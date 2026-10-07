---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "conformance-fixture-guide"
domain: "proto-ring-rust-migration"
severity: "strict"
name: "Proto-ring shared conformance fixtures"
---

# Shared conformance fixtures

This directory owns checked-in byte artifacts only when multiple vectors reuse
the same exact bytes. The current vectors embed their exact bytes directly in
`inline`, `repository_plan`, or `provider_observation` fixtures, so no shared
binary artifact is required.

A future shared artifact must remain language-neutral, byte-exact, and
referenced from a versioned vector. Descriptive scenario files are forbidden.
