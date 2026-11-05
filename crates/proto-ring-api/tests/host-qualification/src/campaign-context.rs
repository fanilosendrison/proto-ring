//! Controller-owned allocation context, separate from launcher expectations.
use crate::{
    Result, ensure,
    evidence::Evidence,
    subject::{Subject, identity},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Component, Path, PathBuf};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub campaign: String,
    pub source_sha: String,
    pub run_id: String,
    pub run_attempt: String,
    pub owned_root: PathBuf,
    pub subject_root: PathBuf,
    pub program_root: PathBuf,
    pub source_root: PathBuf,
    pub uid: u32,
    pub gid: u32,
    pub root_identity: Value,
    pub network_name: String,
}
impl Context {
    pub fn capture(
        e: &Evidence,
        s: &Subject,
        source_root: &Path,
        user: &str,
        work: &crate::evidence::Campaign,
    ) -> Result<Self> {
        let sha = &work.source_sha;
        let run = &work.run_id;
        let attempt = &work.run_attempt;
        let program = source_root.join("crates/proto-ring-api/tests/host-qualification/fixtures");
        let (uid, gid) = user
            .split_once(':')
            .ok_or("numeric controller identity absent")?;
        let campaign = format!("api03-{run}-{attempt}");
        let context = Self {
            network_name: format!("{campaign}-network"),
            campaign,
            source_sha: sha.clone(),
            run_id: run.clone(),
            run_attempt: attempt.clone(),
            owned_root: s.owned.path().canonicalize()?,
            subject_root: s.root.clone(),
            program_root: program.to_owned(),
            source_root: source_root.to_owned(),
            uid: uid.parse()?,
            gid: gid.parse()?,
            root_identity: identity(s.owned.path())?,
        };
        e.json("context.json", &context)?;
        Ok(context)
    }
    pub fn instance(&self, prefix: &str) -> String {
        format!("{}-{prefix}", self.campaign)
    }
    pub fn validate(&self, sha: &str, run: &str, attempt: &str) -> Result<()> {
        let source = &self.source_root;
        ensure(
            self.source_sha == sha
                && self.run_id == run
                && self.run_attempt == attempt
                && self.campaign == format!("api03-{run}-{attempt}"),
            "campaign context identity mismatch",
        )?;
        ensure(
            self.uid > 0 && self.gid > 0,
            "launch policy: nonroot controller identity required",
        )?;
        for path in [
            &self.owned_root,
            &self.subject_root,
            &self.program_root,
            &self.source_root,
        ] {
            ensure(exact_absolute(path), "ambiguous allocation path")?;
        }
        ensure(
            self.subject_root == self.owned_root.join("subject")
                && !self.owned_root.starts_with(source)
                && !source.starts_with(&self.owned_root),
            "allocation domain is not an independent subject",
        )?;
        ensure(
            self.program_root
                == source.join("crates/proto-ring-api/tests/host-qualification/fixtures"),
            "program allocation is not authorized source",
        )?;
        ensure(
            self.network_name == format!("{}-network", self.campaign),
            "foreign network allocation",
        )?;
        ensure(
            self.root_identity["device"].as_u64().is_some()
                && self.root_identity["inode"].as_u64().is_some_and(|n| n > 0)
                && self.root_identity["mode"]
                    .as_u64()
                    .is_some_and(|m| m & 0o170000 == 0o040000 && m & 0o077 == 0),
            "owned allocation identity/permissions absent",
        )
    }
}
pub fn exact_absolute(path: &Path) -> bool {
    path.is_absolute()
        && path
            .components()
            .all(|p| matches!(p, Component::RootDir | Component::Normal(_)))
        && path.to_str().is_some_and(|s| {
            s.strip_prefix('/').is_some_and(|s| {
                !s.is_empty() && s.split('/').all(|v| !v.is_empty() && v != "." && v != "..")
            })
        })
}
