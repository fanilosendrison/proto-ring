//! Exclusive controller evidence files and complete campaign inventory.
use crate::{Result, ensure, policy};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Pass,
    Fail,
    Unavailable,
    Indeterminate,
    NotRun,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub status: Status,
    pub reason: String,
    pub checks: Vec<Check>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Check {
    Equal {
        observed: Value,
        required: Value,
    },
    Bytes {
        path: String,
        hex: String,
    },
    Process {
        prefix: String,
        code: i64,
        stdout_hex: String,
        stderr_hex: String,
    },
    DifferentCanonical {
        before: Value,
        after: Value,
    },
    Closed {
        inspect: String,
        container_id: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Campaign {
    pub repository: String,
    pub source_sha: String,
    pub source_tree: String,
    pub workflow_sha: String,
    pub run_id: String,
    pub run_attempt: String,
    pub cases: BTreeMap<String, Case>,
    pub cleanup_verified: bool,
    pub cleanup_containers: Vec<String>,
    pub cleanup_networks: Vec<String>,
}
impl Campaign {
    pub fn new(
        source_sha: String,
        source_tree: String,
        run_id: String,
        run_attempt: String,
    ) -> Self {
        let cases = policy::REQUIRED
            .iter()
            .flat_map(|(group, cases)| {
                cases.iter().map(move |name| {
                    (
                        format!("{group}/{name}"),
                        Case {
                            status: Status::NotRun,
                            reason: "not observed".into(),
                            checks: Vec::new(),
                        },
                    )
                })
            })
            .collect();
        Self {
            repository: "fanilosendrison/proto-ring".into(),
            workflow_sha: source_sha.clone(),
            source_sha,
            source_tree,
            run_id,
            run_attempt,
            cases,
            cleanup_verified: false,
            cleanup_containers: Vec::new(),
            cleanup_networks: Vec::new(),
        }
    }
}
#[derive(Clone)]
pub struct Evidence {
    pub root: PathBuf,
}
impl Evidence {
    pub fn create(root: &Path) -> Result<Self> {
        fs::create_dir(root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(root, fs::Permissions::from_mode(0o700))?;
        }
        Ok(Self {
            root: root.to_owned(),
        })
    }
    pub fn bytes(&self, name: &str, bytes: &[u8]) -> Result<()> {
        ensure(safe_name(name), "unsafe evidence name")?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(self.root.join(name))?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    }
    pub fn json(&self, name: &str, value: &impl Serialize) -> Result<()> {
        self.bytes(name, &serde_json::to_vec_pretty(value)?)
    }
    pub fn inventory(&self) -> Result<()> {
        let entries = inventory(&self.root)?;
        self.json("inventory.json", &entries)
    }
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub bytes: u64,
    pub sha256: String,
}
pub fn inventory(root: &Path) -> Result<BTreeMap<String, Entry>> {
    let mut entries = BTreeMap::new();
    for item in fs::read_dir(root)? {
        let item = item?;
        let name = item
            .file_name()
            .into_string()
            .map_err(|_| "non UTF8 evidence name")?;
        ensure(safe_name(&name), "unsafe inventory path")?;
        let metadata = fs::symlink_metadata(item.path())?;
        ensure(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "evidence is not regular",
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            ensure(metadata.nlink() == 1, "linked evidence")?;
        }
        if name == "inventory.json" {
            continue;
        }
        let bytes = fs::read(item.path())?;
        ensure(bytes.len() <= 4 * 1024 * 1024, "oversized evidence file")?;
        entries.insert(
            name,
            Entry {
                bytes: bytes.len() as u64,
                sha256: policy::sha256(&bytes),
            },
        );
    }
    Ok(entries)
}
pub fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
}
