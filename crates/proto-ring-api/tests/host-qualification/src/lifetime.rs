//! Explicit rendezvous, live runtime identity, capability and namespace closure.
use crate::{
    Result,
    campaign_context::Context,
    docker::Docker,
    ensure,
    evidence::Check,
    rendezvous::{self, Binding},
    subject::Subject,
    transport,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    thread,
    time::{Duration, Instant},
};
pub fn ready(path: &Path) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(25);
    while !path.is_file() {
        ensure(
            Instant::now() < deadline,
            "rendezvous unavailable; no readiness inferred",
        )?;
        thread::sleep(Duration::from_millis(25));
    }
    Ok(())
}
pub fn descendants(d: &mut Docker, s: &Subject, name: &str, kind: &str) -> Result<Vec<Check>> {
    let c: Context = serde_json::from_slice(&fs::read(d.evidence.root.join("context.json"))?)?;
    let dir = s.root.join(name);
    fs::create_dir(&dir)?;
    fs::write(dir.join("writer"), b"before\n")?;
    let instance = c.instance(name);
    let id = d.create(
        name,
        "parent",
        &[
            dir.to_string_lossy().into_owned(),
            kind.into(),
            c.campaign.clone(),
            instance.clone(),
        ],
        &s.root,
        &[s.mount(true)],
        None,
    )?;
    let mut events = vec!["created"];
    fs::write(
        dir.join("launch-binding.json"),
        serde_json::to_vec(&json!({"campaign":c.campaign,"instance":instance,"container_id":id}))?,
    )?;
    d.start(name, &id)?;
    events.push("started");
    let parent = rendezvous::wait(
        &dir.join(format!("{instance}-parent-ready.json")),
        &Binding {
            campaign: &c.campaign,
            instance: &instance,
            container_id: &id,
            kind: "parent",
            uid: c.uid,
        },
    )?;
    events.push("parent_ready");
    let child = rendezvous::wait(
        &dir.join(format!("{instance}-child-ready.json")),
        &Binding {
            campaign: &c.campaign,
            instance: &instance,
            container_id: &id,
            kind,
            uid: c.uid,
        },
    )?;
    events.push("child_ready");
    let live = d.command("descendants-live", &["inspect", &id])?;
    d.evidence.bytes(&format!("{name}-live.json"), &live)?;
    let live: Value = serde_json::from_slice(&live)?;
    ensure(
        live[0]["Id"] == id && live[0]["State"]["Running"] == true,
        "same live container unavailable",
    )?;
    events.push("runtime_live");
    let top = d.command("descendants-top", &["top", &id, "-eo", "pid,ppid,args"])?;
    d.evidence.bytes(&format!("{name}-top.txt"), &top)?;
    crate::process_evidence::capability(
        &d.evidence,
        name,
        &c,
        &child,
        &live,
        &top,
        &dir.join("writer"),
    )?;
    events.push("effect_capability");
    let writer = fs::read(dir.join("writer"))?;
    fs::write(dir.join("release-parent"), b"release\n")?;
    events.push("release_parent");
    d.finish(name, &id)?;
    events.extend(["principal_exited", "perimeter_closed", "streams_collected"]);
    d.evidence.json(&format!("{name}-rendezvous.json"),&json!({"campaign":c.campaign,"instance":instance,"container_id":id,"parent":parent,"child":child,"writer_before_parent_exit":writer,"principal_release_requested":true,"descendant_release_requested":false,"closure":"namespace_terminated_by_principal_exit","events":events}))?;
    // Namespace destruction is containment, not normal descendant/validator success.
    let _ = s.observation()?;
    let stdout = if kind == "stream" {
        b"child-stream\nparent-exit\n".as_slice()
    } else {
        b"parent-exit\n".as_slice()
    };
    Ok(vec![
        transport::stream_check(name, 0, stdout, b""),
        Docker::closed(name, &id),
    ])
}
pub fn network(d: &mut Docker, s: &Subject) -> Result<Vec<(String, Vec<Check>)>> {
    let name = format!("{}-network", d.campaign);
    let label = format!("{}={}", crate::policy::LABEL, d.campaign);
    let created = d.command(
        "network-create",
        &["network", "create", "--internal", "--label", &label, &name],
    )?;
    let id = std::str::from_utf8(&created)?
        .trim_end_matches('\n')
        .to_owned();
    d.network = Some((name.clone(), id.clone()));
    d.evidence.bytes("network-id.txt", &created)?;
    let bytes = d.command("network-inspect", &["network", "inspect", &id])?;
    d.evidence.bytes("network.json", &bytes)?;
    let v: Value = serde_json::from_slice(&bytes)?;
    ensure(
        v[0]["Internal"] == true && v[0]["Id"] == id && v[0]["Driver"] == "bridge",
        "network policy not applied",
    )?;
    let dir = s.root.join("network-control");
    fs::create_dir(&dir)?;
    let listener = d.create(
        "listener",
        "listener",
        &[dir.to_string_lossy().into_owned()],
        &s.root,
        &[s.mount(true)],
        Some(&name),
    )?;
    d.start("listener", &listener)?;
    ready(&dir.join("listener-ready"))?;
    let bytes = d.command("listener-inspect", &["inspect", &listener])?;
    d.evidence.bytes("listener-ready-inspect.json", &bytes)?;
    let v: Value = serde_json::from_slice(&bytes)?;
    let ip = v[0]["NetworkSettings"]["Networks"][&name]["IPAddress"]
        .as_str()
        .ok_or("listener address unavailable")?
        .to_owned();
    let address: std::net::Ipv4Addr = ip.parse()?;
    ensure(
        !address.is_loopback() && !address.is_unspecified() && !address.is_multicast(),
        "invalid controlled endpoint",
    )?;
    let mut cases = Vec::new();
    for (suffix, negative, network) in [
        ("before", false, Some(name.as_str())),
        ("denied", true, None),
        ("after", false, Some(name.as_str())),
    ] {
        let prefix = format!("network-{suffix}");
        let cid = d.run(
            &prefix,
            "connect",
            &[
                ip.clone(),
                if negative {
                    "negative".into()
                } else {
                    "positive".into()
                },
            ],
            &s.root,
            &[s.mount(false)],
            network,
        )?;
        let check = if negative {
            transport::stream_check(&prefix, 42, b"", b"started\nENETUNREACH\n")
        } else {
            transport::stream_check(&prefix, 0, b"api03-pong\n", b"")
        };
        crate::verification::checks(&d.evidence.root, std::slice::from_ref(&check))?;
        cases.push((
            format!("H08/{prefix}"),
            vec![check, Docker::closed(&prefix, &cid)],
        ));
    }
    d.finish("listener", &listener)?;
    Ok(cases)
}
