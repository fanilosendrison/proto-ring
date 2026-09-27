---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-adr-metadata"
severity: "strict"
name: "Canonical shared ADR metadata primitive contract"
---

# Canonical shared ADR metadata primitive contract

## Purpose

The shared ADR metadata primitive layer provides consumer-independent mechanisms
for byte-exact Architecture Decision Record processing.

The layer is deliberately smaller than an ADR engine or universal ADR profile.
A consumer composes these mechanisms with its own profile, corpus, authority,
exceptions, migration evidence, rendering policy, generated artifacts, and
validation orchestration.

## Exact text and hashing

ADR text processed by exact-text primitives must be UTF-8 without a byte-order
mark and must use LF line endings. A carriage return is an error; the primitives
do not normalize source bytes.

The decision body is the exact byte suffix beginning at the unique line whose
heading starts with `## Context` followed by a space or end of line. Missing or
multiple boundaries are errors.

The preserved authored payload is the exact byte suffix beginning at the unique
level-one heading. It is available only for data that also has one valid
decision-body boundary.

SHA-256 operates over exact input bytes and returns a lowercase hexadecimal
digest.

## Safe structured-data loading

YAML loading uses the maintained safe loader and does not enable Python object
constructors. JSON and YAML are read as strict UTF-8 text. Parse, decoding, and
I/O failures become controlled `AdrMetadataError` results.

ADR frontmatter parsing requires an opening `---` line and one exact closing
delimiter. Frontmatter is loaded through the safe YAML mechanism and must be a
mapping. Parsing returns the mapping and the exact decision-body bytes without
normalizing either authored body bytes or final-newline state.

`parse_adr_bytes` is the single exact parser for both repository file bytes and
historical Git blob bytes. `parse_adr` contributes only path-specific byte
reading and delegates the returned bytes to that parser without normalization.

## JSON Schema validation

Metadata is validated independently against a base JSON Schema and a consumer
overlay. Both schemas are checked as Draft 2020-12 schemas before use. Format
checking is enabled, including calendar-aware `date` validation.

A malformed base or overlay is a validation error and cannot disable validation
by the other schema. Error results identify the schema label and JSON path.

This contract does not assign canonical ownership of any base schema or local
overlay.

## Repository path containment

A repository-relative path must be a non-empty string, must not be absolute, and
must resolve within the resolved repository root. Resolution includes symlink
resolution; a symlink that escapes the repository is rejected.

The primitive determines containment only. Consumers own every allowed path and
the meaning of the artifact at that path.

## Structured configuration requirements

Generic requirements are available for:

- mappings;
- non-empty strings; and
- duplicate-free lists of non-empty strings.

The consumer supplies labels and owns the configuration vocabulary.

## Relation-reference checking

The relation helper checks a consumer-supplied source identity, relation type,
target collection, and known-identity collection. It reports self-reference and
missing targets.

The helper owns no relation vocabulary, inverse-relation model, lifecycle
meaning, relation completeness rule, or consumer identity format.

## Public primitive surface

`proto_ring.adr_metadata` owns these public concepts:

- `AdrMetadataError`;
- `load_yaml`;
- `load_json`;
- `sha256_hex`;
- `decision_body_bytes`;
- `h1_text`;
- `preserved_payload_bytes`;
- `parse_adr`;
- `parse_adr_bytes`;
- `schema_errors`;
- `repository_path`;
- `require_mapping`;
- `require_string`;
- `require_string_list`; and
- `relation_target_errors`.

The module depends on the maintained PyYAML and jsonschema libraries at the
versions declared by the proto-ring package.

## Consumer authority boundary

The shared primitive layer does not own or infer:

- an ADR corpus or accepted decision history;
- product meaning;
- a universal profile or profile version;
- schema provenance or canonical schema ownership;
- consumer overlays;
- identity width, filename patterns, heading separators, or date policy;
- relation vocabularies, lifecycle transitions, or relation completeness;
- legacy exceptions;
- migration baselines or migration evidence;
- maintained annotated history;
- rendering or index presentation;
- generated artifact paths or currentness policy;
- qualification or formal evidence;
- command-line behavior; or
- consumer validation membership and ordering.

A consumer adoption must pin an immutable proto-ring identity and demonstrate
exact equivalence, preservation, or an explicitly justified strengthening before
removing local implementation. Consumer-specific constraints compose outside
the shared layer and must not be weakened for reuse.
