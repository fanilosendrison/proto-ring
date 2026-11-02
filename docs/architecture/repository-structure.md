---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "repository-layout-guide"
domain: "proto-ring-repository-architecture"
version: "0.1.0"
severity: "guideline"
name: "Proto-ring repository structure and bounded layout assessment"
---

# Repository structure

## Scope and audited revision

This is a descriptive, non-normative guide. It creates no contract, public API,
implementation architecture requirement, consumer obligation, or relocation
authorization.

The audit inspected the tracked tree, documentation, manifests, source entry
points, tests, tooling, corpus inventory, and CI at source commit
[`111e1dc80dbeb5a4befd35bbd929e9094a0b6326`](https://github.com/fanilosendrison/proto-ring/tree/111e1dc80dbeb5a4befd35bbd929e9094a0b6326),
tree `039a13c1856b08ad73e072bafcb0831395f8c47d`. This guide and the associated
README corrections are documentation additions to that baseline, not claims
that the new guide existed there. No other tracked surface is changed.

The audit distinguished production implementation, normative contracts,
language-neutral conformance, compatibility interfaces, active validation,
empirical extraction, migration history, and repository governance. Historical
wording was checked against current executable paths and immutable provenance,
not treated as an instruction to restore retired code.

## Present layout and authority

```text
.
├── README.md
├── Cargo.toml / Cargo.lock / rust-toolchain.toml
├── .gitattributes / .gitignore
├── .github/workflows/ci.yml
├── crates/
│   ├── proto-ring-engine/
│   ├── proto-ring-conformance/
│   └── proto-ring-compat/
├── conformance/v1/
│   ├── index.json / README.md
│   ├── responsibilities/
│   ├── cases/
│   ├── coverage/
│   ├── schemas/
│   └── fixtures/
├── docs/
│   ├── contracts/
│   ├── bootstrap/
│   ├── repository-governance/
│   └── architecture/                 # added by this documentation change
├── tests/
├── tools/
├── requirements-test.txt
├── review-evidence/
└── target/                           # ignored local build/validation output
```

- [README.md](../../README.md) is the explanatory entry point and contract
  index, not a second specification.
- [Cargo.toml](../../Cargo.toml), [Cargo.lock](../../Cargo.lock), and
  [rust-toolchain.toml](../../rust-toolchain.toml) define the private Rust
  workspace, locked dependencies, and pinned development toolchain. They do
  not turn crate boundaries into normative proto-ring architecture.
- [.gitattributes](../../.gitattributes) preserves exact Rust-owned Unicode
  resource bytes with `-text`; [.gitignore](../../.gitignore) excludes local
  `target/` output. Neither is a second governance implementation.
- [crates/](../../crates/) contains the sole production governance
  implementation and its compatibility and verification adapters.
- [docs/contracts/](../contracts/) contains the language-neutral normative
  governance contracts, authoritative for their accepted scope. Consumer
  product meaning, decisions, policy, configuration, bindings, evidence, and
  generated artifacts retain their consumer-owned authority.
- [conformance/v1/](../../conformance/v1/README.md) contains non-normative,
  language-neutral verification artifacts derived from those contracts.
- [docs/bootstrap/](../bootstrap/) preserves empirical extraction and
  confrontation evidence, historical implementation-transition records, and
  the active compatibility transport description. Its directory name does not
  make all its contents obsolete or give them equal authority.
- [docs/repository-governance/](../repository-governance/) contains the
  [Engineering profile](../repository-governance/proto-ring-engineering.md):
  repository-specific factorization doctrine, Project routing, workflow,
  validation, and completion rules. GitHub owns live work state; this guide
  does not mirror it.
- [docs/architecture/](./) contains this descriptive layout assessment, not
  another normative layer.
- [tests/](../../tests/) and [tools/](../../tools/) contain retained Python
  validation/support facilities. [requirements-test.txt](../../requirements-test.txt)
  pins their test-only dependencies.
- [.github/workflows/ci.yml](../../.github/workflows/ci.yml) defines executable
  CI gates and platform coverage; it is not product-semantic authority.
- [review-evidence/](../../review-evidence/) preserves exact migration evidence
  under `issue-129/` and `issue-130/`, including failed/intermediate runs. Those
  records establish only their original subjects, configurations, and bounds,
  not current conformance or successful replay.
- `target/` is ignored generated output, not a checked-in source or evidence
  authority. Git metadata may be a directory or a linked-worktree Git file;
  the checkout mechanism is not part of governance semantics.

## Rust workspace responsibilities

### proto-ring-engine

[proto-ring-engine](../../crates/proto-ring-engine/src/lib.rs) owns reusable
executable governance logic. Its modules cover structured inputs and patterns,
bootstrap and routing, repository state and governance composition, authority,
objects and bindings, ADRs and terminology, integrity and projections, evidence
requirements and exact binding, and provider observation.

It is the sole production governance engine. Module organization and Rust types
are implementation-private. Their availability does not constitute the coherent
agent-facing public API owned separately by
[Issue #28](https://github.com/fanilosendrison/proto-ring/issues/28).

### proto-ring-conformance

[proto-ring-conformance](../../crates/proto-ring-conformance/src/lib.rs) owns
corpus loading, fixture preparation, candidate invocation, observation comparison,
and verification regressions. Its standalone path invokes the Rust reference
candidate without the retired Python differential channel. Candidate requests
contain responsibility ID, vector ID, and fixture; expected observations and
comparison rules stay outside candidate inputs.

The current complete-corpus gate covers 29 active responsibilities and 1,153
distinct vectors. The engine's `conformance-internal` feature supports bounded
verification instrumentation; it is not a public provider-injection API or a
universal network-interception guarantee.

### proto-ring-compat

[proto-ring-compat](../../crates/proto-ring-compat/Cargo.toml) owns the existing
versioned migration-compatibility process transport. It delegates governance to
the engine and projects values through migration-local DTOs. Its build script
embeds the source commit for `engine.identity`; a qualified release identity
requires an exact clean immutable source.

The [compatibility-v1 guide](../bootstrap/proto-ring-compat-v1.md) remains an
active interface description despite its location under `docs/bootstrap/`.
It describes sixteen operations using one JSON request on standard input and
one JSON response on standard output. Operations include non-executing queries,
explicit integrity evaluation, and live provider observation; read-only
composition does not imply that every operation is non-executing or offline.

This transport is not normative serialization, a generic dispatcher, or #28's
public API. Consumer adapters and immutable consumption pins remain local.
This documentation change selects no future compatibility-evolution policy and
changes neither consumer pins nor transport semantics.

## Corpus and retained Python validation

[conformance/v1/index.json](../../conformance/v1/index.json) is the corpus
discovery entry point. `responsibilities/` records 30 stable dispositions:
29 Rust ports and the accepted retirement of the legacy Shared Governance
Provider V1 bridge. `cases/` contains the 29 active matrices; `coverage/` maps
contract headings; `schemas/` validates corpus structure; `fixtures/` preserves
exact artifacts and immutable contract snapshots. Those snapshots are
provenance, not copies to update when a current contract changes.

The corpus retains semantic baseline
`dedb01a3a9b7a18930c9da75afa3773b5ad67f69` and materialization base
`662d86d445cefed4e17aa3cfa897ac36344cba89`. Tagged values, exact bytes,
responsibility/vector identities, and per-vector comparison rules are unchanged.
Contract precedence remains explicit in the [corpus guide](../../conformance/v1/README.md).

There is no current `src/proto_ring/` engine or root Python distribution
metadata. Retained Python tests check contracts and extraction records, schemas,
corpus inventory and coverage, immutable provenance, and the absence of that
production package. [tools/run_python_tests.py](../../tools/run_python_tests.py)
checks the pinned exact 176-ID inventory in
[tests/retained-python-test-ids.txt](../../tests/retained-python-test-ids.txt),
partitions work, and rejects missing, duplicated, skipped, or failed execution.

Explicit Repository Integrity fixture commands may invoke Python selected by
`PROTO_RING_PYTHON`. The retained fixture interpreter and its pinned test
dependencies do not constitute a second governance engine or a current Python
semantic oracle. Current development commands and the exact CI/platform
coverage are linked from the [root development guide](../../README.md#development).

## Audit findings and historical interpretation

### Authority wording correction

- **Statement:** At the audited revision, the root README incorrectly groups
  `conformance/v1` with normative contracts as semantic authority.
- **Source and evidence:** The baseline README's Rust-engine section contains
  that wording; the corpus guide explicitly identifies itself as non-normative.
- **Existing authority:** The accepted contracts own their scoped semantics;
  the Engineering profile preserves that boundary. Conformance is derived
  verification evidence, not an independent source of semantics.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `repository-governance-or-documentation` and
  `verification-or-qualification-evidence`.
- **Related consequence:** The engine entry-point doc comment also groups the
  contracts and corpus as authority. That transitional comment is outside this
  three-document scope and remains unchanged; it does not promote the corpus.
- **Required authority:** No new semantic decision; the approved documentation
  scope suffices to correct the README.
- **Impact and action:** The README now separates normative contracts from
  conformance evidence. No contract, implementation, or corpus rule changes.

### Implementation-transition wording

The baseline corpus guide calls Rust a future implementation, describes the
legacy bridge retirement as pending, and describes the original differential
harness as current work. Those descriptions are replaced with the established
standalone Rust path and completed retirement. Frozen metadata, historical
observations, and migration disposition names remain intact rather than being
renamed to fit a presentational change.

### Enduring factorization versus migration history

The Engineering profile retains the Turnlock-first extraction direction and
Ruu confrontation, consumer-independence analysis, justified strengthening,
and consumer authority. Neither blind copying nor `Turnlock ∩ Ruu` defines
factorization. The [original audit](../bootstrap/turnlock-extraction-audit.md),
[residual re-audit](../bootstrap/turnlock-residual-governance-reaudit.md), and
individual extraction/confrontation records remain useful empirical evidence
within their original revision and maturity bounds. They are not all migration
history and are not replacements for current doctrine.

The [Python retirement ledger](../bootstrap/python-engine-retirement-ledger.md)
records intermediate inventories and candidate assessments, including deferred
gates and unresolved findings at those historical stages. Those passages are
preserved, not reinterpreted as current task instructions or silently rewritten
to erase the historical sequence. The final migration qualification is recorded
at [its exact successor receipt](https://github.com/fanilosendrison/proto-ring/issues/74#issuecomment-6579160822).
That receipt includes bounded owner admission of documentary historical
attestations; complete original historical packets were not recovered or
recreated. This layout audit does not repeat qualification, replay consumers,
or broaden that evidence claim.

## Bounded structural assessment

**Conclusion: no physical restructuring is necessary.** Every assessed path is
`KEEP`; there is no `MOVE_PROPOSED` destination. Navigation and explicit status
labels address the observed ambiguity without breaking established references.
No relocation or follow-up Issue is introduced by this assessment. Any later
move requires its own approved scope and reference/preservation analysis.

### Normative contracts and conformance — KEEP

- **Current paths:** `docs/contracts/` and `conformance/v1/`.
- **Destination:** Unchanged.
- **Reason and benefit:** Their separation already makes semantics distinct
  from verification; moving them adds no demonstrated organizational benefit.
- **Known references and consumers:** The README indexes contracts; Python
  contract tests read their exact paths. Corpus coverage and authority snapshots
  bind contract paths and immutable Git identities. The Rust loader joins the
  repository root to `conformance/v1`.
- **Preservation checks:** Exact corpus/schema/fixture bytes and identities,
  coverage controls, Rust conformance, contract tests, and link resolution.

### Empirical bootstrap records — KEEP

- **Current path:** `docs/bootstrap/` extraction and confrontation records.
- **Destination:** Unchanged.
- **Reason and benefit:** These records support an enduring repository mission,
  not a disposable migration phase. Clear links and provenance preserve their
  usefulness without an unsupported blanket archive classification.
- **Known references and consumers:** Root README extraction links, Engineering
  profile references, and retained contract/factorization tests read these
  paths. Historical Git references remain revision-specific.
- **Preservation checks:** Existing README links, extraction and doctrine
  tests, immutable source references, and no edits to historical artifacts.

### Compatibility description and retirement ledger — KEEP

- **Current paths:** `docs/bootstrap/proto-ring-compat-v1.md` and
  `docs/bootstrap/python-engine-retirement-ledger.md`.
- **Destination:** Unchanged.
- **Reason and benefit:** Their roles differ, but the active transport is now
  linked under execution and the ledger under history. Relocation would add
  reference churn without improving their actual responsibility boundaries.
- **Known references and consumers:** The README and this guide link both;
  migration and consumer-adoption records refer to the compatibility interface
  and its immutable source subjects. Consumer-owned build/invocation bindings
  continue to use the existing compatibility crate and executable.
- **Preservation checks:** Transport operation/protocol behavior unchanged,
  original ledger bytes and evidence identities retained, links resolved, and
  no change to consumer bindings or pins.

### Implementation and active validation — KEEP

- **Current paths:** `crates/`, `tests/`, `tools/`, and `.github/workflows/`.
- **Destination:** Unchanged.
- **Reason and benefit:** Production logic, verification adapters, and neutral
  support already have distinct locations. Retained Python is not obsolete
  merely because the production implementation changed languages.
- **Known references and consumers:** Cargo workspace/path dependencies, CI
  commands, fixture runners, Rust `include_str!` paths, the Python runner's
  repository-root discovery, and the pinned test-ID inventory depend on these
  locations. Consumer compatibility builds target the existing crate.
- **Preservation checks:** Locked build, Rustfmt, strict Clippy, workspace and
  documentation tests, full neutral Python controls, package absence, and the
  declared platform-specific CI checks. No implementation or CI edits.

At the audited baseline,
[repository_state/git.rs](../../crates/proto-ring-engine/src/repository_state/git.rs)
has 579 physical lines, exceeding the enclosing implementation rules' 400-line
limit. Its bytes remain unchanged. A source-module decomposition would require
separate scope; a documentation layout change does not resolve that existing
implementation-maintainability concern.

### Retained migration evidence — KEEP

- **Current path:** `review-evidence/issue-129/` and `review-evidence/issue-130/`.
- **Destination:** Unchanged.
- **Reason and benefit:** Immutable evidence is most useful with its original
  custody and references intact. A history-directory move has no demonstrated
  benefit over a concise historical entry point.
- **Known references and consumers:** The retirement ledger identifies issue-129
  evidence; historical migration and review records interpret these exact
  subject-bound artifacts. Existing failures remain part of that evidence.
- **Preservation checks:** Byte-for-byte historical artifacts, Git identities,
  and reference reachability retained; old PASS text is not reused as a current
  validation result.
