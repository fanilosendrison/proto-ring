---
evidence_requirements:
  model_version: 1
  authority:
    responsibility: evidence-registry
    source: evidence
  requirements:
    explicit:
      responsibility: evidence-required
      instances:
        kind: single
      evidence_classes:
        kind: explicit
        classes:
          - report
          - trace
      subject:
        source: subject
      context:
        required: false
      candidates:
        source: candidates
      target:
        interface: governance
        object: object-a
    sourced:
      responsibility: evidence-required
      instances:
        kind: source
        source: subject
      evidence_classes:
        kind: source
        source: candidates
      subject:
        source: subject
      context:
        required: true
        source: subject
      candidates:
        source: candidates
---
# Evidence
