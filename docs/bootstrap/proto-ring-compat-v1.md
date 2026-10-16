---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "migration-compatibility-transport"
domain: "proto-ring-consumer-migration"
severity: "strict"
name: "Proto-Ring compatibility transport v1"
---

# Proto-Ring compatibility transport v1

## Status and authority boundary

This executable transport exists only for #70, #71, and #72 migration
compatibility. It is versioned, migration-local, and non-normative. It is not a
Proto-Ring semantic contract, canonical serialization, or #28's public API.
Engine values are manually projected for process interoperability; the
language-neutral contracts remain authoritative, and consumer policy remains
consumer-owned.

The exact executable source exposes exactly twelve operations:

```text
accepted-adr-body.immutability
adr-metadata.parse
adr-metadata.repository-path
adr-metadata.validation
canonical-adr.resolve
engine.identity
evidence-requirements.registry
exact-evidence-binding.evaluate
git-whitespace.validate
github-authoritative-ref-monotonicity.observe
repository-governance-state.compose
repository-integrity.evaluate
```

There is no generic responsibility dispatcher, discovery or negotiation API,
operation-set version, consumer callback protocol, or provider injection
surface.

## Process boundary

`proto-ring-compat` accepts one strict UTF-8 JSON request on standard input and
no command-specific arguments:

```json
{"transport_version":1,"operation":"<operation>","input":{}}
```

Unknown fields, versions, operations, modes, malformed JSON or UTF-8, and
operation-specific transport errors produce `transport_error` with category
`invalid_request` and exit `2`. Results and controlled semantic failures exit
`0`; unexpected runner I/O, projection, or serialization failures exit `70`.
Controlled failures include a stable category, nullable stage, and
non-normative detail. Consumers must not parse detail text and own their local
CLI exit mapping.

## Structured-value migration transport

ADR operations use a recursively tagged compatibility DTO:

```json
{"type":"null","value":null}
{"type":"boolean","value":true}
{"type":"integer","value":"12345678901234567890"}
{"type":"string","value":"text"}
{"type":"sequence","value":[{"type":"string","value":"x"}]}
{"type":"record","value":[{"name":"field","value":{"type":"string","value":"x"}}]}
```

Each tag requires exactly its documented value type. Record names are strings
and unique within that record. Integer text is neither parsed nor normalized by
the compatibility layer. `bytes` is not a structured-value tag. Malformed tags,
wrong value types, duplicate record names, and unknown fields are transport
errors.

The runner privately maps this DTO to and from engine `StructuredValue` using
`record`/`Mapping`, `sequence`/`Sequence`, `string`/`String`,
`boolean`/`Boolean`, `integer`/`Integer`, and `null`/`Null`. Engine mapping order
is preserved. The tagged representation is migration transport only. It is not
canonical Structured Data serialization.

## `engine.identity`

Input is `{}`. The result reports fixed repository
`fanilosendrison/proto-ring`, build-time source commit, and transport version
`1`. Runtime input and environment cannot override this identity.

## `repository-governance-state.compose`

Input contains non-empty `repository` and optional unique, non-empty
`binding_ids`. The operation calls `repository_governance_state::load` and
projects composition stage, identity presence, scope paths, component presence,
and requested binding facts. It is read-only and provider-independent.

## `repository-integrity.evaluate`

Input contains a non-empty repository and strict evaluation-context
realizations. Each realization has a unique non-empty environment ID, exact
UTF-8 identity, non-empty command prefix, and exact process-environment map.
The runner merges no ambient environment. It obtains the routed profile through
RGS and delegates realization and evaluation to the engine.

## `accepted-adr-body.immutability`

Input contains non-empty `repository`. The operation delegates to
`accepted_adr_body::check`. Empty diagnostics project `PASS`; typed
`BodyChanged` projects `NON_PASS/body_changed`; other diagnostics project
`NON_PASS/historical_state_invalid`. Diagnostic prose is not classified.

## `git-whitespace.validate`

Input contains non-empty `repository` and a required exact environment map.
Only that map reaches `git_whitespace::check`. Result status is derived from
`passes()` and domains are typed, ordered, and deduplicated by first occurrence.

## `github-authoritative-ref-monotonicity.observe`

Input contains non-empty `binding_path`. The operation delegates to the engine
and projects only typed status `SATISFIED`, `VIOLATED`, or `UNDETERMINED`.

## `evidence-requirements.registry`

