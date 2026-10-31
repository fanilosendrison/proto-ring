---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "migration-ledger"
domain: "proto-ring-python-retirement"
severity: "strict"
name: "Proto-Ring Python engine retirement ledger"
---

# Proto-Ring Python Engine Retirement Ledger

## Boundary

This ledger starts the reusable removal inventory required by issue #129 and
continued by issue #130. It records dependency evidence at baseline commit
`4ae1a2a549d7dae8e4cce7f727b9bc5330f3823f`. It does not reclassify semantics,
authorize deletion, or replace the responsibility records under
`conformance/v1/responsibilities/`.

Status terms have only the following meanings in this ledger:

- `CONFIRMED`: the baseline contains the cited dependency.
- `TO_INSTRUCT`: issue #130 must classify the dependency before removal.

## Frozen module inventory

The #61 responsibility records confirm 23 frozen production modules:

- `accepted_adr_body.py`
- `adr_metadata.py`
- `canonical_adr.py`
- `evidence_requirements.py`
- `exact_evidence_binding.py`
- `git_whitespace.py`
- `github_authoritative_ref_monotonicity.py`
- `governance_authority.py`
- `governance_bindings.py`
- `governance_bootstrap.py`
- `governance_routing.py`
- `governed_objects.py`
- `normative_terminology.py`
- `normative_terminology_inventory.py`
- `normative_terminology_matching.py`
- `projection_registry.py`
- `repository_governance_model.py`
- `repository_governance_state.py`
- `repository_integrity.py`
- `repository_integrity_evaluation.py`
- `repository_state.py`
- `shared_governance_provider.py`
- `structured_data.py`

The #61 responsibility records confirm three post-baseline carriers:

- `_normative_terminology_unicode.py`
- `portable_pattern.py`
- `portable_pattern_matching.py`

The same records account for 30 responsibilities: 29 `rust_port` records and
one `retire_without_rust_port` record,
`shared-governance-provider.check`. The #69 closeout reports 29 implemented Rust
responsibilities and 1,153 matching vectors.

## Confirmed global-control dependencies

### Rust differential harness

- `CONFIRMED`: `crates/proto-ring-conformance/src/harness.rs` defines
  `PythonBridge` and launches `tests/conformance_python_bridge.py` through the
  interpreter selected by `PROTO_RING_PYTHON`.
- `CONFIRMED`: `crates/proto-ring-conformance/tests/corpus.rs` sends all 1,153
  reduced requests through `PythonBridge` before calling `run_differential`.
- `CONFIRMED`: `crates/proto-ring-conformance/tests/baseline.rs` independently
  exercises the bridge request and output boundary.
- `CONFIRMED`: `crates/proto-ring-conformance/tests/issue91_rgm_witness.rs`
  directly invokes `PythonBridge` for its selected vectors.
- `CONFIRMED`: the standalone path in
  `crates/proto-ring-conformance/src/standalone.rs` receives only
  `responsibility_id`, `vector_id`, and `fixture`; expected observations and
  comparison rules remain in the harness.

### Python bridge and adapters

- `CONFIRMED`: `tests/conformance_python_bridge.py` imports the three adapter
  groups and emits current Python observations.
- `CONFIRMED`: `tests/conformance_vector_execution.py` imports the same adapter
  groups for the Python corpus suite.
- `CONFIRMED`: the adapter groups are
  `tests/conformance_python_adapter_63_64.py`,
  `tests/conformance_python_adapter_65_66.py`, and
  `tests/conformance_python_adapter_67_69.py`, with additional issue-66 support
  modules.
- `CONFIRMED`: these adapters import production modules from `proto_ring`.
- `TO_INSTRUCT`: classify every adapter assertion and helper as retained active,
  covered by an exact Rust control, historical only, or a gap.

### Python test paths

- `CONFIRMED`: `tests/test_conformance_corpus.py` imports
  `conformance_vector_execution` at module collection and test 08 executes the
  current Python candidate.
- `CONFIRMED`: `tests/test_issue66_corpus_closure.py` includes a separate current
  Python observation gate.
- `CONFIRMED`: many unit and contract tests import `proto_ring` directly or read
  files under `src/proto_ring/`.
- `CONFIRMED`: `tools/run_python_tests.py` groups the corpus tests and injects
  test paths for discovery; its own tests inspect the test 08/09/10 grouping.
- `TO_INSTRUCT`: count and classify every affected test method or homogeneous
  assertion group, including mixed tests 08/09/10 and discovery-time imports.
- `TO_INSTRUCT`: classify evidence written under temporary directories and the
  surviving role of `tools/python_test_runner_evidence.py`.

### Packaging and installed resources

- `CONFIRMED`: root `pyproject.toml` declares the installable `proto-ring`
  version `0.17.0` setuptools distribution.
- `CONFIRMED`: package data includes the Python-side Unicode 14.0.0 resource.
- `CONFIRMED`: `tests/test_package.py` requires `proto_ring` to be importable.
- `CONFIRMED`: `tests/test_normative_terminology_unicode.py` resolves the Unicode
  resource through package metadata.
