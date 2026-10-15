---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "migration-compatibility-transport"
domain: "proto-ring-consumer-migration"
severity: "strict"
name: "Proto-Ring compatibility transport v1"
---

# Proto-Ring compatibility transport v1

## Status and authority boundary

This transport exists only for #70, #71, and #72 migration compatibility.

It is not a Proto-Ring governance contract. It does not define canonical
semantic serialization. It does not define #28's public API. The Rust reference
engine remains downstream of language-neutral contracts. Consumer-owned policy
remains consumer-owned.

The transport is versioned, migration-local, and non-normative. Its JSON values
project existing engine results for process interoperability; they do not become
semantic authority and do not replace the language-neutral contracts.

The exact executable source exposes exactly six operations:

```text
accepted-adr-body.immutability
engine.identity
git-whitespace.validate
github-authoritative-ref-monotonicity.observe
repository-governance-state.compose
repository-integrity.evaluate
```

No internal crate module or generic responsibility dispatcher is exposed.

## Process boundary

The `proto-ring-compat` executable accepts exactly one UTF-8 JSON request on
standard input. It accepts no command-specific command-line arguments,
positional repository argument, or environment-variable request transport.

Every request has this envelope:

```json
{
  "transport_version": 1,
  "operation": "<operation>",
  "input": {}
}
```

Unknown fields are invalid. The only implicit default is an omitted
`binding_ids` field for `repository-governance-state.compose`, which means an
empty sequence.

A successfully parsed and supported request emits exactly one UTF-8 JSON
response followed by one line feed. A result has this envelope:

```json
{
  "transport_version": 1,
  "operation": "<same operation>",
  "outcome": "result",
  "value": {}
}
```

A controlled engine failure has this envelope:

```json
{
  "transport_version": 1,
  "operation": "<same operation>",
  "outcome": "controlled_failure",
  "error": {
    "category": "<stable migration-local category>",
    "stage": null,
    "detail": "<non-normative human diagnostic>"
  }
}
```

The `detail` field is non-normative. Consumers must not parse diagnostic prose
to recover semantic status.

Invalid UTF-8, invalid JSON, unknown fields, unsupported transport versions,
unknown operations, and invalid operation-specific input produce exit code `2`.
When response serialization remains possible, stdout contains:

```json
{
  "transport_version": 1,
  "outcome": "transport_error",
  "error": {
    "category": "invalid_request"
  }
}
```

Exit code `0` means one valid `result` or `controlled_failure` response was
emitted. Exit code `2` means the request was invalid or unsupported. Exit code
`70` means an unexpected runner I/O, projection, or serialization failure
prevented a valid compatibility response. Semantic `NON_PASS`, `VIOLATED`, or
`UNDETERMINED` values do not by themselves change the runner exit code.
Consumer wrappers own their local CLI exit mapping.

## `engine.identity`

Request:

```json
{
  "transport_version": 1,
  "operation": "engine.identity",
  "input": {}
}
```

Result value:

```json
{
  "source_repository": "fanilosendrison/proto-ring",
  "source_commit": "<40 lowercase hexadecimal characters>",
  "transport_version": 1
}
```

The source commit is embedded at build time by `git rev-parse HEAD`. It is
implementation and distribution identity, not semantic Proto-Ring authority.
The repository identity is fixed to `fanilosendrison/proto-ring`. Runtime
requests, consumer repositories, consumer bindings, dependency manifests, and
network state cannot override this identity.

## `repository-governance-state.compose`

Request:

```json
{
  "transport_version": 1,
  "operation": "repository-governance-state.compose",
  "input": {
    "repository": "<non-empty UTF-8 path>",
    "binding_ids": []
  }
}
```

`binding_ids` must contain unique, non-empty strings. When omitted, it is an
empty sequence. The operation calls only the existing
`repository_governance_state::load` boundary. It remains read-only,
non-executing, and provider-independent. It does not run Repository Integrity
validations.

Result value:

```json
{
  "stage": "composition",
  "identity_present": true,
  "scope_paths": [],
  "components": {
    "repository_governance_model": true,
    "governance_authority": true,
    "governed_objects": false,
    "governance_bindings": true,
    "repository_integrity": false,
    "projection_registry": false,
    "evidence_requirements": false
  },
  "bindings": {}
}
```

`scope_paths` preserves engine ordering. `identity_present` reports only whether
the opaque Repository State identity is present; no identity bytes or derived
digest are transported. Required components are always present after successful
composition. Optional component fields report presence only.

Each requested absent binding is represented as:

```json
{
  "present": false
}
```

Each requested present binding includes its existing kind, scope, identity, and
authority facts. Executable-provider identity has this projection:

