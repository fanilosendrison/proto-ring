use sha2::{Digest, Sha256};

use crate::{HexDigest, ModelError};

/// Section 5's H primitive. Digests remain raw bytes when used as constituents.
pub(crate) fn hash(domain: &str, parts: &[&[u8]]) -> Result<[u8; 32], ModelError> {
    let mut hash = Sha256::new();
    frame(&mut hash, b"proto-ring-api01-v1")?;
    frame(&mut hash, domain.as_bytes())?;
    hash.update(count(parts.len())?);
    for part in parts {
        frame(&mut hash, part)?;
    }
    Ok(hash.finalize().into())
}

pub(crate) fn count(value: usize) -> Result<[u8; 8], ModelError> {
    u64::try_from(value)
        .map(u64::to_be_bytes)
        .map_err(|_| ModelError::InvalidScalar("reference framing length"))
}

fn frame(hash: &mut Sha256, bytes: &[u8]) -> Result<(), ModelError> {
    hash.update(count(bytes.len())?);
    hash.update(bytes);
    Ok(())
}

pub(crate) fn public_hex(digest: &[u8; 32]) -> Result<HexDigest, ModelError> {
    HexDigest::try_from(
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
    )
}
