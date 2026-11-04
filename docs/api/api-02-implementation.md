---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "implementation-guide"
domain: "proto-ring-public-api"
severity: "guideline"
name: "API-02 nonexecuting implementation"
version: "0.1.0"
---

# API-02 nonexecuting implementation

## Status and authority

`proto-ring-api` and the `proto-ring` binary provide the nonexecuting API-02
**development build**, not a completed or integrated-qualified public API v1.
The package is nonpublishable. Its package version is separate from wire
`api_version:1` and the accepted contract's document version.

The [accepted API-01 package](v1/README.md), at accepted source
`a1b27ca1e21423973a5557415488e4fcd535d01a`, owns the public contract.
Existing [generic contracts](../contracts/) and the canonical Rust engine retain
their semantic authority. This guide records implementation and evidence scope;
it does not amend either authority or change the accepted conformance inventory.

Available operations are discovery, canonical opening, complete or targeted
inspection, and structured declaration/authority explanation. The v1 catalog
still advertises all five operations in their accepted order. Advertisement
means supported contract syntax, not current service availability, applicability,
authorization, successful execution, or provider attestation.

Both API-03 services remain absent. A syntactically valid
`check_repository_integrity` or `read_execution_output` request returns
`controlled_failure/internal_failure`, with `stage:null`, `attempt_id:null`,
`effects_possible:false`, and an explicit missing-API-03 diagnostic. These paths
do not load RGS, realizations, local context, receipts, or output artifacts.
They are integration failures, not RI verdicts or API-03 qualification evidence.

## Architecture and responsibilities

All paths share one chain: syntax adaptation or strict decoding, typed request,
public operation, canonical response construction, and bounded encoding.

- `src/lib.rs` exports the public types and operations. Consumers need no engine
  imports. `src/observation.rs` privately retains one successful canonical RGS,
  provenance and references; construction calls only the engine's RGS loader.
- `src/native-codec.rs`, `src/native-path.rs` and `src/reference-*.rs` implement
  exact native representation and the accepted framed public hashes. Raw engine
  identities are not exposed by observation serialization, getters or Debug.
- `src/models/`, `src/components.rs` and the selector/result modules own typed
  public declarations. `src/projections/` contains shared pure translations;
  full, targeted and explanatory views do not maintain parallel translations.
- `src/inspection-resolve.rs`, `src/explanation-resolve.rs` and
  `src/subject-resolution.rs` resolve retained declarations. Authority role and
  precedence consultations delegate directly to the engine. No winner, total
  order, evidence validity or projection-currentness claim is inferred.
- `src/transport/contract-schema.rs` embeds the canonical schema at compile time.
  `discovery-catalog.rs` mechanically selects branches and local definition
  closures. No runtime schema file, current directory or network retrieval is
  required. Request and response validation use the same canonical authority.
- `src/transport/strict-json.rs` preserves root fields as raw syntax and rejects
  duplicate names before map decoding. `request-decoder.rs` applies exact version,
  schema, typed syntax and native-unit checks before any dispatch.
- `src/transport/dispatch.rs` invokes the public operations. `failure.rs` uses
  canonical response branches rather than another error matrix.
  `response-encoder.rs` and `bounded-buffer.rs` prepare bounded bytes before stdout.
- `src/transport/cli-arguments.rs` is syntax adaptation only. `cli.rs` manages one
  invocation and I/O; `src/main.rs` obtains native arguments with `args_os`.

## Build and reproducible use

Rust 1.99.0 and Git are required. Python dependencies are test-only; neither the
library nor binary invokes a Python production engine.

```bash
cargo build -p proto-ring-api --lib --bin proto-ring --no-default-features --locked
./target/debug/proto-ring discover --json
printf '%s\n' '{"api_version":1,"operation":"discover","input":{}}' |
  ./target/debug/proto-ring json
```

Discovery works outside Git. Inspection and explanation require a repository
whose loaded governance is composable RGM v2, not merely any Git checkout.
The following Unix-shell fixture needs no commit history, credential or provider
installation. Its provider identity is a declaration, not an installation proof.

