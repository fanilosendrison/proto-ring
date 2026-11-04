# proto-ring

## Overview

`proto-ring` is the bootstrap and factorization repository for generic
software-repository governance mechanisms empirically demonstrated by consumer
repositories such as Turnlock and Ruu. It also provides the established,
reusable contracts and executable mechanisms resulting from that work.

Bootstrap and factorization remain its enduring mission. The completed
Python-to-Rust transition changed the production implementation, not that
mission or the language-neutral authority of its contracts. Rust is now the
sole production governance implementation.

## Bootstrap and factorization methodology

The current bootstrap direction is:

```text
observe Turnlock governance
→ identify the maximum consumer-independent extraction candidate
→ confront the candidate with Ruu and other concrete consumers
→ narrow false generalizations
→ strengthen with better consumer-independent guarantees
→ establish the proto-ring contract or mechanism
→ validate it independently
→ adopt it back into consumers without weakening their guarantees
```

Turnlock is the primary extraction source for this bootstrap direction. That
makes it the starting point for maximum justified extraction, not automatic
proto-ring authority or an unquestioned reference implementation. A Turnlock
property is an extraction candidate only when analysis establishes that it is
consumer-independent repository methodology rather than product semantics,
authority, local configuration, history, topology, or another consumer-local
concern.

Ruu is the principal current confrontation consumer. It can falsify a proposed
generalization, expose a missing distinction, demonstrate a stronger generic
guarantee, or contribute another consumer-independent property. Ruu's absence
of an equivalent mechanism is not evidence against extracting a justified
Turnlock candidate.

`proto-ring` does not invent governance speculatively. It does not copy Turnlock
blindly, and it is not derived as `Turnlock ∩ Ruu`. Common implementation is
evidence for factorization, not a prerequisite. The result is the strongest
justified consumer-independent governance. Adoption uses immutable identities
and consumer-owned bindings, with evidence of equivalence, preservation, or
explicitly justified strengthening; weakening existing guarantees is not an
acceptable result.

## Governance architecture

The repository separates four concerns:

- **Normative contracts** in [docs/contracts/](docs/contracts/) own semantics
  for their accepted proto-ring scope.
- **Executable mechanisms** in [crates/](crates/) implement those contracts.
  The Rust workspace layout and low-level module interfaces are
  implementation-private, not normative architecture or a coherent public API.
- **Conformance evidence** in [conformance/v1/](conformance/v1/README.md) is
  language-neutral and derived from the contracts. Neither corpus observations
  nor implementation behavior become normative authority.
- **Consumer authority** remains local: product semantics, decisions, policy,
  evidence, configuration, profiles, bindings, and generated artifacts remain
  consumer-owned. Adoption does not transfer that authority to proto-ring.

The [repository structure guide](docs/architecture/repository-structure.md)
describes these boundaries, the audited layout, and why no physical
restructuring is necessary.

## Implemented mechanisms and contract index

These groups organize existing responsibilities for navigation, not as a new
normative taxonomy. The linked contracts define their exact semantics.

### Structured inputs, bootstrap, and repository observation

- [Canonical Structured Data](docs/contracts/structured-data.md)
- [Canonical Portable Pattern](docs/contracts/portable-pattern.md)
- [Canonical Governance Bootstrap](docs/contracts/governance-bootstrap.md)
- [Canonical Governance Routing](docs/contracts/canonical-governance-routing.md)
- [Repository Governance Model](docs/contracts/repository-governance-model.md)
- [RepositoryState](docs/contracts/repository-state.md)
- [RepositoryGovernanceState](docs/contracts/repository-governance-state.md)

### Authority, governed objects, and provider bindings

- [Shared Governance Provider](docs/contracts/shared-governance-provider.md)
- [Governance Authority](docs/contracts/governance-authority.md)
- [Governed Objects](docs/contracts/governed-objects.md)

The Rust engine implements the current Governance Binding Registry. The
obsolete Shared Governance Provider V1 Python bridge was retired without a
Rust port; the provider contract itself remains established.

### Decision records and terminology

