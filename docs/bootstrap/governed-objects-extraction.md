---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Canonical Governed Objects extraction evidence"
---

# Canonical Governed Objects extraction evidence

Status: non-normative extraction analysis.

This record preserves the consumer confrontation for proto-ring Issue #42. The
canonical generic semantics are established separately by the
[Canonical Governed Objects contract](../contracts/governed-objects.md). This
record creates no Turnlock or Ruu product authority.

## Evidence baselines

The extraction was confronted with these exact default-branch states:

```text
proto-ring
a1cecad9e5bfd01e4474d28a62fa2ae779534d4e

Turnlock-Rust
fa3bffea8ff8a12e1d271113a1b6fe0dd2143d00

Ruu
2b4eaae0c71919905da024748ec185160c7fd1cc
```

The Turnlock evidence included root directives, the repository Governance
Authority profile, the normative specification, the formal-assurance graph,
the ADR profile, and the complete canonical ADR corpus.

The Ruu evidence included root directives, the repository Governance Authority
profile, both normative specifications, qualification policy, the ADR profile,
and the complete canonical ADR corpus.

## Extraction method

Turnlock supplied the primary evidence for multiple stable governed-object
families and structural traceability. Ruu confronted the candidate with a
consumer whose current stable governed identities and other normative,
architectural, qualification, and repository obligations do not form symmetric
object families.

The resulting model retains only consumer-independent identity, responsibility
membership, catalog structure, relation structure, and referential integrity.
Consumer semantics, authority assignments, vocabulary, and catalog currentness
remain outside the generic contract.

## Turnlock evidence

### ADR identities

Turnlock's canonical ADR corpus supplies exact consumer-owned ADR identities.
ADR frontmatter and the pinned ADR profile govern those identities, lifecycle
metadata, relations, scope, and body integrity. The generic catalog can refer to
those exact identities without copying decision bodies or weakening the
stronger ADR contracts.

### Stable invariant identities

The normative Turnlock specification publishes exact `TL-INV-*` identities.
Those identities remain Turnlock-owned and retain their normative meaning in the
specification. The generic model needs only their exact opaque object identities
and their participation in consumer-owned Governance Authority responsibilities.
It does not copy invariant text or interpret the `TL-INV-` prefix.

### Formal claim identities and traceability

`formal/verification.yaml` publishes exact `TL-CLAIM-*` identities and owns the
formal-assurance and traceability graph. Its claims carry structured
`normative_sources` relations, including references to published `TL-INV-*`
identities.

This evidence demonstrates consumer-owned relation vocabulary and
cross-interface targets within one catalog. The generic model retains the exact
relation identity, governing responsibility, and target
`GovernedObjectRef`. It does not copy claim statements, invariant statements,
or formal-assurance semantics.

### Governance Authority composition

Turnlock's Governance Authority profile already distinguishes ADR metadata,
product semantics, accepted decisions, stable invariant definitions, and the
formal-assurance graph as consumer-owned responsibilities. It also demonstrates
that a known responsibility may validly have `roles: {}`.

This evidence supports responsibility membership by exact existing identity
without requiring that every known responsibility currently have an authority
source and without introducing an object-level authority field.

## Ruu evidence

### ADR identities

Ruu's canonical ADR corpus supplies exact consumer-owned ADR identities with
lifecycle metadata governed by its ADR profile. Those identities can participate
in the same generic model while Ruu's stronger ADR contracts continue to own
identity, lifecycle, relation, scope, and body-integrity semantics.

### Specifications and architectural constraints

The Ruu specification and External Control Plane contract own normative product
semantics. Ruu also records architectural constraints in its consumer authority.
At the audited baseline, specification sections, headings, and architectural
constraints are not independently established as stable governed-object
identities for this extraction.

The generic model therefore retains no Ruu specification section, heading, or
architectural constraint as an invented object identity. Their semantic and
identity policy remains Ruu-owned.

### Qualification obligations

Ruu's qualification policy owns retained and post-baseline evidence boundaries,
replay rules, manifests, lineage, and qualification obligations. Those
obligations and artifacts do not establish a stable non-ADR governed-object ID
family at the audited baseline.

Qualification semantics, obligation meaning, evidence custody, and replay
policy remain Ruu-owned and excluded from the generic governed-object contract.

### Absence of stable non-ADR identities

The audited Ruu authority establishes no canonical `RUU-INV-*`, `RUU-REQ-*`,
`RUU-OBL-*`, or other stable non-ADR governed-object identity family.

That absence does not falsify the generic model: the model permits a consumer to
publish only the interfaces and exact identities it already owns. The absence
also does not authorize proto-ring to mint IDs, derive IDs from headings or
section numbers, or manufacture symmetry with Turnlock.

## Classification

| Classification | Disposition |
| -------------- | ----------- |
| Retained as generic | Exact opaque `(interface_id, object_id)` identity; exact opaque responsibility and relation identities; empty catalogs and interfaces; non-empty object responsibility membership; same-catalog target integrity; forward, self, cyclic, and cross-interface relations; duplicate rejection; non-semantic declaration order |
| Consumer-specific and excluded | Turnlock ADR, invariant, claim, and `normative_sources` meaning; Ruu product and architectural meaning; qualification obligations and evidence semantics; consumer relation vocabularies; semantic bodies and locators; universal object taxonomy, lifecycle, status, or applicability |
| Owned by Governance Authority | Responsibility identities, source-role assignments, responsibility-scoped authority, and optional precedence; the governed-object catalog receives no universal `SourceRole` |
| Deferred to #26 | Immutable binding registries; validation, evidence, projection, and catalog-currentness registries; catalog freshness or synchronization |
| Deferred to later state/API/mutation work | Exact `RepositoryGovernanceState` composition, coherent agent-facing APIs, and mutation behavior |

## Preserved boundary

The retained generic contract can enumerate exact consumer-owned object
identities, associate them with existing Governance Authority responsibilities,
and represent consumer-owned structural relations inside one catalog. It cannot
create consumer semantics, mint or canonicalize consumer IDs, assign universal
catalog authority, resolve semantic content, establish currentness, or replace
stronger consumer-specific contracts.
