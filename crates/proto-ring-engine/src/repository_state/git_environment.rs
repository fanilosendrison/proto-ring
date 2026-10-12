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
        .any(|candidate| name == OsStr::new(candidate))
        || has_ascii_prefix(name, b"GIT_CONFIG_KEY_")
        || has_ascii_prefix(name, b"GIT_CONFIG_VALUE_")
        || has_ascii_prefix(name, b"GIT_TRACE")
}

fn has_ascii_prefix(name: &OsStr, prefix: &[u8]) -> bool {
    name.as_encoded_bytes().starts_with(prefix)
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
}
