---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-projection-integrity"
severity: "strict"
name: "Canonical Projection Integrity contract"
---

# Canonical Projection Integrity contract

## Purpose

Projection Integrity governs mutable repository facts that are mechanically
derivable from another repository authority.

Its core rule is:

```text
one mutable derived fact
→ one canonical owner

every secondary representation
→ Reference
  OR Generated projection
  OR Mechanically validated maintained projection
  OR Bounded historical snapshot
```

A consumer owns the concrete facts, owners, paths, bindings, generation rules,
validation membership, historical boundaries, and authority order to which this
contract is applied.

## Canonical ownership

Every mutable derived fact must have one canonical owner. The owner is the only
repository artifact or external authority whose current value is authoritative
for that fact.

A secondary representation must not become a second current owner. Its mode,
canonical source, and currentness or historical boundary must be unambiguous.

When a representation contains both derived fields and independently authored
content, this contract governs only the derivable portion. Independent content
retains the authority assigned by the consumer.

## Projection modes

Every secondary representation of a mutable derived fact must use exactly one
of the following modes for that fact.

### Reference

A reference identifies the canonical owner without reproducing the mutable
value. References should be preferred when a copied value is unnecessary.

A reference does not transfer authority to the referring artifact.

### Generated projection

A generated projection is deterministically derived directly from the canonical
owner. Its generator and source binding must be sufficient to reproduce the
required output for the same governed source state.

A generated projection does not acquire semantic authority merely because it is
committed, rendered, hashed, indexed, or consumed by another process.

### Mechanically validated maintained projection

A mechanically validated maintained projection contains manually authored
material while reproducing one or more mutable derivable fields. A mechanical
guard must cover every duplicated derivable field and compare it directly with
the canonical owner.

Validation must detect a missing, extra, stale, reordered, or otherwise
inconsistent duplicated field whenever that distinction is significant under
the consumer binding.

### Bounded historical snapshot

A bounded historical snapshot preserves a past representation only under an
explicit immutable scope such as a revision, date, migration, release, evidence
subject, or other fixed boundary.

The snapshot must not present its value as current. Its custody, preservation,
lineage, and replay obligations remain consumer-owned. Historical preservation
does not by itself establish the validity of the historical claim.

## Direct-source rule

Projection chains are forbidden for current derived facts. A generated or
validated projection must derive directly from the canonical owner, not from a
secondary projection.

A consumer may construct intermediate runtime data while computing a projection,
but no intermediate representation may silently become the authoritative source
for a downstream repository projection.

## Currentness and completion

A required current projection must correspond to its canonical source under the
consumer's declared binding. Repository work is incomplete while any required
projection is stale, missing, unexpectedly present, or undetermined.

Projection validation must fail closed when the canonical source, required
projection membership, source-to-projection binding, or currentness result
cannot be established.

A generator is repair, not proof of pre-existing currentness. Running a
generator may produce the correct projection, but it must not make an integrity
check report that the stale pre-generation repository state was current.

## Validation ownership

Validation membership, ordering, prerequisites, and environment are mutable
repository facts and must have one canonical owner. Documentation and automation
must reference that owner or be mechanically checked against it rather than
maintaining independent command lists.

A validation entry point may be a generated or validated projection of that
membership only when its binding preserves the same canonical ownership.

## Synchronization rule

Agent memory, review attention, and ordinary diligence are not synchronization
mechanisms.

When a secondary representation must reproduce a mutable derived value, the same
change that introduces or changes that projection must provide generation or a
mechanical guard. A deferred promise to synchronize later does not satisfy this
contract.

## Authority and evidence boundaries

Projection status does not determine product meaning, decision acceptance,
formal validity, qualification success, or evidence truth. Generated and
validated artifacts retain only the authority assigned by their consumer.

Content hashes, manifests, lineage records, immutable snapshots, and replay may
be strong consumer mechanisms. None is a universal representation requirement.
A consumer may require exhaustive registration for a governed scope without
forcing another consumer to adopt the same mechanism.

