---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-repository-governance-state"
severity: "strict"
name: "Canonical RepositoryGovernanceState contract"
---

# Canonical RepositoryGovernanceState contract

## Purpose

Canonical `RepositoryGovernanceState` version 1 composes the already-owned
repository-governance values for one repository into one coherent, read-only
state. It owns composition, capability-presence rules, exact-observation
binding, observation coherence, and controlled construction failure.

It does not redefine the schemas or semantics owned by the component contracts.

## Model-version boundary

`RepositoryGovernanceState` version 1 composes Repository Governance Model
version 2 only.

Repository Governance Model version 1 remains valid and frozen for its existing
`RepositoryGovernanceModel` responsibility. It is not composable into
`RepositoryGovernanceState` version 1. Construction from a version-1 model must
fail in a controlled manner without migration, inference, synthesis, or default
values for version-2 responsibilities.

This contract introduces neither a Repository Governance Model version nor a
capability. The model-version-2 capability vocabulary and required-route table
remain unchanged.

## Canonical state

A successful state contains exactly these logical components:

- the explicit resolved repository worktree root;
- one exact observed `RepositoryState` identity and its explicit observation
  scope;
- the Canonical Governance Bootstrap result;
- the Repository Governance Model version-2 value;
- the Governance Authority profile;
- the Governed Objects catalog if and only if `governed_objects` is declared;
- the Governance Binding Registry;
- the Consumer Repository Integrity profile if and only if
  `repository_integrity` is declared;
- the Projection Registry if and only if `projection_integrity` is declared;
  and
- the Evidence Requirement Registry if and only if `evidence_requirements` is
  declared.

Governance Authority is always loaded because it is mandatory in Repository
Governance Model version 2. The Governance Binding Registry is always loaded
through the mandatory `shared_governance_provider` registry route.

An optional component is absent only when its corresponding optional capability
is absent. When a capability is declared, an invalid, unavailable, malformed,
or uncomposable required routed resource is a construction failure. A partial
state must not be returned or represented as coherent.

## Composition order

Construction preserves this dependency direction:

```text
Canonical Governance Bootstrap
→ Repository Governance Model version 2
→ Governance Authority
→ Governed Objects if declared
→ Governance Binding Registry
→ binding/capability composition validation
→ Consumer Repository Integrity if declared
→ Projection Registry if declared
→ Evidence Requirements if declared
```

Existing component loaders and their contracts retain ownership of component
validation. Existing Repository Governance Model composition rules retain
ownership of binding/capability validation. State composition creates no new
cross-capability dependency.

## Capabilities retained through the model

The `architecture_decisions` capability is retained through its Repository
Governance Model declaration, configuration, and routes. State construction
must not eagerly enumerate an ADR corpus or create an ADR catalog.

The `authoritative_ref_monotonicity` capability is retained through its
Repository Governance Model declaration, configuration, and routes. State
construction must not perform live or provider-specific observation, inspect
forge rulesets, call a provider API, or access the network.

## Exact-observation invariant

The shared local repository-observation semantics are owned by
[Canonical RepositoryState](repository-state.md). RepositoryGovernanceState owns
which governance paths must participate in its observation scope and the
coherence of that scope across composition; it does not define a competing
RepositoryState identity algorithm.

A `RepositoryGovernanceState` MUST NOT be returned unless every contained
governance value belongs to one stable exact repository observation and one
stable observation scope.

Successful construction establishes all of the following:

- one explicit resolved repository root;
- one explicit observation scope;
- one opaque exact observed-state identity;
- no repository-state change across authoritative construction; and
- no observation-scope change across authoritative construction.

Repository-state drift or observation-scope drift during construction fails
closed. Initial/final RepositoryState identity equality is mandatory drift
detection, not proof that mutation isolation existed. Values obtained from an
observation that did not survive the coherence check must not be returned.

The stable exact-governance-state guarantee inherits RepositoryState's
mutation-isolation interval across discovery, first capture, authoritative
composition, and final capture. No uncoordinated external writer may mutate
contract-relevant state during that interval.

The observation scope covers the root bootstrap carrier and the
consumer-declared governance targets actually used to construct the state. It
must preserve sensitivity to both a declared in-repository filesystem binding
and its resolved repository-contained target when they differ.

Canonical Governance Routing retains ownership of declared route-string
semantics. A route declaration whose spelling is not itself an exact
repository-contained filesystem path identity MUST NOT be handed to
RepositoryState as though RepositoryState were a second routing interpreter.
The serialized declaration remains observed through its governing carrier;
RepositoryGovernanceState supplies the actual in-repository binding path where
one exists and the resolved target required for exact observation.

Exact observed governance state is not a universal snapshot of all consumer
semantics. Construction does not recursively load every source referenced by a
consumer authority or resolve Repository Integrity runtime selectors and
instances.

This contract does not prescribe a programming language, digest algorithm, Git
command, implementation structure, or exact number of observation passes.

## Repository identity boundary

The generic state is bound to the explicit resolved repository worktree root
and the opaque exact observed-state identity. It does not define a universal
forge or repository identity and does not require an origin URL, owner/name
coordinate, provider repository ID, or remote-provider identity.

## Read-only and non-executing construction

State construction is read-only, non-executing, and provider-neutral. It loads
and composes declarations; it does not evaluate the operations described by
those declarations.

Construction must not:

- execute Repository Integrity validators;
- execute Projection Integrity validators or generators;
- resolve runtime evidence candidates;
- evaluate Exact Evidence Binding;
- perform qualification or historical replay;
- perform Authoritative Ref Monotonicity live observation;
- call provider or network APIs;
- repair, bootstrap, migrate, or regenerate repository artifacts; or
- modify repository state.

## Controlled construction failure

A construction failure must identify the failed established composition stage
and preserve controlled underlying detail. The diagnostic surface is limited to
state construction and does not define a universal product or error taxonomy.

Construction failure returns no partial coherent state. In particular, an
unsupported model version, component-loading failure, composition failure,
repository-state drift, or observation-scope drift prevents a state value from
escaping.

Contract-owned `UNKNOWN`, `UNDECLARED`, and future runtime `UNDETERMINED` values
remain valid when their owning component contract admits them. Their possible
impact on a later fail-closed operation does not itself make state construction
fail.

## Authority boundary

The state references and composes consumer-owned authority and values. It does
not become authority for:

- Product Intent or product semantics;
- accepted decision meaning;
- formal semantics or formal claims;
- qualification truth;
- evidence truth or evidence interpretation;
- consumer-specific policy;
- provider live state; or
- the semantics of a routed consumer target.

Consumer repositories retain their authority, repository-specific bindings,
validation membership, evidence, qualification policy, and semantic sources.

## Non-goals

This contract does not define mutation, repair, generation, validation
execution, evidence evaluation, provider observation, universal repository
identity, an ADR catalog, or the later agent-facing open/check/explain API.
