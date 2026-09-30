---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-governed-objects"
severity: "strict"
name: "Canonical Governed Objects contract"
---

# Canonical Governed Objects contract

## Purpose

Canonical Governed Objects provides consumer-owned, machine-readable enumeration
of governed interfaces and governed object identities. A governed object is
identified exactly by:

```text
GovernedObjectRef = (interface_id, object_id)
```

Every governed object participates in one or more responsibilities from the
consumer's Canonical Governance Authority profile and may declare structural,
consumer-owned relations to other objects in the same catalog.

## Carrier and machine-governing payload

The governed-object profile is a consumer-owned Markdown governance artifact
with deterministic structured frontmatter. Optional Markdown body prose is
explanatory and is not machine-governing.

The mechanically interpreted payload is exactly:

```yaml
governed_objects:
  model_version: 1
  interfaces: {}
```

The profile composes with the canonical structured-data parser. No competing
YAML or frontmatter interpretation is part of this contract.

## Model-version-1 structure

The direct keys inside `governed_objects` are exactly:

```text
model_version
interfaces
```

Each interface declaration contains exactly:

```text
objects
```

Each object declaration contains exactly:

```text
responsibilities
relations
```

Each relation declaration contains exactly:

```text
relation
responsibility
target
```

Each target declaration contains exactly:

```text
interface
object
```

Unknown structural keys at any of these model-version-1 layers are invalid and
fail closed.

`model_version` is exactly the structured integer `1`. Boolean `true`, string
`"1"`, floating-point `1.0`, another integer, and `null` are invalid.

The complete logical shape is:

```yaml
governed_objects:
  model_version: 1
  interfaces:
    <interface-id>:
      objects:
        <object-id>:
          responsibilities:
            - <governed-responsibility-id>
          relations:
            - relation: <consumer-owned-relation-id>
              responsibility: <governed-responsibility-id>
              target:
                interface: <target-interface-id>
                object: <target-object-id>
```

## Exact identity semantics

Interface identities, object identities, governed responsibility identities,
and relation identities are opaque, non-empty strings. Proto-ring performs exact
identity comparison only.

Proto-ring performs no:

```text
trimming
case folding
prefix parsing
normalization
namespace derivation from spelling
alias inference
semantic-equivalence inference
consumer object-ID minting
object-ID derivation from headings, paths, titles, sequence numbers, or content
```

`object_id` uniqueness is scoped to one interface. The same `object_id` may
appear in different interfaces and denotes different `GovernedObjectRef` values.
The exact `object_id` supplied by the consumer is already the consumer-owned
canonical identity for that interface; proto-ring does not canonicalize it.

## Cardinalities

The following declarations are valid:

```text
interfaces: {}
interface.objects: {}
object.relations: []
```

Every governed object participates in at least one governed responsibility.
Therefore, this declaration is invalid:

```text
object.responsibilities: []
```

An explicitly empty catalog and an explicitly empty interface remain distinct
from absence of the `governed_objects` capability.

## Canonical Governance Authority composition

A governed-object profile composes with one Governance Authority profile for the
same repository. Every governed responsibility referenced by an object or
relation already exists in that Governance Authority profile. An absent
responsibility is an invalid foreign reference.

A known responsibility with this state remains valid:

```yaml
roles: {}
precedence: []
```

The governed-object layer does not require a known responsibility to have a
current authority source.

## Governed relation integrity

Every relation's `responsibility` belongs to the source object's declared
responsibilities. The target does not need to participate in that
responsibility.

Every target resolves to an existing `GovernedObjectRef` in the same catalog.
Relations may cross interfaces within that catalog. Model version 1 forbids
cross-catalog and cross-repository targets.

Referential integrity is evaluated against the complete catalog. Forward
references, self-relations, and cycles are structurally valid.

Proto-ring imposes no universal DAG, tree, acyclicity, irreflexivity,
transitivity, symmetry, exclusivity, or ordering semantics on governed
relations.

## Duplicate and ordering semantics

`responsibilities` is a semantically unordered, duplicate-free collection.
Exact duplicate responsibility identities are invalid.

`relations` is a semantically unordered collection. An exact duplicate relation
is identified by:

```text
(
  relation identity,
  governing responsibility identity,
  target interface_id,
  target object_id
)
```

An exact duplicate relation is invalid. Relations differing in any one of these
dimensions remain distinct. Declaration order of responsibilities and relations
has no generic semantic meaning.

## Relation vocabulary

Relation identities are consumer-owned. Model version 1 defines no universal
relation registry and requires no prior relation-type declaration.

The source `GovernedObjectRef` is implicit from the object containing the
`relations` collection. A serialized `source` field is forbidden by the exact
model-version-1 shape.

## Semantic-content boundary

The governed-object profile contains no universal semantic locator or semantic
body. It does not add generic fields such as:

```text
path
line
heading
JSON pointer
source span
statement
body
description
```

Resolving consumer-specific semantic content remains outside this contract.

## Catalog authority boundary

The governed-object catalog has no universal `SourceRole`. Proto-ring does not
declare every catalog to be an `authority`, `secondary_representation`, or
`non_authoritative` source.

A consumer may assign the catalog an appropriate `SourceRole` for a governed
responsibility through Canonical Governance Authority. A consumer may make its
catalog authoritative for identities when consumer authority explicitly
establishes that role. The generic contract does not require every object
identity to originate in a different physical artifact.

## Ownership boundary

The contract does not define:

```text
universal object taxonomy or object kind
universal lifecycle or status
universal applicability
universal relation vocabulary
semantic object bodies or locators
validation registry
evidence registry
projection or catalog-currentness registry
catalog freshness checking
exact RepositoryGovernanceState
agent-facing API
mutation behavior
```

Existing ADR-specific contracts retain their stronger guarantees. Detailed
validation, evidence, projection, binding, and currentness responsibilities
remain outside this contract.
