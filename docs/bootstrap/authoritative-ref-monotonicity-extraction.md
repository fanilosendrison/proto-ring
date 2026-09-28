---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Authoritative Ref Monotonicity extraction evidence"
---

# Authoritative Ref Monotonicity extraction evidence

Status: non-normative extraction analysis.

This record preserves the evidence, confrontation results, and factorization
boundary for Authoritative Ref Monotonicity. The shared authority is the
separate contract at
[`docs/contracts/authoritative-ref-monotonicity.md`](../contracts/authoritative-ref-monotonicity.md).

## Turnlock primary evidence

The preparation baseline is:

```text
fanilosendrison/turnlock-rust
d8b8c9f48fae43006a0764ac8542160671ede474
AGENTS.md
docs/repository-governance/turnlock-rust-worktree-management.md
```

Turnlock already requires authoritative publication through an ordinary
fast-forward push and prohibits force-push publication. Its worktree-management
policy additionally prohibits force-with-lease history rewrite and deletion of
the authoritative branch.

The demonstrated repository-governance property is that authoritative
publication already assumes monotonic main ancestry and non-deletion. The
substance, rather than a long quotation or the consumer-local branch identity,
is the extraction evidence.

## Ruu confrontation

The confrontation baseline is:

```text
fanilosendrison/ruu
ed187c0c58b626f82404ab475d9dc4c4b36330fd
```

The confrontation establishes all of the following:

- Ruu does not falsify the generic ref-monotonicity guarantee.
- Ruu is already a mandatory proto-ring governance consumer under ADR-085.
- Accepted ADR Body Immutability is consumed by Ruu and depends on preserved
  historical acceptance anchors remaining reachable.
- Ruu contributes no evidence requiring the generic guarantee to be
  Ruu-specific.
- Absence of an existing Ruu-local equivalent is not evidence against
  extraction under proto-ring factorization doctrine.

The contract is not derived as an intersection of Turnlock and Ruu. Turnlock is
the primary evidence source, while Ruu is a confrontation consumer capable of
falsifying an unjustified generalization.

## Existing proto-ring evidence

The observed proto-ring baseline is:

```text
fanilosendrison/proto-ring
72e6e9615703e4d3293175f023d1d953632c45a8
docs/contracts/accepted-adr-body-immutability.md
```

The Accepted ADR Body Immutability threat-model boundary records that historical
acceptance anchors remain trustworthy only while the authoritative Git history
containing them remains available. Authoritative Ref Monotonicity closes that
dependency for ordinary-writer transitions by preserving reachability of the
previous authoritative tip.

It does not close administrator-authorized force-push after protection is
disabled, provider compromise, or external history replacement.

## Factorization boundary

The generic property is:

```text
for every accepted authoritative ref transition old_tip -> new_tip:
old_tip is ancestor-or-equal to new_tip
and ordinary writers cannot delete the authoritative ref
```

proto-ring owns these generic guarantee semantics. Each consumer retains its
repository coordinates, authoritative ref identity, provider identity,
provider-specific ruleset or policy coordinates, and external-system binding or
configuration.

This allocation follows the Shared Governance Provider contract: external-system
bindings remain consumer-local. Provider-native ref-protection mechanisms may
realize the guarantee without becoming its semantic definition.

## Distinction from state admission

Authoritative Ref Monotonicity is not Authoritative State Admission. It preserves
previously authoritative history but does not prove that every newly published
state was valid before publication.

A valid state A may advance by fast-forward to invalid state B before later
validation detects B. This contract guarantees only that A remains reachable
from B. A future, distinct primitive may govern validation before authoritative
ref advancement; this extraction does not introduce it.

## Composition implication

Accepted ADR Body Immutability can rely on Authoritative Ref Monotonicity to
preserve reachability of historical acceptance anchors. This is an implication
of composition, not a transfer of responsibility.

Authoritative Ref Monotonicity has no knowledge of ADRs, accepted status,
decision bodies, or ADR profiles. Its shared semantics remain entirely generic
to Git refs and reachability.

## Writer and administrator boundary

The required guarantee applies to ordinary repository writers. It does not
claim protection against an actor capable of changing or disabling the
protection mechanism itself.

Repository protection administrators, provider administrators, provider
compromise, destructive history rewrite through an authorized bypass, and
external storage compromise remain outside the threat model. Consumers must not
give their principal ordinary publication identity a bypass that defeats the
guarantee. No universal administrator-management policy is inferred.

## Rejected generalizations

The extraction explicitly rejects:

- mandatory pull requests;
- mandatory status checks;
- a mandatory merge queue;
- mandatory linear history;
- mandatory signed commits;
- the universal branch name `main`;
- a universal GitHub ruleset;
- a pre-publication validity guarantee;
- administrator-compromise protection;
- an external transparency log; and
- external signatures or attestations.

None of these mechanisms is required for the demonstrated generic property.

## No executable implementation

No executable ref-protection implementation, network checker, provider adapter,
or dependency is introduced. Enforcement already exists natively in Git hosting
providers; the currently missing shared artifact is the generic governance
contract, not another ref-protection implementation.

This extraction therefore composes an existing provider mechanism and builds
only the missing contract.
