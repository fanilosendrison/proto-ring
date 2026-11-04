#[path = "support/git-governance.rs"]
mod git_governance;

use git_governance::{GitGovernance, native_units};
use proto_ring_api::{ErrorCode, GovernanceStage, open};
use proto_ring_engine::repository_governance_state;
use sha2::{Digest, Sha256};

// Independent buffered framing oracle, not the production streaming primitive.
fn hash(domain: &str, parts: &[&[u8]]) -> [u8; 32] {
    fn frame(buffer: &mut Vec<u8>, bytes: &[u8]) {
        buffer.extend((bytes.len() as u64).to_be_bytes());
        buffer.extend(bytes);
    }
    let mut buffer = Vec::new();
    frame(&mut buffer, b"proto-ring-api01-v1");
    frame(&mut buffer, domain.as_bytes());
    buffer.extend((parts.len() as u64).to_be_bytes());
    for part in parts {
        frame(&mut buffer, part);
    }
    Sha256::digest(buffer).into()
}

fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn verify_original_identities(fixture: &GitGovernance) {
    let canonical = repository_governance_state::load(&fixture.root).unwrap();
    let observed = open(&fixture.root).unwrap();
    #[cfg(unix)]
    let platform: &[u8] = b"unix";
    #[cfg(windows)]
    let platform: &[u8] = b"windows";
    let repository = hash(
        "rgs-repository-v1",
        &[platform, &native_units(&canonical.repository)],
    );
    let mut units: Vec<_> = canonical
        .observed_state
        .scope_paths
        .iter()
        .map(|p| native_units(p))
        .collect();
    units.sort();
    let count = (units.len() as u64).to_be_bytes();
    let mut parts = vec![repository.as_slice(), count.as_slice()];
    parts.extend(units.iter().map(Vec::as_slice));
    let scope = hash("rgs-scope-v1", &parts);
    let state = hash(
        "rgs-state-v1",
        &[&repository, &scope, &canonical.observed_state.identity.0],
    );
    assert_eq!(observed.reference().repository.as_str(), hex(&repository));
    assert_eq!(observed.reference().scope.as_str(), hex(&scope));
    assert_eq!(observed.reference().state.as_str(), hex(&state));
    assert_eq!(observed.scope_paths(), canonical.observed_state.scope_paths);
    if let Some(profile) = canonical.repository_integrity {
        assert_eq!(profile.order, ["second", "first"]);
        let width = u64::from(usize::BITS).to_be_bytes();
        let expected = hash("ri-profile-v1", &[&width, &profile.identity.0]);
        let actual = observed.profile_reference().unwrap();
        assert_eq!(actual.sha256.as_str(), hex(&expected));
        assert_eq!(u64::from(actual.native_word_bits), u64::from(usize::BITS));
        assert!(
            profile
                .identity
                .0
                .windows(b"RAW-IDENTITY-SENTINEL".len())
                .any(|w| w == b"RAW-IDENTITY-SENTINEL")
        );
        assert!(!format!("{observed:?}").contains("RAW-IDENTITY-SENTINEL"));
    } else {
        assert!(observed.profile_reference().is_none());
    }
}

#[test]
fn real_minimal_and_full_original_engine_identities_match_independent_framing() {
    verify_original_identities(&GitGovernance::minimal());
    verify_original_identities(&GitGovernance::full());
}

#[test]
fn actual_scope_exposure_keeps_engine_order_while_hashing_native_byte_order() {
    let fixture = GitGovernance::minimal();
    std::fs::create_dir(fixture.root.join("a")).unwrap();
    std::fs::write(fixture.root.join("a-b"), "# dash\n").unwrap();
    std::fs::write(fixture.root.join("a/b"), "# slash\n").unwrap();
    let mut authority = fixture.read("authority.md");
    for (source, path) in [("dash", "a-b"), ("slash", "a/b")] {
        authority["governance_authority"]["sources"][source] =
            serde_json::json!({"repository_target":path});
        authority["governance_authority"]["responsibilities"]["quality"]["roles"][source] =
            serde_json::json!("authority");
    }
    fixture.write("authority.md", authority);
    let observed = open(&fixture.root).unwrap();
    let slash = observed
        .scope_paths()
        .iter()
        .position(|p| p == std::path::Path::new("a/b"))
        .unwrap();
    let dash = observed
        .scope_paths()
        .iter()
        .position(|p| p == std::path::Path::new("a-b"))
        .unwrap();
    assert!(
        slash < dash,
        "engine Path order differs from raw-byte hashing order"
    );
    verify_original_identities(&fixture);
}

