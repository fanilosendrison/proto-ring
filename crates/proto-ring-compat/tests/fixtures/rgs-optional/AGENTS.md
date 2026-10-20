---
repository_governance:
  model_version: 2
  provider:
    id: proto-ring
    binding:
      capability: shared_governance_provider
      route: registry
  capabilities:
    governance_authority:
      configuration: {}
      routes:
        profile: authority.md
    shared_governance_provider:
      configuration: {}
      routes:
        registry: bindings.md
---
# Carrier