```bash
binary="$PWD/target/debug/proto-ring"
demo="$(mktemp -d)"
git -C "$demo" init -q
cat > "$demo/AGENTS.md" <<'EOF'
---
repository_governance:
  model_version: 2
  provider:
    id: proto-ring
    binding: {capability: shared_governance_provider, route: registry}
  capabilities:
    governance_authority:
      configuration: {}
      routes: {profile: authority.md}
    shared_governance_provider:
      configuration: {}
      routes: {registry: bindings.md}
---
EOF
cat > "$demo/authority.md" <<'EOF'
---
governance_authority:
  model_version: 1
  sources:
    bindings: {repository_target: bindings.md}
  responsibilities:
    quality:
      roles: {bindings: authority}
      precedence: []
---
EOF
cat > "$demo/bindings.md" <<'EOF'
---
governance_bindings:
  model_version: 1
  source: bindings
  bindings:
    provider:
      kind: executable_provider
      scope: {kind: logical_provider}
      identity:
        repository: example/provider
        commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      authority: {responsibility: quality, source: bindings}
---
EOF
git -C "$demo" -c core.autocrlf=false add -A
"$binary" inspect --repository "$demo" --selector all --json
"$binary" inspect --repository "$demo" \
  --selector-json '{"kind":"binding","id":"provider"}' --json
"$binary" explain --repository "$demo" \
  --query-json '{"kind":"authority_role","responsibility":"quality","source":"bindings"}' \
  --json
cargo run -p proto-ring-api --example inspect-governance --locked -- "$demo"
```

The accepted CLI grammar is:

```text
proto-ring json [--local-context-file NATIVE_PATH]
proto-ring discover --json
proto-ring inspect --repository NATIVE_PATH --selector NAME --json
proto-ring inspect --repository NATIVE_PATH --selector-json JSON --json
proto-ring explain --repository NATIVE_PATH --query-json JSON --json
proto-ring check --repository NATIVE_PATH [--local-context-file NATIVE_PATH] [--expected-observation-json JSON] --json
proto-ring output --receipt HEX64 --validation ID --instance U64DEC --stream stdout|stderr --offset U64DEC --max-bytes INT --json
```

Check/output adapters only reach the missing-service failures described above.
`--local-context-file` confers no authority and is not read by any delivered
nonexecuting path or either missing-service path. Duplicate/unknown flags,
missing values and conflicting selectors produce attributed protocol errors.

A JSON inspection uses the same operation and complete projections:

```json
{
  "api_version": 1,
  "operation": "inspect",
  "input": {
    "repository": {"kind": "utf8", "value": "."},
    "selector": {"kind": "all"}
  }
}
```

Here `.` denotes the invocation's chosen valid governance repository. The
fixture above supports the same request directly:

```bash
(cd "$demo" && printf '%s\n' \
  '{"api_version":1,"operation":"inspect","input":{"repository":{"kind":"utf8","value":"."},"selector":{"kind":"all"}}}' \
  | "$binary" json)
```

Native repository arguments and JSON native path variants preserve exact units
without lossy conversion, Unicode normalization, trimming or case folding.

The [Rust example](../../crates/proto-ring-api/examples/inspect-governance.rs)
imports only root public API types. It opens one observation, requests complete
and targeted inspection, explains a capability and handles typed errors without
classifying diagnostic text. Its stdout is a consumer usage summary, not the
wire transport. Its failure exit is example-specific, not a governance verdict.
A separate temporary Cargo package compiles and executes that same source with
only a path dependency on `proto-ring-api`, using Cargo's offline lock handling.

## Presence, errors, freshness and privileges

`inspect(all)` always contains eight components. Bootstrap, model, authority and
bindings are present. Optional absence is exactly `presence:absent,data:null`.
Targeting an absent optional owner returns
`inapplicable/component_not_declared`; a missing subject in a present owner
returns `not_found/subject_not_found`. A construction failure returns
`controlled_failure/governance_load_failed`, its actual engine stage and no
partial successful value. A missing routed RI file fails during model route
resolution; an existing malformed RI profile fails at `repository_integrity`.
Diagnostic text is non-normative and is never parsed to synthesize stages.

Role consultations preserve all three roles plus `UNKNOWN_RESPONSIBILITY` and
`UNDECLARED`. Unknown responsibility is a successful consultation, not a missing
subject. `DOES_NOT_OUTRANK` does not imply the reverse relation. Incomparable
co-authorities remain incomparable.

A retained observation's inspection/explanation does not reload files or Git.
Every result retains the same public reference, exact resolved root and original
explicit scope order. A new standalone invocation opens a new observation.
`ordinary_capture` is not a coherent-cut guarantee; equal relevant state does
not exclude concurrent changes or an intervening A→B→A. Explicit scope paths
are not a universal inventory of all workspace files. Validation references
bind the full original profile and the actual target's native word width.

Local use runs with the invoking user's actual permissions. There is no delegated
sandbox or new privilege boundary. Opening may read Git and governance locally;
inspection/explanation of retained state is pure. None of these paths executes
consumer validators, generators or provider/network observations. API-03 will
need its own actual host admission, realization, effects and custody boundaries.

