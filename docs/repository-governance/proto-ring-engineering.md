---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "agent-directives"
domain: "proto-ring-repository-governance"
severity: "strict"
name: "Proto-Ring Engineering GitHub Project profile"
---

# Proto-Ring Engineering GitHub Project profile

Apply the shared GitHub Engineering Projects operational protocol before using
this profile. This file contains only proto-ring-specific routing, authority,
work-state, classification, scheduling, and workflow policy.

## Fixed routing

- GitHub owner: `fanilosendrison`
- Default repository: `fanilosendrison/proto-ring`
- Project title: `Proto-Ring Engineering`
- Project type: user-owned GitHub Project V2

Resolve an unqualified `Issue #N` as `fanilosendrison/proto-ring#N`.

The Project number, node identifiers, field identifiers, option identifiers,
item identifiers, and Project URL are live GitHub coordinates. Resolve them
mechanically from the owner and exact Project title instead of copying them into
this profile as mutable repository state.

## Authority boundary

Proto-Ring Engineering is the authority for durable proto-ring work existence,
classification, scheduling priority, and workflow state.

It is not authority for Turnlock product semantics, Ruu product semantics,
consumer decision history, consumer formal or qualification evidence, or
semantic choices that belong to another consumer authority.

Proto-ring is an empirical bootstrap and factorization repository.

Its authority extends only to proto-ring contracts and reusable mechanisms
explicitly accepted into this repository for their stated scope.

The current proto-ring repository boundary is:

1. `docs/bootstrap/turnlock-extraction-audit.md` is non-normative historical
   bootstrap analysis. It preserves extraction reasoning but creates no
   consumer authority.
2. Explicitly accepted authority in each consumer repository remains
   authoritative for that consumer.
3. Turnlock ADR-050 authorizes external shared governance implementation while
   retaining Turnlock repository authority.
4. Ruu ADR-083 authorizes external shared governance implementation while
   retaining Ruu repository authority.
5. Existing shared governance already accepted into proto-ring retains the
   authority established for its current proto-ring scope.
6. Consumer-derived shared governance becomes proto-ring-owned only after an
   explicit repository change establishes the applicable consumer-independent
   contract or mechanism and the required consumer preservation, conformance,
   or intentional-strengthening obligations are satisfied.
7. GitHub Issues and this Project govern proto-ring work. They do not themselves
   establish consumer product semantics.

Do not infer new shared governance merely from repository structure, Issues,
Project state, implementation convenience, or speculative future use.

Until concrete consumer evidence justifies a consumer-independent concern, keep
it outside proto-ring.

## Workflow-status mapping

Use exactly these `Status` values:

- `Backlog`
- `Ready`
- `In Progress`
- `Review`
- `Done`

Their meanings are:

- `Backlog`: retained work that is not currently independently executable.
- `Ready`: independently executable work whose required native blockers are
  cleared and whose accepted scope is sufficient for execution.
- `In Progress`: active execution.
- `Review`: execution produced a reviewable result that has not yet satisfied
  completion conditions.
- `Done`: the Issue's own acceptance criteria are satisfied and required
  repository changes and validation are complete.

A direct user request may select work outside normal autonomous pickup order,
but it does not alter dependencies, authority, or accepted semantics.

## No Phase field

Proto-Ring Engineering MUST NOT define a `Phase` field.

The current proto-ring program contains dependency-ordered factorization,
extraction, strengthening, and consumer-adoption work.

Work ordering belongs in native Issue dependencies, not in an artificial
lifecycle-phase taxonomy.

Do not encode `Bootstrap`, `Extraction`, `Adoption`, `Migration`,
`Consolidation`, or similar process stages as Project phases.

A Phase taxonomy may be introduced only by a later explicit governance decision
supported by demonstrated need.

## Live work-state ownership

Every mutable live work-state fact has exactly one canonical GitHub owner.

| Information | Canonical owner |
| --- | --- |
| Durable work item | GitHub Issue |
| Open/closed Issue state | GitHub Issue state |
| Workflow state | Project `Status` |
| Work-item classification | Project `Kind` |
| Scheduling priority | Project `Priority` |
| Parent/sub-issue structure | Native GitHub Issue relationships |
| Blocking/blocked-by structure | Native GitHub dependency relationships |
| Pull Request linkage | Native GitHub relationships |
| Acceptance criteria for Issue X | Issue X body |
| Extracted implementation | proto-ring repository artifacts after accepted extraction |
| Consumer product semantics | consumer repository authority |

Apply:

```text
Reference, do not mirror.
```

Issue bodies MUST NOT maintain a second copy of current `Status`, `Kind`,
`Priority`, dependency state, parent/sub-issue membership, or Pull Request
relationship state.

Do not create a manually maintained roadmap table containing current Issue
states.

## Kind

Use exactly these `Kind` values:

* `Agent Task`
* `Follow-up`
* `Finding`

Definitions:

* `Agent Task`: independently scoped work whose outcome can be directly
  executed once its native blockers are cleared.
