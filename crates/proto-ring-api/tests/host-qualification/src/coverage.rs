//! Experimental coverage rejection and monotone linkage-loss witness.
use crate::{
    Result, ensure,
    evidence::{Check, Evidence},
    subject::{Subject, identity},
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

pub fn counterexamples(
    subject: &Subject,
    evidence: &Evidence,
) -> Result<Vec<(String, Vec<Check>)>> {
    let context: crate::campaign_context::Context =
        serde_json::from_slice(&fs::read(evidence.root.join("context.json"))?)?;
    let owned = subject.owned.path().canonicalize()?;
    let mut results = Vec::new();
    let external = owned.join("external-git");
    let split = owned.join("split");
    fs::create_dir(&split)?;
    subject.git(
        evidence,
        "split-init",
        &[
            "-C",
            split.to_str().ok_or("split path")?,
            "init",
            "-q",
            "--separate-git-dir",
            external.to_str().ok_or("gitdir path")?,
        ],
    )?;
    let bytes = subject.git(
        evidence,
        "split-observe",
        &[
            "-C",
            split.to_str().ok_or("split path")?,
            "rev-parse",
            "--path-format=absolute",
            "--git-dir",
        ],
    )?;
    let observed = PathBuf::from(std::str::from_utf8(&bytes)?.trim()).canonicalize()?;
    evidence.json(
        "external-git-dir.json",
        &json!({"root":split,"git_dir":observed,"covered":observed.starts_with(&split)}),
    )?;
    ensure(
        observed == external.canonicalize()? && !observed.starts_with(&split),
        "external git-dir not established",
    )?;
    results.push((
        "H04/external-git-dir".into(),
        vec![equal(json!(observed.starts_with(&split)), json!(false))],
    ));
    let linked = owned.join("linked-counterexample");
    subject.git(
        evidence,
        "linked-create",
        &[
            "worktree",
            "add",
            "--detach",
            linked.to_str().ok_or("linked path")?,
            "HEAD",
        ],
    )?;
    let bytes = subject.git(
        evidence,
        "linked-common",
        &[
            "-C",
            linked.to_str().ok_or("linked path")?,
            "rev-parse",
            "--path-format=absolute",
            "--git-common-dir",
        ],
    )?;
    let common = PathBuf::from(std::str::from_utf8(&bytes)?.trim()).canonicalize()?;
    evidence.json(
        "external-common-dir.json",
        &json!({"root":linked,"common_dir":common,"covered":common.starts_with(&linked)}),
    )?;
    ensure(
        !common.starts_with(&linked),
        "external common-dir not established",
    )?;
    results.push((
        "H04/external-common-dir".into(),
        vec![equal(json!(common.starts_with(&linked)), json!(false))],
    ));
    #[cfg(unix)]
    {
        let alias = subject.root.join("escape-alias");
        let outside = owned.join("outside-inert");
        fs::write(&outside, b"inert\n")?;
        std::os::unix::fs::symlink(&outside, &alias)?;
        let escaped = !alias.canonicalize()?.starts_with(&subject.root);
        let rejected = proto_ring_engine::repository_state::capture(
            &subject.root,
            &[PathBuf::from("escape-alias")],
        )
        .is_err();
        evidence.json("escaping-path.json",&json!({"alias":alias,"resolved":alias.canonicalize()?,"outside":escaped,"canonical_rejected":rejected}))?;
        ensure(escaped, "escaping alias not established")?;
        results.push((
            "H04/escaping-path".into(),
            vec![equal(json!(escaped), json!(true))],
        ));
        fs::remove_file(alias)?;
    }
    let binding = owned.join("coverage-binding");
    fs::write(&binding, b"first\n")?;
    let mut lease = LinkageLease::new(binding.clone())?;
    fs::rename(&binding, subject.owned.path().join("coverage-original"))?;
    fs::write(&binding, b"second\n")?;
    let active = lease.ensure_active();
    evidence.json("replaced-binding.json",&json!({"campaign":context.campaign,"path":binding,"before":lease.original,"after":identity(&binding)?,"checkpoint_refused":active.is_err()}))?;
    results.push((
        "H04/replaced-binding".into(),
        vec![equal(json!(active.is_err()), json!(true))],
    ));
    Ok(results)
}
pub fn equal(observed: Value, required: Value) -> Check {
    Check::Equal { observed, required }
}

pub struct LinkageLease {
    path: PathBuf,
    original: Value,
    active: bool,
}
impl LinkageLease {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self {
            original: identity(&path)?,
            path,
            active: true,
        })
    }
    pub fn ensure_active(&mut self) -> Result<()> {
        if !self.active {
            return Err("guard already lost".into());
        }
        if identity(&self.path)? != self.original {
            self.active = false;
            return Err("actual owned linkage replaced".into());
        }
        Ok(())
    }
    pub fn guarded_observation(&mut self) -> Result<Value> {
        self.ensure_active()?;
        Ok(serde_json::to_value(
            proto_ring_api::open(&self.path)?.reference(),
        )?)
    }
}
pub fn invalidate(subject: &Subject, evidence: &Evidence) -> Result<Vec<Check>> {
    let context: crate::campaign_context::Context =
        serde_json::from_slice(&fs::read(evidence.root.join("context.json"))?)?;
    let mut lease = LinkageLease::new(subject.root.clone())?;
    lease.ensure_active()?;
    fs::rename(&subject.root, subject.owned.path().join("original-subject"))?;
    fs::create_dir(&subject.root)?;
    let refused = lease.guarded_observation().is_err();
    evidence.json("guard-loss.json",&json!({"campaign":context.campaign,"path":subject.root,"before":lease.original,"after":identity(&subject.root)?,"protection_active":lease.active,"next_capture_refused":refused,"reason":"actual subject root replacement"}))?;
    ensure(
        !lease.active && refused,
        "guard loss not detected before capture",
    )?;
    Ok(vec![
        equal(json!(lease.active), json!(false)),
        equal(json!(refused), json!(true)),
    ])
}
