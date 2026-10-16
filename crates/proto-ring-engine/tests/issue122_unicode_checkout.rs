#![forbid(unsafe_code)]

use proto_ring_engine::normative_terminology::unicode14::{
    CASEFOLD_DATA_SHA256, UNICODE_VERSION, casefold_data_sha256,
};

#[test]
fn embedded_casefold_bytes_match_unicode_14_authority() {
    assert_eq!(UNICODE_VERSION, "14.0.0");
    assert_eq!(
        CASEFOLD_DATA_SHA256,
        "a566cd48687b2cd897e02501118b2413c14ae86d318f9abbbba97feb84189f0f"
    );
    assert_eq!(casefold_data_sha256(), CASEFOLD_DATA_SHA256);
}
