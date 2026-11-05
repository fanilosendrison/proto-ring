//! Read-only verification; fixture-declared PASS is never consumed.
use crate::{
    Result, ensure,
    evidence::{self, Campaign, Check, Entry, Status},
    policy,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub fn read_bytes(root: &Path, name: &str) -> Result<Vec<u8>> {
    ensure(evidence::safe_name(name), "unsafe evidence reference")?;
    let path = root.join(name);
    let metadata = fs::symlink_metadata(&path).map_err(|e| format!("{name}: {e}"))?;
    ensure(
        metadata.is_file()
            && !metadata.file_type().is_symlink()
            && metadata.len() <= 4 * 1024 * 1024,
        &format!("{name}: invalid evidence type/size"),
    )?;
    Ok(fs::read(path).map_err(|e| format!("{name}: {e}"))?)
}
pub fn json(root: &Path, name: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&read_bytes(root, name)?).map_err(|e| format!("{name}: {e}"))?)
}
pub fn checks(root: &Path, checks: &[Check]) -> Result<()> {
    ensure(!checks.is_empty(), "no independent checks")?;
    for check in checks {
        match check {
            Check::Equal { observed, required } => ensure(
                observed == required,
                "observed value contradicts required value",
            )?,
            Check::Bytes { path, hex } => check_bytes(root, path, hex)?,
            Check::DifferentCanonical { before, after } => {
                let _: proto_ring_api::ObservationReference =
                    serde_json::from_value(before.clone())?;
                let _: proto_ring_api::ObservationReference =
                    serde_json::from_value(after.clone())?;
                for name in [
                    "format",
                    "platform",
                    "repository",
                    "scope",
                    "coherence_basis",
                ] {
                    ensure(
                        before.get(name).is_some() && before.get(name) == after.get(name),
                        "canonical comparison domain mismatch",
                    )?;
                }
                ensure(
                    before["coherence_basis"] == "ordinary_capture",
                    "fabricated privileged observation",
                )?;
                ensure(
                    before["state"].as_str().is_some_and(|s| s.len() == 64)
                        && after["state"].as_str().is_some_and(|s| s.len() == 64)
                        && before["state"] != after["state"],
                    "canonical change not established",
                )?;
            }
            Check::Closed {
                inspect,
                container_id,
            } => {
                let value = json(root, inspect)?;
                ensure(
                    value[0]["Id"] == *container_id
                        && value[0]["State"]["Running"] == false
                        && value[0]["State"]["Pid"] == 0,
                    "effect lifetime not closed",
                )?;
            }
            Check::Process {
                prefix,
                code,
                stdout_hex,
                stderr_hex,
            } => {
                let state = json(root, &format!("{prefix}-inspect.json"))?;
                ensure(
                    state[0]["State"]["Running"] == false && state[0]["State"]["Pid"] == 0,
                    "principal exited without closed perimeter",
                )?;
                ensure(
                    state[0]["State"]["ExitCode"] == *code && state[0]["State"]["Error"] == "",
                    "wrong process exit or startup error",
                )?;
                let started = state[0]["State"]["StartedAt"].as_str().unwrap_or("");
                ensure(
                    !started.is_empty() && !started.starts_with("0001-"),
                    "program not started",
                )?;
                for transport in ["start", "wait", "logs"] {
                    let record = json(root, &format!("{prefix}-{transport}-transport.json"))?;
                    ensure(
                        record["code"] == 0 && record["guard_triggered"] == false,
                        "transport failure is not denial",
                    )?;
                }
                check_bytes(
                    root,
                    &format!("{prefix}-wait.stdout"),
                    &policy::hex(format!("{code}\n").as_bytes()),
                )?;
                check_bytes(root, &format!("{prefix}-logs.stdout"), stdout_hex)?;
                check_bytes(root, &format!("{prefix}-logs.stderr"), stderr_hex)?;
            }
        }
    }
    Ok(())
}
fn check_bytes(root: &Path, name: &str, expected: &str) -> Result<()> {
    ensure(evidence::safe_name(name), "unsafe bytes reference")?;
    let bytes = read_bytes(root, name)?;
    ensure(
        bytes.len() <= policy::STREAM_LIMIT && policy::hex(&bytes) == expected,
        "missing, oversized or contradictory bytes",
    )
}
pub fn verify(root: &Path, sha: &str, run: &str, attempt: &str, source_root: &Path) -> Result<()> {
    let campaign: Campaign = serde_json::from_slice(&read_bytes(root, "campaign.json")?)?;
    ensure(
        policy::source_sha(sha)
            && campaign.source_sha == sha
            && campaign.workflow_sha == sha
            && campaign.repository == "fanilosendrison/proto-ring"
            && campaign.run_id == run
            && campaign.run_attempt == attempt
            && !run.is_empty()
            && !attempt.is_empty()
            && policy::source_sha(&campaign.source_tree),
        "source/run/attempt mismatch",
    )?;
    let expected: BTreeSet<_> = policy::REQUIRED
        .iter()
        .flat_map(|(group, cases)| cases.iter().map(move |name| format!("{group}/{name}")))
        .collect();
    ensure(
        campaign.cases.keys().cloned().collect::<BTreeSet<_>>() == expected,
        "missing or extra mandatory subcase",
    )?;
    completion(&campaign)?;
    let expected: BTreeMap<String, Entry> =
        serde_json::from_slice(&read_bytes(root, "inventory.json")?)?;
    ensure(expected == evidence::inventory(root)?, "inventory mismatch")?;
    let context: crate::campaign_context::Context =
        serde_json::from_slice(&read_bytes(root, "context.json")?)?;
    let source_root = source_root.canonicalize()?;
    context.validate(sha, run, attempt)?;
    let authorized = crate::source::trusted(&source_root, sha)?;
    ensure(
        authorized.tree == campaign.source_tree,
        "authorized source tree mismatch",
    )?;
    crate::source_state::verify(root, &authorized)
        .map_err(|e| format!("source inventory/integrity: {e}"))?;
    let controller = json(root, "controller.json")?;
    ensure(
        controller["source_sha"] == sha
            && controller["source_tree"] == authorized.tree
            && controller["binary_sha256"]
                .as_str()
                .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())),
        "controller source/build binding absent",
    )?;
    for (name, case) in &campaign.cases {
        crate::case_verification::required(root, name, case)?;
        // Diagnostics may add contradictions, but cannot select mandatory checks.
        if !case.checks.is_empty() {
            checks(root, &case.checks)?;
        }
    }
    for name in ["cleanup-containers.txt", "cleanup-networks.txt"] {
        ensure(
            fs::read(root.join(name))?.is_empty(),
            "cleanup absence observation missing or nonempty",
        )?;
    }
    ensure(
        !root.join("failure.txt").exists(),
        "controller did not complete campaign",
    )?;
    Ok(())
}

