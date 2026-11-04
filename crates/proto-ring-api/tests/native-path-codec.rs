use std::path::Path;

use proto_ring_api::NativePath;
use serde_json::json;

#[test]
fn synthetic_unicode_paths_round_trip_without_normalization() {
    for text in [".", "a/../b", " space ", "é/😀", "e\u{301}", "MixedCase"] {
        let native = NativePath::from_path(Path::new(text)).unwrap();
        assert_eq!(
            serde_json::to_value(&native).unwrap(),
            json!({"kind":"utf8","value":text})
        );
        assert_eq!(
            native.to_path_buf().unwrap().as_os_str(),
            Path::new(text).as_os_str()
        );
    }
}

#[test]
fn synthetic_empty_and_nul_paths_are_rejected_in_both_directions() {
    assert!(NativePath::from_path(Path::new("")).is_err());
    assert!(NativePath::from_path(Path::new("a\0b")).is_err());
    let nul: NativePath = serde_json::from_value(json!({"kind":"utf8","value":"a\0b"})).unwrap();
    assert!(nul.to_path_buf().is_err());
}

#[test]
fn synthetic_foreign_native_variant_is_rejected() {
    #[cfg(unix)]
    let foreign = json!({"kind":"windows_wide_be_base64","value":"AGE="});
    #[cfg(windows)]
    let foreign = json!({"kind":"unix_bytes_base64","value":"YQ=="});
    let path: NativePath = serde_json::from_value(foreign).unwrap();
    assert!(path.to_path_buf().is_err());
}

#[test]
fn synthetic_native_unicode_input_is_reemitted_as_utf8() {
    #[cfg(unix)]
    let input = json!({"kind":"unix_bytes_base64","value":"w6k="});
    #[cfg(windows)]
    let input = json!({"kind":"windows_wide_be_base64","value":"AOk="});
    let native: NativePath = serde_json::from_value(input).unwrap();
    let output = NativePath::from_path(&native.to_path_buf().unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(output).unwrap(),
        json!({"kind":"utf8","value":"é"})
    );
}

#[cfg(unix)]
#[test]
fn synthetic_unix_non_utf8_units_round_trip_exactly() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let path = std::path::PathBuf::from(OsString::from_vec(b"dir/\xff".to_vec()));
    let encoded = NativePath::from_path(&path).unwrap();
    assert_eq!(
        serde_json::to_value(&encoded).unwrap(),
        json!({"kind":"unix_bytes_base64","value":"ZGlyL/8="})
    );
    assert_eq!(
        encoded.to_path_buf().unwrap().into_os_string().into_vec(),
        b"dir/\xff"
    );
}

#[cfg(windows)]
#[test]
fn synthetic_windows_unpaired_wide_units_round_trip_exactly() {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    let units = [0x0043, 0x003a, 0x005c, 0xd800];
    let path = std::path::PathBuf::from(OsString::from_wide(&units));
    let encoded = NativePath::from_path(&path).unwrap();
    assert_eq!(
        serde_json::to_value(&encoded).unwrap(),
        json!({"kind":"windows_wide_be_base64","value":"AEMAOgBc2AA="})
    );
    assert_eq!(
        encoded
            .to_path_buf()
            .unwrap()
            .as_os_str()
            .encode_wide()
            .collect::<Vec<_>>(),
        units
    );
}
