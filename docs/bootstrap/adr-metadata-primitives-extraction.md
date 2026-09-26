---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "ADR metadata primitive extraction inventory"
---

# ADR metadata primitive extraction inventory

Status: non-normative extraction analysis.

This record preserves the Issue #7 factorization evidence. The canonical shared
scope is defined by
[`docs/contracts/adr-metadata-primitives.md`](../contracts/adr-metadata-primitives.md)
and the implementation at `src/proto_ring/adr_metadata.py`.

## Evidence baselines

The primary extraction source was:

```text
fanilosendrison/turnlock-rust
2f1617b56210b3464a4ce1de2ce78fa7567033f8
scripts/adr-metadata.py
```

The principal confrontation consumer was:

```text
fanilosendrison/ruu
2f693d1aac0ff59d96db988c32fd36b39c1347e0
tools/adr-metadata.py
```

Turnlock ADR-017 and ADR-050 and Ruu ADR-082 and ADR-083 establish the consumer
authority and shared-implementation boundaries. This inventory is evidence, not
consumer authority.

## Extraction method

Every in-scope Turnlock candidate was first tested for consumer independence.
Ruu was then used to falsify, distinguish, or strengthen that candidate. The
result was not derived as an intersection, and lack of a Ruu mechanism was not
used by itself as a reason to reject a justified Turnlock candidate.

## Extracted candidates

### Controlled ADR metadata error

Turnlock and Ruu use the same controlled `ValueError` boundary for parse, path,
and structure failures. The shared `AdrMetadataError` preserves that generic
failure class without owning consumer CLI presentation.

### Safe YAML loading

Turnlock uses `yaml.safe_load`; Ruu independently uses the same mechanism and
failure boundary. The generic primitive preserves strict UTF-8 and controlled
I/O, decode, and YAML failures. Direct regression coverage proves that an unsafe
Python object constructor is rejected without side effects.

### Safe JSON loading

Both consumers use UTF-8 `json.loads` with controlled I/O, decode, and parse
failures. The behavior is extracted unchanged.

### Exact SHA-256 hashing

Both consumers hash exact bytes with SHA-256 and lowercase hexadecimal output.
The primitive has an independent fixed digest vector.

### Exact decision-body detection

Both consumers reject BOM, every carriage return, invalid UTF-8, and any count
other than one `## Context` boundary. They return the exact boundary-to-EOF byte
suffix. The behavior is extracted unchanged with boundary-negative tests.

### Exact preserved payload detection

Both consumers validate the decision body, require exactly one H1, and retain
the exact H1-to-EOF bytes for migration comparison. The mechanism is extracted;
consumer migration baselines and preservation claims remain local.

### Safe ADR frontmatter parsing

Both consumers require the same exact delimiters, safe YAML mapping, and exact
body bytes. The parser is extracted without defining a universal frontmatter
schema or consumer ADR model.

### H1 text detection

Both consumers require exactly one level-one heading. The generic primitive
adds direct controlled invalid-UTF-8 and LF handling consistent with the shared
exact-text contract. Heading separators and title-agreement policy remain
consumer bindings.

### JSON Schema validation

Both consumers validate independently against a base schema and local overlay
with Draft 2020-12 and `FormatChecker`. Turnlock precompiles validators for its
local loop; Ruu constructs them directly. The shared primitive keeps the common
observable behavior and the Turnlock fail-closed handling for malformed schemas.
Compilation caching remains an optional consumer optimization, not shared
semantics.

### Repository-relative containment

Both consumers reject empty, non-string, absolute, and root-escaping paths after
resolution. Direct tests make the existing symlink-escape guarantee explicit.
Consumers continue to own all admitted paths.

### Structured configuration requirements

Turnlock and Ruu have identical mapping, non-empty string, and unique non-empty
string-list checks. Those checks are extracted as vocabulary-neutral
primitives. Consumer profile keys and required sections remain local.

### Relation target checking

Both consumers perform the same self-reference and missing-target checks after
consumer schema validation. The shared helper receives the relation type and
known identity set explicitly. It therefore extracts the repeated mechanism
without owning OKF relation vocabulary, identity format, inverse relations, or
relation-completeness meaning.

## Candidates retained in Turnlock

### Profile constants and profile loading

The profile path, profile version, required sections, keys, exact commands, and
policy booleans are Turnlock bindings. Extracting the loader wholesale would
create the universal profile prohibited by Issue #7.

### Canonical-schema and overlay provenance checks

The generic byte hashing and JSON parsing mechanisms are shared. Turnlock's
canonical source coordinates, pinned digest, vendored path, local overlay,
expected fields, and policy for their composition remain Turnlock-owned.

### ADR discovery and repository identity policy

Filename patterns, ID width and contiguity, retained IDs, excluded Markdown,
H1 separator, and identity derivation are consumer profile semantics. Their
current similarity with Ruu does not transfer ownership.

### Lifecycle and date policy

Lifecycle transitions and accepted-record mutability remain consumer authority.
Turnlock's calendar-aware schema requirement is supported by the shared schema
primitive. No universal current-date policy is introduced.

### Migration evidence

Turnlock's baseline commit, evidence schema, migrated ID set, Git ancestry,
annotated payload expectations, and recorded hashes are historical consumer
evidence. The shared layer provides byte parsing, hashing, and path mechanisms
only.

### Annotated history

Turnlock's markers, narrative-entry grammar, order, links, statuses, and
annotation requirements are unique maintained-history bindings. Ruu does not
have an equivalent contract, but that absence was not the rejection reason; the
mechanism remains outside this narrow primitive extraction because Issue #7
explicitly retains annotated-history ownership and does not extract an ADR
engine.

### Rendering and generated-index policy

Index headings, prose, path links, table presentation, incoming-relation
rendering, output path, freshness policy, and write command remain consumer
rendering and generated-projection policy. Superficially identical Markdown
escaping does not justify transferring rendering ownership in this narrow
primitive layer.

### Commands, orchestration, and diagnostics

Root discovery, check/render commands, exit codes, summaries, error prefixes,
and validation membership remain consumer-owned bindings.

## Ruu confrontation distinctions retained in Ruu

Ruu demonstrates multiple accepted local distinctions:

- multiple historical H1 separators and a preferred separator threshold;
- rejection of future dates;
- exact legacy unstructured and null-date boundaries;
- a larger migration baseline with a different payload-preservation shape;
- `legacy-partial` relation presentation;
- generated index wording and display naming;
- qualification-manifest integration; and
- immutable snapshot and retained-lineage custody.

These distinctions do not falsify the extracted byte, loading, schema, path, or
relation-reference mechanisms. They remain Ruu-owned policy and evidence.

## Strengthening and non-weakening result

The extracted layer makes the existing symlink-aware containment and safe-YAML
properties directly testable and applies controlled exact-text failures to the
public H1 primitive. It does not weaken either consumer's current guarantees.

Turnlock's annotated-history and migration protections remain possible outside
the layer. Ruu's future-date, legacy, migration, qualification, manifest, and
immutable-evidence protections remain possible outside the layer. Consumer
adoption still requires exact preservation or intentional-strengthening proof
before local duplication is removed.
