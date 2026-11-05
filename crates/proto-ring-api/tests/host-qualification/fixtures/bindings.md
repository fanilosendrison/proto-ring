---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "synthetic-fixture"
domain: "api03-host-qualification"
severity: "guideline"
name: "Inert bindings"
version: "0.1.0"
governance_bindings:
  model_version: 1
  source: bindings
  bindings:
    provider:
      kind: executable_provider
      scope: {kind: logical_provider}
      identity:
        repository: example/provider
        commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      authority: {responsibility: quality, source: bindings}
---

# Inert bindings
