---
governance_bindings:
  model_version: 1
  source: bindings
  bindings:
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
