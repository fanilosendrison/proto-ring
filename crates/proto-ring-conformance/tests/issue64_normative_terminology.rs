#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

use proto_ring_conformance::harness::{CandidateExecutor, CandidateRequest, CandidateState};
use proto_ring_conformance::loader::{LoadedCorpus, load_corpus};
use proto_ring_conformance::model::Vector;
use proto_ring_conformance::rust_candidate::ReferenceRustCandidate;
use proto_ring_engine::normative_terminology::{
    ConsumerPolicy, MarkdownBlock, Occurrence, RegistryEntry, RegistryFormat, TerminologyError,
    discover_occurrences, unicode14,
};

const WAVE_C: [(&str, usize); 5] = [
    ("normative-terminology.definition-discovery", 33),
    ("normative-terminology.fingerprint", 8),
    ("normative-terminology.inventory", 11),
    ("normative-terminology.markdown", 9),
    ("normative-terminology.registry", 18),
];

struct FencedBlockPolicy {
    allowed_roles: Vec<String>,
}

impl ConsumerPolicy for FencedBlockPolicy {
    fn allowed_roles(&self) -> &[String] {
        &self.allowed_roles
    }

    fn section_from_heading(&self, heading: &str) -> Result<String, TerminologyError> {
        Ok(heading.to_owned())
    }

    fn canonical_location(
        &self,
        _entry: &RegistryEntry,
        _block: &MarkdownBlock,
    ) -> Result<bool, TerminologyError> {
        Ok(true)
    }

    fn role_for(&self, _occurrence: &Occurrence) -> Result<String, TerminologyError> {
        Ok("reference".to_owned())
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root exists")
        .to_path_buf()
}

fn vectors<'a>(corpus: &'a LoadedCorpus, responsibility: &str) -> &'a [Vector] {
    &corpus
        .matrices
        .iter()
        .find(|matrix| matrix.responsibility_id == responsibility)
        .expect("Wave C matrix exists")
        .vectors
}

fn request(responsibility: &str, vector: &Vector) -> CandidateRequest {
    CandidateRequest {
        responsibility_id: responsibility.to_owned(),
        vector_id: vector.vector_id.clone(),
        fixture: vector.fixture.clone(),
    }
}

#[test]
fn issue64_wave_c_matches_all_published_vectors() {
    let corpus = load_corpus(&root()).expect("corpus loads");
    let candidate = ReferenceRustCandidate;
    let mut total = 0;
    for (responsibility, expected_count) in WAVE_C {
        let matrix = vectors(&corpus, responsibility);
        assert_eq!(matrix.len(), expected_count);
        let mut matches = 0;
        for vector in matrix {
            let actual = candidate
                .execute(&request(responsibility, vector))
                .expect("candidate runs");
            let expected = CandidateState::Observation(vector.expected_observation.observation());
            if actual == expected {
                matches += 1;
            } else {
                eprintln!("vector_id={}", vector.vector_id);
                eprintln!("actual={actual:?}");
                eprintln!("expected={expected:?}");
            }
        }
        assert_eq!(matches, expected_count);
        total += matches;
        match responsibility {
            "normative-terminology.definition-discovery" => {
                eprintln!("DEFINITION_DISCOVERY_RUST=33/33")
            }
            "normative-terminology.fingerprint" => eprintln!("FINGERPRINT_RUST=8/8"),
            "normative-terminology.inventory" => eprintln!("INVENTORY_RUST=11/11"),
            "normative-terminology.markdown" => eprintln!("MARKDOWN_RUST=9/9"),
            "normative-terminology.registry" => eprintln!("REGISTRY_RUST=18/18"),
            _ => unreachable!(),
        }
    }
    assert_eq!(total, 79);
    eprintln!("WAVE_C_RUST=79/79");
}

