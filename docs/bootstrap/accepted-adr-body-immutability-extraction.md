---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Accepted ADR Body Immutability extraction evidence"
---

# Accepted ADR Body Immutability extraction evidence

Status: non-normative extraction analysis.

This record preserves the concrete evidence, confrontation results, and
factorization boundary for Accepted ADR Body Immutability. The shared authority
is the separate contract at
[`docs/contracts/accepted-adr-body-immutability.md`](../contracts/accepted-adr-body-immutability.md).

## Turnlock evidence baseline

The primary evidence source is:

```text
fanilosendrison/turnlock-rust
acb34df30d1d2ddf1ccff9bf2c3667a3ceb61c1c
docs/adr/metadata-migration-evidence.yaml
docs/adr/adr-profile.yaml
scripts/adr-metadata.py
```

Turnlock ADR-001 through ADR-016 already carry mechanical evidence that their
exact decision body bytes were preserved through structured migration. The
migration evidence binds before and after decision-body hashes while preserving
the authored payload boundary.

Modern records, including ADR-051, were created directly as structured accepted
ADRs. They do not require a separate mutable seal registry: their first
structured accepted state in authoritative history is the acceptance anchor.

## Ruu confrontation baseline

The principal confrontation consumer is:

```text
fanilosendrison/ruu
b2240fddf943c8bab9883faae872a47c7aa3388f
docs/adr/metadata-migration-evidence.yaml
docs/adr/adr-profile.yaml
tools/adr-metadata.py
```

Ruu also demonstrates exact decision-body preservation across structured
migration. Its evidence spans the retained migrated corpus and keeps migration
semantics consumer-owned.

Ruu ADR-031 demonstrates why whole-file sealing is too strong. Its current
`status: superseded` lifecycle state is legitimate while its exact decision body
remains historically preserved. Ruu ADR-085 is a modern record created directly
as structured and accepted.

## Common generic mechanism

The demonstrated consumer-independent relation is exactly:

```text
structured ADR first appears accepted on authoritative first-parent history
->
exact decision body becomes sealed for that logical ADR identity
->
every later observable historical body for that identity must remain byte-identical
->
current canonical ADR body must remain byte-identical
```

The shared mechanism derives its seal from committed Git history, retains exact
bytes in memory, inspects every first-parent published state, and composes with
Canonical ADR Identity for current working-tree resolution.

## Current configuration trust boundary

Historical discovery uses the current consumer-owned Canonical ADR Identity
profile, specifically `repository.adr_directory` and
`repository.filename_pattern`. It does not prove historical immutability of
profile routing, profile contents, corpus routing, or filename representation.
Those questions remain part of the broader Governed Identity problem.

## Consumer-local responsibilities

The following remain consumer-local:

```text
lifecycle transition policy
migration evidence
legacy migration semantics
schema and overlay
retained-ID policy
profile routing
corpus routing
generated projections
qualification
product semantics
```

Consumers also retain authority over their accepted decisions, validation order,
and repository-specific conformance evidence.

## Rejected generalizations

The extraction rejects:

```text
whole-file ADR immutability
universal lifecycle
immutable frontmatter
universal decision identity
profile-history immutability
Git branch-protection policy
force-push prevention
external transparency logs
signed acceptance events
```

These stronger claims are neither required by the demonstrated shared mechanism
nor established by the cited evidence.

## Unresolved boundaries

Turnlock #35 and Ruu #49 remain unresolved and are not resolved by this
extraction. The extraction establishes only Accepted ADR Body Immutability under
the current profile and available first-parent Git history.
