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

## Governed identity boundary

Projection Integrity answers where the current source fact is owned and how a
secondary representation relates to that owner. It does not define a universal
identity, serialization, canonicalization, equivalence, or invalidation model
for governed objects.

The consumer must supply whatever state binding its accepted authority requires
to determine that a source and projection correspond. When that required
binding cannot be established, currentness must fail closed. This rule does not
select the binding or promote a representation detail into identity semantics.

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
