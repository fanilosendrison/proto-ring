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
