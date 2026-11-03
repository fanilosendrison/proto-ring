---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "candidate-package-index"
domain: "proto-ring-agent-facing-api"
severity: "guideline"
name: "Proto-Ring API-01 v1 candidate package"
version: "0.1.0"
---

# Proto-Ring API-01 v1 — candidate contract package

**Status: design candidate, not an accepted contract or implemented API.**

Source reviewed: `fanilosendrison/proto-ring@1eb84e1b4474b155d2271d1decb66d0cd86a555f` (2026-11-03). Work graph: [#134](https://github.com/fanilosendrison/proto-ring/issues/134) → [#135](https://github.com/fanilosendrison/proto-ring/issues/135) → [#136](https://github.com/fanilosendrison/proto-ring/issues/136) → [#137](https://github.com/fanilosendrison/proto-ring/issues/137).

The normative prose candidate is `proto-ring-api-01-v1-spec.md`. `schemas/api.schema.json` owns shared Draft 2020-12 definitions; request, response and local-context schemas reference that one authority by URN. `examples/` contains **illustrative wire-shape fixtures**, not observed engine results. `conformance/schema-cases.json` is executable against these schemas. `conformance/semantic-cases.json` lists **unqualified** semantic and hostile-review requirements; none have been executed against a public API implementation.

## Local schema validation

Run from this package root:

```bash
python conformance/validate-wire-schemas.py
```

Requires Python with `jsonschema` and `referencing` installed. This validates JSON Schema 2020-12 metaconformance, example request/response/local-context wire shapes and positive/negative JSON tests, including a duplicate-key parser rejection. It does **not** qualify the engine, host security, process-output custody, cross-platform path round trips or the semantic cases.

Also run `python conformance/check-identity-vectors.py` to check 13 frozen public-reference hashing known-answer vectors and 11 native-path codec cases. These are synthetic representation checks, not actual engine observations or cross-target engine qualification.

The current shape dataset contains 128 explicit-target cases, plus nine output-range request/response shape scenarios. Discovery validation checks the complete [five-operation discovery example](examples/response-discover.json), self-contained schema closure and exact derivation from the canonical definitions. The semantic dataset defines A01–A70, all `qualified:false`; none is a passed public API runtime test.

Run `python conformance/test_schema_controls.py` for five tooling regression controls, including real CLI mutation tests proving that removing either original response restriction is detected. These controls also reject missing/duplicate/sixth discovery operations, schema drift and incorrect effects/preconditions. They do not execute public API governance operations.

## Authority boundary

Accepted Proto-Ring governance contracts remain normative for their own scope. This candidate adds a versioned public-interface proposal but cannot change underlying governance meaning. Zod is unnecessary for Rust production; a future TypeScript client may generate/use Zod only as a conforming consumer of the accepted JSON Schema. Migration-only `proto-ring-compat` current-main compatibility is not a delivery gate for this new agent-facing API. Existing pinned consumer source commits remain unchanged.

## Before implementation

1. Review this candidate against #134 and source at the recorded exact commit, including all cross-operation semantic conditions not enforceable by JSON Schema alone.
2. Conduct independent hostile review, explicitly marking any unresolved guarantee or output-custody problem **NO_GO**.
3. Record an owner-accepted contract revision and exact-source evidence in #134. Only then authorize API-02 and API-03 implementation. API-04 qualifies real coding-agent use.

The initial normalized candidate was published in Draft PR #139 at `d2f3cb4ff70d6a6281ac836e8252f6ade0191d70`. It has not been accepted or installed as a new API in a consumer repository.

## R1 hostile-review remediation

The first completed independent hostile review, on that exact SHA and tree `01baf220baf4e44601d2d05f501a234eed45ae52`, returned **NO_GO**: one blocker, eight majors, two minors, one observation and two uncertainties. R1 applies the owner-directed corrections F01–F12 and U01–U02, including the clarified six-variant declaration projection; the four targeted inspection selectors and five wire operations remain unchanged.

The revision is identified by the immutable Git commit containing these files and the exact SHA/tree recorded in PR #139. A **fresh independent exact-SHA hostile review is required**. R1 does not implement the Rust host interfaces, execute semantic scenarios, qualify host security, or establish normative acceptance. The production engine and accepted governance contracts remain unchanged.

## Import provenance

The immutable original ZIP has SHA-256 `549f34fb743e2cdaf6fb2c6a9ee05f1ee85b61b8e37b377db6a7f3eef13896d8`.

The initial import renamed `MANIFEST.sha256` to `manifest.sha256`, `conformance/check_identity_vectors.py` to `conformance/check-identity-vectors.py`, and `conformance/validate_wire_schemas.py` to `conformance/validate-wire-schemas.py`. Both Markdown documents received OKF frontmatter; command references were normalized. Four introductory Markdown hard breaks were changed from two trailing spaces to backslash hard breaks. At that import revision, 28 file contents, including both scripts, remained byte-identical to the archive. The original archive was not changed.

R1 intentionally revises the candidate prose, shared schemas, examples and conformance controls under owner authorization. The current `manifest.sha256` identifies every revised package file except itself; it is not the original archive manifest and no longer asserts byte identity of the entire revised package with that archive.

This normalization does not qualify or normatively accept the API. The specification remains a candidate, not approved, not implemented, and not independently qualified.
