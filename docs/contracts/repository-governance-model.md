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

    governance_authority:
      configuration: {}
      routes:
        profile: "docs/repository-governance/<consumer>-governance-authority.md"

    governed_objects:
      configuration: {}
      routes:
        profile: "docs/repository-governance/<consumer>-governed-objects.md"

    shared_governance_provider:
      configuration:
        required: true
      routes:
        binding: "docs/repository-governance/<consumer>-shared-governance-provider.md"
```

The route strings in this declaration illustrate consumer-owned targets. They
do not impose a universal physical repository layout.

## Model version 2

Model version 1 is frozen. Model version 2 integrates the persistent governance
capabilities without changing the semantics of their routed resources.

The normative model-version-2 declaration shape is:

```yaml
repository_governance:
  model_version: 2

  provider:
    id: "proto-ring"
    binding:
      capability: "shared_governance_provider"
      route: "registry"

  capabilities:
    shared_governance_provider:
      configuration: {}
      routes:
        registry: "<consumer-owned-target>"
    governance_authority:
      configuration: {}
      routes:
        profile: "<consumer-owned-target>"
```

The exact model-version-2 capability and required-route table is:

| CapabilityId | Required route |
|---|---|
| `architecture_decisions` | `profile` |
| `governance_authority` | `profile` |
| `governed_objects` | `profile` |
| `shared_governance_provider` | `registry` |
| `projection_integrity` | `registry` |
| `repository_integrity` | `profile` |
| `evidence_requirements` | `registry` |
| `authoritative_ref_monotonicity` | `binding` |

This table is exhaustive and normative. Changing its capability vocabulary or
required routes requires a later model version. Additional consumer-local route
IDs remain opaque and do not acquire generic meaning.

Model version 2 requires exactly these capabilities to be present:

```text
shared_governance_provider
governance_authority
```

The other six supported capabilities are optional generically. Mandatory
presence is a model rule and does not require a duplicated
`configuration.required` field. Configuration remains opaque structured data.

The model-version-2 provider binding reference is exactly:

```text
capability = shared_governance_provider
route = registry
```

The target is the canonical Governance Binding Registry. The logical provider
identifier remains exactly `proto-ring`; it remains distinct from executable
provider and governance-contract identities.

## Model-version-2 structural composition

The following cross-capability rules are normative:

- `governed_objects` composes with `governance_authority`, which is already
  mandatory;
- `projection_integrity` requires `repository_integrity` because Projection
  Registry currentness and historical custody use Repository Integrity
  `ValidationId` values;
- `evidence_requirements` does not require `governed_objects` or
  `repository_integrity`;
- `authoritative_ref_monotonicity` has no additional generic capability
  dependency.

A governance-contract binding with `capability` scope must reference a
capability declared by the same model-version-2 consumer. This cross-model check
operates on already-loaded values; Repository Governance Model loading does not
load the Governance Binding Registry.

Capabilities are top-level repository-governance responsibilities, not Python
modules or a one-to-one list of contracts. In particular,
`exact_evidence_binding`, `governance_routing`, `structured_data`,
`canonical_adr`, `accepted_adr_body`, `adr_metadata`, and `git_whitespace` are
not capability IDs. One capability may compose multiple contracts, and module
use does not imply a capability declaration.

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

For model version 1, `model_version` MUST be the structured integer `1`. For
model version 2, it MUST be the structured integer `2`.

It MUST NOT be `true`, a string, a float, another integer, or `null`. A Python
implementation MUST require an exact integer that is one of the two supported
versions; boolean values are not integers for this purpose.

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
governance_authority
governed_objects
shared_governance_provider
```

`shared_governance_provider` is mandatory because the model's logical provider
binding refers to it. `architecture_decisions` is optional because not every
consumer is required to use ADRs. `governance_authority` and `governed_objects`
are optional globally. When `governed_objects` is declared, the same model MUST
also declare `governance_authority`; governed responsibility references compose
with that profile. Declaring `governed_objects` does not make
`architecture_decisions` mandatory.

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
`profile`. When `governance_authority` is declared, its `routes` mapping MUST
contain `profile`. When `governed_objects` is declared, its `routes` mapping
MUST contain `profile`. When `shared_governance_provider` is declared, its
`routes` mapping MUST contain `binding`.

The Repository Governance Model declares the `governance_authority` and
`governed_objects` capabilities and routes each to a consumer-owned profile. The
Canonical Governance Authority contract and Canonical Governed Objects contract
define the detailed semantics of those profiles. This contract does not acquire
or duplicate those semantics.

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

## Responsibility boundary

The responsibility boundary is exact:

```text
Canonical Governance Authority owns authority and source-role semantics.
Governed Objects owns decision, invariant, and obligation interfaces.
The individual registry/profile contracts own their detailed schemas and semantics.
Repository Governance Model owns capability discovery and structural composition.
#27 owns exact-state-bound RepositoryGovernanceState composition.
```

Loading model version 2 validates only model structure, supported capability
IDs, mandatory capabilities, required routes, cross-capability structural
constraints, and route resolution. It does not load or execute routed
registries/profiles, evidence candidates, Repository Integrity validations, or
Projection Integrity validators or generators.
