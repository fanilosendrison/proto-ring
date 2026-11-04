use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;

use super::{observation_reference, profile_reference};
use crate::{NativeWordBits, Platform, reference_hash::hash};

fn vectors() -> Value {
    serde_json::from_str(include_str!(
        "../../../../docs/api/v1/conformance/identity-vectors.json"
    ))
    .unwrap()
}

fn parts(vector: &Value) -> Vec<Vec<u8>> {
    vector["parts_base64"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| STANDARD.decode(p.as_str().unwrap()).unwrap())
        .collect()
}

#[test]
fn all_thirteen_accepted_synthetic_hash_vectors_match_exactly() {
    let document = vectors();
    let vectors = document["vectors"].as_array().unwrap();
    assert_eq!(vectors.len(), 13);
    for vector in vectors {
        let parts = parts(vector);
        let actual = hash(
            vector["domain"].as_str().unwrap(),
            &parts.iter().map(Vec::as_slice).collect::<Vec<_>>(),
        )
        .unwrap();
        let hex = actual
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(
            hex,
            vector["expected_sha256_lowerhex"].as_str().unwrap(),
            "{}",
            vector["id"]
        );
    }
}

#[test]
fn rgs_derivation_uses_raw_digests_and_accepted_constituents() {
    let actual = observation_reference(
        Platform::Unix,
        b"/tmp/repo",
        &[b"docs/profile.md".to_vec(), b"AGENTS.md".to_vec()],
        b"\x00\x01candidate",
    )
    .unwrap();
    assert_eq!(
        actual.repository.as_str(),
        "50ce34743a9561f2f61155628f1a528054d4fc6e36cada4444c01b53d0d11a25"
    );
    assert_eq!(
        actual.scope.as_str(),
        "3d7b58689c8f610a164734d80234e89e9a3bdbaacc66d478beccafd0427b97a0"
    );
    assert_eq!(
        actual.state.as_str(),
        "2f7b80f9f57fd1e349459a78fd9c43b151b86f45c715a93f646b0fec9388311c"
    );
}

#[test]
fn rgs_scope_sort_is_native_byte_order_on_a_working_copy() {
    let input = vec![b"a/b".to_vec(), b"a-b".to_vec()];
    let original = input.clone();
    let actual =
        observation_reference(Platform::Unix, b"/tmp/repo", &input, b"\x00\x01candidate").unwrap();
    // Independent Python hashlib oracle applying section 5, not production code.
    assert_eq!(
        actual.scope.as_str(),
        "e3fe08478ce83932adbe73cee260895d62eb0a62396ce529ecf1d445a426a037"
    );
    assert_eq!(
        actual.state.as_str(),
        "271d9ed9acbab45584ba08b66cab0f0ad764a6f30d580f779890b80086062cba"
    );
    assert_eq!(input, original);
    assert!(
        observation_reference(
            Platform::Unix,
            b"/tmp/repo",
            &[b"a".to_vec(), b"a".to_vec()],
            b"i"
        )
        .is_err()
    );
}

#[test]
fn profile_derivation_binds_original_bytes_and_explicit_synthetic_word_width() {
    for vector in vectors()["vectors"].as_array().unwrap() {
        if vector["domain"] != "ri-profile-v1" {
            continue;
        }
        let parts = parts(vector);
        let bits = NativeWordBits::try_from(vector["native_word_bits"].as_u64().unwrap()).unwrap();
        let actual = profile_reference(&parts[1], bits).unwrap();
        assert_eq!(
            actual.sha256.as_str(),
            vector["expected_sha256_lowerhex"].as_str().unwrap()
        );
        assert_eq!(actual.native_word_bits, bits);
    }
}

#[test]
fn framing_separates_domains_counts_and_component_boundaries() {
    let joined = hash("rgs-state-v1", &[b"ab", b"c"]).unwrap();
    assert_ne!(joined, hash("rgs-state-v1", &[b"a", b"bc"]).unwrap());
    assert_ne!(joined, hash("rgs-scope-v1", &[b"ab", b"c"]).unwrap());
    assert_ne!(hash("x", &[]).unwrap(), hash("x", &[b""]).unwrap());
    assert_ne!(
        profile_reference(b"same", NativeWordBits::Bits32)
            .unwrap()
            .sha256,
        profile_reference(b"same", NativeWordBits::Bits64)
            .unwrap()
            .sha256
    );
}
