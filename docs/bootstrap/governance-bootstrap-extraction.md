---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Canonical Governance Bootstrap extraction evidence"
---

# Canonical Governance Bootstrap extraction evidence

Status: non-normative extraction analysis.

## Evidence baselines

```text
proto-ring
fanilosendrison/proto-ring
3bddcd4b49147f022466fdeb4acbf590e68890ce

Turnlock
fanilosendrison/turnlock-rust
dfe2bde9400a92676a769799f2028e5dce40b7ec

Ruu
fanilosendrison/ruu
19862a6caa6b6d1f86f61204f5a4e5a3cb56665b
```

These immutable revisions bound this extraction evidence. The consumer
repositories remain authoritative for their own configuration, decisions,
product semantics, formal or qualification semantics, local policy, and routed
targets.

## Concrete Turnlock evidence

At the exact Turnlock baseline, root `AGENTS.md` contains:

```text
repository_governance.architecture_decisions.profile_path
→ docs/adr/adr-profile.yaml

repository_governance.shared_governance_provider.required
→ true

repository_governance.shared_governance_provider.binding_path
→ docs/repository-governance/turnlock-rust-shared-governance-provider.md
```

Turnlock's current in-scope frontmatter also demonstrates quoted strings,
mappings, sequences, exact lowercase boolean `true`, and integer
`ruleset_id: 24106121`. The boolean evidence includes `required: true` in root
`AGENTS.md` and `mandatory: true` in the Shared Governance Provider binding.

Turnlock's ADR profile consumes the canonical OKF ADR schema identified as
`urn:fanilosendrison:okf:architecture-decision-record:0.1.0`. That schema admits
ADR dates as a date-formatted string or null. Turnlock's current profile records
an empty `legacy.null_dates` list, so its current ADR corpus does not exercise
the admitted null date representation.

The meanings of these fields remain Turnlock-owned. Their presence demonstrates
representation requirements; it does not transfer their semantics to
proto-ring.

## Concrete Ruu evidence

At the exact Ruu baseline, root `AGENTS.md` independently contains:

```text
repository_governance.architecture_decisions.profile_path
→ docs/adr/adr-profile.yaml

repository_governance.shared_governance_provider.required
→ true

repository_governance.shared_governance_provider.binding_path
→ docs/repository-governance/ruu-shared-governance-provider.md
```

Ruu therefore independently demonstrates the same root-carrier and structured
frontmatter bootstrap pattern, including mappings, sequences, quoted strings,
and exact lowercase boolean `true`.

Ruu also demonstrates an admitted representational difference: ADR-049,
ADR-050, and ADR-055 use exact unquoted lowercase `null` for `date`. Ruu's ADR
profile explicitly lists those identities under `legacy.null_dates`. Ruu
consumes the same canonical OKF ADR schema identity and bytes as Turnlock, and
that schema admits `date` as a date-formatted string or null.

This null evidence establishes a generic structured-data representation need;
it does not transfer Ruu ADR meaning, migration policy, or decision authority to
proto-ring.

## Existing proto-ring parsing duplication

The proto-ring baseline independently interprets governance-relevant
YAML/frontmatter in:

```text
src/proto_ring/adr_metadata.py
src/proto_ring/canonical_adr.py
src/proto_ring/shared_governance_provider.py
src/proto_ring/github_authoritative_ref_monotonicity.py
```

`canonical_adr` already requires BOM-free UTF-8, forbids carriage returns,
requires exact leading frontmatter and a closing delimiter, uses safe YAML
loading, and requires a mapping at the top level.

`shared_governance_provider` independently implements materially the same
frontmatter byte, envelope, safe-loading, and top-level mapping responsibility.

`adr_metadata.parse_adr_bytes` independently parses safe YAML frontmatter after
exact ADR byte validation has already required BOM-free UTF-8 and LF-only
content.

`github_authoritative_ref_monotonicity` independently parses binding
frontmatter. Its current loader decodes UTF-8 but is weaker at the
byte-representation boundary because it does not independently reject a BOM or
carriage returns before parsing.

