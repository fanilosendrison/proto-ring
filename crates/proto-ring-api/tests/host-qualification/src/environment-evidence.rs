//! Mandatory environment and image-interpreter observations, independent of diagnostics.
use crate::{
    Result,
    campaign_context::Context,
    docker::Docker,
    ensure,
    evidence::Evidence,
    policy, transport,
    verification::{self, json as read},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Kernel {
    system: String,
    release: String,
    architecture: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Version {
    client: String,
    server: String,
    api: String,
    os: String,
    architecture: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Runtime {
    server: String,
    kernel: String,
    os: String,
    architecture: String,
    cgroup_driver: String,
    cgroup_version: String,
    security: Vec<String>,
    default_runtime: String,
    logging_drivers: Vec<String>,
    // Docker's version-specific auxiliary runtime descriptors remain provider data.
    runtimes: BTreeMap<String, serde_json::Value>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Process {
    uid: u32,
    gid: u32,
    pid: u32,
    capabilities_zero: bool,
    no_new_privileges: bool,
    seccomp: u8,
    rootfs_erofs: bool,
    credential_names_absent: bool,
    environment_names: Vec<String>,
    cwd: String,
    interpreter: String,
    interpreter_sha256: String,
    python: String,
    namespaces: BTreeMap<String, String>,
    cgroup: String,
    mountinfo: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Interpreter {
    container_id: String,
    image: String,
    path: String,
    sha256: String,
    bytes: usize,
}
fn text(value: &str) -> bool {
    !value.is_empty() && value.len() <= 4096 && !value.contains('\0')
}
fn version(value: &str) -> bool {
    text(value)
        && value
            .split('.')
            .next()
            .is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
}
fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn capture_kernel(e: &Evidence) -> Result<()> {
    let mut values = Vec::new();
    for (name, arg) in [("system", "-s"), ("release", "-r"), ("architecture", "-m")] {
        let bytes = transport::success(transport::capture(
            e,
            &format!("kernel-{name}"),
            Command::new("uname").arg(arg),
        )?)?;
        values.push(
            std::str::from_utf8(&bytes)?
                .strip_suffix('\n')
                .ok_or("kernel framing absent")?
                .to_owned(),
        );
    }
    e.json(
        "kernel.json",
        &Kernel {
            system: values[0].clone(),
            release: values[1].clone(),
            architecture: values[2].clone(),
        },
    )
}
pub fn capture_runtime(d: &mut Docker) -> Result<()> {
    let version=d.command("version",&["version","--format",r#"{"client":{{json .Client.Version}},"server":{{json .Server.Version}},"api":{{json .Server.APIVersion}},"os":{{json .Server.Os}},"architecture":{{json .Server.Arch}}}"#])?;
    let _: Version = serde_json::from_slice(&version)?;
    d.evidence.bytes("runtime-version.json", &version)?;
    let info=d.command("runtime",&["info","--format",r#"{"server":{{json .ServerVersion}},"kernel":{{json .KernelVersion}},"os":{{json .OperatingSystem}},"architecture":{{json .Architecture}},"cgroup_driver":{{json .CgroupDriver}},"cgroup_version":{{json .CgroupVersion}},"security":{{json .SecurityOptions}},"default_runtime":{{json .DefaultRuntime}},"logging_drivers":{{json .Plugins.Log}},"runtimes":{{json .Runtimes}}}"#])?;
    let _: Runtime = serde_json::from_slice(&info)?;
    d.evidence.bytes("runtime-info.json", &info)?;
    d.evidence.bytes("runtime.txt", &info)
}
pub fn capture_interpreter(d: &mut Docker, id: &str) -> Result<()> {
    let target = d.evidence.root.join("image-interpreter.bin");
    ensure(!target.exists(), "interpreter evidence replacement refused")?;
    let source = format!("{id}:/usr/local/bin/python3");
    let destination = target
        .to_str()
        .ok_or("interpreter evidence path encoding")?;
    d.command("interpreter-copy", &["cp", "-L", &source, destination])?;
    let metadata = fs::symlink_metadata(&target)?;
    ensure(
        metadata.is_file() && metadata.len() > 0 && metadata.len() <= 4 * 1024 * 1024,
        "image interpreter copy invalid/oversized",
    )?;
    let bytes = fs::read(target)?;
    d.evidence.json(
        "image-interpreter.json",
        &Interpreter {
            container_id: id.into(),
            image: policy::CONFIG.into(),
            path: "/usr/local/bin/python3".into(),
            sha256: policy::sha256(&bytes),
            bytes: bytes.len(),
        },
    )
}
pub fn kernel(root: &Path) -> Result<()> {
    let k: Kernel = serde_json::from_slice(&verification::read_bytes(root, "kernel.json")?)?;
    ensure(
        k.system == "Linux" && k.architecture == "x86_64" && version(&k.release),
        "kernel observation invalid",
    )
}
pub fn runtime(root: &Path) -> Result<()> {
    kernel(root)?;
    let k: Kernel = serde_json::from_slice(&verification::read_bytes(root, "kernel.json")?)?;
    let v: Version =
        serde_json::from_slice(&verification::read_bytes(root, "runtime-version.json")?)?;
    let r: Runtime = serde_json::from_slice(&verification::read_bytes(root, "runtime-info.json")?)?;
    ensure(
        verification::read_bytes(root, "runtime.txt")?
            == verification::read_bytes(root, "runtime-info.json")?,
        "runtime diagnostic contradicts structured observation",
    )?;
    ensure(
        version(&v.client)
            && version(&v.server)
            && version(&v.api)
            && v.os == "linux"
            && v.architecture == "amd64",
        "runtime version/platform observation invalid",
    )?;
    ensure(
        r.server == v.server
            && r.kernel == k.release
            && matches!(r.architecture.as_str(), "x86_64" | "amd64")
            && text(&r.os)
            && matches!(r.cgroup_driver.as_str(), "systemd" | "cgroupfs")
            && matches!(r.cgroup_version.as_str(), "1" | "2"),
        "runtime/kernel/cgroup correspondence invalid",
    )?;
    ensure(
        r.security.iter().any(|s| s.starts_with("name=seccomp")) && text(&r.default_runtime),
        "runtime security inventory incomplete",
    )?;
    ensure(
        r.logging_drivers.iter().any(|name| name == "local"),
        "required Docker local logging facility unavailable",
    )?;
    let selected = r
        .runtimes
        .get(&r.default_runtime)
        .ok_or("default runtime not inventoried")?;
    ensure(
        selected["path"].as_str().is_some_and(text)
            || selected["runtimeType"].as_str().is_some_and(text),
        "runtime descriptor absent",
    )
}
pub fn process(root: &Path, c: &Context) -> Result<()> {
    let p: Process =
        serde_json::from_slice(&verification::read_bytes(root, "policy-logs.stdout")?)?;
    ensure(
        p.pid == 1
            && p.uid == c.uid
            && p.gid == c.gid
            && p.seccomp == 2
            && p.capabilities_zero
            && p.no_new_privileges
            && p.rootfs_erofs
            && p.credential_names_absent,
        "effective process policy contradicted",
    )?;
    ensure(
        p.cwd == c.subject_root.to_string_lossy()
            && p.interpreter == "/usr/local/bin/python3"
            && hash(&p.interpreter_sha256),
        "process cwd/interpreter observation invalid",
    )?;
    let image = read(root, "image.json")?;
    let env = image[0]["Config"]["Env"]
        .as_array()
        .ok_or("image environment absent")?;
    let mut expected = BTreeSet::new();
    let mut python = None;
    for e in env {
        let (k, v) = e
            .as_str()
            .ok_or("image environment invalid")?
            .split_once('=')
            .ok_or("image environment syntax")?;
        expected.insert(k.to_owned());
        if k == "PYTHON_VERSION" {
            python = Some(v);
        }
    }
    expected.extend(["HOME".into(), "HOSTNAME".into()]);
    let names: BTreeSet<_> = p.environment_names.iter().cloned().collect();
    ensure(
        names == expected && names.len() == p.environment_names.len(),
        "effective environment inventory contradicts image/defaults",
    )?;
    ensure(
        p.python.split_ascii_whitespace().next() == python && python.is_some(),
        "interpreter version correspondence missing",
    )?;
    ensure(
        p.namespaces
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            == ["pid", "mnt", "net", "ipc", "user"].into_iter().collect(),
        "namespace inventory incomplete",
    )?;
    for (kind, link) in &p.namespaces {
        crate::rendezvous::namespace_link(link, kind)?;
    }
    ensure(
        !p.cgroup.is_empty()
            && p.cgroup.lines().all(|line| {
                let f: Vec<_> = line.splitn(3, ':').collect();
                f.len() == 3 && f[0].parse::<u32>().is_ok() && f[2].starts_with('/')
            }),
        "process cgroup observation missing/invalid",
    )?;
    mounts(&p.mountinfo, &c.subject_root)?;
    let i: Interpreter =
        serde_json::from_slice(&verification::read_bytes(root, "image-interpreter.json")?)?;
    let created = read(root, "policy-created.json")?;
    let bytes = verification::read_bytes(root, "image-interpreter.bin")?;
    ensure(
        i.container_id == created[0]["Id"]
            && i.image == policy::CONFIG
            && i.path == p.interpreter
            && i.bytes > 0
            && i.bytes == bytes.len()
            && hash(&i.sha256)
            && i.sha256 == policy::sha256(&bytes)
            && i.sha256 == p.interpreter_sha256,
        "image/process interpreter identity contradicted",
    )
}
fn mounts(value: &str, subject: &Path) -> Result<()> {
    ensure(
        !value.is_empty() && value.len() <= policy::STREAM_LIMIT,
        "mount observations missing/oversized",
    )?;
    let mut points = BTreeMap::new();
    for line in value.lines() {
        let (before, after) = line.split_once(" - ").ok_or("mountinfo framing invalid")?;
        let fields: Vec<_> = before.split_ascii_whitespace().collect();
        ensure(
            fields.len() >= 6
                && after.split_ascii_whitespace().count() >= 3
                && fields[0].parse::<u64>().is_ok()
                && fields[1].parse::<u64>().is_ok(),
            "mountinfo required fields invalid",
        )?;
        ensure(
            points.insert(fields[4], fields[5]).is_none(),
            "ambiguous mount observation",
        )?;
    }
    for path in [
        "/",
        "/program",
        subject.to_str().ok_or("subject mount encoding")?,
    ] {
        ensure(
            points
                .get(path)
                .is_some_and(|flags| flags.split(',').any(|f| f == "ro")),
            "protected process mount not observed read-only",
        )?;
    }
    Ok(())
}