pub fn completion(campaign: &Campaign) -> Result<()> {
    ensure(
        campaign
            .cases
            .values()
            .all(|case| case.status == Status::Pass),
        "non-PASS mandatory subcase",
    )?;
    ensure(campaign.cleanup_verified, "cleanup unknown")?;
    ensure(
        campaign.cleanup_containers.is_empty() && campaign.cleanup_networks.is_empty(),
        "remaining owned resources",
    )
}

pub fn container_policy(value: &Value, expected: &Value) -> Result<()> {
    let v = &value[0];
    let h = &v["HostConfig"];
    let user = v["Config"]["User"].as_str().unwrap_or("");
    ensure(
        user.split_once(':').is_some_and(|(u, g)| {
            u.parse::<u32>().is_ok_and(|v| v > 0) && g.parse::<u32>().is_ok_and(|v| v > 0)
        }),
        "nonroot numeric identity required",
    )?;
    ensure(
        h["NetworkMode"] != "host" && h["SecurityOpt"] == serde_json::json!(["no-new-privileges"]),
        "forbidden namespace/security policy",
    )?;
    ensure(
        h["LogConfig"]
            == serde_json::json!({"Type":"local","Config":{"max-size":"1m","max-file":"1","compress":"false"}}),
        "lossless bounded logging policy not applied",
    )?;
    ensure(v["Image"] == policy::CONFIG, "unexpected image")?;
    for (name, required) in [
        ("Privileged", serde_json::json!(false)),
        ("ReadonlyRootfs", serde_json::json!(true)),
        ("PidsLimit", serde_json::json!(64)),
        ("Memory", serde_json::json!(536870912)),
        ("NanoCpus", serde_json::json!(1000000000)),
        ("PidMode", serde_json::json!("")),
        ("IpcMode", serde_json::json!("private")),
    ] {
        ensure(h[name] == required, &format!("unexpected {name}"))?;
    }
    ensure(
        h["CapDrop"] == serde_json::json!(["ALL"])
            && h["CapAdd"].as_array().is_none_or(|v| v.is_empty()),
        "capabilities added",
    )?;
    ensure(
        h["SecurityOpt"]
            .as_array()
            .is_some_and(|v| v.contains(&serde_json::json!("no-new-privileges")))
            && !h["SecurityOpt"]
                .as_array()
                .is_some_and(|v| v.contains(&serde_json::json!("seccomp=unconfined"))),
        "unsafe security options",
    )?;
    for name in ["Devices", "DeviceRequests"] {
        ensure(
            h[name].as_array().is_none_or(|v| v.is_empty()),
            "devices added",
        )?;
    }
    ensure(
        h["PortBindings"].as_object().is_none_or(|v| v.is_empty()),
        "published ports",
    )?;
    ensure(
        h["NetworkMode"] == expected["network"]
            && v["Config"]["User"] == expected["user"]
            && v["Config"]["WorkingDir"] == expected["cwd"]
            && v["Config"]["Cmd"] == expected["argv"]
            && v["Config"]["Env"] == expected["env"],
        "effective launch substitution",
    )?;
    let mut actual: Vec<_> = v["Mounts"].as_array().ok_or("mount observations absent")?.iter().map(|m| serde_json::json!({"source":m["Source"],"destination":m["Destination"],"rw":m["RW"],"type":m["Type"]})).collect();
    actual.sort_by_key(Value::to_string);
    let mut required = expected["mounts"]
        .as_array()
        .ok_or("expected mounts absent")?
        .clone();
    required.sort_by_key(Value::to_string);
    ensure(actual == required, "unexpected mount")
}