#[test]
fn real_failed_loads_preserve_canonical_stages_and_diagnostics_exactly() {
    for mode in ["invalid-profile", "missing-profile", "model-one"] {
        let fixture = GitGovernance::full();
        match mode {
            "invalid-profile" => fixture.write(
                "integrity.md",
                serde_json::json!({"repository_integrity":{}}),
            ),
            "missing-profile" => std::fs::remove_file(fixture.root.join("integrity.md")).unwrap(),
            _ => {
                let mut model = fixture.read("AGENTS.md");
                model["repository_governance"]["model_version"] = serde_json::json!(1);
                fixture.write("AGENTS.md", model);
            }
        }
        let canonical = repository_governance_state::load(&fixture.root).unwrap_err();
        let public = open(&fixture.root).unwrap_err();
        assert_eq!(public.code, ErrorCode::GovernanceLoadFailed);
        assert_eq!(
            serde_json::to_value(public.stage.unwrap()).unwrap(),
            canonical.diagnostic.stage.serialized()
        );
        assert_eq!(
            public.detail.as_deref(),
            Some(canonical.diagnostic.detail.as_str())
        );
        assert_eq!(public.effects_possible, Some(false));
        if mode == "invalid-profile" {
            assert_eq!(public.stage, Some(GovernanceStage::RepositoryIntegrity));
        }
        if mode == "model-one" {
            assert_eq!(
                public.stage,
                Some(GovernanceStage::RepositoryGovernanceModel)
            );
        }
    }
}

#[test]
fn open_does_not_mutate_participating_files_index_head_or_local_config() {
    let fixture = GitGovernance::full();
    let probes = [
        vec!["status", "--porcelain=v1", "-z"],
        vec!["ls-files", "--stage", "-z"],
        vec!["symbolic-ref", "HEAD"],
        vec!["config", "--local", "--list", "-z"],
    ];
    let before: Vec<_> = probes.iter().map(|p| fixture.git(p)).collect();
    let before_files: Vec<_> = [
        "AGENTS.md",
        "authority.md",
        "bindings.md",
        "integrity.md",
        "objects.md",
        "projections.md",
        "evidence.md",
    ]
    .map(|p| std::fs::read(fixture.root.join(p)).unwrap())
    .to_vec();
    let _observed = open(&fixture.root).unwrap();
    let after: Vec<_> = probes.iter().map(|p| fixture.git(p)).collect();
    let after_files: Vec<_> = [
        "AGENTS.md",
        "authority.md",
        "bindings.md",
        "integrity.md",
        "objects.md",
        "projections.md",
        "evidence.md",
    ]
    .map(|p| std::fs::read(fixture.root.join(p)).unwrap())
    .to_vec();
    assert_eq!(before, after);
    assert_eq!(before_files, after_files);
}

#[test]
fn independent_observations_allow_a_return_to_the_same_relevant_state() {
    let fixture = GitGovernance::minimal();
    let path = fixture.root.join("AGENTS.md");
    let original_bytes = std::fs::read(&path).unwrap();
    let original = open(&fixture.root).unwrap();
    let mut changed = original_bytes.clone();
    changed.extend_from_slice(b"changed body\n");
    std::fs::write(&path, changed).unwrap();
    let intermediate = open(&fixture.root).unwrap();
    assert_ne!(original.reference().state, intermediate.reference().state);
    std::fs::write(&path, original_bytes).unwrap();
    let returned = open(&fixture.root).unwrap();
    assert_eq!(original.reference(), returned.reference());
}
