---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Canonical Repository Governance Model extraction evidence"
---

# Canonical Repository Governance Model extraction evidence

Status: non-normative extraction analysis.

This record preserves the concrete consumer evidence and classification behind
the [Canonical Repository Governance Model contract](../contracts/repository-governance-model.md).
It creates no Turnlock or Ruu authority.

## Turnlock evidence

The current Turnlock root governance declaration contains:

```text
architecture_decisions.profile_path
shared_governance_provider.required
shared_governance_provider.binding_path
```

Current accepted Turnlock governance establishes:

```text
proto-ring = mandatory provider for applicable generic repository governance
ADR profile remains consumer-owned
Shared Governance Provider binding remains consumer-owned
product/formal/evidence authority remains Turnlock-owned
```

## Ruu evidence

The current Ruu root governance declaration contains the same three logical
facts:

```text
architecture_decisions.profile_path
shared_governance_provider.required
shared_governance_provider.binding_path
```

The binding path is Ruu-specific. Current accepted Ruu governance establishes:

```text
proto-ring = mandatory provider for applicable generic repository governance
ADR profile remains consumer-owned
Shared Governance Provider binding remains consumer-owned
product/qualification/evidence authority remains Ruu-owned
```

## Extraction matrix

The candidate fields and structures have this exact classification:

```text
repository root
→ generic
→ supplied externally to loader, not serialized

model_version
→ generic representation-integrity field
→ introduced by the canonical model
→ not consumer product semantics

logical provider id
→ generic
→ both consumers already adopt proto-ring as mandatory generic provider

provider binding reference
→ generic
→ both consumers already route to a consumer-owned Shared Governance Provider binding

capabilities namespace
→ generic
→ separates model structure from capability identity

architecture_decisions capability
→ demonstrated by both consumers
→ optional generically

shared_governance_provider capability
→ demonstrated by both consumers
→ mandatory in model version 1 because provider binding references it

configuration namespace
→ generic
→ prevents capability-local scalar/configuration semantics from becoming model-wide semantics

routes namespace
→ generic
→ replaces *_path naming heuristics with explicit routing declarations

architecture_decisions.routes.profile
→ demonstrated by both consumers

shared_governance_provider.routes.binding
→ demonstrated by both consumers

shared_governance_provider.configuration.required
→ capability-local
→ preserved but NOT assigned universal model semantics

executable provider SHA
→ deferred to #26

contract pins
→ deferred to #26

authority/source roles
→ deferred to #24

decision/invariant/obligation semantics
→ deferred to #25

validation/projection/evidence registries
→ deferred to #26

exact repository state identity
→ deferred to #27
```

This is not Turnlock ∩ Ruu.

The model is the strongest justified consumer-independent structure supported by
concrete evidence and the already accepted bootstrap and routing
responsibilities.

## Unknown capability rationale

Bootstrap preserves unknown valid descendants because parsing must not destroy
data.

RepositoryGovernanceModel fails closed on an unknown declared capability because
successfully loading a governance model while silently ignoring an applicable
governance responsibility would falsely claim complete understanding.

Capability-local configuration descendants remain preserved because the generic
model does not own their arbitrary meaning. Additional explicit routes remain
admitted for supported capabilities because Canonical Governance Routing can
resolve them without inventing their target semantics.

## Representation migration

The expected future consumer migration is exact:

```text
legacy:
repository_governance.architecture_decisions.profile_path

canonical:
repository_governance.capabilities.architecture_decisions.routes.profile
```

```text
legacy:
repository_governance.shared_governance_provider.required

canonical:
repository_governance.capabilities.shared_governance_provider.configuration.required
```

```text
legacy:
repository_governance.shared_governance_provider.binding_path

canonical:
repository_governance.capabilities.shared_governance_provider.routes.binding
```

No target or consumer authority changes.

## Excluded and deferred responsibilities

The model records a logical provider and a route to a consumer-owned provider
binding. It does not define executable-provider SHA, package installation,
contract-pin, projection, validation, evidence-registry, authority-role,
decision-interface, or exact-state semantics.

Routed ADR profiles and Shared Governance Provider bindings remain consumer-owned
artifacts. Turnlock retains its product, formal, and evidence authority. Ruu
retains its product, qualification, and evidence authority.
