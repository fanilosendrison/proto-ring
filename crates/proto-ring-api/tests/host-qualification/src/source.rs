//! Source authority comes from caller-selected immutable Git objects, not the dossier.
use crate::{Result, ensure, evidence::Evidence, policy};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Command, Stdio},
};
const ROOTS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "crates/proto-ring-api",
    "crates/proto-ring-engine",
    "docs/api/v1",
    "docs/api/api-03-host-qualification.md",
    ".github/workflows/api03-host-qualification.yml",
];
const MINIMUM: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "crates/proto-ring-api/Cargo.toml",
    "crates/proto-ring-api/src/lib.rs",
    "crates/proto-ring-engine/Cargo.toml",
    "crates/proto-ring-engine/src/lib.rs",
    "docs/api/v1/proto-ring-api-01-v1-spec.md",
    "docs/api/api-03-host-qualification.md",
    ".github/workflows/api03-host-qualification.yml",
    "crates/proto-ring-api/tests/host-qualification/Cargo.toml",
    "crates/proto-ring-api/tests/host-qualification/Cargo.lock",
    "crates/proto-ring-api/tests/host-qualification/src/main.rs",
    "crates/proto-ring-api/tests/host-qualification/src/lib.rs",
    "crates/proto-ring-api/tests/host-qualification/src/policy.rs",
    "crates/proto-ring-api/tests/host-qualification/fixtures/probe.py",
    "crates/proto-ring-api/tests/host-qualification/fixtures/rendezvous.py",
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub sha: String,
    pub tree: String,
    #[serde(deserialize_with = "unique_files")]
    pub files: BTreeMap<String, File>,
}
fn unique_files<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, File>, D::Error> {
    struct Unique;
    impl<'de> serde::de::Visitor<'de> for Unique {
        type Value = BTreeMap<String, File>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("unique source paths")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut m: M,
        ) -> std::result::Result<Self::Value, M::Error> {
            let mut files = BTreeMap::new();
            while let Some((k, v)) = m.next_entry::<String, File>()? {
                if files.insert(k, v).is_some() {
                    return Err(serde::de::Error::custom("duplicate source path"));
                }
            }
            Ok(files)
        }
    }
    d.deserialize_map(Unique)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub mode: String,
    pub blob: String,
    pub bytes: usize,
    pub sha256: String,
}
fn git(root: &Path) -> Command {
    let mut c = Command::new("git");
    c.env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .arg("-C")
        .arg(root);
    c
}
fn output(c: &mut Command) -> Result<Vec<u8>> {
    let out = c.output()?;
    ensure(out.status.success(), "source Git observation unavailable")?;
    Ok(out.stdout)
}
pub fn trusted(root: &Path, sha: &str) -> Result<Source> {
    ensure(policy::source_sha(sha), "source SHA syntax")?;
    let commit = output(git(root).args(["cat-file", "-t", sha]))?;
    ensure(
        commit == b"commit\n",
        "authorized source is not a Git commit",
    )?;
    let tree = String::from_utf8(output(
        git(root).args(["rev-parse", &format!("{sha}^{{tree}}")]),
    )?)?
    .trim_end_matches('\n')
    .to_owned();
    let entries = output(git(root).args(["ls-tree", "-rz", sha, "--"]).args(ROOTS))?;
    let mut process = git(root)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut input = process.stdin.take().ok_or("Git batch input missing")?;
    let mut reader = BufReader::new(process.stdout.take().ok_or("Git batch output missing")?);
    let mut files = BTreeMap::new();
    for raw in entries.split(|b| *b == 0).filter(|v| !v.is_empty()) {
        let (meta, path) = std::str::from_utf8(raw)?
            .split_once('\t')
            .ok_or("ambiguous Git source entry")?;
        let fields: Vec<_> = meta.split(' ').collect();
        ensure(
            fields.len() == 3
                && fields[1] == "blob"
                && matches!(fields[0], "100644" | "100755")
                && safe_relative(path),
            "source mode/path invalid",
        )?;
        input.write_all(fields[2].as_bytes())?;
        input.write_all(b"\n")?;
        input.flush()?;
        let mut header = String::new();
        reader.read_line(&mut header)?;
        let h: Vec<_> = header.split_ascii_whitespace().collect();
        ensure(
            h.len() == 3 && h[0] == fields[2] && h[1] == "blob",
            "Git blob identity mismatch",
        )?;
        let size: usize = h[2].parse()?;
        ensure(
            size <= 4 * 1024 * 1024,
            "source blob exceeds experiment bound",
        )?;
        let mut data = vec![0; size];
        reader.read_exact(&mut data)?;
        let mut lf = [0];
        reader.read_exact(&mut lf)?;
        ensure(lf == [10], "malformed Git batch framing")?;
        ensure(
            files
                .insert(
                    path.into(),
                    File {
                        mode: fields[0].into(),
                        blob: fields[2].into(),
                        bytes: size,
                        sha256: policy::sha256(&data),
                    },
                )
                .is_none(),
            "duplicate source identity",
        )?;
    }
    drop(input);
    ensure(process.wait()?.success(), "Git source reader failed")?;
    ensure(
        MINIMUM.iter().all(|p| files.contains_key(*p)),
        "source inventory required roots/files absent",
    )?;
    Ok(Source {
        sha: sha.into(),
        tree,
        files,
    })
}
pub fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !Path::new(path).is_absolute()
        && path
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
}
pub fn source_unchanged(root: &Path, source: &Source) -> Result<()> {
    #[cfg(unix)]
    let mut identities = std::collections::BTreeSet::new();
    for (path, entry) in &source.files {
        let p = root.join(path);
        let metadata = fs::symlink_metadata(&p).map_err(|e| format!("{path}: {e}"))?;
        ensure(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            &format!("{path}: source is not a regular file"),
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            ensure(
                metadata.nlink() == 1 && identities.insert((metadata.dev(), metadata.ino())),
                &format!("{path}: source alias/hardlink"),
            )?;
            ensure(
                (metadata.permissions().mode() & 0o111 != 0) == (entry.mode == "100755"),
                &format!("{path}: source mode drift"),
            )?;
        }
        let bytes = fs::read(p).map_err(|e| format!("{path}: {e}"))?;
        ensure(
            bytes.len() == entry.bytes && policy::sha256(&bytes) == entry.sha256,
            &format!("{path}: source bytes drift"),
        )?;
    }
    // Include ignored additions in executable source directories: no unbound module/import.
    for prefix in [
        "crates/proto-ring-api/src",
        "crates/proto-ring-engine/src",
        "crates/proto-ring-api/tests/host-qualification/src",
        "crates/proto-ring-api/tests/host-qualification/fixtures",
    ] {
        let mut pending = vec![root.join(prefix)];
        while let Some(path) = pending.pop() {
            if !path.exists() {
                continue;
            }
            for e in fs::read_dir(&path)? {
                let e = e?;
                let m = fs::symlink_metadata(e.path())?;
                ensure(!m.file_type().is_symlink(), "source alias")?;
                if m.is_dir() {
                    pending.push(e.path());
                } else {
                    let rel = e
                        .path()
                        .strip_prefix(root)?
                        .to_str()
                        .ok_or("source path encoding")?
                        .to_owned();
                    ensure(
                        source.files.contains_key(&rel),
                        &format!("untracked executable source: {rel}"),
                    )?;
                }
            }
        }
    }
    Ok(())
}
pub fn inventory(e: &Evidence, sha: &str) -> Result<Source> {
    for (key, required) in [
        ("GITHUB_SHA", sha),
        ("GITHUB_WORKFLOW_SHA", sha),
        ("GITHUB_REF", "refs/heads/main"),
        ("GITHUB_EVENT_NAME", "workflow_dispatch"),
        ("GITHUB_REPOSITORY", "fanilosendrison/proto-ring"),
    ] {
        ensure(
            std::env::var(key).is_ok_and(|v| v == required),
            "execution source identity mismatch",
        )?;
    }
    let root = std::env::current_dir()?.canonicalize()?;
    ensure(
        output(git(&root).args(["rev-parse", "HEAD"]))? == format!("{sha}\n").as_bytes(),
        "checkout is not authorized source",
    )?;
    let source = trusted(&root, sha)?;
    source_unchanged(&root, &source)?;
    e.json("source.json", &source)?;
    Ok(source)
}
