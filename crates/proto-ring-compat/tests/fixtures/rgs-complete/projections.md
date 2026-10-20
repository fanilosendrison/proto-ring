---
projection_registry:
  model_version: 1
  authority:
    responsibility: projection-registry
    source: projections
  projections:
    object-projection:
      responsibility: object-governance
      canonical_source: authority
      secondary_source: secondary
      mode: generated
      validation: validate-paths
      generator_source: generator
      target:
        interface: governance
        object: object-a
      binding: contract
---
# Projections