The accepted contract intentionally establishes one stronger uniform
representation boundary. Later caller migration owns mechanical adoption of
that strengthening. This extraction does not modify any of these modules.

## Scalar and representation audit

A token- and node-level mechanical audit examined the in-scope frontmatter at
the exact consumer baselines:

```text
Turnlock: 54 files
Ruu: 92 files
```

Required data types are:

- mapping;
- sequence;
- string;
- boolean;
- integer; and
- null.

Concrete null evidence is:

- Ruu ADR-049 uses `date: null`;
- Ruu ADR-050 and ADR-055 independently use the same exact null representation;
- Ruu's profile explicitly records ADR-049, ADR-050, and ADR-055 under
  `legacy.null_dates`;
- the shared canonical OKF ADR schema admits `date` as string or null; and
- Turnlock consumes the same canonical schema even though its current corpus
  has no null-date ADRs.

The finding widens the initially hypothesized scalar subset based on concrete
consumer authority. It is not a compatibility workaround.

The audit found no demonstrated requirement for:

- floating-point values;
- timestamp typing;
- anchors;
- aliases;
- merge keys;
- explicit tags;
- multiple documents;
- non-string mapping keys;
- duplicate keys;
- block scalars; or
- omitted-value null syntax.

Versions and non-null dates observed in current canonical ADR frontmatter are
quoted strings rather than YAML numeric or timestamp authority.
`ruleset_id: 24106121` demonstrates the need for an integer type.
`required: true` and `mandatory: true` demonstrate the need for a boolean type.

The audit inspected YAML tokens and composed nodes rather than relying only on
`safe_load`. It checked anchors, aliases, tags, directives, document markers,
duplicate keys at every mapping depth, `<<`, mapping-key types, scalar
spellings/types, and block-scalar styles. The deterministic contract rules
remove reliance on PyYAML's implementation-specific implicit scalar resolution.

## Maximum justified extraction

The maximum justified responsibility is:

```text
consumer supplies:
    repository root
    +
    exact root AGENTS.md bytes containing canonical structured frontmatter

proto-ring bootstrap contract:
    addresses exactly root AGENTS.md
    validates exact byte/envelope requirements
    constructs one deterministic restricted structured-data tree
    requires top-level mapping
    requires repository_governance mapping
    preserves unknown valid descendants
    fails closed on malformed/ambiguous/unavailable bootstrap
    never reconstructs machine governance from prose

consumer retains:
    semantic meaning of consumer-owned configuration
    authority of routed targets
    product semantics
    decision semantics
    formal/qualification semantics
    local policy
```

This extraction establishes representation and bootstrap only. It does not
assign universal meaning to the configuration exposed by that bootstrap.

## Candidate disposition

