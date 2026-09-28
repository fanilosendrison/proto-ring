---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-authoritative-ref-monotonicity"
severity: "strict"
name: "Authoritative Ref Monotonicity contract"
---

# Authoritative Ref Monotonicity contract

## Purpose

An authoritative ref is a consumer-designated Git ref whose current target
represents authoritative published repository history for the consumer's
declared scope.

This contract governs continuity of authoritative Git history. It does not
select which ref is authoritative. That designation remains consumer-local.

## Monotonic advancement

For every accepted authoritative ref transition:

```text
old_tip -> new_tip
```

The previous tip MUST be an ancestor-or-equal of the successor tip:

```text
old_tip in ancestors_or_equal(new_tip)
```

The external enforcement mechanism MUST reject a non-fast-forward update by an
ordinary repository writer. Equality is admissible: `old_tip == new_tip` is a
no-op observation and need not change the ref.

This guarantee means that previously authoritative history cannot normally
disappear through an ordinary-writer transition.

## Non-deletion

While a ref remains designated as authoritative, ordinary repository writers
MUST NOT be able to perform its deletion. The external enforcement mechanism
MUST reject an attempted deletion by an ordinary writer.

This contract does not decide how a consumer retires an authoritative ref or
migrates authority to another ref. Deliberate retirement and migration are
separate consumer-authorized responsibilities.

## External enforcement

The guarantee MUST be imposed at an enforcement boundary outside ordinary
writer instructions. A documentary convention such as `do not force push` is
not a sufficient realization of this contract.

The enforcement mechanism MUST mechanically reject both a non-monotonic update
and deletion for an ordinary writer. A native ref-protection mechanism supplied
by an external Git hosting provider MAY realize this contract. Such a mechanism
is an external realization, not part of the contract definition. proto-ring
does not reimplement Git ref access control.

## Provider-neutral realization

No hosting product, named protection feature, or provider-specific API defines
these semantics. A provider-specific realization is conforming only when its
native ref-protection behavior supplies the generic guarantees above.

## Writer and control-plane boundary

An ordinary repository writer is an actor permitted to publish ordinary
repository updates but not to alter the protection mechanism that governs the
authoritative ref.

A protection-control-plane administrator is an actor capable of configuring,
disabling, or bypassing that protection mechanism. The consumer MUST ensure
that its principal ordinary publication identity has no bypass permitting a
non-fast-forward authoritative update or authoritative ref deletion.

A control-plane administrator MAY technically possess such a bypass. This
contract does not establish administrator immutability and does not define a
universal administrator-management policy.

The threat model excludes:

- a repository protection administrator changing or disabling protection;
- a provider administrator;
- provider compromise;
- destructive history rewrite through an authorized bypass; and
- external storage compromise.

## Authoritative State Admission distinction

Authoritative Ref Monotonicity is not Authoritative State Admission.

This contract guarantees only continuity: a previous authoritative tip remains
reachable from each accepted successor tip. It does not guarantee that every
new authoritative state was valid before publication.

The following sequence remains possible:

```text
valid state A
-> fast-forward invalid state B
-> validation later detects B
```

The guarantee establishes only that A remains reachable from B. Validation
before authoritative ref advancement is a separate future governance concern
and is not introduced by this contract.

## Local binding

Each consumer adoption MUST provide a local binding that identifies at least:

- the authoritative repository identity;
- the authoritative ref;
- the external provider or enforcement mechanism;
- the provider-specific protection identity sufficient to inspect or configure
  that mechanism; and
- the immutable proto-ring contract identity being adopted.

These values remain consumer-owned. They are not encoded in this shared
contract.

## Non-goals

This contract does not guarantee or require:

- repository state or commit correctness;
- test success;
- semantic validity or product semantics;
- pre-publication validation;
- provenance or authorship of commits;
- signature validity or signed commits;
- review approval;
- CI completion or status checks;
- linear history;
- merge topology or merge queue use;
- pull request use;
- branch naming;
- use of any provider-specific API;
- availability of the remote provider;
- administrator honesty; or
- external transparency.
