---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-repository-governance-model"
severity: "strict"
name: "Canonical Repository Governance Model contract"
---

# Canonical Repository Governance Model contract

## Purpose

Canonical Repository Governance Model defines the versioned logical governance
structure constructed from Canonical Governance Bootstrap output. It identifies
the logical provider, declared supported capabilities, capability-local
configuration, and explicit routes to consumer-owned targets.

The normative model-version-1 declaration shape is:

```yaml
repository_governance:
  model_version: 1

  provider:
    id: "proto-ring"
    binding:
      capability: "shared_governance_provider"
      route: "binding"

  capabilities:
    architecture_decisions:
      configuration: {}
      routes:
        profile: "docs/adr/adr-profile.yaml"

    shared_governance_provider:
      configuration:
        required: true
      routes:
        binding: "docs/repository-governance/<consumer>-shared-governance-provider.md"
```

## Model composition

The model is constructed through this exact composition:

```text
explicit repository root
→ Canonical Governance Bootstrap
→ repository_governance structured mapping
→ Canonical Repository Governance Model
→ Canonical Governance Routing for declared routes
```

The model MUST NOT parse `AGENTS.md` independently, duplicate structured-data
parsing, or duplicate repository path and containment logic.

## Repository root

Repository and root identity MUST come from the explicit `repository` argument
supplied to the loader. It MUST NOT be declared inside the YAML model.

Exact repository-state identity is not owned by this contract. It remains
deferred to #27.

## Structural keys

For model version `1`, the direct keys of `repository_governance` MUST be
exactly:

```text
model_version
provider
capabilities
```

Missing structural keys MUST fail closed. Unknown structural keys at this model
layer MUST fail closed.

This requirement does not alter Canonical Governance Bootstrap behavior.
Bootstrap continues to preserve unknown valid descendants. The model layer may
reject descendants that it cannot interpret as a valid version-1 model.

## Model version

`model_version` MUST be the structured integer:

```text
1
```

It MUST NOT be `true`, `"1"`, `1.0`, another integer, or `null`. A Python
implementation MUST require exactly:

```python
type(value) is int and value == 1
```

## Logical provider

`provider` MUST be a mapping with exactly these keys:

```text
id
binding
```

`provider.id` MUST be exactly `proto-ring`.

This identifier is a logical provider identity only. It is not a Git SHA,
package version, repository URL, executable identity, or contract identity.

`provider.binding` MUST be a mapping with exactly these keys:

```text
capability
route
```

For model version `1`, the binding reference MUST be exactly:

```text
capability = shared_governance_provider
route = binding
```

The reference declares where the consumer-owned provider binding is routed. It
does not define the content or immutable executable identity of that binding.
Detailed executable-provider and contract-binding identity remains deferred to
#26.

## Capabilities namespace

`capabilities` MUST be a mapping. Model version `1` supports exactly these
capability identifiers:

```text
architecture_decisions
shared_governance_provider
```

`shared_governance_provider` is mandatory because the model's logical provider
binding refers to it. `architecture_decisions` is optional because not every
consumer is required to use ADRs.

An unknown declared capability MUST fail closed. A capability MUST NOT be
silently ignored or inferred from prose, Python module names, or filesystem
structure.

## Capability declaration shape

Every capability declaration MUST be a mapping with exactly these keys:

```text
configuration
routes
```

Both keys are mandatory. `configuration` and `routes` MUST each be mappings.

Unknown descendants inside `configuration` MUST be preserved exactly as
admitted `StructuredValue` data. The model MUST NOT assign universal meaning to
arbitrary configuration descendants. It MUST NOT assume that all capabilities
share fields such as `required`, `profile_path`, `binding_path`, `mode`, or
`enabled`.

## Required routes

When `architecture_decisions` is declared, its `routes` mapping MUST contain
`profile`. When `shared_governance_provider` is declared, its `routes` mapping
MUST contain `binding`.

Additional route identifiers MAY exist inside a supported capability. Every
route identifier and every route value MUST be a non-empty string.

Every declared route MUST be resolved through existing Canonical Governance
Routing. The model MUST NOT implement its own path normalization, repository
containment, symlink containment, target-existence checking, or fallback
discovery.

## Route result

For each declared route, an implementation MUST retain the existing
`ResolvedGovernanceRoute` result containing:

```text
exact route tuple
exact declared path string
resolved repository-contained target
```

The target remains consumer-owned. The model does not interpret routed target
content.

## Unknown and invalid state handling

The following MUST fail closed:

```text
unsupported model_version
unknown logical provider
unknown capability
missing provider binding reference
provider binding reference that does not identify the required capability/route
missing required capability route
malformed capability structure
malformed route declaration
unresolvable route
```

The following MUST be preserved:

```text
unknown valid capability-local configuration descendants
unknown valid additional route IDs inside a supported capability
```

## Provider boundary

The model MUST NOT contain or infer:

```text
proto-ring executable Git SHA
effective installed proto-ring identity
requirements.txt semantics
pyproject.toml semantics
container/image identity
contract pin semantics
executable-provider binding schema
projection/currentness from binding to installation surface
```

Those responsibilities belong to #26.

## Authority boundary

The model MUST NOT become authority for:

```text
consumer product semantics
Product Intent
accepted consumer decisions
ADR meaning
formal semantics
qualification semantics
evidence truth
consumer-specific policy
routed target semantics
routed target authority
```

## Later-Issue boundary

The responsibility boundary is exact:

```text
#24 owns authority/source-role semantics.
#25 owns decision/invariant/obligation interfaces.
#26 owns detailed immutable bindings and projection/validation/evidence registries.
#27 owns exact-state-bound RepositoryGovernanceState composition.
```
