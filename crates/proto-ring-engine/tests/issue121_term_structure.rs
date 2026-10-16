#![forbid(unsafe_code)]

use proto_ring_engine::normative_terminology::{
    DiagnosticCategory, RegistryFormat, parse_registry,
};

fn registry_format() -> RegistryFormat {
    RegistryFormat::new(
        "<!-- terminology:start -->".to_owned(),
        "<!-- terminology:end -->".to_owned(),
        [
            "Key",
            "Canonical term",
            "Anchor",
            "Aliases",
            "Deprecated",
            "Term structure",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        "^[a-z][a-z0-9-]*$".to_owned(),
        r"\[`[^`]+`\]\(#(term-[a-z0-9-]+)\)".to_owned(),
        r#"<a id="(term-[a-z0-9-]+)"></a>"#.to_owned(),
    )
    .expect("generic terminology format is valid")
}

#[test]
fn internal_backticks_are_preserved_in_opaque_term_structures() {
    let markdown = r#"<!-- terminology:start -->
| Key | Canonical term | Anchor | Aliases | Deprecated | Term structure |
| --- | --- | --- | --- | --- | --- |
| alpha | `Alpha` | [`Alpha`](#term-alpha) | — | — | compound; overloaded modifier: `bounded` |
| beta | `Beta` | [`Beta`](#term-beta) | — | — | compound of `raw-llm-inference` |
<!-- terminology:end -->"#;

    let parsed = parse_registry(markdown, &registry_format());

    assert_eq!(parsed.diagnostics, []);
    assert_eq!(parsed.entries.len(), 2);
    assert_eq!(
        parsed.entries[0].term_structure,
        "compound; overloaded modifier: `bounded`"
    );
    assert_eq!(
        parsed.entries[1].term_structure,
        "compound of `raw-llm-inference`"
    );
}

#[test]
fn optional_outer_backtick_pair_is_removed() {
    let markdown = r#"<!-- terminology:start -->
| Key | Canonical term | Anchor | Aliases | Deprecated | Term structure |
| --- | --- | --- | --- | --- | --- |
| alpha | `Alpha` | [`Alpha`](#term-alpha) | — | — | `singular` |
<!-- terminology:end -->"#;

    let parsed = parse_registry(markdown, &registry_format());

    assert_eq!(parsed.diagnostics, []);
    assert_eq!(parsed.entries.len(), 1);
    assert_eq!(parsed.entries[0].term_structure, "singular");
}

#[test]
fn empty_term_structure_after_outer_pair_decoding_is_rejected() {
    let markdown = r#"<!-- terminology:start -->
| Key | Canonical term | Anchor | Aliases | Deprecated | Term structure |
| --- | --- | --- | --- | --- | --- |
| alpha | `Alpha` | [`Alpha`](#term-alpha) | — | — | `` |
<!-- terminology:end -->"#;

    let parsed = parse_registry(markdown, &registry_format());

    assert_eq!(parsed.entries, []);
    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.category == DiagnosticCategory::RegistryStructure)
    );
}
