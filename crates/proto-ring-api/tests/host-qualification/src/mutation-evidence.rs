//! Mandatory H02 bytes and retained canonical provenance; no producer-selected checks.
use crate::{
    Result,
    campaign_context::Context,
    ensure,
    evidence::{Check, Evidence},
    verification::{self, json as read},
};
use serde_json::{Value, json};
use std::{fs, path::Path};
pub fn provenance(subject: &Path) -> Result<Value> {
    Ok(serde_json::to_value(
        proto_ring_api::open(subject)?.provenance(),
    )?)
}
pub fn capture_before(e: &Evidence, subject: &Path, role: &str) -> Result<Value> {
    let path = subject.join(if role == "governance" {
        "authority.md"
    } else {
        "untracked-witness"
    });
    let bytes = if path.exists() {
        fs::read(path)?
    } else {
        Vec::new()
    };
    e.bytes(&format!("{role}-before.bin"), &bytes)?;
    provenance(subject)
}
pub fn capture_after(e: &Evidence, c: &Context, role: &str, before: Value, id: &str) -> Result<()> {
    let target = c.subject_root.join(if role == "governance" {
        "authority.md"
    } else {
        "untracked-witness"
    });
    let bytes = fs::read(&target)?;
    e.bytes(&format!("{role}-after.bin"), &bytes)?;
    e.json(&format!("{role}-mutation.json"),&json!({"campaign":c.campaign,"container_id":id,"target":target,"before_present":role=="governance","after_present":true,"events":["before_capture","launch","effect_closed","after_capture"],"before":before,"after":provenance(&c.subject_root)?}))
}
pub fn verify(root: &Path, role: &str, c: &Context) -> Result<()> {
    let v = read(root, &format!("{role}-mutation.json"))?;
    let target = if role == "governance" {
        "authority.md"
    } else {
        "untracked-witness"
    };
    let created = read(root, &format!("{role}-created.json"))?;
    ensure(
        v["campaign"] == c.campaign
            && v["container_id"] == created[0]["Id"]
            && v["target"] == json!(c.subject_root.join(target)),
        "mutation attribution/target mismatch",
    )?;
    ensure(
        v["events"] == json!(["before_capture", "launch", "effect_closed", "after_capture"])
            && v["before_present"] == (role == "governance")
            && v["after_present"] == true,
        "mutation event/presence mismatch",
    )?;
    let before = if role == "governance" {
        include_bytes!("../fixtures/authority.md").to_vec()
    } else {
        Vec::new()
    };
    let after = if role == "governance" {
        [
            include_bytes!("../fixtures/authority.md").as_slice(),
            b"\nBounded inert change.\n",
        ]
        .concat()
    } else {
        b"inert-untracked\n".to_vec()
    };
    ensure(
        fs::read(root.join(format!("{role}-before.bin")))? == before
            && fs::read(root.join(format!("{role}-after.bin")))? == after,
        "mutation bytes differ from frozen fixture",
    )?;
    if role == "governance" {
        ensure(
            fs::read(root.join("governance-effect.bin"))? == after,
            "mutation bytes effect proof missing/contradictory",
        )?;
        ensure(
            read(root, "rgs-before.json")? == v["before"]["observation"]
                && read(root, "rgs-after.json")? == v["after"]["observation"],
            "retained canonical observation contradicted",
        )?;
    }
    for observation in [&v["before"], &v["after"]] {
        ensure(
            observation["repository"] == json!({"kind":"utf8","value":c.subject_root})
                && observation["scope_paths"]
                    == read(root, "canonical-scopes.json")?["observation"]["scope_paths"],
            "canonical mutation subject/scope mismatch",
        )?;
    }
    let check = Check::DifferentCanonical {
        before: v["before"]["observation"].clone(),
        after: v["after"]["observation"].clone(),
    };
    verification::checks(root, &[check])?;
    if role == "untracked" {
        let first = read(root, "governance-mutation.json")?;
        ensure(
            v["before"] == first["after"],
            "canonical temporal chain mismatch",
        )?;
    }
    // Never emit internal RepositoryState identity bytes; only public references above.
    Ok(())
}