Input contains non-empty `repository` and optional unique, non-empty
`requirement_ids`. The runner loads RGS, lists every registry ID in engine order,
and projects only requested entries. It does not reconstruct authority,
routing, Governed Objects, or loader dependencies. Absent registry and missing
requested IDs are controlled failures without partial output.

## `exact-evidence-binding.evaluate`

Modes are exactly `evaluate_binding` and `evaluate_requirement_history`.
Identity fields are nullable lowercase even-length hexadecimal byte transport;
empty hex is valid. The runner does not hash, decode as UTF-8, normalize,
case-fold, validate digest length, or interpret identities as Git objects.
Duplicate admitted classes are rejected before `BTreeSet` construction.
Structural validation and comparison delegate to `EvidenceRequirement::new`
and `evaluate`. History constructs every requirement before evaluating any
comparison and preserves request order.

## `adr-metadata.parse`

Input has exactly one of these modes and lowercase even-length `bytes_hex`:

```text
load_yaml
load_json
parse_adr
decision_body
h1
preserved_payload
sha256
```

Empty bytes are transport-valid. The runner decodes bytes without prior UTF-8
interpretation and delegates respectively to `adr_metadata::load_yaml`,
`load_json`, `parse_adr`, `decision_body`, `h1_text`, `preserved_payload`, or
`sha256`.

`load_yaml` and `load_json` return `structured`; `parse_adr` returns tagged
record `metadata` and `body_hex`; the remaining modes return `body_hex`, `text`,
`payload_hex`, or lowercase `sha256`. Engine errors are controlled category
`adr-metadata.parse.rejected`; diagnostic text does not determine status.

## `adr-metadata.repository-path`

Request input contains non-empty `repository` and raw JSON `relative`.
Non-string values are semantic controlled rejections, not transport errors.
Strings are passed only to `adr_metadata::resolve_repository_path`. The result
is the resolved target relative to the canonical repository root with portable
`/` separators, so a contained symlink projects its contained target. The
runner implements no containment, parent, absolute-path, symlink, or
canonicalization policy. Failures use category
`adr-metadata.repository-path.rejected`.

## `adr-metadata.validation`

Exactly eight modes are supported:

```text
require_mapping
require_string
require_string_list
relation_target_categories
relation_target_errors
schema_error_labels
schema_error_locations
schema_errors
```

The `require_*` modes accept tagged `value` and delegate to the corresponding
engine helper, returning tagged record `value`, `text`, or ordered `items`.
Relation modes accept tagged `source_id`, `relation_type`, `targets`, and
`known_ids`. Source and relation type must become non-empty engine strings;
target and known-ID values pass through `require_string_list`. Typed relation
errors project `self_reference` or `missing_target` with target.

Schema modes accept tagged `value`, `schema`, and `overlay_schema` and call only
`adr_metadata::validate_schemas`. The runner does not instantiate JSON Schema.
Typed errors project `path` and `schema` from `SchemaLabel::as_str`; label mode
returns base/overlay booleans and location mode also returns `rejected`.
Structural semantic failures use category `adr-metadata.validation.rejected`.

## `canonical-adr.resolve`

This operation has exactly two modes.

`configured_profile_path` accepts only non-empty `repository`. It calls
`repository_governance_model::load`, selects
`architecture_decisions/routes/profile/target`, strips the model repository
root, and returns portable `relative_path`. This mode is a migration helper over
existing Repository Governance Model routing, not a new normative
responsibility. It does not parse `AGENTS.md` or invoke governance routing
directly.

`resolve_adr` accepts non-empty `repository`, raw JSON `adr_id`, and optional
boolean `include_decision_body` defaulting to false. A non-string ADR ID is a
semantic rejection. String IDs are passed only to `canonical_adr::resolve`.
The result contains `adr_id` and portable `relative_path`, plus
`decision_body_hex` only when requested. Canonical metadata is not transported.

Both modes use controlled category `canonical-adr.resolve.rejected`. The runner
implements no filename matching, Portable Pattern logic, ID width, candidate
enumeration, symlink handling, profile loading, or ADR parsing.

## Consumer boundary and side effects

All four ADR compatibility operations are non-executing and read-only with
respect to governed source repositories. ADR profile semantics, schema pinning,
overlays, migration
evidence, annotated histories, legacy-ID policy, contiguous-ID policy,
relation vocabularies, H1 policy, generated index layout, and rendering remain
consumer-owned. Consumer rendering is not part of this transport.