- `TO_INSTRUCT`: classify every package-metadata, resource-loading, editable
  install, and importability assertion before packaging removal.

### Continuous integration

- `CONFIRMED`: `.github/workflows/ci.yml` runs `python -m pip install .` in the
  Python matrix.
- `CONFIRMED`: `.github/workflows/ci.yml` runs a second
  `python -m pip install .` in the Linux Rust job.
- `CONFIRMED`: the Linux Rust job exports `PROTO_RING_PYTHON=python` for the
  differential corpus path.
- `CONFIRMED`: Windows runs targeted Rust conformance tests without installing
  the root Python distribution.
- `TO_INSTRUCT`: classify each Python matrix check and preserve required Ubuntu
  and Windows coverage before either install step is removed.

### Fixture interpreter boundary

- `CONFIRMED`: the Rust reference candidate may invoke a Python interpreter for
  explicitly declared fixture commands in Repository Integrity vectors.
- `CONFIRMED`: this fixture interpreter path is distinct from
  `PythonBridge` and does not import the production `proto_ring` package.
- `CONFIRMED`: the checked-in fixture runner imports test support requiring
  `jsonschema`, `PyYAML`, and `truststore`; an isolated interpreter therefore
  needs those test dependencies even when the `proto-ring` distribution is
  deliberately absent.
- `TO_INSTRUCT`: preserve explicit fixture interpreter dependencies without
  treating them as a retained production Python governance engine.

## Issue #129 preservation boundary

Issue #129 changes no file under these surfaces:

- `src/proto_ring/`
- `docs/contracts/`
- `conformance/v1/`
- `pyproject.toml`
- `.github/workflows/`

No module, package, corpus vector, contract, or CI job is removed by issue #129.

## Issue #130 active-control dispositions

Issue #130 applies the following dispositions at base commit
`3bbfaa4184d39c58a28abf189a2a6cceb51f25dd`. A reference to a Rust vector means
that the current observation is produced by `ReferenceRustCandidate` and
compared by `run_standalone`; expected observations are not candidate inputs.

### RETAIN_ACTIVE

| Assertion group | Protected guarantee | Active destination and proof |
| --- | --- | --- |
| `tests/test_conformance_corpus.py` tests 01–06, 07a, 09, and 11 | Canonical JSON, strict schemas, exact inventory and ownership, qualification, authorities, semantic coverage, fixture allocation, historical observation consistency, transport projection, and repository boundaries | The module has no production-package or adapter import and is discovered with `PYTHONPATH=tests`. Its 10 retained tests execute in an environment where `proto_ring` is neither installed nor importable. |
| `tests/test_issue66_corpus_closure.py` tests 01–03, 07, and 09–12 | Exact 555-vector inventory, raw duplicate discrimination, fixture preparation, metadata and coverage, loaded-state completeness, and identity relations | The module has no production-package or adapter import. Eight retained tests discover and execute without `src/` on `sys.path`. |
| `tests/test_issue90_final_audits.py` projection-support tests | F-001: one support variant per family and exact agreement with the owning Repository Integrity and Governance Bindings loader witnesses | Three restored engine-independent audits load the neutral corpus directly without importing `proto_ring` or a retired adapter. |
| `tests/test_issue90_final_audits.py` exact-cardinality tests | F-002: one canonical vector for each non-mapping distinction and exactly two Repository Governance Model Projection Integrity witnesses | Two restored engine-independent audits compare the exact vector-ID sets. |
| `tests/conformance_corpus_test_support.py`, `tests/conformance_fixture_runner.py`, schema and transport support | Corpus decoding, schema validation, authority snapshot reads, coverage checks, and explicit fixture preparation | Imported only by retained corpus controls. The fixture interpreter may use declared fixture dependencies but does not import `proto_ring`. |
| `tools/run_python_tests.py` and `tools/python_test_runner_protocol.py` | Exact non-zero full-suite discovery, module partitioning, child accounting, and failure propagation | Discovery and child `PYTHONPATH` contain `tests/` only. The runner no longer injects repository `src/` or owns fixed global `/tmp` artifacts. The package-installed suite remains a transitional distribution check. |
| `tests/test_package.py`, direct `proto_ring` unit tests, contract source readers, and `tests/test_normative_terminology_unicode.py` | The Python distribution remains installable and its transitional package behavior remains tested until issue #131 | These tests run only in the normal intermediate environment after installation. They are not part of the isolated active conformance control. |
| Both `python -m pip install .` steps and all current CI jobs | Intermediate package installation, Python matrix, Linux Rust, and Windows coverage | `.github/workflows/ci.yml` remains byte-for-byte unchanged in issue #130. |

### COVERED_BY_RUST

