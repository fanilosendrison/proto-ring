---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Canonical Governance Authority extraction evidence"
---

# Canonical Governance Authority extraction evidence

Status: non-normative extraction analysis.

This record preserves the consumer confrontation for proto-ring Issue #38. The
canonical generic semantics are established separately by
[`docs/contracts/governance-authority.md`](../contracts/governance-authority.md).
This record creates no Turnlock or Ruu product authority.

## Evidence baselines

The extraction was confronted with these exact default-branch states:

```text
proto-ring
07fa4e96dc2f762e5fbc99a8a3ffc820d5336b93

Turnlock-Rust
182be7f761a5bc50367a50e729aa393aaa20b219

Ruu
e7536db754c319b4b989bae2d376aeb3fb58dcb7
```

The evidence included each consumer's root directives, specifications, accepted
decisions, Projection Integrity binding, Shared Governance Provider binding,
formal or qualification governance, validation ownership, evidence custody,
historical material, explanatory material, and GitHub work-state ownership.

## Extraction method

Turnlock supplied the primary responsibility-based authority evidence. Ruu
confronted the candidate with a consumer-owned product-semantic order,
qualification responsibilities, retained historical evidence, and current
repository projections. The result is not a blind copy of Turnlock and is not
the intersection of Turnlock and Ruu.

Only source identity, responsibility identity, one role assignment per
source/responsibility pair, and optional responsibility-scoped authority
precedence are retained. Consumer artifact categories and consumer semantic
meaning remain local.

## Turnlock evidence

### Authority by responsibility

Turnlock assigns authority according to responsibility rather than one absolute
artifact rank. The normative specification owns product meaning, stable
invariant definitions, and canonical terminology. Accepted ADRs own accepted
decision history and may be co-authoritative with the specification for product
semantics where applicable.

The specification and accepted ADRs are therefore valid co-authorities for that
shared responsibility. Current Turnlock authority requires inconsistencies to be
reported; it does not define precedence between those co-authorities. The
generic model consequently preserves both authorities with no invented edge and
no global winner.

### Formal assurance and evidence

`formal/verification.yaml` owns the formal-assurance and traceability graph,
including provenance, assurance claims and domains, coverage, residual
assurance, bindings, review requirements, and evidence contracts. It is not
product semantics, canonical formal semantics, or verification evidence.

An accepted canonical formal representation would own checked abstract
mathematical semantics for its declared scope. That responsibility is currently
known but unassigned. `formal/reviews/` owns exact hostile semantic-review
evidence, while `formal/results/` owns bounded verification evidence for exact
mechanism-specific runs. These are distinct authorities rather than one
universal formal hierarchy.

### ADR metadata and projections

Canonical ADR records and frontmatter own ADR identity, lifecycle, outgoing
relations, governed scope, and body integrity. The ADR profile owns the
representation rules. The generated ADR index and derivable portions of the
maintained ADR history are secondary representations; independently authored
history narrative remains authoritative for its own responsibility.

`docs/formal/invariant-mapping.md` is a generated projection of the
formal-assurance graph. The terminology inventory is reviewed lexical material,
not terminology or product authority. Generated mappings, indexes, README
files, and explanatory projections do not acquire authority from their
representation.

### Repository governance and mixed roles

Root structured frontmatter owns repository-operational routes. Root directives
own agent guardrails. The Repository Integrity entry point owns validation
membership and order, while CI owns bootstrap execution. The local provider and
discovery-classification bindings own their repository-governance
responsibilities without becoming product authority.

`AGENTS.md` is therefore mixed-role: structured frontmatter owns routes, agent
prose owns operational guardrails, and neither becomes product semantics. The
ADR history is also mixed-role because its narrative is independently authored
while its derivable metadata is a secondary representation.

The repository Git tree owns current artifact state. GitHub Issue state,
Project fields, and native relationships remain the external authority for
engineering work state. Explanatory vision and other explicitly
non-authoritative material remain non-authoritative for product semantics.

## Ruu evidence

### Product-semantic authority order

Ruu declares a consumer-owned order for overlapping product-semantic authority:

```text
Ruu specification
> external control-plane contract
> accepted ADRs
```

The specification, external control-plane contract, and accepted ADRs remain
authorities within their declared scopes. The order is represented only as
Ruu-owned precedence for the product-semantic responsibility. It is not a
universal proto-ring hierarchy and does not make a lower source globally
non-authoritative.

The architecture overview, problem statement, historical design material,
audits, reports, and recorded outputs do not create product semantics.

### Qualification and historical authority

Ruu owns qualification policy through its directives and qualification policy
document. Retained qualification evidence and post-baseline qualification
evidence own their distinct bounded execution-evidence scopes without invented
precedence between them.

The immutable ADR-080 release package owns its bounded historical snapshot. The
retained lineage is a secondary representation of that immutable source.
Retained evidence projections preserve byte identity and historical custody;
they do not become current product authority or prove a current claim merely by
containing recorded success text.

Post-baseline registrations own their registration and replay bindings outside
the retained lineage. Qualification ownership, replay semantics, evidence
interpretation, and claims remain Ruu-owned.

### Current projections, ADR metadata, and work state

The repository Git tree owns current maintained artifact state. The current
repository manifest is a projection: a generated secondary representation of
that state. ADR frontmatter and canonical records own active ADR metadata; the generated index
and derivable maintained-history fields remain projections. The ADR profile owns
representation rules.

Root structured frontmatter owns repository-operational routes. Root directives
own repository, qualification, and validation guardrails. The Repository
Integrity entry point owns current validation membership and order, while CI
owns bootstrap and distinct replay execution.

GitHub Issue state, Project fields, and native relationships remain external
authority for engineering work state. Architecture, problem, and history
material remain explanatory or non-authoritative for product semantics unless a
separate Ruu-owned responsibility assigns authority.

## Classification

| Classification | Disposition |
|----------------|-------------|
| Consumer-independent and retained                   | Opaque logical source and responsibility IDs; exact three roles; lookup-state distinction; valid co-authorities; optional responsibility-scoped authority precedence; profile-v1 shape |
| Consumer-specific and excluded                      | Turnlock product, formal, review, evidence, terminology, and vision semantics; Ruu product ordering categories, qualification, evidence, history, and repository topology              |
| Already owned by existing proto-ring contract       | Repository-contained route resolution and containment by Canonical Governance Routing; projection modes, owner bindings, generation, currentness, and projection validation by Projection Integrity |
| Deferred to #25                                     | Decision, invariant, and obligation interfaces, lifecycles, identities, graphs, statuses, and semantic meaning                                         |
| Deferred to #26                                     | Provider and contract binding registries; detailed projection, validation, and evidence registries; generator and validation membership bindings       |
| Deferred to #27                                     | Exact-state-bound `RepositoryGovernanceState` composition                                                                                              |

No Product Intent requirement, universal artifact taxonomy, universal authority
hierarchy, external provider integration, evidence truth rule, formal or
qualification semantics, or mutation behavior is extracted.

## Preserved boundary

The retained contract can state that a source is an authority, secondary
representation, or explicit non-authority for one consumer-owned responsibility.
It cannot determine product meaning, select a universal authority winner,
identify the canonical owner of a particular projection, validate currentness,
interpret evidence, or transfer consumer authority into proto-ring.
