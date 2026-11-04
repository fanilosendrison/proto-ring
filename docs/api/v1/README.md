---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "public-api-package-index"
domain: "proto-ring-agent-facing-api"
severity: "guideline"
name: "Proto-Ring API-01 v1 accepted contract package"
version: "0.1.0"
---

# Proto-Ring API-01 v1 — accepted contract package

**Status: ACCEPTED API-01 CONTRACT — NOT IMPLEMENTED — NOT RUNTIME QUALIFIED.**

Source reviewed: `fanilosendrison/proto-ring@1eb84e1b4474b155d2271d1decb66d0cd86a555f` (2026-11-03). Work graph: [#134](https://github.com/fanilosendrison/proto-ring/issues/134) → [#135](https://github.com/fanilosendrison/proto-ring/issues/135) → [#136](https://github.com/fanilosendrison/proto-ring/issues/136) → [#137](https://github.com/fanilosendrison/proto-ring/issues/137).

The normative public-interface contract is `proto-ring-api-01-v1-spec.md`. `schemas/api.schema.json` owns shared Draft 2020-12 definitions; request, response and local-context schemas reference that one authority by URN. `examples/` contains **illustrative wire-shape fixtures**, not observed engine results. `conformance/schema-cases.json` is executable against these schemas. `conformance/semantic-cases.json` lists **unqualified** semantic and hostile-review requirements; none have been executed against a public API implementation.

## Local schema validation

Run from this package root:

```bash
export PYTHONDONTWRITEBYTECODE=1
python -B conformance/validate-wire-schemas.py
python -B conformance/check-identity-vectors.py
python -B -m unittest discover -s conformance -p 'test_*.py'
```

Requires Python with `jsonschema` and `referencing` installed. This validates JSON Schema 2020-12 metaconformance, example request/response/local-context wire shapes and positive/negative JSON tests, including a duplicate-key parser rejection. It does **not** qualify the engine, host security, process-output custody, cross-platform path round trips or the semantic cases.

Also run `python conformance/check-identity-vectors.py` to check 13 frozen public-reference hashing known-answer vectors and 11 native-path codec cases. These are synthetic representation checks, not actual engine observations or cross-target engine qualification.

The current shape dataset contains 197 explicit-target cases, plus nine output-range request/response shape scenarios. Discovery validation checks the complete [five-operation discovery example](examples/response-discover.json), self-contained schema closure and exact derivation from canonical per-operation response restrictions. An independent fixed matrix oracle checks all allowed and forbidden operation/outcome/code combinations against both canonical and advertised schemas. Five local-context controls separately check exact environment-ID uniqueness; duplicate-ID examples are shape-valid but candidate-semantically invalid. The semantic dataset preserves A01–A70 and adds A71–A87, all `qualified:false`; none is a passed public API runtime test.

The unittest command runs ten tooling controls: five original schema/discovery regression tests, three operation/error and interruption tests, and two local-context uniqueness tests. Real CLI mutation tests prove that removing either original response restriction is detected. Controls reject missing/duplicate/sixth discovery operations, schema drift and incorrect effects/preconditions. They do not execute public API governance operations or prove host isolation, process quiescence or a working Rust API.

## Direct local checking limitation

An ordinary same-user mutable checkout does not automatically supply an enforceable coherence capability. Without an actually compliant host-established `exclusive_workspace` or `versioned_workspace`, the standalone check returns `admission_refused/execution_guarantee_unavailable` before launching any validator. The local realization file cannot authorize coherence, privileged dispatch or protected custody. A controlled local host may establish those properties using real OS/substrate mechanisms; no vendor, VM or remote server is mandated. Validator mutations must remain observable to canonical RI drift detection. Non-executing `discover`, `inspect` and `explain` retain their existing behavior.

## Authority boundary

Accepted Proto-Ring governance contracts remain normative for their own scope. This accepted contract adds a versioned public interface but cannot change underlying governance meaning. Zod is unnecessary for Rust production; a future TypeScript client may generate/use Zod only as a conforming consumer of the accepted JSON Schema. Migration-only `proto-ring-compat` current-main compatibility is not a delivery gate for this new agent-facing API. Existing pinned consumer source commits remain unchanged.

## Contract acceptance and downstream gates

The [owner acceptance decision](https://github.com/fanilosendrison/proto-ring/issues/134#issuecomment-6637682924) accepts **only the API-01 contract** at:

- Source: `a1b27ca1e21423973a5557415488e4fcd535d01a`.
- Tree: `5ad78576922a5bfa209322130f989e4f1d555b49`.
- API subtree: `effe8bb52de4899e666c800683ad17db8ff7efeb`.
- Independent exact-source Pi RPC hostile review: **GO**, all four R1 findings resolved, no new findings.
- Review evidence manifest SHA-256: `15b27044d44bba225491a2ca7852d8e9fe96248d8175a0eafba9476d31211c97`.

This subsequent editorial status update changes no accepted API semantics and is not a replacement independent-review subject. API-02 may proceed after formal completion of #134; API-03 remains dependent on API-02. API-04 qualifies the integrated implementation and real coding-agent use. Actual engine-derived cross-target identity/native-path evidence and runtime host security, coherence, process lifetime, capture and custody remain downstream qualification obligations. All 87 semantic scenarios remain unqualified, and no production public API implementation exists.

The initial normalized candidate was published in Draft PR #139 at `d2f3cb4ff70d6a6281ac836e8252f6ade0191d70`. That initial revision was not accepted. No new public API has been installed in a consumer repository.

## R1 hostile-review remediation

The first completed independent hostile review, on that exact SHA and tree `01baf220baf4e44601d2d05f501a234eed45ae52`, returned **NO_GO**: one blocker, eight majors, two minors, one observation and two uncertainties. R1 applies the owner-directed corrections F01–F12 and U01–U02, including the clarified six-variant declaration projection; the four targeted inspection selectors and five wire operations remain unchanged.

R1 at `3e9a6bf5f1eb217ae42401f5b34b64810a07d127` received a second independent **NO_GO**: one blocker, two majors and one minor. Eleven original findings were reported resolved, F06 partially resolved, U01 partially resolved and U02 resolved. One secondary historical F07 citation exceeded the source file's line count; the original report remains unchanged and the separate citation audit does not claim that range was valid.

R2 applies the owner's four bounded decisions: exact canonical operation/error attribution, an exhaustive immutable host-plan view with completion/abort/coherence contracts, rejection of exact duplicate environment IDs, and truthful non-null post-effect interruption metadata. The exact R2 revision identified above subsequently received independent **GO** and the owner's contract-only acceptance. R2 does not implement Rust host interfaces, execute semantic scenarios or qualify host security. The production engine and accepted governance contracts remain unchanged.

## Import provenance

The immutable original ZIP has SHA-256 `549f34fb743e2cdaf6fb2c6a9ee05f1ee85b61b8e37b377db6a7f3eef13896d8`.

The initial import renamed `MANIFEST.sha256` to `manifest.sha256`, `conformance/check_identity_vectors.py` to `conformance/check-identity-vectors.py`, and `conformance/validate_wire_schemas.py` to `conformance/validate-wire-schemas.py`. Both Markdown documents received OKF frontmatter; command references were normalized. Four introductory Markdown hard breaks were changed from two trailing spaces to backslash hard breaks. At that import revision, 28 file contents, including both scripts, remained byte-identical to the archive. The original archive was not changed.

R1 and R2 intentionally revise the candidate prose, schemas, examples and conformance controls under owner authorization. The current `manifest.sha256` identifies every revised package file except itself; it is not the original archive manifest and no longer asserts byte identity of the entire revised package with that archive.

Import normalization did not qualify or accept the API. Contract-only acceptance was subsequently established for the exact reviewed R2 source above. The public API remains not implemented and not runtime qualified.
