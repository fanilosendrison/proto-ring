---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Projection Integrity extraction inventory"
---

# Projection Integrity extraction inventory

Status: non-normative extraction analysis.

The canonical result is
[`docs/contracts/projection-integrity.md`](../contracts/projection-integrity.md).
This record preserves derivation evidence without becoming consumer authority.

## Evidence baselines

The primary extraction source was:

```text
fanilosendrison/turnlock-rust
eb9d26d6f27a164272cd5a2ca734c19fc4459c90
docs/repository-governance/turnlock-rust-projection-integrity.md
```

The principal confrontation consumer was:

```text
fanilosendrison/ruu
fb76cc340a9167c73b455259114d3d3691a9f692
```

Ruu evidence included its repository directives, active manifest generation,
qualification-layout verification, immutable release snapshot, retained lineage,
post-baseline registration, generated ADR projection, Repository Integrity
binding, and qualification/replay separation.

## Method

The analysis began with the maximum justified Turnlock candidate rather than a
symmetric intersection. Each Turnlock rule was tested for consumer independence,
then subjected to Ruu confrontation for falsification, distinction,
strengthening, or additional generic evidence. Equivalent Ruu implementation
was not required, and absence alone was not used to reject a justified generic
rule.

## Turnlock candidate dispositions

### Scope over mechanically derivable mutable facts

Disposition: extracted and generalized.

The method applies to mutable repository facts mechanically derivable from an
owner. Turnlock's concrete ADR, formal, filesystem, and live-work examples remain
local mappings and do not appear in the shared contract.

### One canonical owner

Disposition: extracted.

The rule is repository-governance methodology rather than product or formal
semantics. Ruu's canonical active sources, generated indexes, manifests, and
lineage provide independent supporting evidence.

### Reference mode

Disposition: extracted.

Referencing an owner without copying current value is generic and avoids a
synchronization obligation. Consumer reference targets remain local.

### Generated projection mode

Disposition: extracted and strengthened.

Turnlock requires direct generation from the owner. Ruu demonstrates
deterministic index, lineage, and manifest generation plus rejection of stale
committed output. The generic contract states reproducibility and preserves the
rule that generation does not confer semantic authority.

### Validated maintained projection mode

Disposition: extracted and strengthened.

Turnlock requires mechanical validation of duplicated derivable fields. Ruu's
exhaustive layout checks demonstrate that a guard may need to detect missing,
extra, stale, and unexpectedly present entries. The generic wording requires
coverage of every duplicated derivable field while leaving significance and
membership to the consumer.

### Bounded historical snapshot mode

Disposition: extracted and strengthened.

Turnlock recognizes revision- or migration-bounded historical representations.
Ruu adds concrete evidence for immutable release snapshots, retained byte
identity, lineage, and separation from current active state. The generic
contract requires explicit immutable scope and forbids presenting a snapshot as
current, while leaving custody and replay claims local.

### Projection-chain prohibition

Disposition: extracted.

Both repositories rely on comparison with canonical sources rather than another
projection. Ruu's generation and verification flow supports the direct-source
rule. Runtime intermediates are distinguished from authoritative repository
projection chains.

### Forbidden manual current-state mirrors

Disposition: extracted.

Turnlock's examples are consumer mappings, but the generic prohibition on
unguarded manual mirroring is consumer-independent. Ruu's current manifest and
generated ADR index provide additional evidence that ordinary diligence is not a
synchronization mechanism.

### Validation membership ownership

Disposition: extracted and confirmed by Ruu.

Turnlock gives the canonical suite one local executable owner. Ruu now gives its
Repository Integrity profile one local owner while Qualification invokes it as a
distinct prerequisite. The shared contract owns only the single-owner rule, not
either command set or ordering.

### Change protocol

Disposition: extracted.

The reference/generate/validate/snapshot decision sequence is generic. Consumer
paths, generated artifacts, authority mappings, and validation commands remain
local.

### Stale-projection completion barrier

Disposition: extracted and strengthened fail-closed.

Turnlock states that work is incomplete while a required projection is stale.
Ruu demonstrates current-manifest, layout, index, lineage, and replay checks that
reject missing, stale, unexpected, or undetermined state. The generic contract
makes those outcomes non-complete without mandating Ruu's manifest mechanism.

### No checker solely for policy prose

Disposition: retained as mechanism neutrality rather than copied literally.

The shared contract requires mechanical enforcement where an obligation is
mechanically derivable, but it does not mandate one universal checker or a
checker that merely searches for policy wording.

## Turnlock mappings retained locally

All concrete owner mappings remain Turnlock-owned, including mappings for ADR
frontmatter, maintained histories, formal traceability, generated formal
mappings, bounded verification evidence, repository artifact presence,
accepted commitments, validation entry points, automation, and live engineering
work state.

The extraction transfers no product semantics, formal-assurance semantics,
accepted decision history, repository paths, Project coordinates, or validation
membership.

## Ruu confrontation results

### Exhaustive active manifest

Ruu demonstrates a strong projection and state-registration mechanism. It does
not justify a universal manifest requirement because Turnlock satisfies
projection currentness through other mechanisms. The generic result requires
complete currentness for consumer-declared scope, not one representation.

### Immutable historical evidence

Ruu strengthens bounded historical snapshot semantics by distinguishing
immutable historical custody from current active state and from replayed claim
validity. The shared contract incorporates that distinction without moving
qualification evidence or replay authority.

### Retained lineage

Ruu lineage demonstrates direct source binding, exhaustive registration, content
hashing, path/provenance checks, and fail-closed drift detection for its retained
scope. These support direct-source and currentness guarantees. Exact lineage
shape and artifact mapping remain local.

### Generated active artifacts

Ruu's deterministic generators demonstrate that a generated projection remains
a consumer artifact and does not become authority. Its workflow also confirms
that generation is preparation or repair, not evidence that the prior stale
state was current.

### Qualification separation

Ruu demonstrates that projection currentness and artifact custody can be
Repository Integrity obligations while qualification replay remains a distinct
claim-evidence layer. The shared contract preserves that boundary.

### Consumer-specific behavior

Ruu product authority, ADR policy, qualification claims, manifests, lineage,
retained snapshots, post-baseline registrations, generated paths, and replay
commands remain local. None is copied as a generic owner mapping.

## Governed Identity exclusion

The findings `fanilosendrison/turnlock-rust#35` and
`fanilosendrison/ruu#49` remain open. Their comments contain adjudication and
cross-consumer analysis, but no resulting accepted consumer repository contract
has independently become authoritative.

Their proposed Governed Identity semantics are therefore not incorporated into
the Projection Integrity contract. In particular, this extraction does not
select a minimum-sufficient identity rule, canonicalization model, identity
domain taxonomy, or universal invalidation contract.

Projection Integrity states only that a consumer must supply the state binding
required by its existing authority and must fail closed when required
currentness cannot be established. The separate findings remain separate work.

## Result

The resulting proto-ring contract is the strongest consumer-independent
Projection Integrity governance supported by current concrete evidence. It is
not a blind Turnlock copy, not `Turnlock ∩ Ruu`, and not a universal repository
profile or governed-knowledge model.