```json
{
  "present": true,
  "kind": "executable_provider",
  "scope": {
    "kind": "logical_provider",
    "capability": null,
    "interface": null,
    "object": null
  },
  "identity": {
    "kind": "executable_provider",
    "repository": "<repository>",
    "commit": "<commit>"
  },
  "authority": {
    "responsibility": "<responsibility>",
    "source": "<source>"
  }
}
```

Governance-contract identity additionally contains its existing `path` and uses
`"kind": "governance_contract"`. These objects are migration projections, not
canonical Governance Binding representations.

A `RepositoryGovernanceStateError` produces category
`repository-governance-state`, its typed serialized stage, and its diagnostic
text as non-normative detail.

## `repository-integrity.evaluate`

Request:

```json
{
  "transport_version": 1,
  "operation": "repository-integrity.evaluate",
  "input": {
    "repository": "<non-empty UTF-8 path>",
    "evaluation_context": {
      "realizations": [
        {
          "environment_id": "<non-empty ID>",
          "identity_utf8": "<opaque text>",
          "command_prefix": ["<program>"],
          "process_environment": {
            "NAME": "VALUE"
          }
        }
      ]
    }
  }
}
```

Each command prefix is non-empty. `identity_utf8` is converted to its exact
UTF-8 bytes without hashing, normalization, or interpretation. The supplied
process-environment mapping is exact: the runner does not merge its ambient
environment.

The caller does not serialize a Consumer Repository Integrity profile. The
operation loads Repository Governance State from the supplied repository and
uses its routed `repository_integrity` profile. An RGS failure retains category
`repository-governance-state`. An absent profile uses category
`repository-integrity-profile-absent`. A profile-realization failure uses
category `repository-integrity-realization`.

The operation delegates realization and evaluation to the existing engine. It
does not reproduce selector resolution, identities, Repository State capture,
state coherence, prerequisites, aggregation, or process execution.

Result value:

```json
{
  "verdict": "PASS",
  "state_relation": "EQUAL",
  "state_coherence_halt": false,
  "validations": [
    {
      "id": "<validation ID>",
      "status": "SATISFIED",
      "obligations": [
        {
          "id": "<validation ID>",
          "status": "SATISFIED",
          "executed": true,
          "exit_code": "0"
        }
      ]
    }
  ],
  "obligations": [
    {
      "id": "<validation ID>",
      "status": "SATISFIED",
      "executed": true,
      "exit_code": "0"
    }
  ]
}
```

An unavailable exit code is `null`. Verdict, state relation, and obligation
status spellings come from the existing typed engine serializers. Profile,
evaluation-context, baseline-state, and final-state identities remain opaque
and are not transported.

## `accepted-adr-body.immutability`

Request:

```json
{
  "transport_version": 1,
  "operation": "accepted-adr-body.immutability",
  "input": { "repository": "<non-empty UTF-8 path>" }
}
```

Result value is exactly one of:

```json
{ "status": "PASS", "category": null }
{ "status": "NON_PASS", "category": "body_changed" }
{ "status": "NON_PASS", "category": "historical_state_invalid" }
```

The operation delegates to `accepted_adr_body::check`. Empty diagnostics pass;
any typed `BodyChanged` diagnostic selects `body_changed`; every other
non-empty diagnostic set selects `historical_state_invalid`. Diagnostic prose,
historical commits, and paths are not transported. Every valid request is a
result with exit code `0`, including `NON_PASS`.

## `git-whitespace.validate`

Request:

```json
{
  "transport_version": 1,
  "operation": "git-whitespace.validate",
  "input": {
    "repository": "<non-empty UTF-8 path>",
    "environment": { "NAME": "VALUE" }
  }
}
```

Result value:

```json
{ "status": "PASS", "domains": [] }
```

Status is derived only from `WhitespaceReport::passes`. Domains preserve typed
diagnostic first-occurrence order while deduplicating and are limited to
`unstaged`, `staged`, `push_committed_range`,
`pull_request_committed_range`, `range_determination`, and `git_observation`.
The operation delegates to `git_whitespace::check` with exactly the required
request environment; it does not merge ambient event or CI variables. Git
output and diagnostic internals are not transported. `NON_PASS` remains a
result with exit code `0`.

## `github-authoritative-ref-monotonicity.observe`

Request:

```json
{
  "transport_version": 1,
  "operation": "github-authoritative-ref-monotonicity.observe",
  "input": { "binding_path": "<non-empty UTF-8 path>" }
}
```

Result value contains only the typed serialized status:

```json
{ "status": "SATISFIED" }
{ "status": "VIOLATED" }
{ "status": "UNDETERMINED" }
```

The operation delegates to `github_authoritative_ref_monotonicity::observe`.
Binding parsing, provider addressing, HTTP behavior, and diagnostics remain
engine-owned. No provider injection surface is exposed. Every valid status is
a result with exit code `0`; later consumer wrappers own local exit mapping.