| Former active assertion group | Exact Rust destination | Demonstrated guarantee |
| --- | --- | --- |
| `test_conformance_corpus.py` former tests 08 and 10 and the current-observation half of former test 09 | `crates/proto-ring-conformance/tests/corpus.rs` and `tests/standalone-rust-conformance.rs`; all 29 responsibility matrices and all 1,153 distinct vector IDs | Current candidate execution, exact comparison, complete/non-duplicate inventory, and wrong, missing, duplicate, unimplemented, or fixture-failure rejection. |
| `test_issue66_corpus_closure.py` former tests 04, 04b, 05, 06, and 08 | `standalone-rust-conformance.rs::issue66_support_failure_is_distinct_from_semantic_rejection`, plus vectors `governance-bindings.registry.root-non-mapping`, `exact-evidence-binding.evaluate.requirement-empty-classes`, and all 555 issue-66 vectors in `corpus.rs` | Raw-data duplication remains a Python structural audit; malformed support is a harness error; valid semantic rejection is a candidate observation; current issue-66 results match exact expectations. |
| Issue #91 Projection Integrity / Repository Integrity dependency comparison | `issue91_rgm_witness.rs` vectors `repository-governance-model.load.projection-integrity-requires-repository-integrity` and `repository-governance-model.load.projection-integrity-with-repository-integrity` | Both current Rust witnesses execute and match; no Python MATCH verdict is synthesized. |
| Foreign-catalog adapter tests from issues 99, 100, and 102 | Vectors `governance-bindings.registry.foreign-catalog-repository`, `repository-integrity.profile.foreign-catalog-repository`, and `evidence-requirements.registry.foreign-catalog-repository` | Repository identity is the sole invalid distinction and produces the owned semantic rejection. |
| Issue #94 Projection Registry mutation tests | Vectors `projection-registry.registry.foreign-integrity-repository`, `projection-registry.registry.foreign-catalog-repository`, `projection-registry.registry.foreign-bindings-repository`, and `projection-registry.registry.target-without-responsibility-participation` | Each formerly mutated distinction has an exact current Rust observation and expectation. |
| Issue #98 exact-evidence mutation and transport tests and issue #90 adapter boundary tests | The complete `exact-evidence-binding.evaluate` matrix (44 vectors), including malformed transport and precedence vectors | Transport/support failure remains distinct from owned controlled rejection, and all semantic distinctions are checked by the Rust candidate. |
| Rust Unicode 14.0.0 control | `crates/proto-ring-engine/tests/issue122_unicode_checkout.rs` and `crates/proto-ring-conformance/tests/issue64_normative_terminology.rs` | F-003: the Rust-owned bytes are `-text` and retain independent fixed SHA-256 `a566cd48687b2cd897e02501118b2413c14ae86d318f9abbbba97feb84189f0f`; the conformance test also requires byte equality with the transitional Python package copy until issue #131 disposes of that resource. |

### HISTORICAL_ONLY

| Historical surface at the issue #130 base | Preserved evidence and reason |
| --- | --- |
| `crates/proto-ring-conformance/tests/baseline.rs` | The old bridge request/output protocol is not a current product guarantee. Exact base bytes remain at `3bbfaa4184d39c58a28abf189a2a6cceb51f25dd:crates/proto-ring-conformance/tests/baseline.rs`. |
| `crates/proto-ring-conformance/tests/standalone-rust-conformance-evidence.rs` | This was the one-time #129 differential transition witness. Its published outputs remain under `review-evidence/issue-129/`; replaying the Python channel is not an active control. |
| `tests/conformance_python_bridge.py`, `conformance_vector_execution.py`, all `conformance_python_adapter_*` modules, and `tests/test_conformance_vector_execution.py` | Their sole responsibility was executing or validating the retired Python comparison channel. Exact source and test IDs remain recoverable from the base commit; retaining them active would preserve a second engine. |
| Adapter-specific tests removed from issues 90, 94, 98, 99, 100, and 102 | Their semantic guarantees are listed under `COVERED_BY_RUST`; adapter framing, mutation implementation, and bridge-failure protocol are historical implementation evidence at the base commit. In particular, `test_projection_adapter_does_not_reload_support` remains `HISTORICAL_ONLY` because it inspected the retired Python adapter. |
| `tools/python_test_runner_evidence.py`, `tests/test_python_test_runner_evidence.py`, and fixed `/tmp/proto-ring-61-*` artifacts | They exported the former Python current-observation channel. Issue #129 review evidence preserves the accepted transition bytes; no active proof may consume global artifacts from an earlier run. |

The baseline contains 883 Python test method IDs. The corrected issue #130
candidate contains 807: 83 historical IDs were removed and seven replacement
IDs were added, for a net reduction of 76. The isolated active conformance
control contains 23 tests: the previous 18 plus the five restored F-001 and
F-002 audits. The complete removed-ID set is mechanically recoverable by
comparing `tests/test_*.py` at the base commit with the candidate; the semantic
groups are partitioned above between `RETAIN_ACTIVE`, `COVERED_BY_RUST`, and
`HISTORICAL_ONLY` rather than being equated with the 1,153 Rust vectors.

### GAP

The independent review of the original issue #130 candidate identified three
active-control gaps. F-001 is closed by three restored projection-support
corpus audits, F-002 is closed by two restored exact-cardinality audits, and
F-003 is closed by retaining both the fixed Rust digest and cross-copy byte
equality. No active guarantee is currently classified `GAP`. Any later
validation that finds an uncovered active assertion changes this disposition
and blocks the issue #130 handoff until a targeted regression is added or the
former control is retained.
