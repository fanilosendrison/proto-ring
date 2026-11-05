use api03_host_qualification::{
    case_verification,
    coverage::LinkageLease,
    evidence::{Campaign, Case, Check, Evidence, Status},
    policy,
    subject::Subject,
    verification,
};
use serde_json::json;
use std::fs;

#[test]
fn missing_mandatory_evidence_prevents_qualification() {
    let d = tempfile::tempdir().unwrap();
    assert!(verification::verify(d.path(), &"a".repeat(40), "1", "1", d.path()).is_err());
}
#[test]
fn declared_pass_cannot_replace_observations() {
    let d = tempfile::tempdir().unwrap();
    let case = Case {
        status: Status::Pass,
        reason: "PASS".into(),
        checks: vec![Check::Equal {
            observed: json!(true),
            required: json!(true),
        }],
    };
    assert!(case_verification::required(d.path(), "H03/denied-write", &case).is_err());
}
#[test]
fn absent_or_weakened_positive_control_refused() {
    let d = tempfile::tempdir().unwrap();
    for name in [
        "H03/positive-write",
        "H05/positive-write",
        "H07/positive-write",
        "H08/network-before",
        "H08/network-after",
    ] {
        let case = Case {
            status: Status::Pass,
            reason: "declared".into(),
            checks: vec![],
        };
        assert!(case_verification::required(d.path(), name, &case).is_err());
    }
}
#[test]
fn all_unknown_states_prevent_qualification() {
    let c = Campaign::new("a".repeat(40), "b".repeat(40), "1".into(), "1".into());
    assert!(!c.cleanup_verified);
    assert!(c.cases.values().all(|v| v.status == Status::NotRun));
    for status in [
        Status::Fail,
        Status::Unavailable,
        Status::Indeterminate,
        Status::NotRun,
    ] {
        assert_ne!(status, Status::Pass);
    }
}
#[test]
fn any_nonpass_subcase_or_unknown_cleanup_prevents_completion() {
    let mut campaign = Campaign::new("a".repeat(40), "b".repeat(40), "1".into(), "1".into());
    for case in campaign.cases.values_mut() {
        case.status = Status::Pass;
    }
    let error = verification::completion(&campaign).unwrap_err().to_string();
    assert_eq!(error, "cleanup unknown");
    campaign.cleanup_verified = true;
    for status in [
        Status::Fail,
        Status::Unavailable,
        Status::Indeterminate,
        Status::NotRun,
    ] {
        campaign.cases.get_mut("H08/network-after").unwrap().status = status;
        assert_eq!(
            verification::completion(&campaign).unwrap_err().to_string(),
            "non-PASS mandatory subcase"
        );
    }
}
#[test]
fn canonical_reference_requires_real_change_in_same_domain() {
    let d = tempfile::tempdir().unwrap();
    let a = json!({"format":"proto-ring-rgs-ref-v1","platform":"unix","repository":"a".repeat(64),"scope":"b".repeat(64),"state":"c".repeat(64),"coherence_basis":"ordinary_capture"});
    let mut b = a.clone();
    let check = Check::DifferentCanonical {
        before: a.clone(),
        after: b.clone(),
    };
    assert!(verification::checks(d.path(), &[check]).is_err());
    b["state"] = json!("d".repeat(64));
    verification::checks(
        d.path(),
        &[Check::DifferentCanonical {
            before: a.clone(),
            after: b.clone(),
        }],
    )
    .unwrap();
    b["scope"] = json!("e".repeat(64));
    assert!(
        verification::checks(
            d.path(),
            &[Check::DifferentCanonical {
                before: a,
                after: b
            }]
        )
        .is_err()
    );
}
#[test]
fn false_privileged_observation_rejected() {
    let d = tempfile::tempdir().unwrap();
    let a = json!({"format":"proto-ring-rgs-ref-v1","platform":"unix","repository":"a","scope":"b","state":"c".repeat(64),"coherence_basis":"host_established"});
    let mut b = a.clone();
    b["state"] = json!("d".repeat(64));
    assert!(
        verification::checks(
            d.path(),
            &[Check::DifferentCanonical {
                before: a,
                after: b
            }]
        )
        .is_err()
    );
}
#[test]
fn wrong_program_fingerprint_rejected() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("source.json"),br#"{"files":{"crates/proto-ring-api/tests/host-qualification/fixtures/probe.py":{"sha256":"wrong"}}}"#).unwrap();
    let case = Case {
        status: Status::Pass,
        reason: String::new(),
        checks: vec![],
    };
    assert!(case_verification::required(d.path(), "H05/launch-binding", &case).is_err());
}
#[test]
fn exclusive_evidence_rejects_replacement() {
    let d = tempfile::tempdir().unwrap();
    let e = Evidence::create(&d.path().join("evidence")).unwrap();
    e.bytes("witness", b"original").unwrap();
    assert!(e.bytes("witness", b"replacement").is_err());
    assert_eq!(fs::read(e.root.join("witness")).unwrap(), b"original");
}
#[test]
fn inventory_detects_changed_bytes() {
    let d = tempfile::tempdir().unwrap();
    let e = Evidence::create(&d.path().join("evidence")).unwrap();
    e.bytes("witness", b"original").unwrap();
    let first = api03_host_qualification::evidence::inventory(&e.root).unwrap();
    fs::write(e.root.join("witness"), b"changed").unwrap();
    assert_ne!(
        first,
        api03_host_qualification::evidence::inventory(&e.root).unwrap()
    );
}
#[test]
fn real_linkage_replacement_latches_guard_loss() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("binding");
    fs::write(&p, b"original").unwrap();
    let mut lease = LinkageLease::new(p.clone()).unwrap();
    lease.ensure_active().unwrap();
    fs::rename(&p, d.path().join("old")).unwrap();
    fs::write(&p, b"replacement").unwrap();
    assert!(lease.ensure_active().is_err());
    fs::remove_file(&p).unwrap();
    fs::rename(d.path().join("old"), &p).unwrap();
    assert!(lease.ensure_active().is_err());
}
#[test]
fn source_syntax_is_exact() {
    for bad in ["main", "ABCDEF", "a/b", "$(anything)"] {
        assert!(!policy::source_sha(bad));
    }
    assert!(policy::source_sha(&"a".repeat(40)));
}
#[test]
fn standalone_subject_loads_and_pure_scope_is_canonical() {
    let d = tempfile::tempdir().unwrap();
    let e = Evidence::create(&d.path().join("evidence")).unwrap();
    let s = Subject::new(d.path(), &e).unwrap();
    assert!(
        s.coverage(&e, "offline").unwrap(),
        "{}",
        String::from_utf8(std::fs::read(e.root.join("offline-git-coverage.json")).unwrap())
            .unwrap()
    );
    let before = s.observation().unwrap();
    assert!(
        s.canonical_scope()
            .unwrap()
            .iter()
            .any(|p| p.to_str() == Some("target"))
    );
    fs::write(s.root.join("target"), b"after\n").unwrap();
    let after = s.observation().unwrap();
    verification::checks(&e.root, &[Check::DifferentCanonical { before, after }]).unwrap();
}
