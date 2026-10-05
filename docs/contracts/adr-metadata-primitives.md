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
level-one heading. For this primitive, a level-one heading is exactly one line
whose bytes begin with `# ` and contain at least one byte after that ASCII
space. No alternate Markdown H1 syntax participates. The exact UTF-8 text after
the first `# ` is the H1 text; it is not trimmed or normalized.

The preserved payload is available only for data that also has one valid
decision-body boundary.

SHA-256 operates over exact input bytes and returns a lowercase hexadecimal
digest.

## Canonical structured-data loading

Machine-readable YAML consumed by this primitive layer composes with
[Canonical Structured Data](structured-data.md).

The standalone YAML loading operation consumes the complete YAML file as one
Canonical Structured Data document. It does not inherit a YAML library's
implicit scalar typing, object construction, merge behavior, duplicate-key
precedence, or multi-document behavior.

The standalone root may be any Canonical Structured Data value. Callers that
require a mapping use the existing generic mapping requirement.

JSON loading remains strict UTF-8 JSON and returns the corresponding structured
value without adding YAML semantics.

ADR frontmatter parsing uses the exact ADR/frontmatter carrier boundary and
constructs its mapping through Canonical Structured Data. Parsing returns the
mapping and the exact decision-body bytes without normalizing authored body
bytes or final-newline state.

One exact ADR-byte parsing operation owns both repository-file bytes and
historical Git blob bytes. A path-based convenience operation contributes only
path-specific byte reading and delegates the exact returned bytes without
normalization.

Parse, decoding, representation, and I/O failures remain controlled
`AdrMetadataError` results at the ADR metadata boundary.

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

## Canonical operation set

This contract owns the following language-neutral operations:

- standalone Canonical Structured Data YAML loading through the ADR error
  boundary;
- strict UTF-8 JSON loading;
- lowercase hexadecimal SHA-256 over exact input bytes;
- exact decision-body boundary extraction;
- exact H1 text extraction;
- exact preserved authored-payload extraction;
- exact ADR-frontmatter parsing over supplied bytes;
- path-based ADR byte reading composed with that exact parser;
- independent base/overlay JSON Schema Draft 2020-12 validation;
- repository-relative containment;
- generic mapping, non-empty-string, and duplicate-free string-list
  requirements; and
- generic self/missing relation-target checking.

Concrete module names, function names, exception class names, parser libraries,
container types, and package dependencies are implementation surfaces, not
language-neutral contract authority.

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
