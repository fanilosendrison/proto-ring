//! Inert write attempts and controller-owned byte observations.
use crate::{
    Result,
    docker::Docker,
    evidence::{Check, Evidence},
    policy,
    subject::Subject,
    transport,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
pub(crate) fn run_output(
    docker: &mut Docker,
    subject: &Subject,
    name: &str,
    mode: &str,
    args: &[String],
    rw: bool,
    stdout: &[u8],
) -> Result<Vec<Check>> {
    let id = docker.run(name, mode, args, &subject.root, &[subject.mount(rw)], None)?;
    Ok(vec![
        transport::stream_check(name, 0, stdout, b""),
        Docker::closed(name, &id),
    ])
}
pub(crate) fn writable_control(
    docker: &mut Docker,
    subject: &Subject,
    name: &str,
) -> Result<Vec<Check>> {
    let target = subject.root.join(format!("{name}-target"));
    fs::write(&target, b"before\n")?;
    let mut checks = run_output(
        docker,
        subject,
        name,
        "write",
        &[target.to_string_lossy().into_owned()],
        true,
        b"written\n",
    )?;
    let bytes = fs::read(target)?;
    docker
        .evidence
        .bytes(&format!("{name}-effect.bin"), &bytes)?;
    checks.push(Check::Bytes {
        path: format!("{name}-effect.bin"),
        hex: policy::hex(b"after\n"),
    });
    Ok(checks)
}
pub(crate) fn denied(
    docker: &mut Docker,
    subject: &Subject,
    name: &str,
    path: &str,
    binds: &[(PathBuf, String, bool)],
) -> Result<Vec<Check>> {
    let id = docker.run(name, "write", &[path.into()], &subject.root, binds, None)?;
    Ok(vec![
        transport::stream_check(name, 30, b"", b"started\nEROFS\n"),
        Docker::closed(name, &id),
    ])
}
pub(crate) fn target_unchanged(
    evidence: &Evidence,
    name: &str,
    path: &Path,
    expected: &[u8],
) -> Result<Check> {
    let bytes = fs::read(path)?;
    let file = format!("{name}-effect.bin");
    evidence.bytes(&file, &bytes)?;
    Ok(Check::Bytes {
        path: file,
        hex: policy::hex(expected),
    })
}
