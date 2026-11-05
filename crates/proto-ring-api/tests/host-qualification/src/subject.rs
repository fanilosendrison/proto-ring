//! Standalone synthetic Git subject and canonical observations, never evaluation.
use crate::{
    Result,
    evidence::Evidence,
    transport::{capture, success},
};
use proto_ring_engine::repository_state;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;

pub struct Subject {
    pub owned: TempDir,
    pub root: PathBuf,
    pub home: PathBuf,
}
impl Subject {
    pub fn new(parent: &Path, evidence: &Evidence) -> Result<Self> {
        let owned = tempfile::Builder::new()
            .prefix("api03-subject-")
            .tempdir_in(parent)?;
        let root = owned.path().join("subject");
        let home = owned.path().join("git-home");
        fs::create_dir(&root)?;
        fs::create_dir(&home)?;
        let root = root.canonicalize()?;
        let home = home.canonicalize()?;
        let subject = Self { owned, root, home };
        subject.git(evidence, "init", &["init", "-q", "--initial-branch=main"])?;
        subject.git(
            evidence,
            "config-autocrlf",
            &["config", "core.autocrlf", "false"],
        )?;
        subject.git(
            evidence,
            "config-hooks",
            &["config", "core.hooksPath", "/dev/null"],
        )?;
        for (name, bytes) in [
            (
                "AGENTS.md",
                include_bytes!("../fixtures/AGENTS.md").as_slice(),
            ),
            (
                "authority.md",
                include_bytes!("../fixtures/authority.md").as_slice(),
            ),
            (
                "bindings.md",
                include_bytes!("../fixtures/bindings.md").as_slice(),
            ),
            (
                "integrity.md",
                include_bytes!("../fixtures/integrity.md").as_slice(),
            ),
            ("target", b"before\n".as_slice()),
        ] {
            fs::write(subject.root.join(name), bytes)?;
        }
        subject.git(evidence, "add", &["add", "--all"])?;
        subject.git(
            evidence,
            "commit",
            &[
                "-c",
                "user.name=Qualification Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "inert fixture",
            ],
        )?;
        Ok(subject)
    }
    pub fn git(&self, evidence: &Evidence, name: &str, args: &[&str]) -> Result<Vec<u8>> {
        let mut c = Command::new("git");
        c.env_clear()
            .env("PATH", std::env::var_os("PATH").ok_or("PATH missing")?)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C")
            .arg("-C")
            .arg(&self.root)
            .args(args);
        success(capture(evidence, &format!("git-{name}"), &mut c)?)
    }
    pub fn observation(&self) -> Result<Value> {
        let observation = proto_ring_api::open(&self.root)?;
        Ok(serde_json::to_value(observation.reference())?)
    }
    pub fn canonical_scope(&self) -> Result<Vec<PathBuf>> {
        let scopes = crate::scope_evidence::canonical(&self.root, "offline-canonical-control")?;
        Ok(crate::scope_evidence::union(&scopes).into_iter().collect())
    }
    pub fn coverage(&self, evidence: &Evidence, name: &str) -> Result<bool> {
        let root = repository_state::resolve_worktree_root(&self.root)?;
        let git = self.git(
            evidence,
            &format!("{name}-gitdir"),
            &["rev-parse", "--path-format=absolute", "--git-dir"],
        )?;
        let common = self.git(
            evidence,
            &format!("{name}-common"),
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        )?;
        let git = PathBuf::from(std::str::from_utf8(&git)?.trim());
        let common = PathBuf::from(std::str::from_utf8(&common)?.trim());
        let contained =
            git.canonicalize()?.starts_with(&root) && common.canonicalize()?.starts_with(&root);
        if !contained {
            return Ok(false);
        }
        for file in [
            git.join("objects/info/alternates"),
            common.join("objects/info/alternates"),
            git.join("commondir"),
        ] {
            if file.exists() {
                return Ok(false);
            }
        }
        crate::scope_evidence::capture(self, evidence, name, &git, &common)
    }
    pub fn mount(&self, rw: bool) -> (PathBuf, String, bool) {
        (
            self.root.clone(),
            self.root.to_string_lossy().into_owned(),
            rw,
        )
    }
}
pub fn identity(path: &Path) -> Result<Value> {
    Ok(serde_json::to_value(
        crate::filesystem_identity::Identity::observe(path)?,
    )?)
}
