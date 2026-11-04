use std::path::Path;

use base64::{Engine, engine::general_purpose::STANDARD};

use crate::{Base64Text, Identifier, NativePath, NativePathError, Platform};

pub(crate) fn current_platform() -> Result<Platform, NativePathError> {
    #[cfg(unix)]
    {
        Ok(Platform::Unix)
    }
    #[cfg(windows)]
    {
        Ok(Platform::Windows)
    }
    #[cfg(not(any(unix, windows)))]
    {
        Err(NativePathError::UnsupportedPlatform)
    }
}

pub(crate) fn native_units(path: &Path) -> Result<Vec<u8>, NativePathError> {
    #[cfg(unix)]
    let units = {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes().to_vec()
    };
    #[cfg(windows)]
    let units = {
        use std::os::windows::ffi::OsStrExt;
        path.as_os_str()
            .encode_wide()
            .flat_map(u16::to_be_bytes)
            .collect::<Vec<_>>()
    };
    #[cfg(not(any(unix, windows)))]
    let units = {
        let _ = path;
        return Err(NativePathError::UnsupportedPlatform);
    };
    validate_units(current_platform()?, &units)?;
    Ok(units)
}

fn validate_units(platform: Platform, units: &[u8]) -> Result<(), NativePathError> {
    if units.is_empty() {
        return Err(NativePathError::EmptyPath);
    }
    match platform {
        Platform::Unix if units.contains(&0) => return Err(NativePathError::NulUnit),
        Platform::Windows => {
            if !units.len().is_multiple_of(2) {
                return Err(NativePathError::OddWindowsByteLength);
            }
            if units.as_chunks::<2>().0.contains(&[0, 0]) {
                return Err(NativePathError::NulUnit);
            }
        }
        Platform::Unix => {}
    }
    Ok(())
}

pub(crate) fn decode_units(
    path: &NativePath,
    platform: Platform,
) -> Result<Vec<u8>, NativePathError> {
    let units = match path {
        NativePath::Utf8 { value } => match platform {
            Platform::Unix => value.as_str().as_bytes().to_vec(),
            Platform::Windows => value
                .as_str()
                .encode_utf16()
                .flat_map(u16::to_be_bytes)
                .collect(),
        },
        NativePath::UnixBytesBase64 { value } if platform == Platform::Unix => STANDARD
            .decode(value.as_str())
            .map_err(|_| NativePathError::InvalidBase64)?,
        NativePath::WindowsWideBeBase64 { value } if platform == Platform::Windows => STANDARD
            .decode(value.as_str())
            .map_err(|_| NativePathError::InvalidBase64)?,
        _ => return Err(NativePathError::ForeignPlatform),
    };
    validate_units(platform, &units)?;
    Ok(units)
}

pub(crate) fn encode_units(
    platform: Platform,
    units: &[u8],
) -> Result<NativePath, NativePathError> {
    validate_units(platform, units)?;
    let unicode = match platform {
        Platform::Unix => std::str::from_utf8(units).ok().map(str::to_owned),
        Platform::Windows => {
            let wide: Vec<_> = units
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u16::from_be_bytes(*pair))
                .collect();
            String::from_utf16(&wide).ok()
        }
    };
    if let Some(text) = unicode {
        return Ok(NativePath::Utf8 {
            value: Identifier::try_from(text).map_err(|_| NativePathError::EmptyPath)?,
        });
    }
    let value =
        Base64Text::try_from(STANDARD.encode(units)).map_err(|_| NativePathError::InvalidBase64)?;
    Ok(match platform {
        Platform::Unix => NativePath::UnixBytesBase64 { value },
        Platform::Windows => NativePath::WindowsWideBeBase64 { value },
    })
}

#[cfg(test)]
#[path = "tests/native-codec.rs"]
mod tests;