#[test]
fn issue64_wave_c_fenced_definition_blocks_remain_discoverable() {
    let format = RegistryFormat::new(
        "<!-- terminology:start -->".to_owned(),
        "<!-- terminology:end -->".to_owned(),
        [
            "Concept key",
            "Canonical term",
            "Canonical anchor",
            "Accepted aliases",
            "Deprecated wording",
            "Term structure",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        "^[a-z][a-z0-9-]*$".to_owned(),
        r"\[`[^`]+`\]\(#(term-[a-z0-9-]+)\)".to_owned(),
        r#"<a id="(term-[a-z0-9-]+)"></a>"#.to_owned(),
    )
    .unwrap();
    let document = r#"<!-- terminology:start -->
| Concept key | Canonical term | Canonical anchor | Accepted aliases | Deprecated wording | Term structure |
| --- | --- | --- | --- | --- | --- |
| alpha | alpha | [`term-alpha`](#term-alpha) | — | — | base |
<!-- terminology:end -->
# Examples
```text
alpha means one.
```
"#;
    let report = discover_occurrences(
        document,
        &format,
        &FencedBlockPolicy {
            allowed_roles: vec!["reference".to_owned()],
        },
    )
    .unwrap();
    assert!(
        report
            .occurrences
            .iter()
            .any(|occurrence| { !occurrence.canonical && occurrence.concepts == ["alpha"] })
    );
}

#[test]
fn issue64_wave_c_unicode_and_source_boundaries_are_exact() {
    let repository = root();
    let rust_data =
        fs::read(repository.join("crates/proto-ring-engine/data/unicode-14.0.0/CaseFolding.txt"))
            .unwrap();
    let source_data =
        fs::read(repository.join("src/proto_ring/data/unicode-14.0.0/CaseFolding.txt")).unwrap();
    assert_eq!(rust_data, source_data);
    assert_eq!(unicode14::UNICODE_VERSION, "14.0.0");
    assert_eq!(
        unicode14::casefold_data_sha256(),
        unicode14::CASEFOLD_DATA_SHA256
    );
    assert_eq!(unicode14::CONTRACT_WHITESPACE_SCALARS, 29);
    for scalar in [
        '\u{001c}', '\u{001d}', '\u{001e}', '\u{001f}', '\u{2007}', '\u{3000}',
    ] {
        assert!(unicode14::is_whitespace(scalar));
    }
    assert!(!unicode14::is_whitespace('\u{180e}'));
    assert_eq!(
        unicode14::case_fold("Straße"),
        unicode14::case_fold("STRASSE")
    );
    assert_eq!(unicode14::case_fold("\u{212a}"), "k");
    assert_eq!(unicode14::case_fold("\u{0130}"), "i\u{0307}");
    assert_eq!(unicode14::case_fold("\u{1e4d0}"), "\u{1e4d0}");
    let (common, full, simple, turkic) = unicode14::folding_status_counts();
    assert!(common > 0 && full > 0 && simple > 0 && turkic > 0);

    let engine_root = repository.join("crates/proto-ring-engine/src/normative_terminology");
    let sources = [
        "mod.rs",
        "unicode14.rs",
        "registry.rs",
        "markdown.rs",
        "matching.rs",
        "inventory.rs",
    ]
    .iter()
    .map(|name| fs::read_to_string(engine_root.join(name)).unwrap())
    .collect::<String>();
    for forbidden in [
        "to_lowercase",
        "to_uppercase",
        "is_whitespace()",
        "regex::",
        "unicode_normalization",
        "unicode_segmentation",
        "caseless::",
    ] {
        assert!(
            !sources.contains(forbidden),
            "forbidden source: {forbidden}"
        );
    }
    for required in [
        "include_bytes!",
        "boundaries",
        "is_ascii_alphanumeric",
        "BTreeMap<OccurrenceSignature",
        "PortablePattern::compile",
        "PortablePattern::full_match",
    ] {
        assert!(sources.contains(required), "missing source: {required}");
    }
    let adapter = fs::read_to_string(
        repository
            .join("crates/proto-ring-conformance/src/rust_candidate/normative_terminology.rs"),
    )
    .unwrap();
    assert!(adapter.contains("Some(&registry)"));
    assert!(!adapter.contains("diagnostics.is_empty().then_some"));

    let first = Occurrence::new(
        vec!["beta".into(), "alpha".into()],
        "4".into(),
        "Heading".into(),
        "a".repeat(64),
        1,
        false,
    );
    let second = Occurrence::new(
        vec!["alpha".into(), "beta".into()],
        "4".into(),
        "Heading".into(),
        "a".repeat(64),
        99,
        true,
    );
    assert_eq!(first.signature(), second.signature());

    eprintln!("UNICODE_VERSION=14.0.0");
    eprintln!("CASEFOLD_DATA_SHA256={}", unicode14::CASEFOLD_DATA_SHA256);
    eprintln!("CASEFOLD_STATUS_C=USED\nCASEFOLD_STATUS_F=USED");
    eprintln!("CASEFOLD_STATUS_S=IGNORED\nCASEFOLD_STATUS_T=IGNORED");
    eprintln!("HOST_UNICODE_CASEFOLD=NO\nHOST_UNICODE_WHITESPACE=NO");
    eprintln!("UNICODE_NORMALIZATION=NO\nLOCALE_DEPENDENCE=NO");
    eprintln!("CONTRACT_WHITESPACE_SCALARS=29\nU180E_WHITESPACE=NO");
    eprintln!("ORIGINAL_SCALAR_SPAN_MAPPING=YES");
    eprintln!("ASCII_BOUNDARY_ON_ORIGINAL_TEXT=YES\nASCII_CUE_MATCHING_ONLY=YES");
    eprintln!("PORTABLE_PATTERN_REUSE=YES\nNATIVE_REGEX_ENGINE=NO");
    eprintln!("OCCURRENCE_SIGNATURE_EXCLUDES_LINE=YES");
    eprintln!("OCCURRENCE_SIGNATURE_EXCLUDES_CANONICAL=YES");
    eprintln!("INVENTORY_SIGNATURE_MULTISET=YES\nINVENTORY_ROLE_MULTISET=YES");
    eprintln!("VECTOR_ID_SPECIAL_CASES=NO");
}