## Protocol and resource boundaries

The shared decoder accepts one UTF-8 root object, rejects duplicate keys at all
depths before map construction, and rejects concatenated JSON, unknown fields,
enums, authorizing fields, NaN/Infinity, invalid native units and type/version
coercions. `api_version` is the exact integer token `1`, not `1.0` or `1e0`.
The request limit is 1,048,576 bytes including whitespace; overflow is an explicit
error, never successful prefix dispatch. Operation attribution comes from parsed
root fields or a known CLI verb, not diagnostic-text matching.

The encoded JSON response budget is 16,777,216 bytes; one separate framing LF
follows it. Encoding is bounded before the first stdout write. A complete
inspection/explanation that cannot fit returns `response_too_large`, never a
summary or truncated success. There is one JSON response, without stdout logs.
The canonical schema enforces operation/outcome/code combinations before emission.
After bounded serialization establishes the byte budget, validation uses a JSON
value produced from that exact immutable typed candidate, rather than reparsing
its bytes with a hidden parser-depth limit. Overflow validates only the small
replacement response. Recursive opaque configurations retain every descendant;
80-level sequence and mapping fixtures exercise the depth amplification caused
by wire tags through Rust, JSON and both CLI adapters. This bounded evidence does
not establish arbitrary-depth runtime support: loading, projection, serialization,
value construction, schema traversal and destruction still consume finite stack
and memory. No new configuration-depth limit is introduced by this correction.

Exit 0 means normal structured processing, including controlled failures; exit 2
means an invalid request; exit 70 means internal processing or I/O could not be
completed normally. A mid-write or flush failure cannot roll back bytes already
handed to stdout, and is not reported as successful structured processing.

## Requirement, implementation and evidence map

Test paths below are relative to `crates/proto-ring-api/tests/`, except where an
engine/conformance path is written explicitly. These are bounded implementation
evidence references, not changes to the accepted scenarios' qualification flags.

| Requirement | Implementation | Behavioral evidence / limit |
| ----------- | -------------- | --------------------------- |
| Section 12 A; A01, A20 | Canonical open and exact stage conversion | `observation-opening.rs`, `observation-identities.rs`, `qualification/loading-boundaries.rs`: real failures, optional absence, missing/invalid RI, RGM v1 and no partial value; private load-stage tests cover all ten conversions. |
| Construction drift | Canonical engine controls, not a façade injection API | Existing `issue68_rust_port` executes all 25 RGS vectors including repository/scope drift. `qualification/engine-controls.rs` runs an isolated test-only control package: valid body drift → repository state, corrupt binding → governance bindings, scope drift → composition, plus equal-scope control. |
| Section 12 B; A28 | Native codec and framed references | `native-path-codec.rs`, `observation-identities.rs`, private codec/reference vectors and `cli/equivalence.rs`: exact actual-root/native arguments and synthetic platform cases are separate. |
| A22 | Repository/scope reference domain separation | `qualification/loading-boundaries.rs` distinguishes real roots/scopes and rejects expected-observation input on inspect. Actual check admission/incomparable-observation handling remains API-03; A22 is not fully qualified here. |
| A19, A32 | Complete shared typed projectors and bounded encoder | `inspection/completeness.rs`, `inspection/components.rs`, `transport/bounds.rs`: empty keys, huge signed integers, every declaration, real boundary-size mappings, all 256 RI validations/order and explicit overflow. |
| A19, A32; deep opaque configuration | Shared recursive projections and typed-candidate schema validation after bounded serialization | `deep-configuration.rs`: real 80-level sequence/mapping configurations, inspect all/model/capability and explain capability, complete Rust/JSON/CLI equality and exact provenance; `response-validation.rs` and the exhaustive matrix retain invalid-code rejection and replacement validation. This is finite-depth evidence, not arbitrary-depth qualification. |
| A23 | Direct engine role and precedence consultation | `explanation/authority.rs`: five role outcomes, direct/transitive edges, reverse/incomparable queries and unknown/empty responsibilities. |
| A29 and protocol | Duplicate-safe decoding and syntax-only adapters | `transport/strict-decoding.rs`, `transport/request-adversaries.rs`, `cli/grammar.rs`: nested duplicates, exact bytes/types/version, unknown fields and attribution. |
| A33–A36 | Shared capability/validation/binding/object declarations | `inspection/subjects.rs`, `explanation/declarations.rs`: exact IDs/targets and equality to complete views; validation retains the full profile reference. |
| A37–A38, A64 | Distinct absent owner and missing subject errors | `inspection/subjects.rs`, `explanation/declarations.rs`, `qualification/loading-boundaries.rs`: actual absent/present/invalid owners. |
| A40 | Mechanically derived full catalog and local closures | `transport/discovery.rs`, `cli/grammar.rs`: complete accepted example equality, all ten schema closures, non-Git/other-CWD binary use. |
| A41 observation part only | Original provenance on explanations | `explanation.rs` validation helper and `explanation/retention.rs`: exact root/scope/reference retained after edits/removal. CheckReceipt provenance remains API-03, not qualified here. |
| A61–A62 | Complete projection/evidence subjects and registry metadata | `explanation/declarations.rs`: every mode, authority, carrier, model version, nested fields and explicit nulls; no evidence-validity/currentness assertion. |
| A63 | Exact query/subject identity correlation | `qualification/declaration-correlation.rs`: real outputs from all six families reject outer, inner and subject ID/target mismatches. |
| Section 12 F; A21 | No implicit executable/provider effects | Existing sentinels and Git/carrier preservation tests; isolated canonical controls measure zero provider entries through real façade/JSON paths. Existing `issue133-rgs-observation` qualifies the engine control itself. |
| Section 12 G | Root-API-only external consumer | `qualification/external-consumer.rs` compiles the example in a separate offline package, executes valid fixtures and exercises typed load-error handling. |
| API-02/API-03 boundary | Explicit missing services without context/artifact reads | `transport/dispatch.rs`, `cli/api03-adapters.rs`, `cli/grammar.rs`: valid API-03 syntax, hostile context path, no fabricated receipt/verdict. |
| Section 12 H | Existing repository regression gates | Workspace all-feature Clippy/tests, workspace doctests, neutral Python/package-absence controls and accepted API-01 controls remain mandatory; hosted CI and independent review are separate evidence. |

