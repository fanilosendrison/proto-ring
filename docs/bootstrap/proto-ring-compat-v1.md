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

This transport exists only for #70, #71, and #72 migration compatibility. It
is migration-local, versioned, and non-normative. It is not a Proto-Ring
semantic contract, canonical serialization, or #28's public API. Engine values
are manually projected for process interoperability. Language-neutral
contracts remain authoritative, and consumer-owned policy remains
consumer-owned.

The exact executable source exposes exactly eight operations:

```text
accepted-adr-body.immutability
engine.identity
evidence-requirements.registry
exact-evidence-binding.evaluate
git-whitespace.validate
github-authoritative-ref-monotonicity.observe
repository-governance-state.compose
repository-integrity.evaluate
```

No generic responsibility dispatcher, operation negotiation, discovery API,
consumer callback protocol, ADR operation, or Terminology operation is exposed.

## Process boundary

`proto-ring-compat` accepts exactly one strict UTF-8 JSON request on standard
input and no command-specific arguments or environment request transport:

```json
{
  "transport_version": 1,
  "operation": "<operation>",
  "input": {}
}
```

Unknown fields, unsupported versions or operations, invalid JSON or UTF-8, and
operation-specific transport errors produce exit code `2` and, when
serialization remains possible:

```json
{
  "transport_version": 1,
  "outcome": "transport_error",
  "error": { "category": "invalid_request" }
}
```

A valid result has `outcome: "result"`. A controlled engine or semantic
rejection has `outcome: "controlled_failure"` and a migration-local error with
`category`, nullable `stage`, and non-normative `detail`. Consumers must not
parse diagnostic prose. Both outcomes exit `0`; unexpected runner I/O,
projection, or serialization failure exits `70`. Consumers own local CLI exit
mapping.

## `engine.identity`

Request input is `{}`. Result:

```json
{
  "source_repository": "fanilosendrison/proto-ring",
  "source_commit": "<40 lowercase hexadecimal characters>",
  "transport_version": 1
}
```

The build-time `git rev-parse HEAD` result and fixed repository identify the
exact executable source. Runtime requests, environment, consumer bindings,
network state, and dependency manifests cannot override them.

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

`binding_ids` defaults to `[]` and otherwise contains unique non-empty IDs. The
operation calls only `repository_governance_state::load`; it is read-only,
non-executing, and provider-independent. The result projects composition stage,
opaque identity presence, ordered scope paths, component presence, and only
explicitly requested Governance Binding facts. It exposes no opaque identity
bytes or digest. An RGS failure uses category
`repository-governance-state` and its typed stage.

## `repository-integrity.evaluate`

Request input contains a non-empty `repository` and an `evaluation_context`
with strict realizations. Each realization contains a unique non-empty
`environment_id`, exact `identity_utf8`, non-empty `command_prefix`, and exact
`process_environment` string map. The runner merges no ambient environment.

The operation obtains the routed profile through RGS, then delegates to
`repository_integrity::realize_consumer_profile` and
`repository_integrity::evaluate`. It does not reproduce selectors, identities,
state capture/coherence, prerequisites, aggregation, or execution. The result
projects verdict, state relation, coherence halt, validations, and obligations;
engine identities remain opaque.

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
a typed `BodyChanged` selects `body_changed`; every other non-empty diagnostic
set selects `historical_state_invalid`. Diagnostic prose and historical paths
are not transported. `NON_PASS` remains a result with exit code `0`.

## `git-whitespace.validate`

Request input contains a non-empty `repository` and required exact
`environment` string map. The runner passes only that map to
`git_whitespace::check`; it does not merge ambient event or CI variables.

Result value contains `status` (`PASS` or `NON_PASS`) and ordered, deduplicated
`domains`. Domain values are limited to `unstaged`, `staged`,
`push_committed_range`, `pull_request_committed_range`,
`range_determination`, and `git_observation`. Git output and diagnostic
internals are not transported. `NON_PASS` exits `0`.

## `github-authoritative-ref-monotonicity.observe`

Request input contains only non-empty `binding_path`. The operation delegates
to `github_authoritative_ref_monotonicity::observe`. Result value contains only
`status`: `SATISFIED`, `VIOLATED`, or `UNDETERMINED`. Binding parsing, provider
addressing, HTTP behavior, and diagnostics remain engine-owned. No provider
injection surface is exposed; every valid status exits `0`.

## `evidence-requirements.registry`

Request:

```json
{
  "transport_version": 1,
  "operation": "evidence-requirements.registry",
  "input": {
    "repository": "<non-empty UTF-8 path>",
    "requirement_ids": []
  }
}
```

`requirement_ids` defaults to `[]` and otherwise contains unique non-empty IDs.
The operation loads RGS and uses only its composed `evidence_requirements`
registry. It does not reconstruct authority, routing, Governed Objects, or
loader dependencies. RGS failure retains category
`repository-governance-state`; absent registry uses
`evidence-requirements-absent`; any missing requested ID uses
`evidence-requirement-not-found` without partial output.

Successful value:

```json
{
  "authority_source": "<source>",
  "model_version": 1,
  "registry_authority": {
    "responsibility": "<responsibility>",
    "source": "<source>"
  },
  "requirement_ids": ["<all registry keys in engine order>"],
  "requirements": {
    "<requested ID>": {
      "id": "<ID>",
      "responsibility": "<responsibility>",
      "instances": { "kind": "single", "source": null },
      "evidence_classes": {
        "kind": "explicit",
        "classes": ["<class>"],
        "source": null
      },
      "subject_source": "<source>",
      "context": { "required": false, "source": null },
      "candidate_source": "<source>",
      "target": null
    }
  }
}
```

Source-backed instances and class admissions carry their declared `source`;
explicit classes preserve the engine `BTreeSet` order. A target, when present,
contains only `interface` and `object`. This is a migration representation, not
the canonical Evidence Requirements serialization. It performs no validation,
provider access, artifact generation, or mutation.

## `exact-evidence-binding.evaluate`

This operation has exactly two local modes. `evaluate_binding` request input:

```json
{
  "mode": "evaluate_binding",
  "requirement": {
    "admitted_classes": ["check"],
    "subject_hex": "7375626a656374",
    "context_hex": null,
    "context_required": false
  },
  "binding": {
    "evidence_class": "check",
    "subject_hex": "7375626a656374",
    "context_hex": null
  }
}
```

`binding` may be `null`. A valid semantic result contains only:

```json
{ "status": "MATCH" }
{ "status": "MISMATCH" }
{ "status": "UNDETERMINED" }
```

The runner detects duplicate admitted classes before constructing
`EvidenceRequirement`, delegates structural validation to
`EvidenceRequirement::new`, and delegates comparison to `evaluate`. Structural
requirement rejection is controlled category
`exact-evidence-binding.evaluate.rejected`; candidate empty values remain
engine-evaluated data.

`evaluate_requirement_history` contains ordered `comparisons`, each with
`label`, `historical_requirement`, `current_requirement`, and nullable
`binding`. All transport is validated and all requirements are constructed
before any comparison executes. Result order equals request order:

```json
{
  "comparisons": [
    {
      "label": "subject-change",
      "historical": "MATCH",
      "current": "MISMATCH"
    }
  ]
}
```

Identity fields are nullable lowercase hexadecimal byte encoding. Empty hex is
transport-valid; uppercase, odd-length, or non-hex input is invalid transport.
Hex encoding belongs only to this compatibility transport. Opaque identity
semantics remain defined by the language-neutral contract: the runner does not
hash, normalize, UTF-8 decode, case-fold, validate digest length, or interpret
identities as Git objects. Consumer identity construction remains
consumer-owned.