* `Follow-up`: consumer adoption, binding, migration, or other downstream work
  made necessary by a prior extracted result.
* `Finding`: a validated concern that requires separate adjudication,
  correction, or an explicit semantic decision.

An observation is not a `Finding` until validated.

A Finding never ratifies its proposed resolution merely by existing.

## Priority

Use exactly:

* `P0`
* `P1`
* `P2`
* `P3`

Definitions:

* `P0`: work whose delay currently blocks meaningful progress on the active
  proto-ring critical path or whose omission would permit an integrity breach
  in an established proto-ring mechanism.
* `P1`: work required by the current proto-ring program but whose delay does not
  currently block all meaningful progress.
* `P2`: important retained downstream extraction, adoption, or governance work
  that is not on the current critical path.
* `P3`: useful retained work that can safely wait without meaningful current
  scheduling cost.

Priority is scheduling state only.

Priority never overrides:

* native dependencies;
* consumer authority;
* preservation, equivalence, intentional-strengthening, or conformance obligations;
* validation requirements;
* immutable pinning requirements;
* repository-specific evidence boundaries.

Priority does not imply readiness.

A blocked `P0` item may remain `Backlog`.

## Priority revalidation

Re-evaluate all open Project Issues after any of these events:

1. creation of a durable Issue;
2. Issue completion, closure, or reopening;
3. creation or removal of a native dependency;
4. an accepted proto-ring result that changes which downstream work is
   executable;
5. an accepted authority change in proto-ring, Turnlock, or Ruu that changes an
   extraction prerequisite or integrity boundary;
6. a validated cross-cutting governance Finding.

Do not trigger a full priority pass solely because of formatting, comments,
assignees, labels, or a Priority edit itself.

For each priority pass:

1. read canonical live Issue state, Project fields, and native dependencies;
2. read the current proto-ring governance profile and the exact consumer
   authority relevant to affected work;
3. identify the current proto-ring critical path;
4. assign `P0`, then `P1`, then `P2`, then `P3` according to the definitions
   above;
5. mutate only Project `Priority` fields whose value changes;
6. re-read every changed item and verify that no unrelated field or repository
   state changed.

Do not invent numeric scores, weights, severity scales, or secondary ranking
fields.

## Autonomous pickup

Autonomous pickup uses this exact order:

1. candidates MUST have `Status = Ready`;
2. choose the highest `Priority`: `P0`, then `P1`, then `P2`, then `P3`;
3. never choose work with an unresolved native blocker;
4. when equal-priority Ready Issues are independent, retain the Project's
   existing manual order as the tie-break;
5. a direct user selection overrides pickup order for that action only.

Do not use Priority as a dependency, cancellation, or preemption mechanism.

## Project views

The intended Project views are:

### Now

Board view filtered to:

```text
Status = In Progress
```

### Agent Queue

Table view filtered to:

```text
Status = Ready OR Status = In Progress
```

Visible fields MUST include:

```text
Status
Kind
Priority
```

### Backlog

Table view filtered to:

```text
Status = Backlog
```

Visible fields MUST include:

```text
Status
Kind
Priority
```

### Findings

Table view filtered to:

```text
Kind = Finding
```

Visible fields MUST include:

```text
Status
Kind
Priority
```

Do not create Phase-specific views.

## Issue requirements

Every normal proto-ring Issue MUST state:

* the concrete normative, extraction, adoption, or implementation outcome;
* the accepted normative authority or concrete source mechanism that justifies
  the work;
* the authority boundary that must remain unchanged;
* exact repository surfaces allowed to change;
* preservation, equivalence, intentional-strengthening, or conformance
  obligations as applicable;
* mechanically checkable acceptance criteria;
* explicit non-goals where scope expansion would otherwise be plausible.

Do not copy current dependency state, Status, Kind, or Priority into Issue prose.

Dependencies belong only in native GitHub relationships.

An Issue may identify stable semantic prerequisites without restating their live
relationship state.

## Factorization discipline

This section governs proto-ring's defining consumer-derived factorization work.

Consumer-derived factorization establishes proto-ring contracts and mechanisms
for their accepted proto-ring scope. The current bootstrap direction is:

```text
Turnlock
→ primary source of the governance method
→ maximum consumer-independent extraction candidate

Ruu
→ principal current confrontation consumer
→ falsification, distinction, strengthening, and additional-generic-property
   evidence

→ strongest justified consumer-independent proto-ring contract or mechanism
```

Turnlock is currently the primary source from which the governance method is
being maximally factorized. This is an extraction direction, not a transfer of
Turnlock authority. Turnlock remains a consumer authority for Turnlock-owned
semantics, decisions, evidence, configuration, topology, and bindings, and a
Turnlock implementation is not an unquestioned proto-ring reference
implementation.

### Maximum justified extraction

For every relevant Turnlock governance property or mechanism under
consideration, apply this rule:

```text
consumer-independent repository governance
→ candidate for proto-ring extraction

intrinsically Turnlock-specific
→ remains Turnlock-owned
```