## Mechanically significant embedded assertions

A carrier's structural validity does not establish the semantic currentness or
truth of an assertion contained by that carrier.

A mechanically significant assertion is an ordinary Projection Integrity
relationship only when the consumer has already declared:

- the governed responsibility;
- a consumer-owned authority source for the fact;
- the carrying representation as a secondary representation; and
- the required source-to-representation relation and currentness obligation.

Artifact hashing, registration, manifest inclusion, byte identity, replay,
filesystem metadata, Git metadata, or persistence MUST NOT be promoted into
fact authority merely because it exists.

When required authority cannot be established, Projection Integrity MUST NOT
invent a canonical source or create a false projection relation. The unresolved
authority state remains governed by Canonical Governance Authority, and any
mandatory consumer currentness obligation MUST fail closed through its owning
evaluation path.

Projection Integrity establishes correspondence and currentness only. It does
not establish qualification truth, evidence truth, or the semantic truth of a
consumer claim.

## Governed identity boundary

Projection Integrity answers where the current source fact is owned and how a
secondary representation relates to that owner. It does not define a universal
identity, serialization, canonicalization, equivalence, or invalidation model
for governed objects.

The consumer must supply whatever state binding its accepted authority requires
to determine that a source and projection correspond. When that required
binding cannot be established, currentness must fail closed. This rule does not
select the binding or promote a representation detail into identity semantics.

## Persistent Projection Registry

Model version 1 persists consumer-owned projection relations in canonical
structured frontmatter:

```yaml
projection_registry:
  model_version: 1
  authority:
    responsibility: <GovernedResponsibilityId>
    source: <GovernedSourceId>
  projections:
    <ProjectionId>:
      responsibility: <GovernedResponsibilityId>
      canonical_source: <GovernedSourceId>
      secondary_source: <GovernedSourceId>
      mode: reference
      validation: <ValidationId>
```

The registry carrier is the authoritative source for its declared Projection
Registry responsibility. That authority does not make the registry authority
over projected facts.

A `ProjectionId` is opaque, non-empty, stable, consumer-owned, and identifies a
projection relationship rather than a path. Proto-ring infers no meaning from
its spelling and does not canonicalize physical paths into projection identity.

The registry records only relations where a secondary representation
mechanically represents a fact whose authority is elsewhere. It is not an
inventory of every repository authority.

## Authority and direct-source composition

Every projection names one existing responsibility and two distinct existing
sources. `canonical_source` must have Governance Authority role `authority` for
the responsibility. `secondary_source` must have role
`secondary_representation` for the same responsibility.

A source that is only secondary for the responsibility cannot become canonical
for another current projection under that responsibility. These rules
mechanically preserve the direct-source rule without duplicating source or
responsibility identity.

## Persistent mode fields

The registry uses exactly the four modes already defined by this contract:

```text
reference
generated
mechanically_validated_maintained
bounded_historical_snapshot
```

A `generated` projection additionally requires:

```yaml
generator_source: <GovernedSourceId>
```

The generator source must exist and have role `non_authoritative` for the
projection responsibility. It is a logical production/repair binding, not a
command line. All other modes forbid `generator_source`. The registry defines
no generator execution, arguments, environment, provisioning, repair, or
mutation. Generation never proves that pre-generation state was current.

A `bounded_historical_snapshot` additionally requires:

```yaml
boundary_source: <GovernedSourceId>
```

The boundary source must exist and have role `authority` for the projection
responsibility. All current modes forbid `boundary_source`. Model version 1
creates no `HistoricalBoundaryId`. Historical validation establishes only the
declared boundary, custody, or integrity condition and never historical claim
truth.

## Repository Integrity validation binding

Every projection declares one `ValidationId` from the consumer's persistent
Repository Integrity profile. Current modes use that validation for declared
currentness or conformance. Bounded historical mode uses it only for the
boundary, custody, or integrity condition described above.

