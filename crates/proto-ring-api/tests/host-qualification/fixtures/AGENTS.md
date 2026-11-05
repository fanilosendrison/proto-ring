---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "synthetic-fixture"
domain: "api03-host-qualification"
severity: "guideline"
name: "Inert qualification bootstrap"
version: "0.1.0"
repository_governance:
  model_version: 2
  provider:
    id: proto-ring
    binding: {capability: shared_governance_provider, route: registry}
  capabilities:
    governance_authority:
      configuration: {}
      routes: {profile: authority.md}
    shared_governance_provider:
      configuration: {}
      routes: {registry: bindings.md}
    repository_integrity:
      configuration: {}
      routes: {profile: integrity.md}
---

# Inert bootstrap fixture

This is synthetic governance data, not executable consumer policy.
