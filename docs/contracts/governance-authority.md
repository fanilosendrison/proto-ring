---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-governance-authority"
severity: "strict"
name: "Canonical Governance Authority contract"
---

# Canonical Governance Authority contract

## Purpose

Canonical Governance Authority represents consumer-owned authority as the
responsibility-scoped relation:

```text
GovernedSourceId × GovernedResponsibilityId → SourceRole
```

`GovernedSourceId` and `GovernedResponsibilityId` are opaque, non-empty,
consumer-owned string identities. Proto-ring does not infer semantics,
hierarchy, type, scope, or relationship from identifier spelling.

## Source roles

Model version 1 has exactly three serialized source roles:

```text
authority
secondary_representation
non_authoritative
```

Their meanings are:

- `authority`: the consumer declares the source to own the governed
  responsibility within the declared scope;
- `secondary_representation`: the source represents governed information whose
  authority is owned elsewhere; and
- `non_authoritative`: the consumer explicitly declares that the source does
  not own the governed responsibility within the declared scope.

The same source may have different roles for different responsibilities.

## Lookup states

Authority lookup has two states that are not serialized source roles:

```text
UNKNOWN_RESPONSIBILITY
UNDECLARED
```

`UNKNOWN_RESPONSIBILITY` means that the responsibility identity is absent from
the profile. `UNDECLARED` means that the responsibility exists but the requested
source has no role assignment for it.

Therefore:

```text
undeclared != non_authoritative
```

Absence of a declaration does not establish non-authority. Required authority
that cannot be established remains unresolved and fails closed for the operation
that requires it.

## Co-authorities

A responsibility may have zero, one, or multiple authority sources. Multiple
authority sources are valid co-authorities. Their coexistence is not an
ambiguity and does not elect one global winning authority.

Proto-ring defines no universal unique-authority resolver. A known
responsibility with no role assignments is valid and represents a responsibility
whose authority is not currently declared.

## Precedence

Authority ownership and precedence are separate concepts. Precedence is:

```text
consumer-owned
responsibility-scoped
optional
strict partial order
authority-to-authority only
```

Each precedence edge identifies one `higher_source` and one `lower_source` that
both have role `authority` for the same responsibility. A self edge, cycle, or
exact duplicate edge is invalid. A transitively redundant edge is allowed.

Precedence does not remove authority from a lower source and does not install a
universal hierarchy. Incomparable co-authorities remain incomparable. Proto-ring
does not silently choose between them.

## Secondary representations

If any source has role `secondary_representation` for a responsibility, at least
one source must have role `authority` for that responsibility.

This rule establishes authority existence only. It does not identify the
canonical projection owner of a particular secondary representation and does
not define projection mode, generation, currentness, or projection validation.

## Governed sources

A `GovernedSourceId` is a logical identity, not a repository path. A source may
declare one optional `repository_target`. Absence of `repository_target` is
valid and implies no source type or taxonomy. Multiple distinct source IDs may
share the same `repository_target`.

Every declared `repository_target` composes with Canonical Governance Routing.
Every declared source must participate in at least one responsibility's `roles`
mapping. An unused source declaration is invalid.

Product Intent is optional and consumer-owned. Proto-ring does not require,
invent, or prioritize Product Intent.

## Model-version-1 profile

The authority profile is a consumer-owned Markdown governance artifact with
deterministic structured frontmatter. Optional body prose is explanatory and is
not machine-governing.

The mechanically interpreted payload is exactly:

```yaml
governance_authority:
  model_version: 1
  sources: {}
  responsibilities: {}
```

The direct keys inside `governance_authority` are exactly:

```text
model_version
sources
responsibilities
```

`model_version` is the structured integer `1`. `sources` and
`responsibilities` are mappings and may be empty.

Each source declaration is exactly either:

```yaml
{}
```

or:

```yaml
repository_target: "non-empty repository-relative path"
```

Each responsibility declaration contains exactly:

```yaml
roles: {}
precedence: []
```

Both keys are mandatory. `roles` maps a declared source ID to exactly one of the
three serialized source roles. Because `roles` is a mapping, one
source/responsibility pair has exactly one serialized role.

Each `precedence` entry contains exactly:

```yaml
higher_source: <source-id>
lower_source: <source-id>
```

Both endpoints must be declared sources with role `authority` for that
responsibility.

## Repository Governance Model integration

Canonical Repository Governance Model exposes the supported capability:

```text
governance_authority
```

Its required route is:

```text
profile
```

The routed consumer-owned profile contains the detailed authority declarations.
The Repository Governance Model routes that profile; it does not acquire the
profile's semantic authority.

## Ownership boundary

Canonical Governance Authority does not own or define:

```text
Product Intent semantics
consumer product semantics
decision/invariant/obligation interfaces
projection mode
projection currentness
generator binding
exact projection source binding
validation registries
evidence registries
evidence truth
formal semantics
qualification semantics
provider/contract binding registry
exact RepositoryGovernanceState
external transport/authentication
mutation behavior
```

Projection Integrity continues to own projection modes, exact canonical-source
bindings, generation, currentness, and projection validation. Decision,
invariant, and obligation interfaces remain deferred to #25. Detailed binding,
projection, validation, and evidence registries remain deferred to #26. Exact
`RepositoryGovernanceState` composition remains deferred to #27.

Consumer source and responsibility identities, role assignments, precedence,
product meaning, decisions, formal assurance, qualification, evidence, and
external work-state ownership remain consumer-owned.
