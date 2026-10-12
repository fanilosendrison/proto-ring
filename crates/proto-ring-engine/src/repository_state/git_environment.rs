//! Git subprocess environment-name sanitization policy.

use std::ffi::OsStr;

const REMOVED_GIT_ENVIRONMENT: &str = concat!(
    "GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_CONFIG GIT_CONFIG_PARAMETERS GIT_CONFIG_COUNT ",
    "GIT_CONFIG_GLOBAL GIT_CONFIG_SYSTEM GIT_CONFIG_NOSYSTEM GIT_OBJECT_DIRECTORY ",
    "GIT_DIR GIT_WORK_TREE GIT_IMPLICIT_WORK_TREE GIT_GRAFT_FILE GIT_INDEX_FILE ",
    "GIT_NO_REPLACE_OBJECTS GIT_REPLACE_REF_BASE GIT_PREFIX GIT_SHALLOW_FILE ",
    "GIT_COMMON_DIR GIT_NAMESPACE GIT_CEILING_DIRECTORIES GIT_DISCOVERY_ACROSS_FILESYSTEM",
);

pub(super) fn removed_git_name(name: &OsStr) -> bool {
    REMOVED_GIT_ENVIRONMENT
        .split_ascii_whitespace()
        .any(|candidate| native_name_eq(name, candidate.as_bytes()))
        || native_name_starts_with(name, b"GIT_CONFIG_KEY_")
        || native_name_starts_with(name, b"GIT_CONFIG_VALUE_")
        || native_name_starts_with(name, b"GIT_TRACE")
}

#[cfg(unix)]
fn native_name_eq(name: &OsStr, ascii_name: &[u8]) -> bool {
    name.as_encoded_bytes() == ascii_name
}

#[cfg(unix)]
fn native_name_starts_with(name: &OsStr, prefix: &[u8]) -> bool {
    name.as_encoded_bytes().starts_with(prefix)
}

#[cfg(windows)]
fn native_name_eq(name: &OsStr, ascii_name: &[u8]) -> bool {
    windows_ascii_name_eq(name, ascii_name)
}

#[cfg(windows)]
fn native_name_starts_with(name: &OsStr, prefix: &[u8]) -> bool {
    windows_ascii_name_starts_with(name, prefix)
}

#[cfg(windows)]
fn windows_ascii_name_eq(name: &OsStr, ascii_name: &[u8]) -> bool {
    use std::os::windows::ffi::OsStrExt;

    let mut units = name.encode_wide();
    for ascii_unit in ascii_name {
        let Some(native_unit) = units.next() else {
            return false;
        };
        if fold_ascii_unit(native_unit) != fold_ascii_unit(u16::from(*ascii_unit)) {
            return false;
        }
    }
    units.next().is_none()
}

#[cfg(windows)]
fn windows_ascii_name_starts_with(name: &OsStr, prefix: &[u8]) -> bool {
    use std::os::windows::ffi::OsStrExt;

    let mut units = name.encode_wide();
    prefix.iter().all(|ascii_unit| {
        units.next().is_some_and(|native_unit| {
            fold_ascii_unit(native_unit) == fold_ascii_unit(u16::from(*ascii_unit))
        })
    })
}

#[cfg(windows)]
fn fold_ascii_unit(unit: u16) -> u16 {
    if unit >= u16::from(b'A') && unit <= u16::from(b'Z') {
        unit + u16::from(b'a' - b'A')
    } else {
        unit
    }
}

#[cfg(test)]
mod tests {
    use super::removed_git_name;
    use std::ffi::OsStr;

    #[test]
    fn exact_names_and_ascii_prefixes_are_classified() {
        for name in [
            "GIT_TRACE",
            "GIT_TRACE2",
            "GIT_TRACE_FUTURE_TARGET",
            "GIT_CONFIG_KEY_0",
            "GIT_CONFIG_VALUE_0",
            "GIT_DIR",
        ] {
            assert!(removed_git_name(OsStr::new(name)), "{name}");
        }
        for name in ["PATH", "PROTO_RING_EXAMPLE", "XGIT_TRACE", "GIT_TRAC"] {
            assert!(!removed_git_name(OsStr::new(name)), "{name}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn unix_git_environment_names_remain_case_sensitive() {
        for name in ["git_dir", "Git_Work_Tree", "gIt_TrAcE2", "git_config_key_0"] {
            assert!(!removed_git_name(OsStr::new(name)), "{name}");
        }
        eprintln!("UNIX_GIT_ENVIRONMENT_CASE_SENSITIVE=PASS");
    }

    #[cfg(unix)]
    #[test]
    fn git_trace_prefix_with_non_utf8_suffix_is_removed() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        for prefix in [
            b"GIT_TRACE".as_slice(),
            b"GIT_CONFIG_KEY_".as_slice(),
            b"GIT_CONFIG_VALUE_".as_slice(),
        ] {
            let mut bytes = prefix.to_vec();
            bytes.push(0xff);
            assert!(removed_git_name(&OsString::from_vec(bytes)));
        }
        assert!(!removed_git_name(&OsString::from_vec(
            b"NOT_GIT_TRACE_\xff".to_vec()
        )));
        eprintln!("NON_UTF8_GIT_TRACE_PREFIX_REMOVED=PASS");
        eprintln!("NON_UTF8_GIT_CONFIG_KEY_PREFIX_REMOVED=PASS");
        eprintln!("NON_UTF8_GIT_CONFIG_VALUE_PREFIX_REMOVED=PASS");
        eprintln!("NONPREFIX_NON_UTF8_ENV_RETAINED=PASS");
    }

    #[cfg(windows)]
    #[test]
    fn windows_git_environment_names_are_ascii_case_insensitive() {
        for fixed_name in super::REMOVED_GIT_ENVIRONMENT.split_ascii_whitespace() {
            for variant in [
                fixed_name.to_owned(),
                fixed_name.to_ascii_lowercase(),
                deterministic_mixed_case(fixed_name),
            ] {
                assert!(removed_git_name(OsStr::new(&variant)), "{variant}");
            }
        }
        for prefix in ["GIT_TRACE", "GIT_CONFIG_KEY_", "GIT_CONFIG_VALUE_"] {
            for variant in [
                format!("{prefix}FUTURE"),
                format!("{}future", prefix.to_ascii_lowercase()),
                format!("{}Future", deterministic_mixed_case(prefix)),
            ] {
                assert!(removed_git_name(OsStr::new(&variant)), "{variant}");
            }
        }
        for retained in ["PATH", "PathLikeGitDir", "XGIT_DIR", "MY_GIT_TRACE"] {
            assert!(!removed_git_name(OsStr::new(retained)), "{retained}");
        }
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        let mut opaque_suffix: Vec<u16> = "gIt_TrAcE".encode_utf16().collect();
        opaque_suffix.push(0xd800);
        assert!(removed_git_name(&OsString::from_wide(&opaque_suffix)));
        eprintln!("WINDOWS_FIXED_GIT_NAMES_CASE_INSENSITIVE=PASS");
        eprintln!("WINDOWS_GIT_PREFIXES_CASE_INSENSITIVE=PASS");
    }

    #[cfg(windows)]
    fn deterministic_mixed_case(value: &str) -> String {
        value
            .bytes()
            .enumerate()
            .map(|(index, byte)| {
                if index % 2 == 0 {
                    byte.to_ascii_lowercase()
                } else {
                    byte.to_ascii_uppercase()
                }
            })
            .map(char::from)
            .collect()
    }
}
