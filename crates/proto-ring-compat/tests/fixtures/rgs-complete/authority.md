---
governance_authority:
  model_version: 1
  sources:
    authority:
      repository_target: canonical.md
    bindings:
      repository_target: bindings.md
    boundary:
      repository_target: boundary.md
    candidates:
      repository_target: candidates.md
    evidence:
      repository_target: evidence.md
    generator:
      repository_target: generator.md
    integrity:
      repository_target: integrity.md
    projections:
      repository_target: projections.md
    secondary:
      repository_target: secondary.md
    subject:
      repository_target: subject.md
  responsibilities:
    evidence-registry:
      roles:
        evidence: authority
      precedence: []
    evidence-required:
      roles:
        authority: authority
        candidates: non_authoritative
        subject: non_authoritative
      precedence: []
    integrity-profile:
      roles:
        integrity: authority
      precedence: []
    object-governance:
      roles:
        authority: authority
        bindings: secondary_representation
        boundary: authority
        generator: non_authoritative
        secondary: secondary_representation
      precedence:
        - higher_source: authority
          lower_source: boundary
    projection-registry:
      roles:
        projections: authority
      precedence: []
    provider-binding:
      roles:
        bindings: authority
      precedence: []
    repository-validation:
      roles:
        integrity: authority
      precedence: []
---
# Authority