The Projection Registry contains no command, environment, exit code, validator
implementation, ordering, prerequisite, or execution policy. Those facts
remain owned by Repository Integrity.

The cross-capability declaration rule is:

```text
projection_integrity declared
-> repository_integrity must also be declared
```

Projection Registry currentness uses Repository Integrity ValidationIds rather
than creating a second validation system.

## Optional governed-object target

A projection may narrow its relation to one existing Governed Object:

```yaml
target:
  interface: <InterfaceId>
  object: <ObjectId>
```

The target does not replace canonical or secondary source identity. Targets are
not required for corpus-wide or repository-wide projections. One physical
Governed Objects profile may participate in multiple relations under distinct
responsibilities without moving semantic bodies into the catalog.

## Optional Governance Binding reference

A projection may reference one existing Governance `BindingId`:

```yaml
binding: <BindingId>
```

When present, the projection responsibility must equal the binding authority
responsibility and its canonical source must equal the binding authority
source. A binding is optional for ordinary repository projections and does not
create a new binding identity namespace.

## Executable realization composition

An authoritative executable-provider pin may have two distinct direct
projections:

```text
authoritative executable pin source
    |-> structured Governance Binding Registry representation
    `-> effective installed/executed provider realization
```

Both relations point directly to the authority; neither projects from the
other. An effective realization is expected to use `generated` mode with a
non-authoritative environment or provisioning source and a ValidationId that
checks correspondence to the authoritative binding.

The generic model names no package file, package manager, programming language,
virtual environment, derivation, container, or installation mechanism. Logical
sources may have no repository path, and distinct logical source IDs may share
one physical repository target.

## Active ambiguity

For one effective current combination of:

```text
responsibility
secondary_source
optional target
optional binding
```

the registry must not contain conflicting canonical-source or mode
declarations. Distinct facts represented by one physical carrier require
distinct logical source IDs. Bounded historical snapshots are not active
current declarations.

## Current consumer confrontation

The model represents Turnlock-Rust relations for generated ADR indexes,
mechanically validated maintained ADR history metadata, generated formal
invariant mappings, governed-object catalog currentness under distinct ADR,
invariant, and formal-claim responsibilities, Governance Binding Registry
representation, and effective provider realization.

It does not preserve the obsolete Shared Governance Provider binding-file
projection or treat a former Python validation-membership entry point as
permanent authority after persistent Repository Integrity adoption.

The model represents Ruu relations for generated ADR indexes, mechanically
validated maintained ADR history, generated active repository manifests,
generated retained lineage, bounded historical retained qualification
artifacts under custody validation, current ADR catalog identity, Governance
Binding Registry representation, and effective provider realization.

Registrations that are themselves consumer authorities remain outside the
registry unless they are actually secondary representations. Qualification
replay truth remains outside Projection Integrity.

These confrontation examples do not add consumer paths, installation choices,
product meaning, or qualification semantics to the generic registry.

## Change protocol

Before introducing or changing a mutable derived fact or secondary
representation, the consumer must:

1. identify the canonical owner;
2. classify the secondary representation in exactly one projection mode;
3. prefer reference when copying is unnecessary;
4. bind generation or validation directly to the canonical owner;
5. identify required currentness and historical boundaries;
6. keep consumer-specific authority and evidence local; and
7. make every mechanically checkable synchronization obligation mechanical in
   the same change.

The change is incomplete until every required projection is current and every
applicable validation passes for the resulting repository state.

## Relationship to Repository Integrity

Projection Integrity defines ownership, projection modes, direct-source
relationships, and currentness obligations. Repository Integrity evaluates
whether one exact current repository state satisfies the consumer-owned
obligations that include required projection currentness.

Projection repair remains outside both integrity verdicts. Qualification,
formal assurance, and historical replay remain separate evidence layers.
