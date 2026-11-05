//! Ownership-scoped Docker resources; no product sandbox abstraction.
use crate::{
    Result, ensure,
    evidence::{Check, Evidence},
    policy,
    transport::{capture, success},
    verification,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

pub struct Docker {
    pub evidence: Evidence,
    pub campaign: String,
    pub user: String,
    pub env: Value,
    pub program: PathBuf,
    pub containers: Vec<(String, String)>,
    pub network: Option<(String, String)>,
    sequence: usize,
}
impl Docker {
    pub fn new(evidence: Evidence, campaign: String, program: PathBuf) -> Result<Self> {
        let uid = success(capture(&evidence, "uid", Command::new("id").arg("-u"))?)?;
        let gid = success(capture(&evidence, "gid", Command::new("id").arg("-g"))?)?;
        let uid = std::str::from_utf8(&uid)?.trim();
        let gid = std::str::from_utf8(&gid)?.trim();
        ensure(
            uid != "0" && gid != "0" && uid.parse::<u32>().is_ok() && gid.parse::<u32>().is_ok(),
            "nonroot runner identity unavailable",
        )?;
        Ok(Self {
            evidence,
            campaign,
            user: format!("{uid}:{gid}"),
            env: Value::Null,
            program,
            containers: Vec::new(),
            network: None,
            sequence: 0,
        })
    }
    pub fn command(&mut self, prefix: &str, arguments: &[&str]) -> Result<Vec<u8>> {
        self.sequence += 1;
        success(capture(
            &self.evidence,
            &format!("d{}-{prefix}", self.sequence),
            Command::new("docker").args(arguments),
        )?)
    }
    pub fn prepare_runtime(&mut self) -> Result<()> {
        crate::environment_evidence::capture_runtime(self)?;
        crate::environment_evidence::runtime(&self.evidence.root)
    }
    pub fn prepare_image(&mut self) -> Result<()> {
        self.command(
            "pull",
            &["pull", "--platform", "linux/amd64", policy::IMAGE],
        )?;
        let bytes = self.command("image", &["image", "inspect", policy::IMAGE])?;
        self.evidence.bytes("image.json", &bytes)?;
        let image: Value = serde_json::from_slice(&bytes)?;
        ensure(
            image[0]["Id"] == policy::CONFIG
                && image[0]["Architecture"] == "amd64"
                && image[0]["Os"] == "linux",
            "candidate image identity unavailable",
        )?;
        ensure(
            image[0]["RepoDigests"].as_array().is_some_and(|v| {
                v.iter().any(|s| {
                    s.as_str().is_some_and(|s| {
                        s.ends_with(policy::IMAGE.split('@').nth(1).unwrap_or("invalid"))
                    })
                })
            }),
            "repository digest mismatch",
        )?;
        self.env = image[0]["Config"]["Env"].clone();
        ensure(self.env.is_array(), "image environment missing")
    }
    pub fn create(
        &mut self,
        name: &str,
        mode: &str,
        args: &[String],
        subject: &Path,
        binds: &[(PathBuf, String, bool)],
        network: Option<&str>,
    ) -> Result<String> {
        let full = format!("{}-{name}", self.campaign);
        ensure(
            !self.containers.iter().any(|(n, _)| n == &full),
            "container name reused",
        )?;
        // Reserve the owned name before create: even uncertain create enters cleanup.
        self.containers.push((full.clone(), String::new()));
        let network = network.unwrap_or("none");
        let argv: Vec<String> = ["python3", "-I", "-B", "/program/probe.py", mode]
            .into_iter()
            .map(str::to_owned)
            .chain(args.iter().cloned())
            .collect();
        let mut command = Command::new("docker");
        command.args([
            "create",
            "--name",
            &full,
            "--label",
            &format!("{}={}", policy::LABEL, self.campaign),
            "--platform",
            "linux/amd64",
            "--network",
            network,
            "--read-only",
            "--cap-drop",
            "ALL",
            "--security-opt",
            "no-new-privileges",
            "--user",
            &self.user,
            "--pids-limit",
            "64",
            "--memory",
            "512m",
            "--cpus",
            "1",
            "--ipc",
            "private",
            "--log-driver",
            "local",
            "--log-opt",
            "max-size=1m",
            "--log-opt",
            "max-file=1",
            "--tmpfs",
            "/tmp:rw,nosuid,nodev,noexec,size=16m,mode=1777",
            "--workdir",
        ]);
        command.arg(subject);
        let mut mounts = vec![(self.program.clone(), "/program".to_string(), false)];
        mounts.extend_from_slice(binds);
        for (src, dst, rw) in &mounts {
            ensure(
                src.is_absolute() && !src.to_string_lossy().contains(',') && !dst.contains(','),
                "unsafe owned mount path",
            )?;
            command.args([
                "--mount",
                &format!(
                    "type=bind,src={},dst={dst}{}",
                    src.display(),
                    if *rw { "" } else { ",readonly" }
                ),
            ]);
        }
        command.arg(policy::IMAGE).args(&argv);
        let output = success(capture(
            &self.evidence,
            &format!("{name}-create"),
            &mut command,
        )?)?;
        let id = std::str::from_utf8(&output)?.trim().to_string();
        ensure(
            id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit()),
            "container ID not captured",
        )?;
        self.containers
            .last_mut()
            .ok_or("container reservation absent")?
            .1 = id.clone();
        let inspected = self.command("inspect-created", &["inspect", &id])?;
        self.evidence
            .bytes(&format!("{name}-created.json"), &inspected)?;
        let expected = json!({"network":network,"user":self.user,"cwd":subject,"argv":argv,"env":self.env,"mounts":mounts.iter().map(|(src,dst,rw)| json!({"source":src,"destination":dst,"rw":rw,"type":"bind"})).collect::<Vec<_>>()});
        self.evidence
            .json(&format!("{name}-launch.json"), &expected)?;
        let value: Value = serde_json::from_slice(&inspected)?;
        verification::container_policy(&value, &expected)?;
        ensure(
            value[0]["HostConfig"]["Tmpfs"]["/tmp"] == "rw,nosuid,nodev,noexec,size=16m,mode=1777",
            "scratch policy not applied",
        )?;
        Ok(id)
    }
    pub fn start(&mut self, name: &str, id: &str) -> Result<()> {
        success(capture(
            &self.evidence,
            &format!("{name}-start"),
            Command::new("docker").args(["start", id]),
        )?)?;
        Ok(())
    }
    pub fn finish(&mut self, name: &str, id: &str) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let bytes = self.command("inspect-wait", &["inspect", id])?;
            let value: Value = serde_json::from_slice(&bytes)?;
            ensure(value[0]["Id"] == id, "container identity replaced")?;
            if value[0]["State"]["Running"] == false {
                self.evidence
                    .bytes(&format!("{name}-inspect.json"), &bytes)?;
                break;
            }
            ensure(
                Instant::now() < deadline,
                "experiment rendezvous guard triggered; no retry",
            )?;
            thread::sleep(Duration::from_millis(50));
        }
        success(capture(
            &self.evidence,
            &format!("{name}-wait"),
            Command::new("docker").args(["wait", id]),
        )?)?;
        success(capture(
            &self.evidence,
            &format!("{name}-logs"),
            Command::new("docker").args(["logs", id]),
        )?)?;
        Ok(())
    }
    pub fn run(
        &mut self,
        name: &str,
        mode: &str,
        args: &[String],
        subject: &Path,
        binds: &[(PathBuf, String, bool)],
        network: Option<&str>,
    ) -> Result<String> {
        let id = self.create(name, mode, args, subject, binds, network)?;
        self.start(name, &id)?;
        self.finish(name, &id)?;
        Ok(id)
    }
    pub fn closed(name: &str, id: &str) -> Check {
        Check::Closed {
            inspect: format!("{name}-inspect.json"),
            container_id: id.into(),
        }
    }
    pub fn cleanup(&mut self) -> Result<(Vec<String>, Vec<String>)> {
        let resources = self.containers.clone();
        for (name, id) in resources {
            let inspected = self.command("cleanup-inspect", &["inspect", &name]);
            if let Ok(bytes) = inspected {
                let v: Value = serde_json::from_slice(&bytes)?;
                ensure(
                    v[0]["Config"]["Labels"][policy::LABEL] == self.campaign
                        && (id.is_empty() || v[0]["Id"] == id),
                    "cleanup ownership mismatch",
                )?;
                self.command("remove", &["rm", "-f", &name])?;
            }
        }
        if let Some((name, id)) = self.network.clone() {
            let bytes = self.command("cleanup-net-inspect", &["network", "inspect", &name])?;
            let v: Value = serde_json::from_slice(&bytes)?;
            ensure(
                v[0]["Labels"][policy::LABEL] == self.campaign && v[0]["Id"] == id,
                "network cleanup ownership mismatch",
            )?;
            self.command("remove-network", &["network", "rm", &id])?;
        }
        let filter = format!("label={}={}", policy::LABEL, self.campaign);
        let containers = self.command(
            "cleanup-container-list",
            &["ps", "-aq", "--filter", &filter],
        )?;
        let networks = self.command(
            "cleanup-network-list",
            &["network", "ls", "-q", "--filter", &filter],
        )?;
        self.evidence.bytes("cleanup-containers.txt", &containers)?;
        self.evidence.bytes("cleanup-networks.txt", &networks)?;
        Ok((
            String::from_utf8(containers)?
                .lines()
                .map(str::to_owned)
                .collect(),
            String::from_utf8(networks)?
                .lines()
                .map(str::to_owned)
                .collect(),
        ))
    }
}
