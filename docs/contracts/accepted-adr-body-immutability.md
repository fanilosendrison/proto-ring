---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-accepted-adr-body-immutability"
severity: "strict"
name: "Accepted ADR Body Immutability contract"
---

# Accepted ADR Body Immutability contract

## Purpose

Once a logical ADR identity has appeared as a structured record with
`status: accepted` in committed first-parent Git history, the exact decision-body
bytes associated with that identity are immutable.

This contract protects the decision body only. It does not make the whole ADR
file, lifecycle metadata, profile configuration, corpus routing, projections,
migration evidence, or external work-management state immutable.

## Acceptance anchor

Inspect committed history in the exact order returned by:

```text
git rev-list --first-parent --reverse HEAD
```

The first structured historical state whose `metadata.id` equals the logical ADR
identity and whose `metadata.status` equals `accepted` establishes the acceptance
anchor.

The seal is the exact decision-body byte sequence parsed from that state. It is
not a configured hash, mutable registry, or path. No seal file or mutable seal
registry is introduced.

## Before acceptance

Before the first structured accepted state, proposed body changes are allowed.
Historical unstructured ADRs may exist without establishing an acceptance seal.

## After acceptance

After the acceptance anchor, every later structured occurrence of the same
logical ADR identity observed in a changed canonical ADR slot must carry exact
decision-body bytes equal to the seal.

A coordinated change to `decision_body_sha256` does not make changed decision
body bytes admissible. A temporary rewrite followed by a later restoration is
still a violation because every published first-parent state is inspected.

## Lifecycle boundary

Consumers continue to interpret lifecycle validity. This contract does not
define or validate lifecycle transitions.

An accepted ADR may later become `deprecated` or `superseded` when consumer
policy permits that transition. Its decision body remains sealed and must remain
byte-identical.

The contract does not make status, date, name, relations, governs, or other
frontmatter fields immutable.

## Legacy non-accepted structured state

A logical ADR whose first historical structured representation is, for example,
`status: superseded`, with no earlier structured accepted occurrence, does not
acquire an acceptance seal from this contract.

Preservation of that legacy material remains governed by consumer migration
evidence and policy.

## Current working state

For every logical ADR identity with a historical seal, current state is resolved
through:

```text
canonical_adr.resolve(repository, adr_id)
```

The exact current decision body must equal the sealed bytes. Resolution uses the
working tree, so an uncommitted body rewrite is detected.

If no historical accepted anchor exists and the working tree contains a first
acceptance candidate, the candidate is not rejected merely because the future
published anchor does not exist yet. The committed accepted state becomes the
anchor during later validation.

## Historical discovery

The repository must be an exact Git worktree root, have a committed `HEAD`, and
provide complete non-shallow Git history.

Only `HEAD` first-parent history is authoritative for this contract. Commits
reachable only through second parents do not independently establish published
history, while each merge commit on the first-parent chain is inspected as a
published state.

Every Git subprocess must use `GIT_NO_REPLACE_OBJECTS=1`. Local replace refs
must not alter the inspected commits, trees, or blobs.

Changed paths are derived from exact tree comparisons without rename heuristics.
Only direct children of the configured ADR directory whose filenames fully match
the configured pattern participate. Deleted paths have no current blob in that
historical state and are skipped.

Historical entries are read by object identity. Symbolic links and other
non-regular entries are never followed. After acceptance, a matching slot that
is non-regular, unparseable, identity-inconsistent, or body-inconsistent is a
controlled failure.

## Current configuration boundary

The current Canonical ADR Identity profile and configured corpus are the
interpretation boundary for historical ADR discovery.

Historical immutability of AGENTS routing, profile contents, corpus routing,
and filename representation is outside this contract. In particular, profile routing
outside this contract remains part of the broader governed-identity problem.

## Threat-model boundary

This contract detects violations observable in the repository history available
to the checker. It does not prevent or recover evidence destroyed by:

- authoritative Git history rewrite;
- force-push destroying the original anchor;
- administrator-level history replacement; or
- compromised remote storage.

Force-push and history rewrite protection remain responsibilities of a separate
future Git history protection layer.