The isolated engine-control package intentionally enables `conformance-internal`
only in its own test process. The normal API dependency and its production
feature graph do not enable it. It does not inject state into a public
`GovernanceObservation` or add a supported façade construction path.

Synthetic wide-surrogate and 32/64-bit framing vectors are not Windows filesystem
or actual 32-bit engine evidence. Unix arguments can contain non-UTF8 bytes even
when the host filesystem canonicalizes the resolved name to Unicode. Tests
compare exact engine-resolved units; they do not claim such a filename survived
unchanged. Windows CI execution is required for native Windows evidence; adding
a workflow step is not evidence that it ran. No all-87-scenario qualification,
CheckReceipt qualification or full API-v1 qualification is claimed.

## Validation and remaining boundaries

The [workflow](../../.github/workflows/ci.yml) retains the workspace all-target,
all-feature Linux gates, adds a separate normal-production API test/build/lint
path, includes API tests in the existing Windows job, and runs accepted API-01
controls in the Python matrix. Permissions remain read-only. Python stays a
validation/fixture dependency, not a runtime dependency of the public binary.

The normal engine configuration currently emits three warnings tracked by
[Issue #140](https://github.com/fanilosendrison/proto-ring/issues/140). This is a
known independent defect, not a clean-lint claim, suppression or waiver of the
canonical all-feature workspace gate. The API-only strict command uses
`--no-deps`; the engine sources remain unchanged.

Local validation uses the pinned toolchain and a fresh worktree-local test venv:

```bash
python3 -m venv target/dev-venv
target/dev-venv/bin/python -I -B tests/test_package.py
target/dev-venv/bin/python -m pip install -r requirements-test.txt
export PROTO_RING_PYTHON="$PWD/target/dev-venv/bin/python"
export PYTHONDONTWRITEBYTECODE=1
cargo test -p proto-ring-api --all-targets --no-default-features --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo test --workspace --doc --locked
cargo build -p proto-ring-api --lib --bin proto-ring --no-default-features --locked
cargo clippy -p proto-ring-api --all-targets --no-deps --locked -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p proto-ring-api --no-deps --locked
target/dev-venv/bin/python tools/run_python_tests.py
target/dev-venv/bin/python -B docs/api/v1/conformance/validate-wire-schemas.py
target/dev-venv/bin/python -B docs/api/v1/conformance/check-identity-vectors.py
target/dev-venv/bin/python -B -m unittest discover \
  -s docs/api/v1/conformance -p 'test_*.py' -v
git diff --check
```

[Issue #136](https://github.com/fanilosendrison/proto-ring/issues/136) owns actual
check execution, host admission/realization, receipts and output custody/read
services. [Issue #137](https://github.com/fanilosendrison/proto-ring/issues/137)
owns integrated qualification of the completed interface. Local tests, this
guide, a successful build or a schema-shaped response do not establish independent
review, hosted CI, publication, consumer repinning or completion of either Issue.