| Candidate | Disposition | Evidence/result explanation |
| --------- | ----------- | --------------------------- |
| Root `AGENTS.md` is the canonical agent-entry governance carrier | `EXTRACT` | Both consumers use the exact root carrier for the demonstrated bootstrap. |
| Machine-governing bootstrap state is structured, not reconstructed from prose | `EXTRACT` | Both root carriers expose the required facts in structured frontmatter. |
| Exact opening/closing frontmatter envelope | `EXTRACT` | The current parsers and audited carriers demonstrate exact leading and closing delimiters. |
| Valid UTF-8, BOM forbidden, CR forbidden | `EXTRACT` | Current stronger consumer parsing guarantees justify one uniform byte boundary. |
| Top-level structured value is a mapping | `EXTRACT` | Every audited payload and current bootstrap consumer requires a mapping. |
| `repository_governance` is required and is a mapping | `EXTRACT` | Both root carriers use this exact structured bootstrap root. |
| Safe construction without arbitrary YAML objects | `EXTRACT` | Existing consumers already use safe loading; arbitrary construction adds no justified representation need. |
| Duplicate mapping keys fail closed | `STRENGTHEN` | Rejecting duplicates removes parser-selected key precedence at every depth. |
| Anchors and aliases fail closed | `STRENGTHEN` | The audited corpus does not require graph references, so rejecting them removes hidden identity and expansion semantics. |
| Merge semantics and `<<` fail closed | `STRENGTHEN` | No audited input requires merge behavior or a `<<` mapping key. |
| Multiple YAML documents fail closed | `STRENGTHEN` | One frontmatter envelope owns exactly one structured payload. |
| Mapping keys are strings | `NARROW` | Every audited governance mapping uses string keys; other key types are not demonstrated. |
| Structured values are limited to mapping/sequence/string/boolean/integer/null | `NARROW` | The complete audited corpus and shared ADR schema require exactly these data types. |
| Only lowercase `true`/`false` are booleans | `NARROW` | Current boolean evidence uses exact lowercase spellings; YAML alternatives are not demonstrated requirements. |
| Only canonical decimal grammar produces integers | `NARROW` | The integer evidence is canonical decimal; alternate YAML integer spellings are not demonstrated. |
| Exact lowercase `null` constructs null; alternative YAML null spellings do not | `NARROW` | Explicit null is required by the canonical ADR schema and current Ruu evidence; alternative YAML spellings are not demonstrated requirements. |
| Other ordinary scalars are strings rather than implementation-dependent implicit types | `STRENGTHEN` | Deterministic construction prevents float, timestamp, and legacy YAML coercions unsupported by the evidence. |
| Unknown valid `repository_governance` descendants are preserved | `EXTRACT` | Bootstrap must expose valid consumer configuration without inventing or erasing its meaning. |
| Semantic meaning of governance descendants | `CONSUMER` | Each consumer remains authoritative for its configuration semantics. |
| Consumer product semantics and authority | `CONSUMER` | Structured bootstrap does not transfer product or decision authority. |
| Target semantics and authority | `CONSUMER` | Routing to a target does not make proto-ring authoritative for that target. |
| Full repository-governance model | `DEFER` | Bootstrap representation does not establish the later logical model. |
| Universal capability/registry model | `DEFER` | The evidence does not justify a universal registry vocabulary. |
| Mutation/bootstrap administration API | `DEFER` | This extraction is read-only representation and bootstrap contract work. |

## Relationship to Canonical Governance Routing

Governance Bootstrap owns representation before routing.

Canonical Governance Routing continues to receive an already-parsed mapping and
one exact caller-supplied route.

Canonical Governance Routing still does not own carrier parsing.

The existence of the new upstream contract therefore does not retroactively
change the historical extraction boundary of Canonical Governance Routing.

The contracts compose as follows:

```text
repository root
    ↓
Canonical Governance Bootstrap
    - exact root AGENTS.md
    - exact frontmatter envelope
    - deterministic structured-data semantics
    - repository_governance mapping
    ↓
already-parsed structured mapping
    ↓
Canonical Governance Routing
    - caller supplies one exact route
    - exact route traversal
    - repository-relative path
    - symlink-aware containment
    - target existence
    ↓
consumer-owned target
```

The historical routing contract and extraction record remain unchanged.

## Consumer authority retained

Turnlock retains authority over Turnlock configuration, decisions, product
semantics, formal-verification semantics, local policy, and routed targets.

Ruu retains authority over Ruu configuration, decisions, product semantics,
qualification semantics, local policy, ADR migration exceptions, and routed
targets.

Proto-ring gains no authority over the semantic meaning of arbitrary
`repository_governance` descendants, consumer-owned target artifacts, or the
facts represented by the admitted null values. Bootstrap exposes the structured
configuration; separate consumer and proto-ring contracts determine any later
interpretation.

## Deferred responsibilities

This extraction intentionally does not establish:

```text
full repository-governance logical model
capability discovery model
authority graph
decision/invariant/obligation interfaces
projection/validation/evidence registries
mutation operations
consumer adoption changes
```

It also does not migrate existing proto-ring callers or modify either consumer.

## Resulting boundary

The resulting contract owns exact root `AGENTS.md` addressing, byte and envelope
validation, deterministic construction of the admitted structured-data model,
extraction of a `repository_governance` mapping, preservation of unknown valid
descendants, and controlled fail-closed behavior.

Canonical Governance Routing remains the separate downstream owner of exact
route traversal and repository-contained target resolution after an
already-parsed mapping exists. Consumers retain semantic authority over their
configuration and targets.
