//! Independently authored SYNTHETIC observations, never Docker campaign evidence.
//! No launcher, collector, policy constructor or verifier generates expected facts.
#[path = "environment-fixture.rs"]
mod environment_fixture;
#[path = "filesystem-fixture.rs"]
mod filesystem_fixture;
#[path = "process-fixture.rs"]
mod process_fixture;
pub use process_fixture::PROCESSES;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
pub struct Bundle {
    pub _directory: tempfile::TempDir,
    pub root: PathBuf,
    pub source: PathBuf,
    pub sha: String,
    pub tree: String,
}
pub fn digest(b: &[u8]) -> String {
    hex(&Sha256::digest(b))
}
pub fn hex(b: &[u8]) -> String {
    b.iter().map(|v| format!("{v:02x}")).collect()
}
pub fn put(root: &Path, name: &str, v: &Value) {
    fs::write(root.join(name), serde_json::to_vec_pretty(v).unwrap()).unwrap();
}
pub fn get(root: &Path, name: &str) -> Value {
    serde_json::from_slice(&fs::read(root.join(name)).unwrap()).unwrap()
}
fn git(root: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap())
        .env("HOME", root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout)
        .unwrap()
        .trim_end_matches('\n')
        .to_owned()
}
const OTHER: &[&str] = &[
    "H01/runner",
    "H01/image",
    "H01/runtime",
    "H01/process-policy",
    "H04/positive-coverage",
    "H04/external-git-dir",
    "H04/external-common-dir",
    "H04/escaping-path",
    "H04/replaced-binding",
    "H05/launch-binding",
    "H05/environment",
    "H08/guard-loss",
];
impl Bundle {
    pub fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("evidence");
        let source = directory.path().join("source");
        fs::create_dir(&root).unwrap();
        fs::create_dir(&source).unwrap();
        let source = source.canonicalize().unwrap();
        git(&source, &["init", "-q"]);
        let names = [
            "Cargo.toml",
            "Cargo.lock",
            "rust-toolchain.toml",
            "crates/proto-ring-api/Cargo.toml",
            "crates/proto-ring-api/src/lib.rs",
            "crates/proto-ring-engine/Cargo.toml",
            "crates/proto-ring-engine/src/lib.rs",
            "docs/api/v1/proto-ring-api-01-v1-spec.md",
            "docs/api/api-03-host-qualification.md",
            ".github/workflows/api03-host-qualification.yml",
            "crates/proto-ring-api/tests/host-qualification/Cargo.toml",
            "crates/proto-ring-api/tests/host-qualification/Cargo.lock",
            "crates/proto-ring-api/tests/host-qualification/src/main.rs",
            "crates/proto-ring-api/tests/host-qualification/src/lib.rs",
            "crates/proto-ring-api/tests/host-qualification/src/policy.rs",
            "crates/proto-ring-api/tests/host-qualification/fixtures/probe.py",
            "crates/proto-ring-api/tests/host-qualification/fixtures/rendezvous.py",
        ];
        for name in names {
            let p = source.join(name);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(
                p,
                if name.ends_with("/probe.py") {
                    include_bytes!("../../fixtures/probe.py").as_slice()
                } else if name.ends_with("/rendezvous.py") {
                    include_bytes!("../../fixtures/rendezvous.py").as_slice()
                } else {
                    b"synthetic source observation only\n"
                },
            )
            .unwrap();
        }
        git(&source, &["add", "--all"]);
        git(
            &source,
            &[
                "-c",
                "user.name=Inert Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "synthetic source",
            ],
        );
        let sha = git(&source, &["rev-parse", "HEAD"]);
        let tree = git(&source, &["rev-parse", "HEAD^{tree}"]);
        let mut files = serde_json::Map::new();
        for name in names {
            let b = fs::read(source.join(name)).unwrap();
            files.insert(name.into(),json!({"mode":"100644","blob":git(&source,&["rev-parse",&format!("HEAD:{name}")]),"sha256":digest(&b),"bytes":b.len()}));
        }
        put(
            &root,
            "source.json",
            &json!({"sha":sha,"tree":tree,"files":files}),
        );
        put(
            &root,
            "context.json",
            &json!({"campaign":"api03-123-1","source_sha":sha,"run_id":"123","run_attempt":"1","source_root":source,"owned_root":"/owned/run","subject_root":"/owned/run/subject","program_root":source.join("crates/proto-ring-api/tests/host-qualification/fixtures"),"uid":1001,"gid":1001,"root_identity":{"device":1,"inode":100,"mode":16832,"links":2},"network_name":"api03-123-1-network"}),
        );
        put(
            &root,
            "source-state.json",
            &json!({"invalidated":false,"checks":[{"phase":"before_probes","unchanged":true},{"phase":"after_probes","unchanged":true}]}),
        );
        for phase in ["before_probes", "after_probes"] {
            put(
                &root,
                &format!("source-{phase}.json"),
                &json!({"phase":phase,"unchanged":true}),
            );
        }
        put(
            &root,
            "image.json",
            &json!([{"Id":"sha256:141fe9b385f3bb6d618c07ace1836c0b3c72bf525566ac3eb13cd1f405069877","Architecture":"amd64","Os":"linux","RepoDigests":["python@sha256:8a00ecb83ed9d0864dfd102a1bdb2067fda907ceb30bb86d09da9ee558b19590"],"Config":{"Env":["PATH=/usr/local/bin:/usr/bin:/bin","LANG=C.UTF-8"]}}]),
        );
        put(
            &root,
            "runner.json",
            &json!({"RUNNER_OS":"Linux","RUNNER_ARCH":"X64","ImageOS":"ubuntu24","ImageVersion":"synthetic-not-a-runner"}),
        );
        let mut cases = serde_json::Map::new();
        for name in OTHER {
            cases.insert((*name).into(),json!({"status":"PASS","reason":"synthetic oracle","checks":[{"kind":"equal","observed":true,"required":true}]}));
        }
        let mut b = Self {
            _directory: directory,
            root,
            source,
            sha,
            tree,
        };
        put(
            &b.root,
            "controller.json",
            &json!({"source_sha":b.sha,"source_tree":b.tree,"binary_sha256":"f".repeat(64),"executable":"/synthetic/controller"}),
        );
        put(
            &b.root,
            "network.json",
            &json!([{"Id":"e".repeat(64),"Name":"api03-123-1-network","Labels":{"org.proto-ring.api03-qualification":"api03-123-1"},"Internal":true,"Driver":"bridge"}]),
        );
        fs::write(
            b.root.join("network-id.txt"),
            format!("{}\n", "e".repeat(64)),
        )
        .unwrap();
        for (index, (case, prefix, code, out, err)) in PROCESSES.iter().enumerate() {
            b.process(prefix, index + 1, *code, out, err);
            let mut checks = vec![
                json!({"kind":"process","prefix":prefix,"code":code,"stdout_hex":hex(out),"stderr_hex":hex(err)}),
                json!({"kind":"closed","inspect":format!("{prefix}-inspect.json"),"container_id":format!("{:064x}",index+1)}),
            ];
            if case.starts_with("H02/") {
                checks.push(json!({"kind":"different_canonical","before":reference(if *prefix=="untracked"{"b"}else{"a"}),"after":reference(if *prefix=="untracked"{"c"}else{"b"})}));
            }
            cases.insert(
                (*case).into(),
                json!({"status":"PASS","reason":"synthetic oracle","checks":checks}),
            );
        }
        b.process("policy", 90, 0, b"", b"");
        b.process("listener", 91, 0, b"", b"");
        environment_fixture::observations(&b);
        filesystem_fixture::observations(&b);
        let authority = include_bytes!("../../fixtures/authority.md").as_slice();
        fs::write(
            b.root.join("governance-effect.bin"),
            [authority, b"\nBounded inert change.\n"].concat(),
        )
        .unwrap();
        for (role, target, before, after) in [
            (
                "governance",
                "authority.md",
                authority.to_vec(),
                [authority, b"\nBounded inert change.\n"].concat(),
            ),
            (
                "untracked",
                "untracked-witness",
                Vec::new(),
                b"inert-untracked\n".to_vec(),
            ),
        ] {
            fs::write(b.root.join(format!("{role}-before.bin")), &before).unwrap();
            fs::write(b.root.join(format!("{role}-after.bin")), &after).unwrap();
            put(
                &b.root,
                &format!("{role}-mutation.json"),
                &json!({"campaign":"api03-123-1","container_id":get(&b.root,&format!("{role}-created.json"))[0]["Id"],"target":format!("/owned/run/subject/{target}"),"before_present":role=="governance","after_present":true,"events":["before_capture","launch","effect_closed","after_capture"],"before":provenance(if role=="untracked"{"b"}else{"a"}),"after":provenance(if role=="untracked"{"c"}else{"b"})}),
            );
        }
        put(&b.root, "rgs-before.json", &reference("a"));
        put(&b.root, "rgs-after.json", &reference("b"));
        let all = [
            "AGENTS.md",
            "authority.md",
            "bindings.md",
            "integrity.md",
            "target",
        ];
        put(
            &b.root,
            "canonical-scopes.json",
            &json!({"campaign":"api03-123-1","repository":"/owned/run/subject","governance":["AGENTS.md","authority.md","bindings.md","integrity.md"],"ri":["target"],"discovered":all,"observation":provenance("a")}),
        );
        let paths:Vec<_>=all.iter().enumerate().map(|(index,p)|json!({"path":p,"resolved":format!("/owned/run/subject/{p}"),"identity":{"device":1,"inode":200+index,"mode":33152,"links":1},"protection":"subject-bind"})).collect();
        put(
            &b.root,
            "positive-git-coverage.json",
            &json!({"root":"/owned/run/subject","git_dir":"/owned/run/subject/.git","common_dir":"/owned/run/subject/.git","contained":true,"alternates_absent":true,"paths":paths}),
        );
        put(
            &b.root,
            "external-git-dir.json",
            &json!({"root":"/owned/run/split","git_dir":"/owned/run/external-git","covered":false}),
        );
        put(
            &b.root,
            "external-common-dir.json",
            &json!({"root":"/owned/run/linked-counterexample","common_dir":"/owned/run/subject/.git","covered":false}),
        );
        put(
            &b.root,
            "escaping-path.json",
            &json!({"outside":true,"alias":"/owned/run/subject/escape-alias","resolved":"/owned/run/outside-inert"}),
        );
        filesystem_fixture::invalidations(&b);
        for (case, prefix, _, _, _) in PROCESSES {
            if case.contains("positive-write")
                || case.ends_with("denied-write")
                || case.ends_with("control-denied")
                || case.ends_with("evidence-denied")
                || case.ends_with("program-denied")
            {
                fs::write(
                    b.root.join(format!("{prefix}-effect.bin")),
                    if case.contains("positive-write") {
                        b"after\n".as_slice()
                    } else if case.ends_with("program-denied") {
                        include_bytes!("../../fixtures/probe.py").as_slice()
                    } else {
                        b"before\n".as_slice()
                    },
                )
                .unwrap();
            }
        }
        for prefix in ["descendant-stream", "descendant-writer"] {
            let v = get(&b.root, &format!("{prefix}-created.json"));
            let id = v[0]["Id"].clone();
            let kind = if prefix.ends_with("stream") {
                "stream"
            } else {
                "writer"
            };
            put(
                &b.root,
                &format!("{prefix}-rendezvous.json"),
                &json!({"campaign":"api03-123-1","instance":format!("api03-123-1-{prefix}"),"container_id":id,"parent":{"pid":1,"ppid":0,"write_fd":null,"kind":"parent","campaign":"api03-123-1","instance":format!("api03-123-1-{prefix}"),"container_id":id,"hostname":id.as_str().unwrap()[..12],"pid_namespace":"pid:[123]"},"closure":"namespace_terminated_by_principal_exit","child":{"pid":2,"ppid":1,"write_fd":3,"kind":kind,"campaign":"api03-123-1","instance":format!("api03-123-1-{prefix}"),"container_id":id,"hostname":id.as_str().unwrap()[..12],"pid_namespace":"pid:[123]"},"parent_ready":b"parent\n".to_vec(),"writer_before_parent_exit":b"before\nchild-started\n".to_vec(),"principal_release_requested":true,"descendant_release_requested":false,"events":["created","started","parent_ready","child_ready","runtime_live","effect_capability","release_parent","principal_exited","perimeter_closed","streams_collected"]}),
            );
            fs::write(b.root.join(format!("{prefix}-top.txt")),b"PID PPID ARGS\n900 1 python3 /program/probe.py parent\n901 900 python3 /program/probe.py child\n").unwrap();
            let mut live = v.clone();
            live[0]["State"] = json!({"Running":true,"Pid":900,"StartedAt":"2026-01-01T00:00:00Z"});
            put(&b.root, &format!("{prefix}-live.json"), &live);
            put(
                &b.root,
                &format!("{prefix}-capability.json"),
                &json!({"container_id":id,"campaign":"api03-123-1","host_parent":900,"host_child":901,"namespace_child":2,"host_ppid":900,"parent_birth":1000,"child_birth":1001,"pid_namespace":"pid:[123]","fd_identity":{"device":1,"inode":300,"mode":33152,"links":1},"target_identity":{"device":1,"inode":300,"mode":33152,"links":1}}),
            );
        }
        put(
            &b.root,
            "campaign.json",
            &json!({"repository":"fanilosendrison/proto-ring","source_sha":b.sha,"source_tree":b.tree,"workflow_sha":b.sha,"run_id":"123","run_attempt":"1","cases":cases,"cleanup_verified":true,"cleanup_containers":[],"cleanup_networks":[]}),
        );
        fs::write(b.root.join("cleanup-containers.txt"), b"").unwrap();
        fs::write(b.root.join("cleanup-networks.txt"), b"").unwrap();
        b.rehash();
        b
    }
    pub fn rehash(&self) {
        let mut files = BTreeMap::new();
        for p in fs::read_dir(&self.root).unwrap() {
            let p = p.unwrap();
            let name = p.file_name().into_string().unwrap();
            if name == "inventory.json" {
                continue;
            }
            let bytes = fs::read(p.path()).unwrap();
            files.insert(name, json!({"bytes":bytes.len(),"sha256":digest(&bytes)}));
        }
        put(&self.root, "inventory.json", &json!(files));
    }
    pub fn edit(&self, name: &str, change: impl FnOnce(&mut Value)) {
        let mut v = get(&self.root, name);
        change(&mut v);
        put(&self.root, name, &v);
        self.rehash();
    }
}
pub fn reference(state: &str) -> Value {
    json!({"format":"proto-ring-rgs-ref-v1","platform":"unix","repository":"1".repeat(64),"scope":"2".repeat(64),"state":state.repeat(64),"coherence_basis":"ordinary_capture"})
}
pub fn provenance(state: &str) -> Value {
    json!({"repository":{"kind":"utf8","value":"/owned/run/subject"},"scope_paths":[{"kind":"utf8","value":"AGENTS.md"},{"kind":"utf8","value":"authority.md"},{"kind":"utf8","value":"bindings.md"},{"kind":"utf8","value":"integrity.md"}],"observation":reference(state)})
}
