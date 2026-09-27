---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-shared-governance-provider"
severity: "strict"
name: "Canonical Shared Governance Provider contract"
---

# Canonical Shared Governance Provider contract

## Purpose

This contract defines the ownership and provider-selection boundary for generic
repository-governance mechanisms consumed from proto-ring.

A consumer adopting this contract retains authority over its own repository,
product semantics, decisions, local policy, evidence, configuration, bindings,
and consumer-specific obligations.

For generic or reusable repository-governance mechanisms, proto-ring is the
mandatory provider.

This contract governs where generic governance belongs and how a consumer binds
to it. It does not transfer consumer authority to proto-ring.

## Consumer authority remains local

A consumer continues to own every consumer-specific fact and decision,
including:

- product semantics;
- accepted decisions and decision history;
- repository-specific authority mappings;
- local profiles, overlays, and configuration;
- consumer-specific policy;
- consumer-specific evidence and evidence interpretation;
- generated consumer artifacts;
- repository coordinates, paths, workflow configuration, and external-system
  bindings; and
- every obligation whose meaning depends on that consumer.

A proto-ring contract or implementation consumes those local authorities through
an explicit binding. It does not replace them and does not become their
canonical owner.

## Generic versus consumer-specific governance

Before introducing, replacing, or materially changing a repository-governance
mechanism, the consumer must classify the governed responsibility.

A responsibility is consumer-specific when its governing semantics depend on
consumer-specific product meaning, accepted consumer decisions,
consumer-specific evidence semantics, consumer-specific repository facts, or
another authority that cannot be stated as a reusable governance contract
independent of that consumer.

A responsibility is generic or reusable when its governing semantics can be
stated independently of a particular consumer and consumer-specific facts can
be supplied through configuration, profiles, bindings, or other explicit local
inputs.

Generic governance must not be made consumer-specific merely because its first
implementation or first demonstrated need occurs in one consumer.

Consumer-specific governance must not be moved into proto-ring merely to
centralize code.

## Mandatory provider rule

When an applicable proto-ring contract or implementation exists for a generic
repository-governance responsibility, an adopting consumer MUST use the
immutably bound proto-ring mechanism.

The consumer MUST NOT maintain a competing local implementation of the same
generic governance responsibility.

The consumer MUST NOT copy the generic mechanism into its own repository,
maintain a functionally equivalent local fork, or bypass the applicable
proto-ring mechanism through ad hoc repository-local machinery.

Consumer-local bindings, profiles, adapters, configuration, and strengthening
checks remain permitted when they express consumer-specific facts or
obligations and do not duplicate, replace, or weaken the generic contract.

## Generic capability not yet present in proto-ring

The absence of a required generic capability from proto-ring does not create a
permanent consumer-local ownership exception.

When a newly required repository-governance mechanism is classified as generic
or reusable and proto-ring does not yet provide it, the permanent generic
contract or mechanism MUST be established in proto-ring and then consumed by
the repository through an explicit immutable binding.

A temporary consumer-local bridge is permitted only when an explicit accepted
consumer governance decision records all of the following:

1. that the responsibility is generic rather than consumer-specific;
2. why the required proto-ring capability cannot yet be consumed;
3. the exact bounded scope of the temporary local mechanism;
4. the condition under which the temporary mechanism must be removed; and
5. the obligation to migrate to the proto-ring mechanism.

Without that explicit bounded exception, absence from proto-ring MUST NOT be
treated as permission to create a permanent local alternative.

## Immutable consumption

A consumer MUST bind every adopted proto-ring governance contract or executable
provider to an immutable identity sufficient to reproduce the exact governing
contract or implementation being consumed.

Mutable proto-ring state, including an unpinned branch such as `main`, MUST NOT
become ambient consumer authority.

A contract-authority pin and an executable-provider pin MAY differ when they
identify different responsibilities.

Neither pin implicitly authorizes, upgrades, or determines the other.

Changing an immutable proto-ring binding is an explicit consumer repository
change.

If the change alters an accepted governance contract, authority boundary,
evidence boundary, or another accepted consumer governance decision, the
consumer must apply its own decision process before adopting the new identity.

## Local composition

Consumer-local governance may compose with proto-ring only at an explicit
responsibility boundary.

A local extension MAY:

- provide consumer-specific configuration;
- bind consumer-owned authorities;
- add consumer-specific obligations;
- add stronger consumer-specific checks; or
- adapt a generic mechanism to consumer-local paths or external systems.

A local extension MUST NOT:

- redefine the generic contract;
- weaken a generic requirement;
- duplicate the generic mechanism as an independent implementation;
- silently substitute another provider; or
- transfer consumer-specific authority into proto-ring.

## Existing and future proto-ring mechanisms

This contract governs provider selection and responsibility placement.

Individual proto-ring contracts and mechanisms continue to own their own
specific generic semantics, including contracts such as Repository Integrity,
Projection Integrity, ADR metadata primitives, Git whitespace mechanisms, and
future extracted governance primitives.

Adopting this contract does not automatically change the immutable identity of
an already adopted proto-ring contract or executable package.

Each distinct binding retains its own explicitly governed immutable identity.

## Change protocol

Before completing a repository-governance change, an adopting consumer must:

1. classify each new or changed governance responsibility as
   consumer-specific or generic/reusable;
2. keep consumer-specific authority local;
3. use the applicable immutably bound proto-ring mechanism for every generic
   responsibility already provided by proto-ring;
4. establish missing permanent generic mechanisms in proto-ring rather than as
   silent permanent consumer-local alternatives;
5. bind the resulting generic contract or implementation explicitly and
   immutably in the consumer;
6. preserve any distinct existing proto-ring contract or executable pins unless
   that change explicitly upgrades them; and
7. satisfy the consumer's own validation and completion rules.

The repository-governance change is incomplete while an applicable generic
responsibility is implemented as an unauthorized competing consumer-local
mechanism.

## Authority boundary

This contract is a repository-governance contract.

It does not define consumer product semantics, consumer architecture,
qualification truth, formal-verification truth, evidence truth, accepted
consumer decisions, or consumer-specific policy.

proto-ring is the mandatory provider of applicable generic governance
mechanisms; it is not the owner of the consumer authority those mechanisms
govern.
