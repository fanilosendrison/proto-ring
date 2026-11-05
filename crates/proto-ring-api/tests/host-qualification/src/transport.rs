//! Bounded controller subprocess transport. Never retries uncertain starts.
use crate::{Result, ensure, evidence::Evidence, policy};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[derive(Serialize)]
pub struct Transport {
    pub code: Option<i32>,
    pub guard_triggered: bool,
}
pub struct Captured {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub code: Option<i32>,
    pub guard_triggered: bool,
}
pub fn capture(evidence: &Evidence, prefix: &str, command: &mut Command) -> Result<Captured> {
    ensure(
        crate::evidence::safe_name(prefix),
        "unsafe transport prefix",
    )?;
    let description = serde_json::json!({"program":command.get_program(), "argv":command.get_args().collect::<Vec<_>>()});
    evidence.json(&format!("{prefix}-command.json"), &description)?;
    let stdout = evidence.root.join(format!("{prefix}.stdout"));
    let stderr = evidence.root.join(format!("{prefix}.stderr"));
    let output = exclusive(&stdout)?;
    let error = exclusive(&stderr)?;
    let mut child = command
        .stdout(Stdio::from(output))
        .stderr(Stdio::from(error))
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut guard = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            guard = true;
            child.kill()?;
            break child.wait()?;
        }
        thread::sleep(Duration::from_millis(25));
    };
    let record = Transport {
        code: status.code(),
        guard_triggered: guard,
    };
    evidence.json(&format!("{prefix}-transport.json"), &record)?;
    ensure(
        fs::metadata(&stdout)?.len() <= 4 * 1024 * 1024
            && fs::metadata(&stderr)?.len() <= 4 * 1024 * 1024,
        "controller output exceeded experiment bound",
    )?;
    Ok(Captured {
        stdout: fs::read(stdout)?,
        stderr: fs::read(stderr)?,
        code: record.code,
        guard_triggered: guard,
    })
}
fn exclusive(path: &Path) -> Result<fs::File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    Ok(options.open(path)?)
}
pub fn success(output: Captured) -> Result<Vec<u8>> {
    ensure(
        output.code == Some(0) && !output.guard_triggered,
        "controller transport failed or uncertain (no retry)",
    )?;
    Ok(output.stdout)
}
pub fn stream_check(
    prefix: &str,
    code: i64,
    stdout: &[u8],
    stderr: &[u8],
) -> crate::evidence::Check {
    crate::evidence::Check::Process {
        prefix: prefix.into(),
        code,
        stdout_hex: policy::hex(stdout),
        stderr_hex: policy::hex(stderr),
    }
}
