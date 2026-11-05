//! Complete technical filesystem witnesses, not RepositoryState identities.
use crate::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub device: u64,
    pub inode: u64,
    pub mode: u32,
    pub links: u64,
}
impl Identity {
    pub fn observe(path: &Path) -> Result<Self> {
        Self::metadata(&fs::symlink_metadata(path)?)
    }
    pub fn metadata(m: &fs::Metadata) -> Result<Self> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Ok(Self {
                device: m.dev(),
                inode: m.ino(),
                mode: m.mode(),
                links: m.nlink(),
            })
        }
        #[cfg(not(unix))]
        {
            let _ = m;
            Err("Unix filesystem identity unavailable".into())
        }
    }
    pub fn validate(&self, kind: &str) -> Result<()> {
        ensure(
            self.inode > 0 && self.links > 0 && self.mode & 0o022 == 0,
            "filesystem identity/permissions incomplete",
        )?;
        match kind {
            "directory" => ensure(
                self.mode & 0o170000 == 0o040000,
                "directory identity required",
            ),
            "regular" => ensure(
                self.mode & 0o170000 == 0o100000 && self.links == 1,
                "regular singly-linked identity required",
            ),
            _ => Err("unapproved identity kind".into()),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Linkage {
    pub path: PathBuf,
    pub resolved: Option<PathBuf>,
    pub present: bool,
    pub identity: Option<Identity>,
}
impl Linkage {
    pub fn observe(path: &Path) -> Result<Self> {
        match fs::symlink_metadata(path) {
            Ok(m) => Ok(Self {
                path: path.into(),
                resolved: Some(path.canonicalize()?),
                present: true,
                identity: Some(Identity::metadata(&m)?),
            }),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Self {
                path: path.into(),
                resolved: None,
                present: false,
                identity: None,
            }),
            Err(e) => Err(e.into()),
        }
    }
    pub fn require(&self, path: &Path, kind: &str) -> Result<()> {
        ensure(
            crate::campaign_context::exact_absolute(&self.path)
                && self
                    .resolved
                    .as_deref()
                    .is_none_or(crate::campaign_context::exact_absolute),
            "ambiguous filesystem linkage",
        )?;
        ensure(self.path == path, "filesystem linkage path mismatch")?;
        if kind == "absent" {
            return ensure(
                !self.present && self.resolved.is_none() && self.identity.is_none(),
                "external Git dependency not excluded",
            );
        }
        ensure(
            self.present && self.resolved.as_deref() == Some(path),
            "filesystem linkage/alias unresolved",
        )?;
        self.identity
            .as_ref()
            .ok_or("filesystem identity missing")?
            .validate(kind)
    }
}
