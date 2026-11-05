//! Separate always-step cleanup after controller failure/cancellation where observable.
use crate::{
    Result, ensure,
    evidence::Evidence,
    policy,
    transport::{capture, success},
};
use serde_json::{Value, json};
use std::{path::PathBuf, process::Command};
pub fn fallback() -> Result<()> {
    let run = std::env::var("GITHUB_RUN_ID")?;
    let attempt = std::env::var("GITHUB_RUN_ATTEMPT")?;
    ensure(
        !run.is_empty()
            && !attempt.is_empty()
            && run.bytes().all(|b| b.is_ascii_digit())
            && attempt.bytes().all(|b| b.is_ascii_digit()),
        "invalid cleanup identity",
    )?;
    let campaign = format!("api03-{run}-{attempt}");
    let filter = format!("label={}={campaign}", policy::LABEL);
    let root =
        PathBuf::from(std::env::var("RUNNER_TEMP")?).join("api03-host-evidence/fallback-cleanup");
    let e = Evidence::create(&root)?;
    let mut errors = Vec::new();
    for (kind, list, inspect, remove) in [
        (
            "container",
            vec!["ps", "-aq", "--filter", &filter],
            vec!["inspect"],
            vec!["rm", "-f"],
        ),
        (
            "network",
            vec!["network", "ls", "-q", "--filter", &filter],
            vec!["network", "inspect"],
            vec!["network", "rm"],
        ),
    ] {
        let ids = success(capture(
            &e,
            &format!("{kind}-before"),
            Command::new("docker").args(&list),
        )?)?;
        for (i, id) in std::str::from_utf8(&ids)?
            .lines()
            .filter(|s| !s.is_empty())
            .enumerate()
        {
            let cleanup = (|| -> Result<()> {
                let inspected = success(capture(
                    &e,
                    &format!("{kind}-{i}-inspect"),
                    Command::new("docker").args(&inspect).arg(id),
                )?)?;
                let v: Value = serde_json::from_slice(&inspected)?;
                let label = if kind == "container" {
                    &v[0]["Config"]["Labels"][policy::LABEL]
                } else {
                    &v[0]["Labels"][policy::LABEL]
                };
                ensure(label == &json!(campaign), "fallback ownership mismatch")?;
                success(capture(
                    &e,
                    &format!("{kind}-{i}-remove"),
                    Command::new("docker").args(&remove).arg(id),
                )?)?;
                Ok(())
            })();
            if let Err(error) = cleanup {
                errors.push(error.to_string());
            }
        }
        let remaining = success(capture(
            &e,
            &format!("{kind}-after"),
            Command::new("docker").args(&list),
        )?)?;
        if !remaining.is_empty() {
            errors.push(format!("owned {kind} still present"));
        }
    }
    e.json("result.json",&json!({"campaign":campaign,"verified_absent":errors.is_empty(),"errors":errors,"does_not_upgrade_campaign_completion":true}))?;
    e.inventory()?;
    ensure(errors.is_empty(), "fallback cleanup not verified")
}
