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
consumer decision history, consumer formal or qualification evidence, or the
Product Intent of any future Ring product.

The current proto-ring repository boundary is:

1. `docs/bootstrap/turnlock-extraction-audit.md` is non-normative historical
   bootstrap analysis. It preserves extraction reasoning but creates no
   consumer authority.
2. Explicitly accepted authority in each consumer repository remains authoritative
   for that consumer.
3. Turnlock ADR-050 authorizes external shared governance implementation while
   retaining Turnlock repository authority.
4. Ruu ADR-083 authorizes external shared governance implementation while
   retaining Ruu repository authority.
5. Shared governance becomes proto-ring-owned only after an explicit repository
   change establishes a consumer-independent contract or mechanism derived from
   demonstrated consumer needs and the required consumer preservation,
   conformance, or intentional-strengthening obligations are satisfied.
6. GitHub Issues and this Project govern work. They do not themselves establish
   shared governance semantics or consumer product semantics.

Do not infer a complete future Ring Product Intent from the current proto-ring
repository or this Project.

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

The current proto-ring extraction program is a dependency-ordered migration of
already demonstrated governance mechanisms. Migration ordering belongs in native
Issue dependencies, not in an artificial lifecycle-phase taxonomy.

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
  extraction critical path or whose omission would permit an integrity breach
  in the extraction mechanism.
* `P1`: work required by the current extraction program but whose delay does not
  currently block all meaningful progress.
* `P2`: important retained downstream extraction or governance work that is not
  on the current critical path.
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
4. an extraction result that changes which downstream adoption work is
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
3. identify the current extraction critical path;
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

Every normal proto-ring extraction Issue MUST state:

* the concrete extraction or adoption outcome;
* the source mechanism or contract being factored;
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

Proto-ring factorization MUST NOT treat any consumer as a privileged reference
implementation.

Shared governance is also NOT limited to behavior that is already identical
across consumers.

Every shared-governance factorization follows this sequence:

```text
observe concrete governance in multiple consumers
→ compare their strengths, constraints, evidence, and failure modes
→ derive the strongest justified consumer-independent contract
→ preserve every demonstrated consumer guarantee unless an explicit
  strengthening intentionally supersedes it
→ implement the generic contract or mechanism in proto-ring
→ validate proto-ring independently
→ pin an immutable proto-ring identity in each consumer
→ bind the shared mechanism through consumer-owned authority and configuration
→ prove preservation, equivalence, or intentional strengthening as applicable
→ remove only the local duplication that has actually become shared
→ revalidate the complete affected consumer
````

"The strongest justified contract" means the strongest contract supported by
demonstrated consumer needs and evidence. It does not mean the strongest
contract that can be imagined.

A difference between consumers is evidence to analyze. It is not automatically
generic and it is not automatically local.

When one consumer demonstrates a stronger governance property than another,
proto-ring MUST determine whether that property is consumer-independent and
justified for the other consumer before either extracting it or leaving it
local.

Do not weaken a demonstrated guarantee merely to make implementations look
identical.

Do not strengthen a consumer merely to make extraction aesthetically uniform.

Do not invent speculative governance unsupported by a concrete demonstrated
need.

Consumer-specific authority, validation membership, ordering, bindings,
profiles, product semantics, qualification claims, evidence, provenance,
generated artifacts, and historical snapshots remain consumer-owned unless an
explicit later authority change says otherwise.

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
