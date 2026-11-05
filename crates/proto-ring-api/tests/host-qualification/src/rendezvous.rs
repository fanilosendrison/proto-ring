//! Bounded no-follow reads of atomically published experiment rendezvous records.
use crate::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Read},
    path::Path,
    thread,
    time::{Duration, Instant},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ready {
    pub campaign: String,
    pub instance: String,
    pub container_id: String,
    pub hostname: String,
    pub pid_namespace: String,
    pub pid: u32,
    pub ppid: u32,
    pub write_fd: Option<u32>,
    pub kind: String,
}
pub struct Binding<'a> {
    pub campaign: &'a str,
    pub instance: &'a str,
    pub container_id: &'a str,
    pub kind: &'a str,
    pub uid: u32,
}
pub fn read(path: &Path, b: &Binding<'_>) -> Result<Option<Ready>> {
    ensure(
        b.container_id.len() == 64 && b.container_id.bytes().all(|v| v.is_ascii_hexdigit()),
        "invalid rendezvous container binding",
    )?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = match options.open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let metadata = file.metadata()?;
    ensure(
        metadata.is_file() && metadata.len() <= 4096,
        "rendezvous not a bounded regular record",
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ensure(metadata.uid() == b.uid, "rendezvous owner mismatch")?;
    }
    let mut data = Vec::new();
    file.take(4097).read_to_end(&mut data)?;
    ensure(data.len() <= 4096, "oversized rendezvous")?;
    let named = fs::symlink_metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ensure(
            named.is_file() && named.dev() == metadata.dev() && named.ino() == metadata.ino(),
            "rendezvous binding changed",
        )?;
    }
    let record: Ready = serde_json::from_slice(&data)?;
    record.require(b)?;
    Ok(Some(record))
}
impl Ready {
    pub fn require(&self, b: &Binding<'_>) -> Result<()> {
        ensure(
            b.container_id.len() == 64 && b.container_id.bytes().all(|v| v.is_ascii_hexdigit()),
            "invalid rendezvous container binding",
        )?;
        ensure(
            self.campaign == b.campaign
                && self.instance == b.instance
                && self.container_id == b.container_id
                && self.kind == b.kind
                && self.hostname == b.container_id[..12],
            "rendezvous attribution mismatch",
        )?;
        namespace_link(&self.pid_namespace, "pid")?;
        if b.kind == "parent" {
            ensure(
                self.pid == 1 && self.ppid == 0 && self.write_fd.is_none(),
                "principal readiness invalid",
            )
        } else {
            ensure(
                self.pid > 1 && self.ppid == 1 && self.write_fd.is_some_and(|v| v >= 3),
                "child capability readiness invalid",
            )
        }
    }
}
pub fn namespace_link(value: &str, kind: &str) -> Result<u64> {
    let prefix = format!("{kind}:[");
    let digits = value
        .strip_prefix(prefix.as_str())
        .and_then(|v| v.strip_suffix(']'))
        .ok_or("namespace observation missing/invalid")?;
    ensure(
        !digits.is_empty() && digits.bytes().all(|v| v.is_ascii_digit()),
        "namespace inode missing/invalid",
    )?;
    let inode = digits.parse()?;
    ensure(inode > 0, "namespace inode absent")?;
    Ok(inode)
}
pub fn wait(path: &Path, b: &Binding<'_>) -> Result<Ready> {
    let deadline = Instant::now() + Duration::from_secs(25);
    loop {
        if let Some(record) = read(path, b)? {
            return Ok(record);
        }
        ensure(
            Instant::now() < deadline,
            "rendezvous observation guard; no readiness inferred",
        )?;
        thread::sleep(Duration::from_millis(25));
    }
}
