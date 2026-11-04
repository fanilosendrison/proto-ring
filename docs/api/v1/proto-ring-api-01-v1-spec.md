---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "public-api-specification"
domain: "proto-ring-agent-facing-api"
severity: "guideline"
name: "Proto-Ring API-01 Public API v1 accepted specification"
version: "0.1.0"
---

# Proto-Ring API-01 — Agent-first public API v1

**Status:** **ACCEPTED API-01 CONTRACT** — **NOT IMPLEMENTED** — **NOT RUNTIME QUALIFIED**; document version `0.1.0`\
**Owner:** Proto-Ring #134 (API-01); downstream implementation #135/#136, qualification #137, parent #28\
**Inspected source:** `fanilosendrison/proto-ring@1eb84e1b4474b155d2271d1decb66d0cd86a555f` (2026-11-03)\
**Wire protocol:** `api_version: 1`; JSON Schema Draft 2020-12\
**Companion files:** `schemas/api.schema.json`, `schemas/request.schema.json`, `schemas/response.schema.json`, `schemas/local-context.schema.json`, `examples/`, `conformance/`.

The first independent review of `d2f3cb4ff70d6a6281ac836e8252f6ade0191d70` returned **NO_GO** with twelve findings and two uncertainties. R1 at `3e9a6bf5f1eb217ae42401f5b34b64810a07d127` (tree `a56f067205703a7e5f40f9003436bedecc894d42`) also received **NO_GO**: one blocker, two majors and one minor. R2 at `a1b27ca1e21423973a5557415488e4fcd535d01a` (tree `5ad78576922a5bfa209322130f989e4f1d555b49`, API subtree `effe8bb52de4899e666c800683ad17db8ff7efeb`) received an independent exact-source **GO**. The [owner acceptance decision](https://github.com/fanilosendrison/proto-ring/issues/134#issuecomment-6637682924) accepts that semantic contract only. This editorial status synchronization does not replace the reviewed source or change its semantics. Document version remains `0.1.0`; API version remains `1`.

This document is the **accepted public-interface contract**. Acceptance and independent GO apply to the exact R2 source above, not to implemented or qualified runtime behavior. This contract does not supersede existing accepted language-neutral governance contracts or change RGM or RGS versions; downstream implementation follows the owner decision and API-02/API-03 dependency order. The public façade must preserve the accepted semantics of RepositoryState, RepositoryGovernanceState, Governance Authority, governed objects, Governance Bindings, Repository Integrity, projections and evidence requirements. The old migration-only `proto-ring-compat` transport is neither an authority nor an ongoing compatibility gate. Existing consumer-owned pins to historical exact commits are not changed.

## 1. Purpose and governing principles

The primary consumers are coding agents in active repository sessions (Pi, Claude Code, Codex and equivalents), with humans and other clients also supported. A public operation must be discoverable, structurally typed, effect-explicit and callable without consulting implementation-private engine modules. Agents select targeted or complete information; Proto-Ring MUST NOT impose inferred relevance filters, hidden summaries, inferred validation membership or silent output truncation. A requested complete view either contains all values within its precisely declared scope or returns a controlled explicit failure.

There are **distinct authorities**: the consumer owns policy and the RI validation profile; a host owns admission, runtime realizations, credentials and actual execution effects; the existing engine owns canonical loading, selection, observation, evaluation and verdict. The agent owns its requests, not privileged permission. A log is an observation, not another RI verdict or an instruction. A single completed check has **one canonical CheckReceipt**.

* **No universal `check_all`**. The engine-module/legacy-adapter inventory does not imply public operations or mandatory obligations.
* **No repair, adoption, bootstrap mutation, generation, historical replay, implicit provider network observation or hidden command execution** in v1.
* `open`/`inspect`/declaration-based `explain` may perform local Git observation subprocesses but MUST NOT execute consumer validators, generators or provider requests.
* A check may execute arbitrary code under its effective local OS permissions; `read-only governance` is **not** a promise of physical mutation prevention.
* No new RGM capability is created. Missing/undeclared/unknown/inapplicable and RI `SATISFIED`/`VIOLATED`/`UNDETERMINED`/`PASS`/`NON_PASS` MUST remain separate.

## 2. Public operation catalog (closed v1)

| Wire operation | Rust surface | Effects | Preconditions | Result |
| -------------- | ------------ | ------- | ------------- | ------ |
| `discover` | `discover()` | Pure metadata | v1 supported | Finite operation catalog with full machine-readable request and response schemas |
| `inspect` | `open(repository).inspect(selector)` | Local governance observation/Git reads; no consumer validators or network | RGS load succeeds; repository readable/authorized | One state-bound typed inspection |
| `explain` | `GovernanceObservation.explain(query)` | Local governance observation for standalone CLI; pure projection on loaded Rust observation | RGS load and query resolution | Deterministic structured facts, no inferred consumer meaning |
| `check_repository_integrity` | `check_repository_integrity(request, host)` | Local observation and explicit consumer command execution | RI declared, complete plan admitted, required host guarantees available | One canonical `CheckReceipt` or explicit non-verdict outcome |
| `read_execution_output` | `read_execution_output(request, custodian)` | Protected local artifact read; **no command execution** | Authorized retained receipt and output | Exact captured byte range, no new verdict |

`open` is an in-process typed Rust constructor, **not** a sixth wire operation and not a persistent process session. Every standalone JSON/CLI `inspect`/`explain`/`check` separately establishes current observations. Later MCP/HTTP/other language adapters may wrap these contracts but are not required in v1. Accepted ADR Body Immutability, Git Whitespace, Exact Evidence Binding and GitHub ARM **are not independently invocable public v1 operations**; their existing engine mechanisms are not removed.

`discover` MUST return exactly five entries in the table order. Each has `operation`, `effects`, ordered `preconditions`, `applicability`, `authorization`, `request_schema`, and `response_schema`. The fixed effects and preconditions are:

| Operation | Effects | Preconditions |
| --------- | ------- | ------------- |
| `discover` | `pure_projection` | none |
| `inspect` | `local_observation` | `repository_read`, `rgs_load` |
| `explain` | `local_observation`, `pure_projection` | `repository_read`, `rgs_load`, `query_resolution` |
| `check_repository_integrity` | `local_observation`, `consumer_command_execution` | `repository_read`, `coherent_workspace`, `rgs_load`, `ri_declared`, `profile_admission`, `realization`, `effect_plan_admission`, `receipt_budget` |
| `read_execution_output` | `read_host_output` | `receipt_access`, `retained_output` |

These identifiers state requirements, not established permission. Catalog membership states `SUPPORTED`, not applicable or authorized. Applicability uses the closed identifiers `always`, `rgs_load`, `query_resolution`, `ri_declared`, `retained_output`; authorization separately uses `none`, `repository_read`, `effect_plan_admission`, `receipt_access`. Each entry's exact values are constants in `$defs/catalog_entry`.

`schemas/api.schema.json` is the sole definition authority. Operation-specific discovery schemas are mechanically derived: select the canonical request branch; select its success response and canonical error branches attributed to that operation; include the transitive closure of referenced canonical `$defs` under the same local reference names. Unassigned `operation:null` protocol failures remain described by the global envelope, not an attributed operation result. Every returned schema is self-contained Draft 2020-12, with no network reference acquisition. `conformance/schema_catalog.py` derives and checks this representation; `examples/response-discover.json` freezes the complete five-entry example. Empty schema objects, stale copies, duplicate entries and a sixth operation are invalid discovery descriptions.

## 3. Protocol envelope, errors and versioning

Every JSON invocation consists of **one UTF-8 request object on stdin, one UTF-8 response object on stdout followed by one LF**, then process exit. No daemon, shared in-memory session, implicit cached governance state, streaming JSON lines, HTTP or remote service. Validator stdout/stderr never contaminate protocol stdout. CLI convenience verbs construct the same requests and use the same projection and evaluation code.

Request grammar is the disjoint union by `operation` in `schemas/request.schema.json`. Required envelope: `api_version` (exact JSON integer `1`), `operation` (one of five names), `input` (operation-specific object). All listed properties are required unless explicitly optional in the schema; no unknown fields, unknown enum values, duplicate keys at **any** JSON object depth, NaN, Infinity, implicit type conversions or malformed UTF-8. Duplicate keys MUST be rejected **before** decoding into maps; JSON Schema cannot detect keys already collapsed by a decoder. Whole request maximum: **1 MiB** (1,048,576 bytes), a resource limit, not a semantic governor. An error MUST be explicit; no request truncation.

Successful response grammar: `{ "api_version":1, "operation":..., "outcome":"result", "value":... }`. Non-result grammar: `{ "api_version":1, "operation": <name-or-null>, "outcome": <failure-discriminant>, "error":{...} }`. `value` and `error` never coexist. `operation:null` is permitted only when a malformed request cannot be assigned to an operation. Stable `outcome`: `result`, `inapplicable`, `not_found`, `admission_refused`, `stale_observation`, `incomparable_observation`, `controlled_failure`, `execution_incomplete`, `protocol_error`.

Stable error codes for v1: `invalid_request`, `unsupported_version`, `response_too_large`, `governance_load_failed`, `repository_integrity_not_declared`, `component_not_declared`, `subject_not_found`, `repository_not_authorized`, `profile_not_authorized`, `execution_plan_not_authorized`, `execution_guarantee_unavailable`, `local_context_invalid`, `stale_observation`, `incomparable_observation`, `output_not_started`, `output_expired`, `output_access_refused`, `output_unavailable`, `range_not_retained`, `invalid_output_range`, `execution_incomplete`, `host_guard_lost_after_start`, `receipt_persistence_failed`, `internal_failure`. The canonical machine inventory is `$defs/error_code`, composed from the named `$defs/error_codes_<outcome>` families. Each error response references its family rather than maintaining a separate code list. `host_guard_lost_after_start` belongs exclusively to `execution_incomplete`. The outer `outcome` and inner `code` MUST agree with the operation's corresponding semantics; neither code nor process exit supplies an RI verdict.

`error.stage` is either `null` or an exact existing `RepositoryGovernanceStage` (`repository_state`, `bootstrap`, `repository_governance_model`, `governance_authority`, `governed_objects`, `governance_bindings`, `repository_integrity`, `projection_integrity`, `evidence_requirements`, `composition`). `error.detail` is non-normative nullable text and MUST NOT be parsed to synthesize a stable category; privileged hosts may redact it. Every `execution_incomplete` requires the actual admitted non-null lowercase hex64 `attempt_id` and `effects_possible:true`. Controlled `receipt_persistence_failed` and `local_context_invalid` require established `effects_possible:false`. Other error families retain their declared nullability where information is genuinely unavailable. A controlled RGS load failure never returns a partial RGS.

The canonical response branches in `api.schema.json` restrict operation, outcome and code together. Each code has one `$defs/error_code_<code>` constant; each outcome family references those constants. Operation branches reference only their permitted subsets. The following v1 matrix is explanatory prose derived from that authority, not a second dispatch implementation:

| Operation | Permitted non-result outcomes and codes |
| --------- | --------------------------------------- |
| `discover` | `protocol_error`: invalid_request, unsupported_version; `controlled_failure`: internal_failure |
| `inspect` | `inapplicable`: component_not_declared; `not_found`: subject_not_found; `admission_refused`: repository_not_authorized; `controlled_failure`: governance_load_failed, response_too_large, internal_failure; `protocol_error`: invalid_request, unsupported_version |
| `explain` | `inapplicable`: component_not_declared; `not_found`: subject_not_found; `admission_refused`: repository_not_authorized; `controlled_failure`: governance_load_failed, response_too_large, internal_failure; `protocol_error`: invalid_request, unsupported_version |
| `check_repository_integrity` | `inapplicable`: repository_integrity_not_declared; `admission_refused`: repository_not_authorized, profile_not_authorized, execution_plan_not_authorized, execution_guarantee_unavailable; `stale_observation`: stale_observation; `incomparable_observation`: incomparable_observation; `controlled_failure`: governance_load_failed, local_context_invalid, response_too_large, receipt_persistence_failed, internal_failure; `execution_incomplete`: execution_incomplete, host_guard_lost_after_start, receipt_persistence_failed; `protocol_error`: invalid_request, unsupported_version |
| `read_execution_output` | `not_found`: subject_not_found; `controlled_failure`: output_not_started, output_expired, output_access_refused, output_unavailable, range_not_retained, invalid_output_range, internal_failure; `protocol_error`: invalid_request, unsupported_version |

Each operation additionally admits its own successful `result`. No other combination is permitted. Unassigned `operation:null` admits only `protocol_error/invalid_request` or `protocol_error/unsupported_version`, and only when the malformed request cannot be assigned to an operation. It is excluded from operation-specific discovery schemas. `controlled_failure/receipt_persistence_failed` is pre-effect only; if any execution effects are possible, use `execution_incomplete` with the admitted attempt ID instead.

Exit code **0** means a structured protocol response was successfully emitted, including a returned RI `NON_PASS`, admission refusal or `execution_incomplete`. Exit **2** means invalid wire request. Exit **70** means an internal failure prevented normal structured processing; an exit code alone cannot prove no commands started. No automatic retry after an unknown potentially effectful attempt. Response maximum: **16 MiB** (16,777,216 encoded bytes). A requested complete observation view exceeding this limit MUST produce `response_too_large`, not a successful partial representation; the agent may make targeted observation queries. Checks instead require the pre-effect complete-receipt budget in Section 8; targeted observation queries cannot recover a truncated check receipt.

The wire version is separate from RGM versions, RepositoryGovernanceState v1 and any historical migration transport. After acceptance or public release, an incompatible schema or reference algorithm requires a new wire/reference version and corresponding conformance vectors; no silent coercion. This accepted API-01 contract retains the owner-fixed API version `1` and document version `0.1.0`; it is not a software release.

## 4. Exact NativePath type (lossless)

`NativePath` is a disjoint tagged union:

```json
{"kind":"utf8","value":"."}
{"kind":"unix_bytes_base64","value":"Li4u"}
{"kind":"windows_wide_be_base64","value":"AC4="}
```

The latter two example values are **illustrative encodings**, not equivalent spellings of `.`. UTF-8 input converts to the platform's native path without case folding, Unicode normalization, trimming, alias inference or loss. On Unix native path units are raw `OsStr` bytes; on Windows they are exactly the big-endian bytes of the `u16` sequence emitted by `OsStr::encode_wide()`, matching the engine's existing `repository_state::path::os_units` convention. A mismatched native platform variant is rejected. Base64 uses RFC 4648 standard alphabet with padding and canonical re-encoding equality; reject malformed padding, unused nonzero pad bits, invalid NUL units, empty path and odd-length Windows bytes. Decode exact units; never use lossy Unicode conversions. Unsupported representability is a controlled error, not a fabricated alternative path.

Output MUST emit `utf8` when exact Unicode conversion is possible, otherwise the matching platform-native base64 form, and MUST preserve the same path identity. A readable display string, if supplied by an external client, has no identity authority. Exact path/root containment and observation are established by the engine, not by a second JSON path resolver.

A `NativePath` is a location representation, **not a global repository ID**. The RGS reference uses the resolved exact worktree root, not a remote URL or mutable branch reference.

## 5. Exact public observation/reference algorithm

Internal `RepositoryStateIdentity` contains framed observations and can include entire governed file bytes. **Never emit its raw bytes** in the public interface. Define `frame(x) = u64_be(length_of_x) || x`. Define `H(domain, parts)` as `SHA256(frame(UTF8("proto-ring-api01-v1")) || frame(UTF8(domain)) || u64_be(number_of_parts) || frame(part_1) || ... || frame(part_n))`. All SHA-256 outputs are 32 raw octets when used internally and lowercase 64-character hex when serialized. `u64_be` means eight exact big-endian octets. The scheme is versioned; a change to an identity's underlying domain representation requires an incompatible reference version, not a silent digest change. The byte-for-byte known-answer tests in `conformance/identity-vectors.json` are controlling **for this proposed public hash algorithm only**; they do not claim to be observed Rust engine-state digests.

* `repository_ref = H("rgs-repository-v1", [UTF8(platform), native_units(resolved_root)])`.
* `scope_ref = H("rgs-scope-v1", [repository_ref_raw, u64_be(scope_count), native_units(path_1), ..., native_units(path_n)])`, with exact RGS explicit-scope paths sorted by native-unit bytes, duplicate identities rejected.
* `state_ref = H("rgs-state-v1", [repository_ref_raw, scope_ref_raw, internal_rgs_repository_state_identity_bytes])`.
* Public RGS observation JSON: `{ "format":"proto-ring-rgs-ref-v1", "platform":"unix"|"windows", "repository":hex, "scope":hex, "state":hex, "coherence_basis":"ordinary_capture"|"host_established" }`.
* `ri_scope_ref = H("ri-scope-v1", [repository_ref_raw, u64_be(scope_count), native_units(path_1), ...])` over the **RI evaluation scope**, not the RGS scope. Deduplicate only exact native path identities and sort by native-unit byte encoding before hashing. Do not depend on `BTreeSet<PathBuf>` order, and do not change the engine governed-path vector or evaluation order. In particular `a-b` precedes `a/b` in the public byte ordering.
* `ri_state_ref = H("ri-state-v1", [repository_ref_raw, ri_scope_ref_raw, internal_ri_state_identity_bytes])`.
* `ri_profile_ref = H("ri-profile-v1", [u64_be(native_word_bits), internal_profile_identity_bytes])`, with `native_word_bits` exactly `32` or `64`. JSON is `{ "format":"ri-profile-v1", "native_word_bits":32|64, "sha256":hex }`. Bind the actual engine target width; preserve its original internal `usize` framing without reconstruction or normalization. Different widths are not interchangeable even for identical persistent semantics. Actual engine-derived cross-target comparison remains a qualification requirement. `ri_context_ref = H("ri-context-v1", [internal_evaluation_context_identity_bytes])`.

Every successful RGS-bearing `inspect`, `explain` and completed `CheckReceipt` MUST expose the actual RGS root and explicit paths separately from these digests. Discovery and output retrieval do not load RGS and do not manufacture RGS provenance. A conditional request reference is **only a comparison hint**, never access or execution authorization; untrusted callers may forge its values. Comparable RGS references must have equal `format`, `platform`, `repository` and `scope`; their `state` fields are then compared. A mismatch in domain -> `incomparable_observation`; a mismatch in `state` alone -> `stale_observation`. Equal comparable values establish equality of the **contract-relevant observed state**, assuming cryptographic collision resistance; they do not establish global transactional coherence or absence of intervening A→B→A events. Return-to-A is acceptable between independent observations.

RGS, RI scope, profile, context and baseline/final RI identities are distinct domains. Do not compare across domains or claim that RGS and RI cover the same paths. An engine-created context identity **does not** prove the exact program or process environment that ran; it is not host admission. Claims of an isolated/coherent cut require independent host evidence that actually covers the claimed boundary. `coherence_basis:host_established` MUST NOT be populated solely because two ordinary captures compared equal. Non-executing local observations may report `ordinary_capture` without asserting absence of concurrent writers or in-evaluation ABA. A completed qualified RI receipt must instead carry an actually established coherence basis as specified below; ordinary capture is not sufficient check admission.

## 6. Structured values, inspection and explanation

`StructuredValue` remains the tagged union `null`, `boolean`, `integer`, `string`, `sequence`, `mapping`. Integers use canonical arbitrary-precision signed decimal strings, never IEEE-754 numbers. Mapping entries have unique exact string keys sorted by Rust/Unicode lexical order. Empty configuration keys are valid and MUST survive `inspect(model)` and `inspect(all)` unchanged; identifier restrictions do not apply to opaque configuration descendants. This projection does not redefine accepted Structured Data syntax.

All successful inspection and explanation results carry the separate RGS observation reference and the flattened `$defs/rgs_provenance` fields: `repository:NativePath`, the exact resolved RGS worktree root, and `scope_paths:NativePath[]`, its exact explicit scope. No digest substitutes for these concrete paths. Component carriers retain exact loaded paths. Consumer declaration strings are not inferred product semantics, provider-installation attestations, projection-currentness claims or evidence-validity claims.

`inspect_value` is a closed discriminated union. Every variant contains `observation`, `repository`, `scope_paths`, `selector`, `result_kind`:

- `result_kind:all`, `selector.kind:all`: only `components` is added. All eight keys are required: `bootstrap`, `model`, `authority`, `governed_objects`, `bindings`, `repository_integrity`, `projections`, `evidence_requirements`. Optional absence is exactly `{presence:absent,data:null}`; mandatory bootstrap/model/authority/bindings must be present.
- `result_kind:component`: a non-parameterized component selector and `component:{name,data}`. `name` equals the selected component, and `data` is its complete present typed projection. Successful absent targeted components are forbidden.
- `result_kind:subject`: one of the unchanged four parameterized selectors (`capability`, `validation`, `binding`, `governed_object`) and only `subject`, the matching exact `$defs/subject_projection`. Never fabricate or filter a whole consumer profile to answer a subject query.

The canonical `subject_projection` has exactly six variants:

| Kind | Required fields in addition to `kind` |
| ---- | ------------------------------------- |
| `capability` | `id`, `model_version`, `declaration` (configuration and resolved routes) |
| `validation` | `id`, `profile_ref` (full original profile), `declaration:validation_decl` |
| `binding` | `id`, `registry_source`, `declaration:binding_decl` |
| `governed_object` | `target`, `declaration:governed_object_decl` |
| `projection` | `id`, `registry_model_version:1`, `registry_authority:authority_ref`, `carrier:NativePath`, `declaration:projection_decl` |
| `evidence_requirement` | `id`, `registry_model_version:1`, `registry_authority:authority_ref`, `carrier:NativePath`, `declaration:evidence_requirement_decl` |

All variants are closed. IDs/targets exactly match the requested subject. Full and targeted projections reference the same extracted declaration definitions, not parallel DTO definitions. `projection_decl` preserves responsibility, canonical_source, secondary_source, mode, validation, generator_source, boundary_source, target, binding, including all existing nullability. `evidence_requirement_decl` preserves responsibility, instances, evidence_classes, subject_source, context, candidate_source, target, including nested shapes. Registry model, authority and carrier come directly from the successfully loaded registry, never from an identifier. The registry mapping key supplies the ID outside the declaration.

`explain` retains its exact finite query set: the six declaration kinds above plus `authority_role` and `authority_precedence`. A declaration explanation is `{kind:declaration,subject:<exact query>,declaration:<matching subject_projection>}`. The result contains `observation`, RGS provenance, `query`, `explanation`. Schema branches correlate query and declaration kinds. Exact ID/target equality, including the echoed query, is an additional semantic requirement. No arbitrary StructuredValue fact container, invented reason for consumer necessity, inferred authority winner or LLM-generated normative prose is permitted. Role outputs retain `authority`, `secondary_representation`, `non_authoritative`, `UNKNOWN_RESPONSIBILITY`, `UNDECLARED`. Authority precedence retains the engine's actual partial-order relation and does not impose a total order.

For targeted inspection and all declaration explanations: absent owning optional component/registry gives `inapplicable/component_not_declared`; missing ID/target in a present component gives `not_found/subject_not_found`; failed RGS construction gives `controlled_failure/governance_load_failed`, with the existing stage and no successful partial state. In-process Rust inspection/explanation may reuse the same retained RGS observation without claiming it stays current. Standalone invocations independently load and expose their own provenance.

## 7. Complete-plan admission and the concrete Rust host boundary

The public check input remains exactly `{repository:NativePath,expected_observation:ObservationReference|null}`. It contains no subset, command prefix, environment map, executable path, host configuration path, privilege flag or agent-minted permit. The agent requests; the consumer profile alone owns mandatory membership and policy; the host admits actual effects; the engine alone resolves, observes, evaluates, aggregates and supplies the RI verdict.

RI undeclared is `inapplicable` with no verdict. A declared malformed/missing profile is a controlled RGS construction failure. Empty profiles preserve canonical possible vacuous PASS, and successful zero-instance `for_each` preserves SATISFIED without an invented file-presence obligation. Permitted missing realizations retain UNDETERMINED/NON_PASS semantics; they are not automatically security refusals.

### 7.1 Intended implementation-boundary signatures

These signatures specify future boundaries, not exports already implemented by the engine. No Rust implementation is introduced by this contract.

```rust
pub trait CheckHost {
    type Authorization;
    type Workspace: CoherentWorkspace;
    type Permit;
    type Process: RunningValidator;

    fn authorize_repository(
        &mut self,
        requested: &Path,
        attempt: &AttemptId,
    ) -> Result<Self::Authorization, AdmissionError>;

    fn acquire_workspace(
        &mut self,
        authorization: Self::Authorization,
    ) -> Result<Self::Workspace, AdmissionError>;

    fn realizations(
        &mut self,
        workspace: &mut Self::Workspace,
        profile: &ConsumerIntegrityProfile,
    ) -> Result<EvaluationContext, AdmissionError>;

    fn admit_plan(
        &mut self,
        workspace: &mut Self::Workspace,
        plan: &ResolvedCheckPlan,
        budget: &ReceiptBudget,
    ) -> Result<Self::Permit, AdmissionError>;

    fn launch(
        &mut self,
        workspace: &mut Self::Workspace,
        permit: &mut Self::Permit,
        effect: PlannedEffect<'_>,
    ) -> Result<Launch<Self::Process>, HostAbort>;
}

pub trait CoherentWorkspace {
    fn root(&self) -> &Path;
    fn evidence(&self) -> CoherenceEvidenceView<'_>;
    fn require_coverage(
        &mut self,
        required: RequiredCoverage<'_>,
    ) -> Result<(), HostAbort>;
    fn ensure_active(
        &mut self,
        checkpoint: CoherenceCheckpoint,
    ) -> Result<(), HostAbort>;
}

pub enum Launch<P> {
    ProvenNotStarted,
    Started(P),
}

pub trait RunningValidator {
    fn wait_and_capture(
        self,
        capture: &mut dyn CaptureSink,
    ) -> Result<CompletedProcess, HostAbort>;
}

pub struct ResolvedCheckPlan {
    subject: CheckSubject,
    realized: RealizedIntegrityProfile,
}

pub struct ObligationKey {
    pub validation_id: String,
    pub instance_index: u64,
}

pub struct PlannedEffect<'a> {
    pub key: &'a ObligationKey,
    pub root: &'a Path,
    pub obligation: &'a RuntimeCommandObligation,
}

pub enum GuardedEvaluation {
    Completed {
        result: IntegrityResult,
        captures: CaptureIndex,
    },
    Interrupted {
        reason: HostAbort,
        effects_possible: bool,
    },
}

pub enum ProcessObservation {
    ProvenNotStarted,
    Completed(CompletedProcess),
}

pub struct CheckSubjectView<'a> {
    pub request_path: &'a Path,
    pub requested_root: &'a Path,
    pub effective_root: &'a Path,
    pub root_binding: RootBindingView<'a>,
    pub governance: &'a RepositoryGovernanceState,
    pub consumer_profile: &'a ConsumerIntegrityProfile,
    pub profile_identity: &'a ProfileIdentity,
    pub evaluation_context_identity: &'a EvaluationContextIdentity,
    pub ri_scope: &'a [PathBuf],
    pub attempt: &'a AttemptId,
}

impl ResolvedCheckPlan {
    pub fn subject(&self) -> CheckSubjectView<'_>;
    pub fn validations(
        &self,
    ) -> impl ExactSizeIterator<Item = ValidationPlanView<'_>> + '_;
}

pub enum InstanceResolutionState {
    ResolvedWithObligations,
    ResolvedZeroInstances,
    InstanceResolutionFailed,
}

pub struct ValidationPlanView<'a> {
    declared: &'a ValidationDefinition,
    canonical: &'a RuntimeValidation,
    position: usize,
}

impl ValidationPlanView<'_> {
    pub fn validation_id(&self) -> &str;
    pub fn canonical_position(&self) -> usize;
    pub fn prerequisites(&self) -> &BTreeSet<String>;
    pub fn resolution(&self) -> InstanceResolutionState;
    pub fn obligations(
        &self,
    ) -> impl ExactSizeIterator<Item = ObligationPlanView<'_>> + '_;
}

pub struct ObligationPosition {
    pub validation_position: usize,
    pub instance_index: u64,
    pub plan_position: usize,
}

pub enum ObligationPlanView<'a> {
    Executable(ExecutableObligationView<'a>),
    MissingRealization(UnlaunchableObligationView<'a>),
    KnownUnlaunchable(UnlaunchableObligationView<'a>),
}

pub struct ExecutableObligationView<'a> {
    pub key: ObligationKey,
    pub position: ObligationPosition,
    pub effective_root: &'a Path,
    pub working_directory: &'a Path,
    pub declared_environment_id: &'a str,
    pub program: &'a str,
    pub arguments: EffectiveArgumentsView<'a>,
    pub process_environment: &'a BTreeMap<String, String>,
    pub profile_identity: &'a ProfileIdentity,
    pub evaluation_context_identity: &'a EvaluationContextIdentity,
}

pub struct EffectiveArgumentsView<'a> {
    pub prefix_tail: &'a [String],
    pub realized_arguments: &'a [String],
}

pub struct UnlaunchableObligationView<'a> {
    pub key: ObligationKey,
    pub position: ObligationPosition,
    pub declared_environment_id: &'a str,
    pub canonical: &'a RuntimeCommandObligation,
}

pub struct CompletedProcess {
    pub process_started: bool,
    pub exit_success: bool,
    pub exit_code: Option<String>,
    pub capture_metadata: CaptureReference,
    pub effect_lifetime_closed: Box<dyn EffectLifetimeClosed>,
}

pub trait EffectLifetimeClosed {
    fn attempt(&self) -> &AttemptId;
    fn obligation(&self) -> &ObligationKey;
    fn generation(&self) -> &ProtectedGeneration;
}

pub enum ProcessStartKnowledge {
    DefinitivelyExcluded,
    PotentiallyOccurred,
}

pub enum HostAbortReason {
    CoherenceLost,
    AdmissionRevoked,
    EffectBindingInvalid,
    ProcessStartIndeterminate,
    ProcessCompletionIndeterminate,
    DescendantEffectsUncontained,
    HostObservationUnavailable,
}

pub struct HostAbort {
    pub reason: HostAbortReason,
    pub process_start: ProcessStartKnowledge,
    pub attempt: AttemptId,
}

pub enum CoherenceClass {
    ExclusiveWorkspace,
    VersionedWorkspace,
}

pub struct RootBindingView<'a> {
    pub requested_root: &'a Path,
    pub effective_root: &'a Path,
    pub protected_subject: &'a ProtectedSubjectIdentity,
    pub generation: &'a ProtectedGeneration,
}

pub struct CoherenceEvidenceView<'a> {
    pub effective_root: &'a Path,
    pub root_binding: RootBindingView<'a>,
    pub class: CoherenceClass,
    pub protected_subject: &'a ProtectedSubjectIdentity,
    pub generation: &'a ProtectedGeneration,
    pub coverage: &'a dyn ProtectedCoverage,
}

pub enum RequiredCoverage<'a> {
    GovernanceAndRepositoryState,
    RiScope(&'a [PathBuf]),
}

pub trait ProtectedCoverage {
    fn covers(&self, required: RequiredCoverage<'_>) -> bool;
}

pub enum CoherenceCheckpoint {
    BeforeRgs,
    BeforeRealization,
    AfterRealization,
    BeforePlanAdmission,
    BeforeEachLaunch,
    AfterEachEffectLifetime,
    BeforeRequiredRiCapture,
    AfterFinalRiObservation,
}
```

The Rust-shaped signatures above are future boundary contracts with method bodies omitted, not compilable implementations or existing exports. `CheckSubject` is the private owner of the immutable values borrowed by `CheckSubjectView`, including the actual EvaluationContext used in realization, not only its identity. `ResolvedCheckPlan` retains its private `subject` and canonical `RealizedIntegrityProfile`; no second plan becomes authoritative. View values are read-only borrows derived from those same objects and their original consumer declarations. Fabricating a view does not mint a plan or permit.

`request_path` preserves the decoded request spelling. `requested_root` is its canonically resolved authorized worktree root; `effective_root` is the root actually observed and used for execution. `RootBindingView` binds both to one host-protected subject and generation. They must be identical unless the authorized host actually establishes their protected-workspace relationship. A relocated workspace is never inferred from path similarity. Public RGS provenance and reference hashing use the actual effective root; expected-observation comparison retains its existing domain rules, without silently equating different roots. The borrowed RGS exposes the exact observation and concrete provenance; the profile/context identities and canonical RI scope remain separate.

`AttemptId` is the 32-octet admitted invocation identity defined in Section 8. `CaptureReference` identifies capture metadata for exactly that attempt and obligation, not an arbitrary output path. `ProtectedSubjectIdentity` and `ProtectedGeneration` are opaque host-owned in-process identity values for the actual protected subject and observation/evaluation generation; no byte encoding, cryptographic proof or public token authority is implied. `EffectLifetimeClosed` and `ProtectedCoverage` are interfaces implemented by a trusted host using actual substrate mechanisms. Their values are not deserialized from public JSON or local realization files, and metadata construction alone establishes no host property.

`validations()` exhaustively follows the original profile's canonical order, including validations that may later be skipped. `canonical_position` is zero-based in that order; declared prerequisites come from the corresponding original `ValidationDefinition`, not an inferred dependency graph. The three resolution classes are respectively `RESOLVED_WITH_OBLIGATIONS`, `RESOLVED_ZERO_INSTANCES`, and `INSTANCE_RESOLUTION_FAILED`: complete resolution with a nonempty ordered obligation vector, successful empty `for_each`, and canonical `resolution_failed`. Failure is never rewritten as successful empty resolution. For `INSTANCE_RESOLUTION_FAILED`, the canonical obligation vector is empty and the resolution failure remains explicit; never fabricate placeholder commands or admit a partially resolved subset.

Within successful resolution, `EXECUTABLE`, `MISSING_REALIZATION` and `KNOWN_UNLAUNCHABLE` are admission-view classifications, not RI statuses. An absent realization under the original declared environment ID is `MISSING_REALIZATION`. A provided realization whose canonical prefix is empty is `KNOWN_UNLAUNCHABLE`; neither view fabricates a program or environment. Other concrete obligations are `EXECUTABLE` launch candidates, not predictions of successful OS execution; a later proven launch failure still preserves canonical uncertainty. These classifications read the same retained context and realized obligation, without selecting or expanding again. Otherwise authorized missing realizations and proven launch failures retain `UNDETERMINED`, `executed=false`; failed instance resolution retains canonical `UNDETERMINED`; successfully resolved zero instances retain `SATISFIED`. Existing prerequisites, stop policy and drift handling still apply and may prevent execution.

`declared_environment_id` is read from `consumer_profile.validations[ValidationId].command.environment`, never guessed from the realized executable. Executable program is exactly `command_prefix[0]`; the full effective argument vector is `command_prefix[1..]` followed by realized obligation arguments. `EffectiveArgumentsView` exposes those two immutable slices as one ordered virtual concatenation: all `prefix_tail` elements followed by all `realized_arguments` elements, with no insertion, selection or reordering. Keys and position metadata are exact derived copies, not mutable access to the plan. The effective process environment is the complete explicit map, with ambient environment cleared; working directory is the exact effective repository root, matching the canonical executor. There is no ambient argument, environment or cwd substitution. `plan_position` follows the flattened canonical obligation order, including non-executable entries; `instance_index` is zero-based within its validation, with checked wire conversion.

The host must inspect every potential executable effect before issuing a permit, including later effects whose prerequisites may eventually fail. A privileged host must establish and bind the actual executable it will launch, not just a mutable filename or command name. Failure to establish required executable identity refuses the whole plan before any validator starts; it is not permission to substitute a tool. Proven unlaunchability is distinct from uncertainty about executable substitution. The permit binds the exact immutable plan and one attempt. Only the engine sequences per-obligation launch and consumes each obligation key at most once. An `AdmittedPlan` binds that private plan, permit, workspace, exact effects and ReceiptBudget; no agent-constructible value grants authority.

### 7.2 Required transitions and guarded dispatch

1. Create the engine-owned attempt identity at invocation entry, before host calls; pass it into repository authorization and retain its binding through workspace acquisition, plan admission, effects and capture. The identity is not a permit. Authorize repository access before protected disclosure. Acquire a sufficient coherent workspace **before the first RGS observation**. Its protection covers relevant governance sources, repository state and the finally resolved RI scope through the last required RI observation.
2. Load canonical RGS, resolve optional observation comparison, admit the actual full profile, and obtain host-controlled realizations. Use `realize_consumer_profile()` as the sole instance/argument resolver. Guard observation and resolution stages as well as evaluation.
3. Resolve the complete plan and establish the conservative complete-receipt budget. Reserve required capture/custody resources before effects.
4. Admit every potential executable obligation, including later obligations that may eventually be skipped by prerequisites: exact root, profile/context, ValidationId, instance ordinal, program, prefix arguments, resolved arguments, process environment, working directory, coherence guard and required resource permissions. A forbidden later effect rejects the entire attempt before any validator starts. Do not remove or reorder consumer membership.
5. Before every launch verify permit validity, expected obligation identity/ordinal, that it has not already launched, effective command/environment, protected root, coherence guard and executable substitution resistance. A mutable executable path or command-name allowlist alone does not establish privileged integrity. `EvaluationContextIdentity` excludes effective prefix/environment values and is neither admission nor attestation.
6. Preserve engine-owned prerequisite handling, captures, obligation status, aggregation, state relation and verdict. Finalize a single canonical receipt only after completion and valid capture/custody metadata.

The engine-owned invocation state initializes `effects_possible=false`. It changes permanently to true once any process has started or might have started, and never returns to false. `Launch::ProvenNotStarted` requires definitive absence of start; uncertain start instead yields `HostAbort` with `PotentiallyOccurred`. Earlier obligations' effects remain possible even when a later launch is definitively excluded. No automatic retry is allowed.

`CompletedProcess` requires `process_started=true`, known principal termination, truthful success/optional numeric-exit facts, and a same-invocation capture reference. `exit_code` preserves the canonical engine's `ExitStatus::code().map(|code| code.to_string())` representation (an i32 decimal or None), not arbitrary diagnostic text. `EffectLifetimeClosed` denotes an established host-enforced condition, not a caller-supplied boolean authorization. `wait_and_capture()` may return it only after relevant descendants have finished or are effectively contained, no remaining process can invalidate the claimed state outside the admitted boundary, and the coherence lease remains valid. The engine verifies attempt/obligation/generation binding and checks `AFTER_EACH_EFFECT_LIFETIME` **before the next required RI capture and before the next obligation**. Parent exit and pipe closure alone are insufficient. Partial stream capture is permitted when process completion and effect closure are independently established. Failure to establish closure returns `DESCENDANT_EFFECTS_UNCONTAINED` or another accurate HostAbort, with effects possible; no universal validator timeout is added.

Stable internal abort reason names are `COHERENCE_LOST`, `ADMISSION_REVOKED`, `EFFECT_BINDING_INVALID`, `PROCESS_START_INDETERMINATE`, `PROCESS_COMPLETION_INDETERMINATE`, `DESCENDANT_EFFECTS_UNCONTAINED`, `HOST_OBSERVATION_UNAVAILABLE`. They are internal diagnostics, not new public codes or RI verdicts. `HostAbort` binds the attempt and whether the interrupted process start is definitively excluded or potentially occurred. The engine combines this fact monotonically with earlier effects. Before any possible execution effects, established absence permits pre-execution `admission_refused/execution_guarantee_unavailable`; admission policy refusals retain their specific permitted codes. After possible effects, any host interruption yields `execution_incomplete`, `effects_possible:true`, the actual admitted attempt ID and no completed IntegrityResult. Guard loss uses `host_guard_lost_after_start`; other interruptions use the existing `execution_incomplete` code. Missing responses prove neither non-start nor quiescence.

The host supplies process facts; only the engine classifies them: successful exit is `SATISFIED`; an explicitly configured nonzero undetermined exit code is `UNDETERMINED`; another unsuccessful exit, including unavailable numeric exit code, is `VIOLATED`; proven start failure is `UNDETERMINED`, `executed=false`. A guarded state capture has boundary `Result<Result<RepositoryState, StateCaptureError>, HostAbort>`: ordinary inner capture failure retains canonical RI observation semantics under a valid guard; outer HostAbort interrupts without a completed verdict. HostAbort is never converted into ProcessOutcome. No new RI status is introduced.

The mandatory extension is one shared evaluation/aggregation core: canonical `realize_consumer_profile()` remains authoritative and is covered by the guard; existing `evaluate()` remains available through an adapter; guarded and existing evaluation share aggregation. HostAbort is not converted into ordinary ProcessOutcome. Actual repository drift continues to produce the engine's existing consequences. In particular realization A, baseline B, final B may yield NOT_EQUAL and coherence halt despite equal baseline/final public digests; the façade MUST preserve that result, never recalculate state relation.

### 7.3 Coherence, local use and interruptions

A host acquires an enforceable coherence capability **before loading RGS**, not after comparing endpoint hashes. `CoherenceEvidenceView` exposes its effective root, bound requested/effective-root relationship, coherence class, protected subject identity, protected observation/evaluation generation and coverage. `ensure_active()` is an enforcement checkpoint on the live host capability, not acceptance of a class string or an independently verifiable security proof.

`exclusive_workspace` prevents uncontrolled external writers from changing contract-relevant state during evaluation while keeping validator-produced mutations visible to normal engine drift detection. `versioned_workspace` binds governance observations, selector resolution, validator reads and RI observations to one coherent protected generation or equivalent versioned view. Its mutable evaluation branch must expose validator-produced modifications to the RI evaluator; an immutable snapshot that hides their effects is not conforming.

`require_coverage(GovernanceAndRepositoryState)` must succeed before the first RGS access. Initial protection must be sufficient for all relevant governance inputs and RepositoryState dimensions even though the full RI scope is unresolved; a host can protect a sufficient whole worktree domain rather than guess a future selector expansion. Metadata-only coverage assertions are insufficient. Each actual authoritative governance read must remain inside that protection. After canonical realization, `require_coverage(RiScope(actual_scope))` checks the actual canonical RI scope before plan admission and before effects. Inadequate coverage causes refusal; never continue a qualified evaluation through uncovered reads or effects.

The API consumes this evidence in-process and requires validity for the same subject/generation through the invocation. Mandatory checkpoints are `BEFORE_RGS`, `BEFORE_REALIZATION`, `AFTER_REALIZATION`, `BEFORE_PLAN_ADMISSION`, `BEFORE_EACH_LAUNCH`, `AFTER_EACH_EFFECT_LIFETIME`, `BEFORE_REQUIRED_RI_CAPTURE`, `AFTER_FINAL_RI_OBSERVATION`, corresponding to the Rust enum above. Checkpoint failure invokes the monotonic effects/abort rules, not ordinary capture failure. A receipt records the asserted class under the declared host trust/custody boundary; it is not independent cryptographic proof that the host enforced it.

An ordinary same-user mutable checkout establishes neither class automatically. Without an actually compliant host workspace, direct local checking returns `admission_refused/execution_guarantee_unavailable`, before any validator starts. There is no silent unqualified ordinary-capture PASS. A controlled local host may acquire either property through real OS/substrate mechanisms; v1 mandates no vendor, VM, remote server or universal timeout. `discover`, `inspect` and `explain` remain non-executing observations.

Direct local CLI uses the invoking OS identity, without elevation or a sandbox promise. `--local-context-file` supplies only nonprivileged realizations; it cannot establish coherence or protected custody by assertion. Its typed schema governs shape. Every `environment_id` in `realizations` must additionally be unique by exact Rust string equality: case-sensitive, no Unicode normalization, trimming or alias inference. Reject both identical and conflicting duplicate-ID entries with `controlled_failure/local_context_invalid`, `effects_possible:false`, before map construction and before the first executable effect. Never deduplicate or select first/last wins. JSON Schema `uniqueItems` cannot enforce selected-property uniqueness; `conformance/local_context_controls.py` checks this candidate rule separately, without executing the API. Default locations remain `$XDG_CONFIG_HOME/proto-ring/local-context.json` (otherwise `$HOME/.config/proto-ring/local-context.json`) on Unix and `%APPDATA%\proto-ring\local-context.json` on Windows. The declared repository must resolve to the same root. A missing default file supplies no realizations; malformed configuration gives `controlled_failure/local_context_invalid` before effects. Even correctly parsed local configuration does not waive coherence admission; ordinary same-user mutable checkout mode cannot claim a qualified completed check without a host-established basis.

Delegated privileged invocation is through an independently controlled Rust host. Public JSON/CLI has no privileged switch or file. The host must actually protect repository, realizations, executables, credentials, dispatch and asserted guards from the untrusted agent/process. Agent-requested prefixes, opaque context identities, environment values and boolean flags cannot grant permission.

Guard loss after any possible launch gives `execution_incomplete`, the actual non-null admitted attempt ID, `effects_possible:true`, no completed RI verdict and no automatic retry. Proven absence of all execution effects permits pre-execution refusal without a verdict. Relevant descendant effects must be finished or effectively contained after **each** obligation, before the next RI capture or obligation, not merely before final receipt publication. Parent exit and pipe closure prove neither condition; incomplete pipe collection may remain partial when completion and effect closure are independently established.

## 8. One canonical receipt, capacity admission and custody

Each check invocation receives an engine-owned random 32-octet attempt identifier at entry, before workspace acquisition or commands; serialized as lowercase hex64. A subsequently admitted attempt retains that same identifier. Identical-state attempts have distinct IDs. IDs are not access capabilities or content digests. A CheckReceipt exists only after canonical engine completion and publication under the declared custody policy. It exposes exactly one engine IntegrityResult projection: PASS/NON_PASS, full ordered validations and obligations, SATISFIED/VIOLATED/UNDETERMINED, executed and exit-code observations, exact profile/context bindings, baseline/final references, state_relation and state_coherence_halt. Instance indices are zero-based per ValidationId and canonical u64 decimal strings. Never recompute a verdict in receipt storage.

The receipt additionally carries RGS observation and flattened `rgs_provenance`, concrete `ri_scope_paths` and distinct RI scope digest, host-established `coherence_basis` (`exclusive_workspace` or `versioned_workspace`), and `custody` (`local_user_controlled` or `host_enforced`). Its governance observation must record `host_established`, not ordinary_capture as proof of qualified checking. RGS and RI path scopes may differ and MUST NOT be substituted for one another.

### 8.1 Fixed v1 receipt-capacity strategy

Pre-execution bounded receipt admission is mandatory. After full canonical realization, ReceiptBudget establishes a conservative upper bound on the complete encoded UTF-8 JSON response, including the envelope, all validations and concrete obligations, maximum serialized statuses and exit codes, exact escaped identifiers, all reference/identity fields, concrete RGS and RI provenance, capture policy, maximum capture metadata and JSON structural overhead. Raw stdout/stderr bytes are not embedded. Size accounting uses checked arithmetic; unknown/unbounded fields require refusal, not a guessed bound. No new global execution timeout or profile-membership limit is imposed.

If a safe bound cannot be established or exceeds **16,777,216 bytes**, return `controlled_failure/response_too_large`, `effects_possible:false` before the first validator. The host admits this budget with the entire effect plan and reserves required resources. An unexpected post-effect bound/serialization failure gives `execution_incomplete` with the actual admitted attempt ID and monotonic `effects_possible:true`, not a truncated completed receipt or an automatic rerun. No sixth operation or receipt pagination exists in v1.

### 8.2 Same-execution output capture

Both stdout and stderr are captured within the existing engine-owned execution path, once per actually executed obligation; the current engine's Stdio::null() cannot supply historical output. A second executor or validator replay is forbidden. Streams are untrusted raw bytes, not instructions or a competing verdict. A success log cannot override RI NON_PASS due to drift.

No process start means `not_started` with absent metrics, not complete-empty output. A started zero-output stream is complete with zero bytes. Preserve exact native bytes, including invalid UTF-8. Retain the last **64 MiB** per stream and drain concurrently, reporting absolute retained start, retained count and observed total when known. Bound memory and storage; never label lost data complete. States remain complete, partial, capture_failed, unavailable, not_started. A descendant-held pipe must not cause indefinite collection after parent exit: drain available bytes and explicitly mark incomplete when necessary. This is not a timeout on a running consumer validator, nor proof that relevant descendant effects ended. Capture failure affecting process behavior/observation must retain its uncertainty.

### 8.3 Custodian and trust boundary

```rust
pub trait ReceiptCustodian {
    type Reservation;

    fn reserve(
        &mut self,
        attempt: AttemptId,
        plan: &ResolvedCheckPlan,
        budget: &ReceiptBudget,
    ) -> Result<Self::Reservation, AdmissionError>;

    fn finalize(
        &mut self,
        reservation: Self::Reservation,
        result: &IntegrityResult,
        captures: CaptureIndex,
    ) -> Result<CheckReceipt, ReceiptError>;

    fn read_output(
        &self,
        authorization: &OutputReadAuthorization,
        request: &OutputReadRequest,
    ) -> Result<OutputChunk, OutputReadError>;
}
```

Reserve before the first validator. Finalize once after canonical result and capture metadata are established. Read only subordinate stored artifacts, never reevaluate. `local_user_controlled` guarantees single-use API finalization and no silent substitution by API reads, appropriate OS access restrictions, and explicit loss/expiry. It does **not** claim independent tamper resistance against the same OS identity, including same-identity validator code and descendants. A neighboring checksum can be rewritten by that actor and is not an independent integrity proof. `host_enforced` requires actual custody protected from the untrusted process by the declared host boundary, not a flag/string. Custody strength does not recalculate RI truth.

Local storage remains `$XDG_STATE_HOME/proto-ring/receipts` (otherwise `$HOME/.local/state/proto-ring/receipts`) on Unix and `%LOCALAPPDATA%\proto-ring\receipts` on Windows, outside the evaluated worktree. Refuse admission if safe resource reservation is unavailable. Use restrictive host-native permissions, reject traversal and symlink/reparse-point substitutions. Defaults remain retention target **604800 seconds**, global output quota **1 GiB**, per-stream tail **64 MiB**, read maximum **64 KiB**. Quota loss before publication is explicitly partial/unavailable; after publication, output expiry is explicit `output_expired`. Retained-byte SHA-256 identifies recorded bytes under custody assumptions, not truth or same-user tamper resistance.

Stage captures, complete RI, finalize capture metadata and atomically publish one finalized manifest before emitting result. Serialize that same finalized object; do not independently compute another receipt. Durability claims require relevant host fsync/durability mechanisms, not rename alone. Crash/finalization failure after possible effects yields `execution_incomplete`, the actual admitted non-null attempt ID and `effects_possible:true` if a response can be emitted. If persistence fails while absence of all execution effects is established, use `controlled_failure/receipt_persistence_failed`, `effects_possible:false`, instead. Silence from a dead process never proves no launch. No recovery/replay of indeterminate attempts is specified in v1.

## 9. Total output-range semantics

The request remains receipt_id, validation_id, instance_index, stream, offset and max_bytes (integer 1..65536). References select one finalized original concrete obligation and stream, never an arbitrary path. Reads do not execute validators, reload RGS or produce a verdict. Canonical padded Base64 preserves exact returned bytes; report offset, returned_bytes, retained_start, retained_bytes, observed_bytes, capture_state and next_offset. Each range begins exactly at requested offset, never a silently clipped prefix.

Let S=retained_start, N=retained_bytes, E=S+N, T=observed_bytes (possibly unknown), O=request offset, L=max_bytes. All arithmetic is checked u64. Apply this table in order:

| Order | Condition | Required outcome |
| ----- | --------- | ---------------- |
| 1 | Before disclosure | Verify authorization, receipt, ValidationId/instance, stream, retention and availability; preserve output_access_refused, output_expired, subject_not_found, output_unavailable and output_not_started distinctions. |
| 2 | O+L overflows u64 | `invalid_output_range`. Invalid S+N metadata must not wrap or be disclosed as valid. |
| 3 | complete | Require S=0, known T, N=T. |
| 4 | complete and O>T | `invalid_output_range`. |
| 5 | complete and O=T | Valid zero bytes, next_offset=null, including complete-empty streams. |
| 6 | complete and O<T | Return exact bytes at O, bounded by L and T; next_offset only if retained bytes remain. |
| 7 | partial and known T with O>T | `invalid_output_range`. |
| 8 | partial and O<S | `range_not_retained`, even if requested interval overlaps S; never clip its start. |
| 9 | partial and S<=O<E | Return exact retained bytes at O, bounded by L and E; next_offset only if more retained bytes remain. |
| 10 | partial and O>=E | `range_not_retained`, including O=T and unknown-total O=E; no complete-stream EOF exception. |
| 11 | partial and N=0 | No fabricated successful range; the preceding partial rules apply. |
| 12 | capture_failed or unavailable | Corresponding controlled unavailability, unless retained partial bytes are explicitly and consistently represented as partial. |

Unknown T never implies E is logical EOF. A process that never started is not a successful empty stream. Capture failure is not reinterpreted as complete-empty. The caller may read either, both or neither stream and request successive retained ranges without summaries or forced bulk disclosure.

## 10. Rust implementation boundary and CLI contract

Suggested Rust crate: one `proto-ring-api` package with a public library and one `proto-ring` binary, delegating to `proto-ring-engine`. The library contains a typed `GovernanceObservation` holding the single successfully composed RGS, and public equivalents of `discover`, `open`, `inspect`, `explain`, `check_repository_integrity` and `read_execution_output`. Execution admission and custody must implement the concrete boundaries and transitions in Sections 7–8; no duplicate governance parser, selector, authority resolver, state capture or command runner. No production implementation is authorized by this contract alone.

Standalone commands (accepted v1 grammar):

```text
proto-ring json [--local-context-file NATIVE_PATH]  # one JSON request from stdin
proto-ring discover --json
proto-ring inspect --repository NATIVE_PATH --selector NAME --json
proto-ring inspect --repository NATIVE_PATH --selector-json JSON --json
proto-ring explain --repository NATIVE_PATH --query-json JSON --json
proto-ring check --repository NATIVE_PATH [--local-context-file NATIVE_PATH] [--expected-observation-json JSON] --json
proto-ring output --receipt HEX64 --validation ID --instance U64DEC --stream stdout|stderr --offset U64DEC --max-bytes INT --json
```

`NAME` is one of the nine non-parameterized `InspectSelector.kind` forms (see schema). `--selector-json` and `--query-json` are exactly one JSON object each matching the wire selector/query schema; unknown CLI flags or ambiguous duplicate flags fail. `--repository` must preserve the OS-native argument spelling (`args_os`) and pass through the canonical root resolver; the JSON path variants remain available to clients requiring exact non-UTF-8 handling. No check or output operation is executed by `inspect`, `explain` or `discover` as a side effect. `proto-ring json` may apply `--local-context-file` **only** to the nonprivileged local realization mode; it never confers a security capability. The CLI is a thin syntactic adapter to one shared engine-backed public API, not a second semantics layer.

## 11. JSON Schema and conformance responsibilities

The JSON Schema Draft 2020-12 documents in `schemas/` describe **syntactic** shapes, discriminants, closed fields, types and ranges. `api.schema.json` owns shared definitions; `request.schema.json`, `response.schema.json` and `local-context.schema.json` reference them using stable URNs (no network retrieval). A client can use a `jsonschema` library or generate a Zod validator from the accepted public schema, but **Zod is optional** and cannot become a second normative contract. Rust `serde` deserialization must agree with the same accepted semantics. `schemars` may assist generation if the generated output is checked against the accepted versioned schema, not silently treated as authority.

**Not expressible solely in JSON Schema:** raw duplicate-key rejection, canonical Base64 pad bits, OS native path validity, `u64` maximum, unique/map key ordering for tagged StructuredValue, relationship of request/response selectors to exactly the published result, RGS component required-presence rules, RI validation order/prerequisite semantics, token computation, host admission, actual executable provenance, RI result aggregation, output capture completeness and absence of effects. Each needs explicit executable conformance/hostile tests.

`conformance/identity-vectors.json` holds frozen deterministic known-answer public-reference hashing vectors, including native path, word-width and byte-order boundaries (the checker reports the actual total); `conformance/schema-cases.json` contains **illustrative schema shape** cases; `conformance/semantic-cases.json` preserves A01–A70 and adds A71–A87 for exact error attribution, host views, effect lifetime, coherence, local-ID collisions and interruption metadata. All remain semantic_review_required and qualified:false. `test_error_attribution.py` contains an independent fixed matrix oracle and checks canonical and advertised schemas across allowed and forbidden combinations. `local-context-cases.json` separately declares exact-ID uniqueness results: duplicate-ID documents are schema-shape-valid but rejected by the candidate semantic control. These Python controls do not execute the API or qualify a host. The executable schema cases declare their request/response/local target explicitly; duplicate keys are rejected before map construction. Discovery schemas are checked for exact canonical derivation and local reference closure. These inventories list unresolved **runtime acceptance tests**, not successful executions. No shape-case pass is evidence of canonical engine semantics or successful hostile review. The current regression reference is the qualified Rust engine and its applicable **1,153 language-neutral conformance vectors across 29 active responsibilities**. Preservation of the legacy 16-operation `proto-ring-compat` transport on current `main` is **not** a new-public-API gate. Downstream consumer repinning remains separately consumer-authorized.

## 12. Required hostile review and acceptance gate

Review source identity, source-contract precedence, completeness of the five-operation inventory, exact binary and path round trips, type/schema agreement, duplicate-key rejection, output access/privacy, incomplete executions, complete-plan admission (especially forbidden *later* command), actual dispatch-to-plan binding, stale/scope drift and A→B→A distinctions, no false privileged claims, RI empty profile/zero-instance semantics, non-executing `inspect`/`explain`, provider-network prohibition, single RI verdict versus misleading log content, output capture and descendant pipe behavior, exact-concrete-instance provenance and negative retry tests. The review must compare concrete Rust/CLI implementability with each contract rule, mark unresolved findings as **NO_GO**, and never report unexecuted semantic cases as PASS.

**The API-01 contract acceptance gate is satisfied** by the independent exact-source hostile GO on `a1b27ca1e21423973a5557415488e4fcd535d01a` and the [owner acceptance decision](https://github.com/fanilosendrison/proto-ring/issues/134#issuecomment-6637682924), with no unresolved material contract finding. #135/#136 implement that accepted semantic contract in dependency order. #137 separately qualifies the actual agent experience on the completed implementation. Runtime implementation, host protections, actual cross-target identity/native-path evidence and execution of all 87 semantic scenarios remain unqualified. This editorial update does not alter live consumer repositories or the Rust engine.
