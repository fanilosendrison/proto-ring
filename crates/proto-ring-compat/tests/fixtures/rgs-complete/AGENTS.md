---
repository_governance:
  model_version: 2
  provider:
    id: proto-ring
    binding:
      capability: shared_governance_provider
      route: registry
  capabilities:
    architecture_decisions:
      configuration:
        enabled: true
        policy:
          strict: true
      routes:
        history: adr-history.md
        profile: adr-profile.yaml
    evidence_requirements:
      configuration: {}
      routes:
        registry: evidence.md
    governance_authority:
      configuration: {}
      routes:
        profile: authority.md
    governed_objects:
      configuration: {}
      routes:
        profile: objects.md
    projection_integrity:
      configuration: {}
      routes:
        registry: projections.md
    repository_integrity:
      configuration: {}
      routes:
        profile: integrity.md
    shared_governance_provider:
      configuration:
        required: true
      routes:
        registry: bindings.md
---
# Carrier
