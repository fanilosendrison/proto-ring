#![forbid(unsafe_code)]

use crate::model::{RecordEntry, TransportValue};

use super::support_authority;

fn record(entries: Vec<(&str, TransportValue)>) -> TransportValue {
    TransportValue::Record(
        entries
            .into_iter()
            .map(|(name, value)| RecordEntry {
                name: name.to_owned(),
                value,
            })
            .collect(),
    )
}

#[test]
fn support_authority_rehydration_does_not_require_target_availability() {
    let temporary = tempfile::tempdir().expect("fixture root must be created");
    let declared_path = "does-not-exist/support-target";
    let support = record(vec![
        ("carrier", TransportValue::String("authority.md".to_owned())),
        ("responsibilities", record(Vec::new())),
        (
            "sources",
            record(vec![(
                "source",
                record(vec![(
                    "repository_target",
                    TransportValue::String(declared_path.to_owned()),
                )]),
            )]),
        ),
    ]);

    let profile = support_authority(temporary.path(), &support)
        .expect("already-loaded support must rehydrate without filesystem routing");
    let binding = profile.sources["source"]
        .repository_target
        .as_ref()
        .expect("support binding must be preserved");
    let expected_target = temporary.path().join(declared_path);

    assert!(!expected_target.exists());
    assert_eq!(binding.declared_path, declared_path);
    assert_eq!(binding.target, expected_target);
    eprintln!("SUPPORT_AUTHORITY_REHYDRATION_DOES_NOT_REQUIRE_TARGET_AVAILABILITY=PASS");
    eprintln!("SUPPORT_AUTHORITY_REHYDRATES_WITHOUT_ROUTING=PASS");
}
