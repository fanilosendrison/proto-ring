---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "synthetic-fixture"
domain: "api03-host-qualification"
severity: "guideline"
name: "Unexecuted synthetic RI scope profile"
version: "0.1.0"
repository_integrity:
  model_version: 1
  authority: {responsibility: qualification-profile, source: integrity}
  environments: [inert-unrealized]
  continue_after_non_satisfied: false
  validations:
    scope-only:
      responsibility: qualification-profile
      prerequisites: []
      instances:
        kind: repository_paths
        mode: append_all
        selectors:
          - {kind: path, path: target}
      command:
        kind: command
        environment: inert-unrealized
        arguments: []
        undetermined_exit_codes: []
  order: [scope-only]
---

# Unexecuted scope fixture

Only pure canonical realization is used. No environment is executable.
