use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Base64Text, Identifier, NativePathError, native_codec};

/// Exact path representation. Conversion validates native units but does not
/// resolve a path, normalize it, or establish repository containment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NativePath {
    Utf8 { value: Identifier },
    UnixBytesBase64 { value: Base64Text },
    WindowsWideBeBase64 { value: Base64Text },
}

impl NativePath {
    /// Encode the original OS units, preferring exact Unicode when possible.
    pub fn from_path(path: &Path) -> Result<Self, NativePathError> {
        native_codec::encode_units(
            native_codec::current_platform()?,
            &native_codec::native_units(path)?,
        )
    }

    /// Decode only on the matching host platform; no filesystem access occurs.
    pub fn to_path_buf(&self) -> Result<PathBuf, NativePathError> {
        let units = native_codec::decode_units(self, native_codec::current_platform()?)?;
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            Ok(std::ffi::OsString::from_vec(units).into())
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStringExt;
            let wide: Vec<_> = units
                .chunks_exact(2)
                .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
                .collect();
            Ok(std::ffi::OsString::from_wide(&wide).into())
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = units;
            Err(NativePathError::UnsupportedPlatform)
        }
    }
}
