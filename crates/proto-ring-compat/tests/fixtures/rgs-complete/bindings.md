---
governance_bindings:
  model_version: 1
  source: bindings
  bindings:
    contract:
      kind: governance_contract
      scope:
        kind: capability
        capability: governed_objects
      identity:
        repository: example/contracts
        commit: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
        path: docs/contracts/governed-objects.md
      authority:
        responsibility: object-governance
        source: authority
    provider:
      kind: executable_provider
      scope:
        kind: logical_provider
      identity:
        repository: example/provider
        commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      authority:
        responsibility: provider-binding
        source: bindings
---
# Bindings