A candidate is consumer-independent only after analysis establishes that it is
repository-governance methodology rather than Turnlock product semantics,
Turnlock-specific authority, local configuration, historical evidence,
repository topology, or another consumer-local concern. This proof of
independence is required for every extraction; Turnlock containing a property
is not sufficient by itself.

The absence of an equivalent mechanism in Ruu MUST NOT by itself prevent
extraction. A property demonstrated in Turnlock does not need prior duplication
in Ruu before it can be considered consumer-independent. Common implementation
in multiple consumers is strong evidence, but it is not a prerequisite.

### Consumer confrontation

Confront every Turnlock-derived candidate with Ruu and with any other concrete
consumer available for the work. Ruu is the principal current confrontation
consumer. The confrontation MUST determine whether Ruu:

* falsifies the proposed generalization;
* exposes a missing distinction;
* demonstrates a stronger consumer-independent guarantee;
* demonstrates an additional consumer-independent governance property absent
  from Turnlock; or
* has only consumer-specific behavior that must remain local to Ruu.

A false generalization MUST be narrowed or its consumer-specific portion
returned to Turnlock. A Ruu-specific difference is not automatically generic.
Ruu's absence of the candidate is not evidence against it. Future consumers
MAY provide the same kind of falsification, distinction, strengthening, or
additional-generic-property evidence; this bootstrap doctrine is not limited to
Ruu.

### Strengthening rule

When Ruu demonstrates a stronger property:

```text
if consumer-independent and justified
→ strengthen proto-ring

if Ruu-specific
→ leave it in Ruu
```

Do not weaken Turnlock merely to match Ruu. Do not preserve a weaker Turnlock
formulation merely because it was established first. The final proto-ring
contract or mechanism MAY therefore be stronger than either consumer's starting
implementation, but the strengthening must remain supported by concrete
consumer evidence and analysis rather than speculation.

### Anti-intersection and anti-copy rules

The following derivation is prohibited:

```text
shared = intersection(Turnlock, Ruu)
```

Proto-ring MUST NOT be reduced to the lowest common denominator, and it MUST
NOT require two-consumer duplication before a property can be considered
consumer-independent.

The opposite error is also prohibited:

```text
Turnlock contains X
≠
X automatically belongs in proto-ring
```

Every extraction requires the consumer-independence analysis above. Neither
blind Turnlock copying nor symmetric intersection is an acceptable method.

The factorization and adoption sequence is therefore:

```text
observe Turnlock governance
→ identify the maximum consumer-independent extraction candidate
→ confront the candidate with Ruu and other concrete consumers
→ narrow false generalizations
→ strengthen with better consumer-independent guarantees
→ establish the proto-ring contract or mechanism
→ validate proto-ring independently
→ pin an immutable proto-ring identity in each adopting consumer
→ bind it through consumer-owned authority and configuration
→ prove preservation, equivalence, or intentional strengthening as applicable
→ remove only the local duplication that has actually become shared
→ revalidate the complete affected consumer
```

"The strongest justified contract" means the strongest contract supported by
concrete consumer needs and evidence. It does not mean the strongest contract
that can be imagined. Proto-ring MUST NOT invent speculative governance.

Consumer-specific authority, validation membership, ordering, bindings,
profiles, product semantics, qualification claims, evidence, provenance,
generated artifacts, and historical snapshots remain consumer-owned unless an
applicable consumer authority change establishes otherwise.

The fact that a property is consumer-local today does not by itself establish
that the property must remain consumer-local permanently.

A consumer adoption proof therefore need not always be exact behavioral parity.
The required proof may be:

```text
exact equivalence
OR
preservation of an existing guarantee
OR
an explicitly justified strengthening
```

but weakening an existing demonstrated guarantee is never an acceptable
factorization result.

## Findings and scope expansion

When execution exposes a problem outside the accepted Issue scope:

* do not silently expand the current Issue;
* validate whether the observation is real;
* create a separate `Finding` only if the concern must survive independently;
* preserve the earliest authority layer requiring resolution;
* keep proposed resolution distinct from accepted authority.

A Finding that requires new shared semantics remains non-executable until the
required authority resolves that choice.

## Repository validation

Use only validation that actually exists in the current proto-ring repository.

Until a stronger canonical validation suite is introduced, every repository
change MUST at minimum pass:

```bash
git diff --check
```

Once proto-ring introduces executable tests or a canonical repository validation
entry point, that repository-owned entry point becomes mandatory in addition to
the minimum patch-whitespace check.

Do not claim nonexistent validation evidence.

## Completion condition

An Issue reaches `Done` only when:

* its own acceptance criteria are satisfied;
* all repository changes it owns are committed;
* required validation passes;
* required generated artifacts are current;
* required consumer preservation, conformance, or intentional-strengthening
  evidence exists for consumer-adoption work;
* no current Issue body or repository document manually mirrors mutable live
  Project state introduced by the work.
