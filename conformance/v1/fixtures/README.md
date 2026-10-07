---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "conformance-fixture-guide"
domain: "proto-ring-rust-migration"
severity: "strict"
name: "Proto-ring shared conformance fixtures"
---

# Shared conformance fixtures

This directory owns checked-in byte artifacts used independently of local Git
history. Vectors embed operational bytes directly in `inline`,
`repository_plan`, or `provider_observation` fixtures.

`authorities/<commit>/<path>` contains byte-exact snapshots of every pinned
repository authority cited by version 1. `index.json` discovers every snapshot.
Qualification compares a snapshot with the Git object when that object is
available and otherwise uses the indexed snapshot, allowing exact authority
validation in shallow or exported checkouts.

A future shared artifact must remain language-neutral, byte-exact, and
discovered from the versioned index. Descriptive scenario files are forbidden.
