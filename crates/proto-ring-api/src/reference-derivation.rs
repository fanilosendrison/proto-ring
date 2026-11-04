use proto_ring_engine::repository_governance_state::RepositoryGovernanceState;

use crate::{
    CoherenceBasis, ModelError, NativePath, NativeWordBits, ObservationFormat,
    ObservationProvenance, ObservationReference, Platform, ProfileFormat, ProfileReference,
    native_codec,
    reference_hash::{count, hash, public_hex},
};

pub(crate) fn provenance(
    state: &RepositoryGovernanceState,
) -> Result<ObservationProvenance, ModelError> {
    let root = native_codec::native_units(&state.repository).map_err(path_error)?;
    let scope = state
        .observed_state
        .scope_paths
        .iter()
        .map(|p| native_codec::native_units(p).map_err(path_error))
        .collect::<Result<Vec<_>, _>>()?;
    let observation = observation_reference(
        native_codec::current_platform().map_err(path_error)?,
        &root,
        &scope,
        &state.observed_state.identity.0,
    )?;
    Ok(ObservationProvenance {
        observation,
        repository: NativePath::from_path(&state.repository).map_err(path_error)?,
        // This is the original engine order, not the hashing working copy.
        scope_paths: state
            .observed_state
            .scope_paths
            .iter()
            .map(|p| NativePath::from_path(p).map_err(path_error))
            .collect::<Result<_, _>>()?,
    })
}

fn path_error(_: crate::NativePathError) -> ModelError {
    ModelError::InvalidScalar("retained native path representation")
}

pub(crate) fn observation_reference(
    platform: Platform,
    root: &[u8],
    scope: &[Vec<u8>],
    identity: &[u8],
) -> Result<ObservationReference, ModelError> {
    let platform_units: &[u8] = match platform {
        Platform::Unix => b"unix",
        Platform::Windows => b"windows",
    };
    let repository = hash("rgs-repository-v1", &[platform_units, root])?;
    let mut ordered = scope.to_vec();
    ordered.sort();
    if ordered.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ModelError::Inconsistent(
            "duplicate RGS scope path identity",
        ));
    }
    let scope_count = count(ordered.len())?;
    let mut parts = vec![repository.as_slice(), scope_count.as_slice()];
    parts.extend(ordered.iter().map(Vec::as_slice));
    let scope = hash("rgs-scope-v1", &parts)?;
    let state = hash("rgs-state-v1", &[&repository, &scope, identity])?;
    Ok(ObservationReference {
        format: ObservationFormat::V1,
        platform,
        repository: public_hex(&repository)?,
        scope: public_hex(&scope)?,
        state: public_hex(&state)?,
        coherence_basis: CoherenceBasis::OrdinaryCapture,
    })
}

pub(crate) fn original_profile_reference(
    state: &RepositoryGovernanceState,
) -> Result<Option<ProfileReference>, ModelError> {
    state
        .repository_integrity
        .as_ref()
        .map(|profile| {
            profile_reference(
                &profile.identity.0,
                NativeWordBits::try_from(u64::from(usize::BITS))?,
            )
        })
        .transpose()
}

pub(crate) fn profile_reference(
    identity: &[u8],
    bits: NativeWordBits,
) -> Result<ProfileReference, ModelError> {
    let width = u64::from(bits).to_be_bytes();
    let digest = hash("ri-profile-v1", &[&width, identity])?;
    Ok(ProfileReference {
        format: ProfileFormat::V1,
        native_word_bits: bits,
        sha256: public_hex(&digest)?,
    })
}

#[cfg(test)]
#[path = "tests/reference-vectors.rs"]
mod tests;
