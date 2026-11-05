//! Frozen per-role authorization. Launcher expectations cannot redefine this policy.
use crate::{
    Result,
    campaign_context::Context,
    ensure, policy,
    verification::{self, json as read},
};
use serde_json::json;
use std::{fs, path::Path};
pub fn verify(root: &Path, prefix: &str, c: &Context) -> Result<()> {
    let a = read(root, &format!("{prefix}-created.json"))?;
    let v = &a[0];
    let requested = read(root, &format!("{prefix}-launch.json"))?;
    let id = v["Id"].as_str().ok_or("creation identity absent")?;
    ensure(
        id.len() == 64
            && id.bytes().all(|b| b.is_ascii_hexdigit())
            && fs::read(root.join(format!("{prefix}-create.stdout")))?
                == format!("{id}\n").as_bytes(),
        "creation result does not bind inspected ID",
    )?;
    ensure(
        v["Config"]["Hostname"] == id[..12],
        "runtime hostname does not bind allocated container",
    )?;
    ensure(
        v["State"]["Running"] == false
            && v["State"]["Pid"] == 0
            && v["State"]["StartedAt"]
                .as_str()
                .is_some_and(|s| s.starts_with("0001-")),
        "creation observation is not pre-start",
    )?;
    let transport = read(root, &format!("{prefix}-create-transport.json"))?;
    ensure(
        transport["code"] == 0 && transport["guard_triggered"] == false,
        "creation transport indeterminate",
    )?;
    ensure(
        v["Name"] == format!("/{}", c.instance(prefix))
            && v["Config"]["Labels"][policy::LABEL] == c.campaign,
        "foreign container allocation",
    )?;
    let cwd = c.subject_root.to_string_lossy();
    let network_role = matches!(prefix, "listener" | "network-before" | "network-after");
    let network = if network_role {
        &c.network_name
    } else {
        "none"
    };
    let (mode, args, writable) = role(root, prefix, c)?;
    let argv: Vec<String> = ["python3", "-I", "-B", "/program/probe.py", mode]
        .into_iter()
        .map(str::to_owned)
        .chain(args)
        .collect();
    let mut mounts = vec![
        json!({"source":c.program_root,"destination":"/program","rw":false,"type":"bind"}),
        json!({"source":c.subject_root,"destination":cwd,"rw":writable,"type":"bind"}),
    ];
    if prefix == "control-denied" {
        mounts.push(json!({"source":c.owned_root.join("control"),"destination":"/control","rw":false,"type":"bind"}));
    }
    if prefix == "evidence-denied" {
        mounts.push(json!({"source":c.owned_root.join("sacrificial-evidence"),"destination":"/evidence-target","rw":false,"type":"bind"}));
    }
    let image = read(root, "image.json")?;
    ensure(
        image[0]["Id"] == policy::CONFIG,
        "image configuration mismatch",
    )?;
    let env = image[0]["Config"]["Env"]
        .as_array()
        .ok_or("image environment absent")?;
    for item in env {
        let key = item
            .as_str()
            .ok_or("environment syntax")?
            .split_once('=')
            .ok_or("environment syntax")?
            .0;
        ensure(
            matches!(
                key,
                "PATH"
                    | "LANG"
                    | "GPG_KEY"
                    | "PYTHON_VERSION"
                    | "PYTHON_SHA256"
                    | "HOME"
                    | "HOSTNAME"
            ),
            "unapproved environment variable",
        )?;
    }
    let authorized = json!({"network":network,"user":format!("{}:{}",c.uid,c.gid),"cwd":cwd,"argv":argv,"env":env,"mounts":mounts});
    // Match the independent authorization first, THEN the launcher's diagnostic request.
    verification::container_policy(&a, &authorized)?;
    verification::container_policy(&a, &requested)?;
    ensure(
        v["Config"]["Entrypoint"].is_null() || v["Config"]["Entrypoint"] == json!([]),
        "entrypoint substitution",
    )?;
    ensure(
        v["HostConfig"]["Tmpfs"]["/tmp"] == "rw,nosuid,nodev,noexec,size=16m,mode=1777",
        "unbounded scratch",
    )?;
    if network_role {
        network_resource(root, c)?;
    }
    let terminal = read(root, &format!("{prefix}-inspect.json"))?;
    ensure(
        terminal[0]["Id"] == id && terminal[0]["Config"]["Labels"][policy::LABEL] == c.campaign,
        "terminal container substitution",
    )
}
fn role(root: &Path, p: &str, c: &Context) -> Result<(&'static str, Vec<String>, bool)> {
    let subject = &c.subject_root;
    let path = |s: &str| subject.join(s).to_string_lossy().into_owned();
    Ok(match p {
        "policy" => ("policy", vec![], false),
        "simple" => ("simple", vec![], false),
        "output" => ("output", vec![], false),
        "governance" => ("governance", vec![path("authority.md")], true),
        "untracked" => ("untracked", vec![path("untracked-witness")], true),
        "writer-positive" | "program-positive" | "evidence-positive" => {
            ("write", vec![path(&format!("{p}-target"))], true)
        }
        "writer-denied" => ("write", vec![path("target")], false),
        "program-denied" => ("write", vec!["/program/probe.py".into()], false),
        "control-denied" => ("write", vec!["/control/target".into()], false),
        "evidence-denied" => ("write", vec!["/evidence-target/target".into()], false),
        "descendant-stream" | "descendant-writer" => (
            "parent",
            vec![
                path(p),
                if p.ends_with("stream") {
                    "stream".into()
                } else {
                    "writer".into()
                },
                c.campaign.clone(),
                c.instance(p),
            ],
            true,
        ),
        "listener" => ("listener", vec![path("network-control")], true),
        "network-before" | "network-after" | "network-denied" => {
            let listener = read(root, "listener-ready-inspect.json")?;
            let id = read(root, "listener-created.json")?;
            ensure(
                listener[0]["Id"] == id[0]["Id"] && listener[0]["State"]["Running"] == true,
                "listener readiness identity mismatch",
            )?;
            let ip = listener[0]["NetworkSettings"]["Networks"][&c.network_name]["IPAddress"]
                .as_str()
                .ok_or("listener address missing")?;
            let address: std::net::Ipv4Addr = ip.parse()?;
            ensure(
                !address.is_loopback() && !address.is_unspecified() && !address.is_multicast(),
                "not an owned local endpoint",
            )?;
            (
                "connect",
                vec![
                    ip.into(),
                    if p == "network-denied" {
                        "negative".into()
                    } else {
                        "positive".into()
                    },
                ],
                false,
            )
        }
        _ => return Err("unapproved probe role".into()),
    })
}
fn network_resource(root: &Path, c: &Context) -> Result<()> {
    let v = read(root, "network.json")?;
    let id = v[0]["Id"].as_str().ok_or("network ID absent")?;
    ensure(
        id.len() == 64
            && fs::read(root.join("network-id.txt"))? == format!("{id}\n").as_bytes()
            && v[0]["Name"] == c.network_name
            && v[0]["Labels"][policy::LABEL] == c.campaign
            && v[0]["Internal"] == true
            && v[0]["Driver"] == "bridge",
        "foreign network resource",
    )
}
