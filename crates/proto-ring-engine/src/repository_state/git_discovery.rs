//! Source-aware decoding of Git path-discovery transport records.

use super::StateCaptureError;

#[derive(Clone, Debug)]
pub(super) struct DiscoveredPath {
    pub(super) git_bytes: Vec<u8>,
    pub(super) identity_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum DiscoverySource {
    Tracked,
    Untracked,
}

pub(super) fn parse_discovered_paths(
    bytes: &[u8],
    source: DiscoverySource,
) -> Result<Vec<DiscoveredPath>, StateCaptureError> {
    nul_fields(bytes)?
        .into_iter()
        .map(|path| {
            let identity_bytes = decode_identity(path, source)?;
            Ok(DiscoveredPath {
                git_bytes: path.to_vec(),
                identity_bytes,
            })
        })
        .collect()
}

fn decode_identity(path: &[u8], source: DiscoverySource) -> Result<Vec<u8>, StateCaptureError> {
    if path.last() != Some(&b'/') {
        return Ok(path.to_vec());
    }
    if matches!(source, DiscoverySource::Tracked) {
        return Err(StateCaptureError::new(
            "tracked Git discovery returned an unexpected directory marker",
        ));
    }
    let identity = &path[..path.len() - 1];
    if identity.is_empty() || identity.last() == Some(&b'/') {
        return Err(StateCaptureError::new(
            "untracked Git discovery returned a malformed directory marker",
        ));
    }
    Ok(identity.to_vec())
}

fn nul_fields(bytes: &[u8]) -> Result<Vec<&[u8]>, StateCaptureError> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    if bytes.last() != Some(&0) {
        return Err(StateCaptureError::new(
            "Git path discovery was not NUL terminated",
        ));
    }
    let fields: Vec<_> = bytes[..bytes.len() - 1].split(|byte| *byte == 0).collect();
    if fields.iter().any(|field| field.is_empty()) {
        return Err(StateCaptureError::new(
            "Git path discovery returned an empty interior field",
        ));
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::{DiscoverySource, parse_discovered_paths};

    #[test]
    fn untracked_terminal_directory_marker_is_transport_not_identity() {
        let paths = parse_discovered_paths(b"nested/\0", DiscoverySource::Untracked)
            .expect("one untracked directory marker is admitted");
        assert_eq!(paths[0].git_bytes, b"nested/");
        assert_eq!(paths[0].identity_bytes, b"nested");
        eprintln!("ISSUE67_UNTRACKED_DIRECTORY_MARKER_DECODE=PASS");
    }

    #[test]
    fn tracked_terminal_directory_marker_is_rejected() {
        assert!(parse_discovered_paths(b"nested/\0", DiscoverySource::Tracked).is_err());
        eprintln!("ISSUE67_TRACKED_UNEXPECTED_DIRECTORY_MARKER=FAIL_CLOSED");
    }

    #[test]
    fn discovery_identity_decoder_does_not_trim_path_units() {
        for path in [
            b"name ".as_slice(),
            b"name\t".as_slice(),
            b"name\r".as_slice(),
            b"name\n".as_slice(),
            b"dir/file".as_slice(),
        ] {
            let mut record = path.to_vec();
            record.push(0);
            let decoded = parse_discovered_paths(&record, DiscoverySource::Untracked)
                .expect("non-marker path units are exact");
            assert_eq!(decoded[0].git_bytes, path);
            assert_eq!(decoded[0].identity_bytes, path);
        }
        eprintln!("ISSUE67_DISCOVERY_NON_SLASH_TRAILING_UNITS_PRESERVED=PASS");
    }

    #[test]
    fn malformed_repeated_terminal_directory_marker_fails_closed() {
        assert!(parse_discovered_paths(b"nested//\0", DiscoverySource::Untracked).is_err());
        eprintln!("ISSUE67_REPEATED_DIRECTORY_MARKER=FAIL_CLOSED");
    }
}
