//! Mandatory obligations are chosen by frozen protocol, never by producer_checks.
use crate::{
    Result,
    campaign_context::Context,
    ensure,
    evidence::{Case, Check},
    policy,
    verification::{self, json as read},
};
use std::{fs, path::Path};
pub fn required(root: &Path, name: &str, _case: &Case) -> Result<()> {
    let c: Context = serde_json::from_slice(&fs::read(root.join("context.json"))?)?;
    if let Some((prefix, code, out, err)) = process(name) {
        crate::launch_policy::verify(root, prefix, &c)
            .map_err(|e| format!("{name}: launch policy: {e}"))?;
        verification::checks(
            root,
            &[crate::transport::stream_check(prefix, code, out, err)],
        )
        .map_err(|e| format!("{name}: process observation: {e}"))?;
        if name.starts_with("H02/") {
            crate::mutation_evidence::verify(root, prefix, &c)
                .map_err(|e| format!("{name}: {e}"))?;
        }
        if name.contains("positive-write")
            || name.ends_with("denied-write")
            || name.ends_with("program-denied")
            || name.ends_with("control-denied")
            || name.ends_with("evidence-denied")
        {
            let expected = if name.contains("positive-write") {
                b"after\n".as_slice()
            } else if name.ends_with("program-denied") {
                include_bytes!("../fixtures/probe.py").as_slice()
            } else {
                b"before\n".as_slice()
            };
            ensure(
                fs::read(root.join(format!("{prefix}-effect.bin")))? == expected,
                &format!("{name}: controller byte observation contradicts boundary"),
            )?;
        }
        if name.starts_with("H06/descendant-") {
            crate::process_evidence::verify(root, prefix, &c)
                .map_err(|e| format!("{name}: process attribution: {e}"))?;
        }
        if name == "H08/network-after" {
            crate::launch_policy::verify(root, "listener", &c)?;
            verification::checks(
                root,
                &[crate::transport::stream_check("listener", 0, b"", b"")],
            )?;
        }
        return Ok(());
    }
    match name {
        "H01/runner" => {
            crate::environment_evidence::kernel(root)
                .map_err(|e| format!("{name}: kernel inventory: {e}"))?;
            let v = read(root, "runner.json")?;
            ensure(
                v["RUNNER_OS"] == "Linux"
                    && v["RUNNER_ARCH"] == "X64"
                    && v["ImageOS"] == "ubuntu24"
                    && v["ImageVersion"].as_str().is_some_and(|s| !s.is_empty()),
                "runner evidence absent",
            )?;
        }
        "H01/image" => {
            let v = read(root, "image.json")?;
            ensure(
                v[0]["Id"] == policy::CONFIG
                    && v[0]["Os"] == "linux"
                    && v[0]["Architecture"] == "amd64"
                    && v[0]["RepoDigests"].as_array().is_some_and(|a| {
                        a.iter().any(|s| {
                            s.as_str().is_some_and(|s| {
                                s.ends_with(policy::IMAGE.split('@').nth(1).unwrap_or("invalid"))
                            })
                        })
                    }),
                "image provenance absent",
            )?;
        }
        "H01/runtime" => crate::environment_evidence::runtime(root)
            .map_err(|e| format!("{name}: runtime inventory: {e}"))?,
        "H01/process-policy" | "H05/environment" => {
            crate::launch_policy::verify(root, "policy", &c)
                .map_err(|e| format!("{name}: launch policy: {e}"))?;
            let bytes = fs::read(root.join("policy-logs.stdout"))?;
            verification::checks(
                root,
                &[Check::Process {
                    prefix: "policy".into(),
                    code: 0,
                    stdout_hex: policy::hex(&bytes),
                    stderr_hex: String::new(),
                }],
            )?;
            crate::environment_evidence::process(root, &c)
                .map_err(|e| format!("{name}: process inventory: {e}"))?;
        }
        "H04/positive-coverage" => crate::scope_evidence::verify(root, &c)
            .map_err(|e| format!("{name}: scope coverage: {e}"))?,
        "H04/external-git-dir" | "H04/external-common-dir" => {
            let (file, field) = if name.ends_with("external-git-dir") {
                ("external-git-dir.json", "git_dir")
            } else {
                ("external-common-dir.json", "common_dir")
            };
            let v = read(root, file)?;
            let base = Path::new(v["root"].as_str().ok_or("fixture root absent")?);
            let actual = Path::new(v[field].as_str().ok_or("fixture administration absent")?);
            ensure(
                v["covered"] == false && exact_negative_git(&c, name, base, actual),
                "negative Git coverage not established",
            )?;
        }
        "H04/escaping-path" => {
            let v = read(root, "escaping-path.json")?;
            let target = Path::new(v["resolved"].as_str().ok_or("negative target absent")?);
            let alias = Path::new(v["alias"].as_str().ok_or("negative alias absent")?);
            ensure(
                v["outside"] == true
                    && crate::campaign_context::exact_absolute(alias)
                    && crate::campaign_context::exact_absolute(target)
                    && alias == c.subject_root.join("escape-alias")
                    && target == c.owned_root.join("outside-inert"),
                "escaping alias not established",
            )?;
        }
        "H04/replaced-binding" | "H08/guard-loss" => {
            let v = read(
                root,
                if name == "H08/guard-loss" {
                    "guard-loss.json"
                } else {
                    "replaced-binding.json"
                },
            )?;
            let before: crate::filesystem_identity::Identity =
                serde_json::from_value(v["before"].clone())
                    .map_err(|e| format!("{name}: invalidation before identity: {e}"))?;
            let after: crate::filesystem_identity::Identity =
                serde_json::from_value(v["after"].clone())
                    .map_err(|e| format!("{name}: invalidation after identity: {e}"))?;
            let kind = if name == "H08/guard-loss" {
                "directory"
            } else {
                "regular"
            };
            before
                .validate(kind)
                .map_err(|e| format!("{name}: invalidation before: {e}"))?;
            after
                .validate(kind)
                .map_err(|e| format!("{name}: invalidation after: {e}"))?;
            let expected_path = if name == "H08/guard-loss" {
                c.subject_root.clone()
            } else {
                c.owned_root.join("coverage-binding")
            };
            ensure(
                v["campaign"] == c.campaign && v["path"] == serde_json::to_value(expected_path)?,
                &format!("{name}: invalidation attribution mismatch"),
            )?;
            ensure(
                c.root_identity["device"] == before.device
                    && before.device == after.device
                    && before.inode != after.inode,
                &format!("{name}: actual object identity replacement absent"),
            )?;
            if name == "H08/guard-loss" {
                let initial: crate::filesystem_identity::Identity = serde_json::from_slice(
                    &verification::read_bytes(root, "subject-root-identity.json")?,
                )?;
                ensure(
                    initial.device == before.device && initial.inode == before.inode,
                    "guard invalidation is not the observed subject",
                )?;
            }
            if name == "H08/guard-loss" {
                ensure(
                    v["protection_active"] == false && v["next_capture_refused"] == true,
                    "guard incorrectly continued",
                )?;
            } else {
                ensure(v["checkpoint_refused"] == true, "replacement not refused")?;
            }
        }
        "H05/launch-binding" => {
            let s = read(root, "source.json")?;
            ensure(
                s["files"]["crates/proto-ring-api/tests/host-qualification/fixtures/probe.py"]["sha256"]
                    == policy::sha256(include_bytes!("../fixtures/probe.py")),
                "executed program differs from verifier source",
            )?;
        }
        _ => return Err("mandatory case has no verifier".into()),
    }
    Ok(())
}
fn exact_negative_git(c: &Context, name: &str, root: &Path, administration: &Path) -> bool {
    if !crate::campaign_context::exact_absolute(root)
        || !crate::campaign_context::exact_absolute(administration)
    {
        return false;
    }
    if name == "H04/external-git-dir" {
        root == c.owned_root.join("split") && administration == c.owned_root.join("external-git")
    } else {
        root == c.owned_root.join("linked-counterexample")
            && administration == c.subject_root.join(".git")
    }
}
fn process(name: &str) -> Option<(&'static str, i64, &'static [u8], &'static [u8])> {
    Some(match name {
        "H02/governance-mutation" => ("governance", 0, b"governance-written\n", b""),
        "H02/untracked-mutation" => ("untracked", 0, b"untracked-written\n", b""),
        "H03/positive-write" => ("writer-positive", 0, b"written\n", b""),
        "H03/denied-write" => ("writer-denied", 30, b"", b"started\nEROFS\n"),
        "H05/positive-write" => ("program-positive", 0, b"written\n", b""),
        "H05/program-denied" => ("program-denied", 30, b"", b"started\nEROFS\n"),
        "H05/control-denied" => ("control-denied", 30, b"", b"started\nEROFS\n"),
        "H06/simple" => ("simple", 0, b"simple\n", b""),
        "H06/descendant-stream" => ("descendant-stream", 0, b"child-stream\nparent-exit\n", b""),
        "H06/descendant-writer" => ("descendant-writer", 0, b"parent-exit\n", b""),
        "H07/streams" => (
            "output",
            23,
            b"api03-stdout\x00\xff\n",
            b"api03-stderr\xfe\n",
        ),
        "H07/positive-write" => ("evidence-positive", 0, b"written\n", b""),
        "H07/evidence-denied" => ("evidence-denied", 30, b"", b"started\nEROFS\n"),
        "H08/network-before" => ("network-before", 0, b"api03-pong\n", b""),
        "H08/network-denied" => ("network-denied", 42, b"", b"started\nENETUNREACH\n"),
        "H08/network-after" => ("network-after", 0, b"api03-pong\n", b""),
        _ => return None,
    })
}
