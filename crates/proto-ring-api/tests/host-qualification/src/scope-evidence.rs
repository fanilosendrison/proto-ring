//! Retention and coverage verification of canonical scope output, not selector resolution.
use crate::{
    Result,
    campaign_context::{Context, exact_absolute},
    ensure,
    evidence::Evidence,
    subject::{Subject, identity},
    verification::json as read,
};
use proto_ring_engine::{repository_governance_state, repository_integrity, repository_state};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scopes {
    pub campaign: String,
    pub repository: PathBuf,
    pub governance: Vec<PathBuf>,
    pub ri: Vec<PathBuf>,
    pub discovered: Vec<PathBuf>,
    pub observation: Value,
}
pub fn canonical(root: &Path, campaign: &str) -> Result<Scopes> {
    let state = repository_governance_state::load(root)?;
    let profile = state
        .repository_integrity
        .as_ref()
        .ok_or("synthetic RI missing")?;
    let realized = repository_integrity::realize_consumer_profile(
        root,
        profile,
        &repository_integrity::EvaluationContext::default(),
    )?;
    ensure(
        !realized
            .profile
            .validations
            .iter()
            .any(|v| v.resolution_failed),
        "canonical RI scope unavailable",
    )?;
    let api = proto_ring_api::open(root)?;
    ensure(
        api.scope_paths() == state.observed_state.scope_paths
            && api.repository() == state.repository,
        "canonical observation domains drifted",
    )?;
    Ok(Scopes {
        campaign: campaign.into(),
        repository: state.repository,
        governance: state.observed_state.scope_paths,
        ri: realized.profile.governed_paths,
        discovered: repository_state::discover_repository_paths(root, &[])?,
        observation: serde_json::to_value(api.provenance())?,
    })
}
pub fn union(scopes: &Scopes) -> BTreeSet<PathBuf> {
    scopes
        .governance
        .iter()
        .chain(&scopes.ri)
        .chain(&scopes.discovered)
        .cloned()
        .collect()
}
pub fn capture(s: &Subject, e: &Evidence, name: &str, git: &Path, common: &Path) -> Result<bool> {
    let before = identity(&s.root)?;
    let campaign = if e.root.join("context.json").exists() {
        read(&e.root, "context.json")?["campaign"]
            .as_str()
            .ok_or("context campaign")?
            .to_owned()
    } else {
        "offline-canonical-control".into()
    };
    let scopes = canonical(&s.root, &campaign)?;
    let mut paths = Vec::new();
    let mut covered = true;
    for p in union(&scopes) {
        let path = s.root.join(&p);
        let resolved = path.canonicalize()?;
        let meta = identity(&path)?;
        covered &= resolved.starts_with(&s.root);
        covered &= meta["links"] == 1 && meta["mode"].as_u64().is_some_and(|m| m & 0o022 == 0);
        paths.push(
            json!({"path":p,"resolved":resolved,"identity":meta,"protection":"subject-bind"}),
        );
    }
    ensure(
        identity(&s.root)? == before,
        "root binding changed during canonical scope capture",
    )?;
    let subject_link = crate::filesystem_identity::Linkage::observe(&s.root)?;
    subject_link.require(&s.root, "directory")?;
    let mut administration = Vec::new();
    for (relative, kind) in ADMINISTRATION {
        let path = s.root.join(relative);
        let link = crate::filesystem_identity::Linkage::observe(&path)?;
        link.require(&path, kind)?;
        administration.push(link);
    }
    e.json("subject-root-identity.json", &subject_link.identity)?;
    e.json(
        "positive-git-identities.json",
        &GitIdentities {
            campaign: campaign.clone(),
            subject: subject_link,
            administration,
        },
    )?;
    e.json("canonical-scopes.json", &scopes)?;
    e.json(&format!("{name}-git-coverage.json"),&json!({"root":s.root,"git_dir":git,"common_dir":common,"contained":covered,"alternates_absent":true,"paths":paths}))?;
    Ok(covered)
}
const ADMINISTRATION: &[(&str, &str)] = &[
    (".git", "directory"),
    (".git/HEAD", "regular"),
    (".git/index", "regular"),
    (".git/config", "regular"),
    (".git/objects", "directory"),
    (".git/objects/info", "directory"),
    (".git/refs", "directory"),
    (".git/refs/heads", "directory"),
    (".git/refs/heads/main", "regular"),
    (".git/objects/info/alternates", "absent"),
    (".git/commondir", "absent"),
];
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GitIdentities {
    campaign: String,
    subject: crate::filesystem_identity::Linkage,
    administration: Vec<crate::filesystem_identity::Linkage>,
}
fn verify_administration(root: &Path, c: &Context) -> Result<BTreeSet<(u64, u64)>> {
    let witness: crate::filesystem_identity::Identity = serde_json::from_slice(
        &crate::verification::read_bytes(root, "subject-root-identity.json")?,
    )?;
    witness.validate("directory")?;
    let allocation: crate::filesystem_identity::Identity =
        serde_json::from_value(c.root_identity.clone())?;
    allocation.validate("directory")?;
    ensure(
        witness.device == allocation.device && witness.inode != allocation.inode,
        "subject is not the allocated child domain",
    )?;
    let retained: GitIdentities = serde_json::from_slice(&crate::verification::read_bytes(
        root,
        "positive-git-identities.json",
    )?)?;
    ensure(
        retained.campaign == c.campaign,
        "Git identity campaign mismatch",
    )?;
    retained.subject.require(&c.subject_root, "directory")?;
    ensure(
        retained.subject.identity.as_ref() == Some(&witness),
        "subject root identity contradicted",
    )?;
    let mut paths = BTreeMap::new();
    let mut objects = BTreeSet::from([
        (allocation.device, allocation.inode),
        (witness.device, witness.inode),
    ]);
    for record in &retained.administration {
        if let Some(identity) = &record.identity {
            ensure(
                identity.device == witness.device
                    && objects.insert((identity.device, identity.inode)),
                "aliased or external Git administrative object",
            )?;
        }
        ensure(
            paths.insert(record.path.clone(), record).is_none(),
            "duplicate Git identity path",
        )?;
    }
    ensure(
        paths.len() == ADMINISTRATION.len(),
        "required Git identity coverage missing",
    )?;
    for (relative, kind) in ADMINISTRATION {
        let path = c.subject_root.join(relative);
        paths
            .get(&path)
            .ok_or_else(|| format!("Git identity absent: {relative}"))?
            .require(&path, kind)?;
    }
    Ok(objects)
}
pub fn verify(root: &Path, c: &Context) -> Result<()> {
    let s: Scopes = serde_json::from_slice(&fs::read(root.join("canonical-scopes.json"))?)?;
    ensure(
        s.campaign == c.campaign
            && s.repository == c.subject_root
            && s.observation["repository"] == json!({"kind":"utf8","value":c.subject_root}),
        "canonical scope attribution mismatch",
    )?;
    for vector in [&s.governance, &s.ri, &s.discovered] {
        let unique: BTreeSet<_> = vector.iter().collect();
        ensure(
            unique.len() == vector.len(),
            "duplicate canonical scope path",
        )?;
        for p in vector {
            ensure(
                crate::source::safe_relative(p.to_str().ok_or("scope encoding")?),
                "canonical scope path outside subject",
            )?;
        }
    }
    let public: Vec<_> = s
        .governance
        .iter()
        .map(|p| json!({"kind":"utf8","value":p}))
        .collect();
    ensure(
        s.observation["scope_paths"] == json!(public),
        "canonical scope/provenance contradiction",
    )?;
    // Frozen fixture obligations independently constrain accidental scope reduction.
    let expected: BTreeSet<_> = ["AGENTS.md", "authority.md", "bindings.md", "integrity.md"]
        .into_iter()
        .map(PathBuf::from)
        .collect();
    ensure(
        s.governance.iter().cloned().collect::<BTreeSet<_>>() == expected
            && s.ri == [PathBuf::from("target")],
        "canonical fixture scope reduced",
    )?;
    let mut expected_discovered = expected;
    expected_discovered.insert(PathBuf::from("target"));
    ensure(
        s.discovered.iter().cloned().collect::<BTreeSet<_>>() == expected_discovered,
        "canonical discovered fixture scope reduced",
    )?;
    let _: proto_ring_api::ObservationProvenance = serde_json::from_value(s.observation.clone())?;
    let mut identities = verify_administration(root, c)?;
    let v = read(root, "positive-git-coverage.json")?;
    ensure(
        v["root"] == json!(c.subject_root)
            && v["contained"] == true
            && v["alternates_absent"] == true,
        "positive protection not established",
    )?;
    for field in ["git_dir", "common_dir"] {
        let p = Path::new(v[field].as_str().ok_or("Git administrative path absent")?);
        ensure(
            exact_absolute(p) && p == c.subject_root.join(".git"),
            "Git administrative dependency outside protection",
        )?;
    }
    let paths = v["paths"].as_array().ok_or("per-path coverage absent")?;
    let mut observed = BTreeMap::new();
    for p in paths {
        let relative = p["path"].as_str().ok_or("path identity absent")?;
        ensure(
            crate::source::safe_relative(relative),
            "ambiguous coverage path",
        )?;
        let resolved = Path::new(
            p["resolved"]
                .as_str()
                .ok_or("resolved coverage path absent")?,
        );
        ensure(
            exact_absolute(resolved)
                && resolved == c.subject_root.join(relative)
                && p["protection"] == "subject-bind",
            "alias or foreign protection",
        )?;
        let metadata: crate::filesystem_identity::Identity =
            serde_json::from_value(p["identity"].clone())?;
        metadata.validate("regular")?;
        ensure(
            c.root_identity["device"] == metadata.device,
            "scope file outside allocated filesystem domain",
        )?;
        let id = (metadata.device, metadata.inode);
        ensure(
            identities.insert(id) && observed.insert(PathBuf::from(relative), p).is_none(),
            "duplicate/alias coverage path",
        )?;
    }
    ensure(
        observed.keys().cloned().collect::<BTreeSet<_>>() == union(&s),
        "canonical required path omitted",
    )?;
    let mount = read(root, "policy-created.json")?;
    let mounts = mount[0]["Mounts"]
        .as_array()
        .ok_or("protected mount absent")?;
    ensure(
        mounts.iter().any(|m| {
            m["Type"] == "bind"
                && m["Source"] == json!(c.subject_root)
                && m["Destination"] == json!(c.subject_root)
                && m["RW"] == false
        }),
        "coverage contradicted by effective mount",
    )
}
