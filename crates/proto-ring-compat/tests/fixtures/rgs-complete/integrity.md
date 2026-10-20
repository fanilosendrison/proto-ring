---
repository_integrity:
  model_version: 1
  authority:
    responsibility: integrity-profile
    source: integrity
  environments:
    - native
    - python
  continue_after_non_satisfied: true
  validations:
    validate-paths:
      responsibility: object-governance
      prerequisites:
        - validate-profile
      instances:
        kind: repository_paths
        mode: append_all
        selectors:
          - kind: path
            path: tracked.txt
          - kind: glob
            glob: "*.md"
      command:
        kind: command
        environment: python
        arguments:
          - marker-must-not-exist
        undetermined_exit_codes:
          - 7
          - 9
      target:
        interface: governance
        object: object-a
    validate-profile:
      responsibility: repository-validation
      prerequisites: []
      instances:
        kind: single
      command:
        kind: command
        environment: native
        arguments:
          - check
          - profile
        undetermined_exit_codes: []
  order:
    - validate-profile
    - validate-paths
---
# Integrity