- [ADR metadata primitives](docs/contracts/adr-metadata-primitives.md)
- [Canonical ADR Identity](docs/contracts/canonical-adr-identity.md)
- [Accepted ADR Body Immutability](docs/contracts/accepted-adr-body-immutability.md)
- [Normative Terminology](docs/contracts/normative-terminology.md)

### Integrity, projections, and external-provider observation

- [Repository Integrity](docs/contracts/repository-integrity.md)
- [Git Whitespace Validation](docs/contracts/git-whitespace-validation.md)
- [Authoritative Ref Monotonicity](docs/contracts/authoritative-ref-monotonicity.md)
- [Projection Integrity](docs/contracts/projection-integrity.md)

GitHub Authoritative Ref Monotonicity effective-rule live conformance is an
implemented provider mechanism composed with existing contracts, not another
normative contract. Its evidence and observability boundary are in the
[GitHub live conformance extraction record](docs/bootstrap/github-authoritative-ref-monotonicity-live-conformance-extraction.md).

### Evidence requirements and exact binding

- [Exact Evidence Binding](docs/contracts/exact-evidence-binding.md)
- [Evidence Requirements](docs/contracts/evidence-requirements.md)

## Execution and integration

The existing process interface is `proto-ring-compat`, a versioned,
migration-local compatibility executable. It is not the final agent-facing
public API, a generic responsibility dispatcher, or a normative serialization.
The coherent read-only public API is separately owned by
[Issue #28](https://github.com/fanilosendrison/proto-ring/issues/28) and is not
implemented by this interface.

The [API-01 public API v1 contract](docs/api/v1/README.md) is formally accepted at source `a1b27ca1e21423973a5557415488e4fcd535d01a`, following an exact-source independent hostile **GO** and the [owner acceptance decision](https://github.com/fanilosendrison/proto-ring/issues/134#issuecomment-6637682924). The new public API is **not implemented yet** or available for execution. API-02/API-03 implement the accepted contract; API-04 qualifies the resulting runtime interface. Existing implemented mechanisms described below are unchanged.

The release build command is:

```bash
cargo build --release --locked -p proto-ring-compat
```

The resulting `target/release/proto-ring-compat` accepts one strict
compatibility-v1 JSON request on standard input and writes one JSON response on
standard output. Its sixteen operations, envelopes, exit codes, projections,
and side-effect boundaries are described by the
[compatibility transport](docs/bootstrap/proto-ring-compat-v1.md).

The interface includes read-only composition and queries, explicitly requested
Repository Integrity evaluation, and live provider observation. Its operations
do not all share the same execution or network behavior; the transport guide
and applicable contracts own those distinctions. Consumer adapters, invocation
policy, and immutable provider pins remain consumer-owned.

A release identity claim requires a build from the exact clean, immutable
commit being qualified. A local build alone is not independent qualification
or consumer-preservation evidence.

The former `proto-ring` Python distribution and `src/proto_ring/` production
engine are retired. There is no current Python package installation step.
Python files under `tests/` and `tools/` are non-authoritative validation and
support facilities, not a second governance engine.

## Conformance and verification

The [language-neutral version-1 corpus](conformance/v1/README.md) retains its
immutable provenance, stable responsibility/vector identities, exact fixtures,
and per-vector comparison rules. Normative contracts take precedence over the
corpus and every implementation.

The Rust standalone conformance gate executes 29 active responsibilities and
1,153 distinct vectors. Candidate inputs exclude expected observations and
comparison rules; the harness compares actual observations independently.
The thirtieth responsibility records the accepted legacy bridge retirement,
not a missing Rust port.

Retained Python controls validate schemas, corpus inventory and coverage,
historical provenance, contract/extraction documentation, package absence, and
test-runner accounting. Explicit conformance fixture commands may also invoke
Python. Their pinned dependencies in
[requirements-test.txt](requirements-test.txt) are test-only; those commands do
not restore the retired governance engine.

The [final migration qualification record](https://github.com/fanilosendrison/proto-ring/issues/74#issuecomment-6579160822)
identifies its exact subject and evidence limits, including owner-admitted
historical documentary attestations. Passing current tests does not reconstruct
missing historical packets or establish unbounded correctness.

## Development

The [layout guide](docs/architecture/repository-structure.md) maps the three
Rust crates, contracts, corpus, retained Python controls, and evidence.
The repository-owned commands are defined in
[CI](.github/workflows/ci.yml), with the toolchain pinned in
[rust-toolchain.toml](rust-toolchain.toml) to Rust 1.99.0, including Rustfmt and
Clippy. Git and Python are needed by conformance fixtures and validation.

A local Python 3.11 or 3.12 test environment can be created under the ignored
`target/` directory:

```bash
python3 -m venv target/dev-venv
. target/dev-venv/bin/activate
python -I tests/test_package.py
python -m pip install --requirement requirements-test.txt
python tools/run_python_tests.py
```

The neutral Python runner checks the pinned complete test-ID inventory and
rejects missing, duplicate, skipped, or failed work. CI runs it on both Python
versions. The package-absence gate precedes dependency installation in CI.

The Linux Rust CI gates are:

```bash
export PROTO_RING_PYTHON="$(command -v python)"
export PYTHONDONTWRITEBYTECODE=1
rustc --version
cargo --version
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo test --workspace --doc --locked
git diff --check
```

`PROTO_RING_PYTHON` selects the interpreter for declared fixture commands, not a
Python production engine. The separate Windows CI job runs the engine and its
listed targeted conformance checks; it is not a full Windows workspace-matrix
claim. The workflow is the source for that exact selection.

Repository work uses the
[Proto-Ring Engineering profile](docs/repository-governance/proto-ring-engineering.md)
and its GitHub Project routing. Live work state belongs in GitHub, not in this
README.

## Historical and factorization records

[Bootstrap records](docs/bootstrap/) preserve empirical derivation and
confrontation evidence, not just the completed implementation migration.
The [historical Turnlock extraction audit](docs/bootstrap/turnlock-extraction-audit.md)
and [baseline-bounded residual re-audit](docs/bootstrap/turnlock-residual-governance-reaudit.md)
remain useful with their recorded provenance and maturity limits. Current
factorization doctrine is governed by the Engineering profile, not by obsolete
assumptions in an earlier audit.

Established extraction records include:

- [Git Whitespace Validation extraction record](docs/bootstrap/git-whitespace-validation-extraction.md)
- [ADR metadata primitive extraction inventory](docs/bootstrap/adr-metadata-primitives-extraction.md)
- [Canonical ADR Identity extraction record](docs/bootstrap/canonical-adr-identity-extraction.md)
- [Canonical Governance Bootstrap extraction record](docs/bootstrap/governance-bootstrap-extraction.md)
- [Canonical Governance Routing extraction record](docs/bootstrap/canonical-governance-routing-extraction.md)
- [Repository Governance Model extraction record](docs/bootstrap/repository-governance-model-extraction.md)
- [RepositoryGovernanceState extraction record](docs/bootstrap/repository-governance-state-extraction.md)
- [Governance Authority extraction record](docs/bootstrap/governance-authority-extraction.md)
- [Governed Objects extraction record](docs/bootstrap/governed-objects-extraction.md)
- [Accepted ADR Body Immutability extraction record](docs/bootstrap/accepted-adr-body-immutability-extraction.md)
- [Authoritative Ref Monotonicity extraction record](docs/bootstrap/authoritative-ref-monotonicity-extraction.md)
- [Projection Integrity extraction record](docs/bootstrap/projection-integrity-extraction.md)
- [Normative Terminology extraction inventory](docs/bootstrap/normative-terminology-extraction.md)
- [Exact Evidence Binding extraction record](docs/bootstrap/exact-evidence-binding-extraction.md)
- [Provenance assertion composition confrontation](docs/bootstrap/provenance-assertion-composition-confrontation.md)

The [Python retirement ledger](docs/bootstrap/python-engine-retirement-ledger.md)
and [retained migration evidence](review-evidence/) preserve the historical
implementation transition, including intermediate failures and superseded
candidate assessments. They are not current installation instructions.
Historical Python replay uses a detached worktree at one full immutable commit
and only that revision's metadata, instructions, and dependencies; the retired
engine is not copied back into current sources.
