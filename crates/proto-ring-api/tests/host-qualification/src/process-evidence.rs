//! Observations of owned namespace processes and exact H06 attribution.
use crate::{
    Result, campaign_context::Context, ensure, evidence::Evidence, rendezvous::Ready,
    verification::json as read,
};
use serde_json::{Value, json};
use std::{fs, path::Path};
fn birth(pid: u64) -> Result<u64> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let tail = stat
        .rsplit_once(')')
        .ok_or("owned process stat malformed")?
        .1;
    Ok(tail
        .split_ascii_whitespace()
        .nth(19)
        .ok_or("owned process birth missing")?
        .parse()?)
}
pub fn capability(
    e: &Evidence,
    prefix: &str,
    c: &Context,
    child: &Ready,
    live: &Value,
    top: &[u8],
    target: &Path,
) -> Result<()> {
    let id = live[0]["Id"].as_str().ok_or("owned container ID missing")?;
    ensure(child.container_id == id, "child container binding differs")?;
    let parent = live[0]["State"]["Pid"]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or("principal host PID unavailable")?;
    let rows = std::str::from_utf8(top)?;
    let mut candidates = Vec::new();
    for row in rows.lines().skip(1) {
        let fields: Vec<_> = row.split_ascii_whitespace().collect();
        ensure(fields.len() >= 3, "owned runtime process table malformed")?;
        let pid: u64 = fields[0].parse()?;
        let ppid: u64 = fields[1].parse()?;
        if ppid == parent && row.contains("/program/probe.py child") {
            candidates.push(pid);
        }
    }
    ensure(
        candidates.len() == 1,
        "unique live owned child not observed",
    )?;
    let host_child = candidates[0];
    let parent_birth = birth(parent)?;
    let child_birth = birth(host_child)?;
    let status = fs::read_to_string(format!("/proc/{host_child}/status"))?;
    let nspid: u32 = status
        .lines()
        .find_map(|line| line.strip_prefix("NSpid:"))
        .ok_or("namespace PID unavailable without elevation")?
        .split_ascii_whitespace()
        .last()
        .ok_or("namespace PID absent")?
        .parse()?;
    let ppid: u64 = status
        .lines()
        .find_map(|line| line.strip_prefix("PPid:"))
        .ok_or("host PPID absent")?
        .trim()
        .parse()?;
    let parent_ns = fs::read_link(format!("/proc/{parent}/ns/pid"))?;
    let child_ns = fs::read_link(format!("/proc/{host_child}/ns/pid"))?;
    ensure(
        parent_ns == child_ns
            && child_ns.to_str() == Some(child.pid_namespace.as_str())
            && nspid == child.pid
            && ppid == parent,
        "namespace/host process relationship differs",
    )?;
    let fd = child.write_fd.ok_or("child write descriptor absent")?;
    let fd_metadata = fs::metadata(format!("/proc/{host_child}/fd/{fd}"))?;
    let fd_identity = crate::filesystem_identity::Identity::metadata(&fd_metadata)?;
    let target_identity = crate::filesystem_identity::Identity::observe(target)?;
    fd_identity.validate("regular")?;
    target_identity.validate("regular")?;
    ensure(
        fd_identity == target_identity,
        "child capability points elsewhere",
    )?;
    ensure(
        birth(parent)? == parent_birth && birth(host_child)? == child_birth,
        "owned process birth changed",
    )?;
    e.json(&format!("{prefix}-capability.json"),&json!({"container_id":id,"campaign":c.campaign,"host_parent":parent,"host_child":host_child,"namespace_child":nspid,"host_ppid":ppid,"pid_namespace":child_ns,"parent_birth":parent_birth,"child_birth":child_birth,"fd_identity":fd_identity,"target_identity":target_identity}))
}
pub fn verify(root: &Path, prefix: &str, c: &Context) -> Result<()> {
    let created = read(root, &format!("{prefix}-created.json"))?;
    let id = created[0]["Id"].as_str().ok_or("container ID absent")?;
    let live = read(root, &format!("{prefix}-live.json"))?;
    let final_state = read(root, &format!("{prefix}-inspect.json"))?;
    let r = read(root, &format!("{prefix}-rendezvous.json"))?;
    let cap = read(root, &format!("{prefix}-capability.json"))?;
    let parent: crate::rendezvous::Ready = serde_json::from_value(r["parent"].clone())?;
    let child: crate::rendezvous::Ready = serde_json::from_value(r["child"].clone())?;
    let instance = c.instance(prefix);
    parent.require(&crate::rendezvous::Binding {
        campaign: &c.campaign,
        instance: &instance,
        container_id: id,
        kind: "parent",
        uid: c.uid,
    })?;
    child.require(&crate::rendezvous::Binding {
        campaign: &c.campaign,
        instance: &instance,
        container_id: id,
        kind: if prefix == "descendant-stream" {
            "stream"
        } else {
            "writer"
        },
        uid: c.uid,
    })?;
    crate::rendezvous::namespace_link(
        cap["pid_namespace"]
            .as_str()
            .ok_or("capability namespace missing")?,
        "pid",
    )?;
    let descriptor: crate::filesystem_identity::Identity =
        serde_json::from_value(cap["fd_identity"].clone())?;
    let target: crate::filesystem_identity::Identity =
        serde_json::from_value(cap["target_identity"].clone())?;
    descriptor.validate("regular")?;
    target.validate("regular")?;
    ensure(
        live[0]["Id"] == id
            && final_state[0]["Id"] == id
            && r["container_id"] == id
            && cap["container_id"] == id
            && r["child"]["container_id"] == id
            && r["parent"]["container_id"] == id,
        "container attribution mismatch",
    )?;
    ensure(
        r["campaign"] == c.campaign
            && cap["campaign"] == c.campaign
            && r["child"]["campaign"] == c.campaign
            && r["parent"]["campaign"] == c.campaign
            && r["instance"] == c.instance(prefix)
            && r["child"]["instance"] == c.instance(prefix)
            && r["parent"]["instance"] == c.instance(prefix),
        "campaign/instance attribution mismatch",
    )?;
    ensure(
        r["events"]
            == json!([
                "created",
                "started",
                "parent_ready",
                "child_ready",
                "runtime_live",
                "effect_capability",
                "release_parent",
                "principal_exited",
                "perimeter_closed",
                "streams_collected"
            ]),
        "causal event sequence mismatch",
    )?;
    ensure(
        live[0]["State"]["Running"] == true
            && live[0]["State"]["Pid"].as_u64().is_some_and(|p| p > 0)
            && live[0]["State"]["StartedAt"] == final_state[0]["State"]["StartedAt"]
            && final_state[0]["State"]["Running"] == false
            && final_state[0]["State"]["Pid"] == 0,
        "principal exit without same perimeter closure",
    )?;
    ensure(
        cap["host_parent"] == live[0]["State"]["Pid"]
            && cap["host_ppid"] == cap["host_parent"]
            && cap["host_child"].as_u64().is_some_and(|p| p > 0)
            && cap["parent_birth"].as_u64().is_some_and(|p| p > 0)
            && cap["child_birth"].as_u64().is_some_and(|p| p > 0),
        "live process/birth witness absent",
    )?;
    ensure(
        r["parent"]["kind"] == "parent"
            && r["parent"]["write_fd"].is_null()
            && r["child"]["kind"]
                == if prefix == "descendant-stream" {
                    "stream"
                } else {
                    "writer"
                },
        "process role mismatch",
    )?;
    ensure(
        cap["namespace_child"] == r["child"]["pid"]
            && r["child"]["pid"].as_u64().is_some_and(|p| p > 1)
            && r["child"]["ppid"] == 1
            && r["parent"]["pid"] == 1
            && r["parent"]["ppid"] == 0
            && r["child"]["write_fd"].as_u64().is_some_and(|p| p >= 3),
        "host/namespace PID capability mismatch",
    )?;
    ensure(
        r["child"]["hostname"] == created[0]["Config"]["Hostname"]
            && r["parent"]["hostname"] == created[0]["Config"]["Hostname"]
            && r["parent"]["pid_namespace"] == cap["pid_namespace"]
            && r["child"]["pid_namespace"] == cap["pid_namespace"]
            && cap["fd_identity"].is_object()
            && cap["fd_identity"] == cap["target_identity"],
        "capability namespace/target mismatch",
    )?;
    ensure(
        r["writer_before_parent_exit"] == json!(b"before\nchild-started\n".to_vec())
            && r["principal_release_requested"] == true
            && r["descendant_release_requested"] == false
            && r["closure"] == "namespace_terminated_by_principal_exit",
        "write capability or closure mechanism not established",
    )?;
    let top = fs::read(root.join(format!("{prefix}-top.txt")))?;
    let rows = std::str::from_utf8(&top)?;
    let mut child_row = false;
    let mut parent_row = false;
    for line in rows.lines().skip(1) {
        let f: Vec<_> = line.split_ascii_whitespace().collect();
        ensure(f.len() >= 3, "runtime row malformed")?;
        let pid: u64 = f[0].parse()?;
        let ppid: u64 = f[1].parse()?;
        child_row |= cap["host_child"] == pid
            && cap["host_parent"] == ppid
            && line.contains("/program/probe.py child");
        parent_row |= cap["host_parent"] == pid && line.contains("/program/probe.py parent");
    }
    ensure(
        child_row && parent_row,
        "runtime process table contradicts namespace witnesses",
    )
}
