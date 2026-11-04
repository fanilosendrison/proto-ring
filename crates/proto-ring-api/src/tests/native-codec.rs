use serde_json::{Value, json};

use super::{decode_units, encode_units};
use crate::{NativePath, Platform};

#[test]
fn accepted_path_vectors_are_synthetic_for_both_platforms_not_filesystem_claims() {
    let document: Value = serde_json::from_str(include_str!(
        "../../../../docs/api/v1/conformance/identity-vectors.json"
    ))
    .unwrap();
    for case in document["native_path_cases"].as_array().unwrap() {
        let platform: Platform = serde_json::from_value(case["platform"].clone()).unwrap();
        let path = serde_json::from_value::<NativePath>(case["path"].clone());
        let units = path
            .map_err(|e| e.to_string())
            .and_then(|path| decode_units(&path, platform).map_err(|e| e.to_string()));
        assert_eq!(
            units.is_ok(),
            case["valid"].as_bool().unwrap(),
            "{}",
            case["id"]
        );
        if let Ok(units) = units {
            let actual = units.iter().map(|b| format!("{b:02x}")).collect::<String>();
            assert_eq!(actual, case["expected_units_hex"].as_str().unwrap());
            let encoded = encode_units(platform, &units).unwrap();
            assert_eq!(decode_units(&encoded, platform).unwrap(), units);
        }
    }
}

#[test]
fn synthetic_wide_encoding_is_big_endian_and_preserves_unpaired_surrogates() {
    let units = [0x00, 0x43, 0x00, 0x3a, 0x00, 0x5c, 0xd8, 0x00];
    let encoded = encode_units(Platform::Windows, &units).unwrap();
    assert_eq!(
        serde_json::to_value(&encoded).unwrap(),
        json!({"kind":"windows_wide_be_base64","value":"AEMAOgBc2AA="})
    );
    assert_eq!(decode_units(&encoded, Platform::Windows).unwrap(), units);
    assert!(decode_units(&encoded, Platform::Unix).is_err());
    assert!(encode_units(Platform::Windows, &[0]).is_err());
    assert!(encode_units(Platform::Windows, &[0, 0]).is_err());
}

#[test]
fn synthetic_unicode_round_trip_prefers_utf8_and_preserves_code_points() {
    for (platform, units) in [
        (Platform::Unix, vec![0xc3, 0xa9]),
        (Platform::Windows, vec![0, 0xe9]),
    ] {
        let encoded = encode_units(platform, &units).unwrap();
        assert_eq!(
            serde_json::to_value(&encoded).unwrap(),
            json!({"kind":"utf8","value":"é"})
        );
        assert_eq!(decode_units(&encoded, platform).unwrap(), units);
    }
    let nul: NativePath = serde_json::from_value(json!({"kind":"utf8","value":"x\0y"})).unwrap();
    assert!(decode_units(&nul, Platform::Unix).is_err());
    assert!(decode_units(&nul, Platform::Windows).is_err());
}
