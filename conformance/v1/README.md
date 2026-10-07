---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "conformance-corpus-guide"
domain: "proto-ring-rust-migration"
severity: "strict"
name: "Proto-ring language-neutral conformance corpus version 1"
---

# Proto-ring language-neutral conformance corpus

Corpus version 1 is a non-normative, reusable conformance and migration
transport derived from proto-ring's normative contracts. The contracts remain
the normative authority. Neither Python nor the future Rust implementation is
semantic authority, and Rust will not become semantic authority.

The corpus is frozen against semantic baseline
`dedb01a3a9b7a18930c9da75afa3773b5ad67f69` and materialized from qualified
base `662d86d445cefed4e17aa3cfa897ac36344cba89`.

The authority direction is always:

```text
normative contract
→ canonical representation or operational fixture
→ conformance matrix/vector
→ implementation
```

It is never reversed.

## Layout

- `index.json` is the sole discovery entry point.
- `schemas/` contains strict Draft 2020-12 schemas for the index,
  responsibilities, matrix cases, and contract coverage.
- `responsibilities/` contains the 30 stable slug-identified responsibility
  records: 29 `rust_port` records and one `retire_without_rust_port` record.
- `cases/` contains one matrix per `rust_port` responsibility and stable
  semantic vector identifiers. Retirement-only responsibilities have no matrix.
- `coverage/` accounts for every H2/H3 heading of each referenced normative
  contract.
- `fixtures/` contains reusable exact byte artifacts.

`migration_disposition` separates implementation work from frozen-module
retirement accounting. `shared-governance-provider.check` accounts for the
obsolete RGM-v1 Python bridge without inventing a Rust conformance obligation;
its retirement remains owned by Issue #73.

Case and vector IDs are stable while their semantic cases remain unchanged.
Tagged structured values preserve mathematical integers without JSON-number
limits, and exact bytes remain bytes. Comparison is selected per vector; there
is no universal JSON equality rule.

Issue #62 consumes this corpus to build the differential harness. The public API
owned by #28 does not originate here. Repository self-adoption owned by #29 does
not originate here.
