---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "synthetic-fixture"
domain: "api03-host-qualification"
severity: "guideline"
name: "Inert authority"
version: "0.1.0"
governance_authority:
  model_version: 1
  sources:
    bindings: {repository_target: bindings.md}
    integrity: {repository_target: integrity.md}
  responsibilities:
    quality:
      roles: {bindings: authority}
      precedence: []
    qualification-profile:
      roles: {integrity: authority}
      precedence: []
---
# Authority
