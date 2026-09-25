# Turnlock governance extraction audit

Status: non-normative bootstrap analysis

Source repository: `fanilosendrison/turnlock-rust`

Source branch: `main`

Source revision: `7534a4667ec9190aec4021f90bfae2fa8a046dae`

Audit date: `2026-09-19`

Source tree size: 85 files

## Purpose

This document preserves the bootstrap analysis that identified governance
currently suitable for shared ownership by `proto-ring`.

It is an observational extraction map derived from the concrete governance of
Turnlock at the exact source revision above.

It is observational bootstrap analysis, not Product Intent and not normative
authority for Turnlock, Ruu, or any other consumer.

Its immediate purpose is to preserve the reasoning required to factor already
demonstrated shared governance before Turnlock and Ruu continue to evolve
independently.

## Classification

Every audited surface is classified using one of four categories.

### PROTO-RING

The rule, contract, or reusable mechanism is already sufficiently generic to
require one canonical shared implementation or authority for Turnlock and Ruu.

### TURNLOCK

The artifact or meaning is specific to Turnlock and remains entirely owned by
`turnlock-rust`.

### MIXED

The current artifact combines generic governance with Turnlock-specific
bindings, data, history, or semantics and must therefore be split rather than
moved wholesale.

### EXTERNAL

The relevant shared authority or mechanism already exists outside Turnlock.
Extraction must not accidentally create a second canonical owner.

## Core extraction boundary

The central ownership boundary identified by this audit is:

```text
proto-ring owns:
    generic governance rules
    generic governance contracts
    reusable governance mechanisms

consumer repository owns:
    product semantics
    accepted decision history
    repository-specific authority bindings
    repository-specific configuration
    product-specific formal claims and models
    concrete review and verification evidence
    historical migrations
    generated repository projections
    product-specific validation
```

The most important instance of this rule is:

```text
proto-ring owns:
    "every mutable projected fact has one canonical owner"

Turnlock owns:
    "formal/verification.yaml owns this specific Turnlock fact"
```

Shared governance owns the rule.

The governed repository owns the binding.

## Root repository surfaces

### `.gitignore`

Classification: TURNLOCK.

No extraction requirement was identified.

### `README.md`

Classification: TURNLOCK.

It describes Turnlock's product, current repository state, architecture, and
formal-verification organization.

### `AGENTS.md`

Classification: MIXED.

The file currently combines generic repository-governance rules with
Turnlock-specific product and architecture directives.

The following responsibilities are candidates for proto-ring ownership:

* authority separation;
* ADR governance;
* projection-integrity discipline;
* work-state authority rules;
* engineering-discovery classification discipline;
* generic formal-assurance governance;
* the rule that evidence does not create product semantics;
* canonical repository-validation-suite ownership;
* diagnostic-check purity;
* generated-projection rules;
* routing semantic findings to the earliest unresolved authoritative layer.

The following remain Turnlock-specific:

* TURNLOCK's architectural center;
* workflow ownership;
* main-agent cognitive lineage;
* TURNLOCK execution forms;
* the Perfect Harness Test;
* harness-specific product boundaries;
* `TL-INV-*` semantics;
* TURNLOCK-specific implementation authorization;
* TURNLOCK-specific paths, artifact identities, and product directives.

A future Turnlock `AGENTS.md` should consume pinned shared governance and retain
only Turnlock-specific bindings and directives rather than duplicating the
shared governance contract.

### `requirements.txt`

Classification: MIXED.

Dependencies required by reusable governance tooling may move with that tooling.
Repository-local bootstrap and product-specific dependencies remain local.

### `.github/workflows/repository-integrity.yml`

Classification: MIXED.

The workflow should remain a repository-local CI entry point while invoking a
shared canonical governance/integrity mechanism.

CI environment/bootstrap remains a consumer concern where required.

Governance semantics and reusable validation orchestration should not be
duplicated between repositories.

## Repository-governance documents

### `docs/repository-governance/turnlock-rust-projection-integrity.md`

Classification: MIXED, with the core policy strongly suitable for PROTO-RING.

The generic invariant is:

```text
one mutable derived fact
    -> one canonical owner

every secondary representation
    -> Reference
       OR Generated projection
       OR Mechanically validated maintained projection
       OR Bounded historical snapshot
```

The following rules are generic:

* never manually mirror mechanically derivable mutable state without a guard;
* avoid projection chains;
* derive or validate against the canonical owner, not another projection;
* generated projections do not acquire semantic authority;
* validation membership itself is mutable repository knowledge and must have a
  canonical owner;
* repository work is incomplete when a canonical change leaves a required
  generated or validated projection stale;
* agent memory or diligence is not a synchronization mechanism.

Turnlock-specific ownership examples remain local bindings.

### `docs/repository-governance/turnlock-rust-engineering.md`

Classification: MIXED.

The following work-governance semantics are candidates for proto-ring ownership:

* workflow `Status` semantics;
* `Phase` semantics;
* `Kind` semantics;
* `Priority` semantics;
* one canonical owner for each live work-state fact;
* Issue prose must not mirror Project fields;
* native GitHub relationships own live dependency and parent/sub-issue state;
* acceptance criteria remain local to the Issue that owns the work;
* accepted repository authority owns accepted technical outcomes;
* closed Issues are historical records;
* `P0` / `P1` / `P2` / `P3` decision semantics;
* priority revalidation triggers;
* portfolio-wide priority revalidation algorithm;
* deterministic autonomous pickup;
* readiness and scheduling importance remain distinct;
* priority does not encode dependencies;
* priority does not implicitly cancel or preempt active work;
* generic Issue requirements;
* durable validated-finding behavior.

The lifecycle-domain taxonomy currently demonstrated by Turnlock is:

```text
Product Semantics
Formal Architecture
Formal Verification
Implementation
```

This taxonomy describes the engineering method rather than TURNLOCK product
semantics and is therefore a candidate shared governance concern.

Turnlock-specific content remains local:

* GitHub owner;
* repository name;
* GitHub Project title and number;
* exact Project views;
* Turnlock-specific authority paths;
* Turnlock-specific repository coordinates.

### `docs/repository-governance/turnlock-rust-discovery-classification.md`

Classification: MIXED.

The generic mechanics include:

```text
derived-from-existing-authority
decision-required
authority-conflict-or-uncertain
```

and the following rules:

* a downstream artifact or tool cannot silently ratify missing upstream product
  semantics;
* discoveries must route according to the earliest unresolved cause;
* an unresolved semantic choice remains locked until accepted authority resolves
  it;
* verification or implementation convenience cannot strengthen or weaken
  product semantics implicitly.

The mapping from generic discovery layers to exact Turnlock artifacts remains a
Turnlock-local binding.

## External shared governance dependencies

Turnlock already depends on shared mechanisms outside the repository, including:

* the `engineering-discovery-classification` agent skill;
* the GitHub Engineering Projects operational protocol;
* the generalized OKF ADR schema currently pinned from another repository.

These are classified EXTERNAL for this bootstrap audit.

Proto-ring must not create duplicate semantic authorities accidentally.

A likely future responsibility boundary is:

```text
proto-ring
    owns the governance contract

agent skill
    exposes or operationalizes that contract for an agent
```

This audit does not decide whether or when those existing external assets move
into proto-ring.

## Architecture Decision Records

All existing Turnlock ADR instances remain TURNLOCK.

This includes every active ADR from:

```text
ADR-001
through
ADR-044
```

No historical Turnlock ADR is to be moved into proto-ring.

Several ADRs contain decisions from which generic governance can be derived,
but they remain Turnlock decision history.

Particularly relevant bootstrap sources include:

* ADR-015 — normative/formal co-evolution and formal traceability;
* ADR-017 — validated ADR metadata and migration integrity;
* ADR-040 — preservation of future abstraction extractability without premature
  generalization;
* ADR-041 — formal-assurance architecture;
* ADR-042 — auditable hostile-review campaign execution and adjudication;
* ADR-043 — binding hostile-review evidence to exact reviewed inputs;
* ADR-044 — unambiguous Gate A review-subject identity.

Generic contracts may be derived from these decisions.

Their historical decision records do not change ownership.

## ADR governance stack

### `docs/adr/adr-profile.yaml`

Classification: MIXED.

The profile binds generic ADR governance to Turnlock-specific paths, migration
history, overlay schema, domain, generated projections, lifecycle constraints,
and commands.

The bindings remain local.

The reusable interpretation and validation mechanism is a proto-ring candidate.

### `docs/adr/index.md`

Classification: TURNLOCK generated artifact.

It remains in Turnlock.

### `docs/adr/metadata-migration-evidence.yaml`

Classification: TURNLOCK historical evidence.

It remains in Turnlock.

### `docs/adr/schemas/architecture-decision-record.schema.json`

Classification: EXTERNAL canonical authority at the audited revision.

Turnlock and Ruu contain byte-identical vendored copies.

The observed Git blob SHA is:

`c196e3708cc39c543a48a29d4dd994692ad53d9f`

The canonical profile is currently pinned to an external source.

Proto-ring must not silently become a second canonical owner.

Any future transfer of schema authority requires an explicit decision.

### `docs/adr/schemas/turnlock-architecture-decision-record.schema.json`

Classification: TURNLOCK.

This is Turnlock's repository-specific overlay.

### `scripts/adr-metadata.py`

Classification: MIXED, with most reusable mechanics suitable for PROTO-RING.

Generic mechanisms include:

* schema loading and validation;
* canonical metadata validation;
* exact decision-body hashing;
* relation validation;
* lifecycle validation;
* migration-evidence validation;
* annotated-history validation;
* generated-index rendering;
* generated-index staleness detection;
* fail-closed behavior.

Turnlock-specific bindings include:

* repository paths;
* repository domain;
* generated Turnlock titles;
* local overlay;
* migration baselines;
* legacy exceptions;
* repository-specific presentation details.

At the audited revision the Turnlock implementation is 949 lines and Ruu
already carries a separate fork of the same class of tool.

This existing duplication is direct evidence for factorization.

### `scripts/tests/test-adr-metadata.py`

Classification: MIXED.

Generic behavior tests should follow the shared engine.

Consumer-specific fixtures, migration baselines, history, and overlays remain
with the consumer repository.

## Existing ADR ownership constraint

Turnlock ADR-017 explicitly adopted:

```text
repository-owned validation, rendering, tests, and CI enforcement
```

Ruu ADR-082 contains the corresponding repository-owned decision.

Therefore shared implementation cannot be moved to proto-ring merely by copying
files.

Before physical extraction, each repository requires an accepted authority
change preserving the following distinction:

```text
consumer repository continues to own:
    ADR corpus
    ADR profile
    local overlay
    migration evidence
    generated repository projections
    repository-specific governance decisions

proto-ring may own:
    shared ADR governance contract
    shared validation implementation
    shared rendering implementation
    reusable generic tests
```

Adopting shared implementation must not transfer product-semantic or
decision-history authority.

This is a normative prerequisite for extraction.

## Formal-assurance instance

### `formal/verification.yaml`

Classification: TURNLOCK.

It is Turnlock's concrete formal-assurance instance.

It contains Turnlock authority, `TL-CLAIM-*`, `TL-INV-*` coverage, formal
domains, formal realization state, and repository-specific assurance data.

It remains entirely local.

### `formal/verification.schema.json`

Classification: MIXED.

Generic schema concepts include:

* authority;
* policy;
* assurance claims;
* normative coverage;
* formal realizations;
* assurance domains;
* behavioral modalities;
* formal semantic domains;
* hostile-review policy;
* mechanical-evidence policy;
* readiness gates;
* full / partial / none coverage semantics.

Observed Turnlock-specific bindings include:

* `project == TURNLOCK`;
* `TL-INV-NNN`;
* `TL-CLAIM-NNN`;
* current TLA+-specific assumptions.

A future extraction should prefer:

```text
proto-ring base assurance contract
+
Turnlock-specific binding or overlay
```

rather than moving the current Turnlock schema unchanged.

## Hostile semantic review

### `formal/reviews/review-evidence.schema.json`

Classification: MIXED.

The following concepts are generic candidates:

* review executions;
* reviewer identity;
* operational independence;
* reviewed subjects;
* review packet identity;
* prompt identity;
* sealed raw output identity;
* normalized findings;
* derived materiality;
* finding routing;
* resolution;
* structured refutation;
* hostile challenge;
* content-addressed evidence.

The current review classes are:

```text
assurance-decomposition
normative-to-formal-semantics
claim-property-correspondence
formal-semantic-projection
verification-profile-adequacy
evidence-interpretation
```

These are not inherently TURNLOCK product semantics.

Turnlock-specific Gate A selectors, exact subject bindings, URNs, and local
policy remain local or consumer-configured.

## Formal traceability implementation

### `scripts/check-formal-traceability.py`

Classification: MIXED.

This file must not be moved wholesale.

At the audited revision it contains approximately 1580 lines and combines a
generic assurance engine with explicit Turnlock bindings.

Generic mechanisms include:

* canonical JSON serialization;
* content hashing;
* schema validation;
* authority-artifact integrity;
* review-record loading and validation;
* reviewer-independence evaluation;
* materiality derivation;
* lossless finding normalization;
* refutation validation;
* hostile-challenge validation;
* semantic-dependency subject construction;
* evidence currentness;
* evidence invalidation;
* coverage consistency;
* claim/source reverse consistency;
* formal-realization integrity;
* readiness-gate derivation;
* generated-projection freshness checking.

Observed Turnlock-specific hardcoding includes:

```text
docs/specification/turnlock-spec.md
formal/Turnlock.tla
TL-INV-[0-9]{3}
TL-CLAIM-[0-9]{3}
exact claim-count assumptions
Turnlock legacy migration commits
Turnlock legacy migration counts
Turnlock-specific migration classifications
```

The intended extraction shape is:

```text
proto-ring formal-assurance engine
+
Turnlock assurance profile/bindings
+
Turnlock-specific historical migration validators
```

### `scripts/tests/test-formal-traceability.py`

Classification: MIXED.

At the audited revision it contains 123 tests.

A substantial majority test generic mechanisms such as:

* schema conformance;
* coverage consistency;
* review independence;
* packet identity;
* canonical serialization;
* materiality;
* finding lifecycle;
* challenge binding;
* stale evidence;
* semantic invalidation;
* symlink/path attacks;
* subject canonicalization;
* gate adequacy.

Turnlock-specific tests include exact `TL-CLAIM` corpus assumptions,
`TL-INV` parsing, Turnlock migration counts, Turnlock model paths, and
repository-self-conformance fixtures.

The suite should eventually be split rather than rewritten from scratch.

## Mechanical verification evidence

### `formal/tlc-result.schema.json`

Classification: MIXED.

The generic evidence contract includes:

* exact repository revision;
* exact executed model/configuration;
* tool identity and version;
* checked properties;
* finite bounds;
* fairness assumptions;
* execution outcome.

Turnlock-specific bindings include its title and `TL-INV` identity syntax.

TLC is currently a real backend used by the repository and may therefore be
factored as a concrete shared backend contract without attempting to design a
universal prover architecture.

This audit does not generalize beyond demonstrated requirements.

## Formal-assurance documentation

The following files are MIXED because they currently restate generic assurance
rules together with Turnlock-local bindings:

* `docs/formal/README.md`;
* `formal/README.md`;
* `formal/reviews/README.md`;
* `formal/results/README.md`;
* `formal/models/focused/README.md`;
* `formal/models/integrated/README.md`.

Generic rules currently repeated there include:

* formal evidence does not create product authority;
* formalization is distinct from formal semantic projection;
* hostile review is evidence, not mathematical proof;
* focused verification does not replace integrated canonical semantics;
* review invalidation follows semantic dependency;
* readiness gates have distinct responsibilities.

Proto-ring is a candidate owner for these generic rules.

Turnlock documentation should eventually bind those rules to Turnlock's exact
artifacts rather than redefine them.

## Normative terminology governance

### `docs/specification/terminology-inventory.yaml`

Classification: TURNLOCK.

It contains Turnlock terminology observations and fingerprints and remains
local.

### `scripts/check-normative-terminology.py`

Classification: MIXED, with most mechanics suitable for PROTO-RING.

Generic mechanisms include:

* canonical terminology-registry parsing;
* canonical-anchor validation;
* aliases and deprecated wording;
* detection of likely competing definitions;
* reviewed-occurrence fingerprints;
* stale inventory detection;
* explicit disclaimer that heuristic consistency checking is not semantic proof.

Turnlock-specific bindings are primarily the default specification and inventory
paths.

### `scripts/tests/test-normative-terminology.py`

Classification: MIXED.

Generic behavior tests are candidates for the shared implementation.

Turnlock fixtures and path bindings remain local.

## Repository-integrity validation

### `scripts/check-repository-integrity.py`

Classification: MIXED.

The reusable orchestration semantics are:

```text
read repository status
run the complete configured validation suite
collect failures
read repository status again
fail if diagnostic validation changed repository state
```

The worktree-purity guard is generic.

The current `canonical_steps()` function is repository-specific because it
contains the exact Turnlock validation membership and execution order.

The target ownership boundary is therefore:

```text
proto-ring owns:
    validation orchestration semantics
    worktree-purity guard

consumer repository owns:
    enabled local validation modules
    product-specific checks
    consumer-specific bindings
```

### `scripts/tests/test-repository-integrity.py`

Classification: MIXED.

Purity-guard and orchestration behavior belong with the shared runner.

Exact Turnlock validation membership remains a consumer binding.

## Git whitespace validation

### `scripts/check-git-whitespace.py`

Classification: PROTO-RING.

No material Turnlock-specific semantic binding was identified.

The implementation already handles:

* local changes;
* staged changes;
* GitHub push ranges;
* GitHub pull-request ranges;
* root commits;
* missing or malformed GitHub event payloads;
* fail-closed behavior.

### `scripts/tests/test-git-whitespace.py`

Classification: PROTO-RING.

At the audited revision this suite contains 22 reusable behavior tests.

## Formal mapping generation

### `scripts/render-formal-mapping.py`

Classification: MIXED.

The generic responsibility is:

```text
canonical assurance graph
    ->
deterministic human-readable projection
```

Turnlock paths, model identities, claim content, migration history, and generated
output remain local bindings.

### `docs/formal/invariant-mapping.md`

Classification: TURNLOCK generated artifact.

It remains in Turnlock.

## Product-owned corpus

The following surfaces remain entirely Turnlock-owned and must not move into
proto-ring:

* `docs/specification/turnlock-spec.md`;
* `docs/specification/terminology-inventory.yaml`;
* `docs/vision/turnlock-vision.md`;
* `docs/vision/future-workflow-run-evaluation.md`;
* every Turnlock ADR instance;
* `formal/verification.yaml`;
* `formal/migrations/verification-v2-to-v3-property-audit.yaml`;
* `docs/formal/invariant-mapping.md`;
* future `formal/Turnlock.tla`;
* future Turnlock TLC configurations;
* concrete hostile-review records;
* concrete verification-result records;
* Turnlock-specific formal claims and invariant identities.

Proto-ring may define how these categories are governed.

It does not own their product-specific content.

## Exhaustive tree-level classification

The 85-file source tree is covered by the following family classification:

| Surface                                         | Classification               |
| ----------------------------------------------- | ---------------------------- |
| `.gitignore`                                    | TURNLOCK                     |
| `README.md`                                     | TURNLOCK                     |
| `AGENTS.md`                                     | MIXED                        |
| `.github/workflows/repository-integrity.yml`    | MIXED                        |
| `requirements.txt`                              | MIXED                        |
| ADR-001 through ADR-044                         | TURNLOCK                     |
| `docs/adr/README.md`                            | TURNLOCK                     |
| `docs/adr/index.md`                             | TURNLOCK generated           |
| `docs/adr/metadata-migration-evidence.yaml`     | TURNLOCK historical evidence |
| `docs/adr/adr-profile.yaml`                     | MIXED                        |
| base ADR schema                                 | EXTERNAL canonical authority |
| Turnlock ADR overlay                            | TURNLOCK                     |
| `docs/formal/README.md`                         | MIXED                        |
| `docs/formal/invariant-mapping.md`              | TURNLOCK generated           |
| all three `docs/repository-governance/*` files  | MIXED                        |
| `docs/specification/turnlock-spec.md`           | TURNLOCK                     |
| `docs/specification/terminology-inventory.yaml` | TURNLOCK                     |
| both `docs/vision/*` files                      | TURNLOCK                     |
| `formal/README.md`                              | MIXED                        |
| formal v2-to-v3 migration audit                 | TURNLOCK historical evidence |
| focused/integrated model READMEs                | MIXED                        |
| results README                                  | MIXED                        |
| reviews README                                  | MIXED                        |
| review-evidence schema                          | MIXED                        |
| TLC-result schema                               | MIXED                        |
| formal-verification schema                      | MIXED                        |
| `formal/verification.yaml`                      | TURNLOCK                     |
| `scripts/adr-metadata.py`                       | MIXED                        |
| `scripts/check-formal-traceability.py`          | MIXED                        |
| `scripts/check-git-whitespace.py`               | PROTO-RING                   |
| `scripts/check-normative-terminology.py`        | MIXED                        |
| `scripts/check-repository-integrity.py`         | MIXED                        |
| `scripts/render-formal-mapping.py`              | MIXED                        |
| ADR tests                                       | MIXED                        |
| formal-traceability tests                       | MIXED                        |
| whitespace tests                                | PROTO-RING                   |
| terminology tests                               | MIXED                        |
| repository-integrity tests                      | MIXED                        |

## GitHub enforcement observation

At the audited revision, GitHub reported the `main` branch as:

```text
protected: false
```

and the repository ruleset collection was empty.

The available integration could not read the dedicated branch-protection
endpoint because that endpoint required unavailable administration permission.

Therefore the supported conclusion is limited to the observed repository state:
no ruleset was returned and the branch resource reported `main` as unprotected.

This reveals a governance concern that is not resolved by this audit:

```text
declared repository governance
    is distinct from
host-enforced repository governance
```

Proto-ring may eventually need to detect or validate divergence between those
layers.

This audit does not define that future contract.

## Cross-repository comparison observation

Turnlock must not be treated wholesale as the canonical implementation of every
governance detail.

For example, its current GitHub Actions workflow uses version tags such as:

```text
actions/checkout@v4
actions/setup-python@v5
```

while Ruu currently pins GitHub Actions to immutable SHAs.

Shared governance should therefore be derived from the strongest demonstrated
invariants across consumers rather than by blindly copying every Turnlock
implementation detail.

## Initial proto-ring ownership candidate

The audit supports the following current bootstrap boundary.

```text
PROTO-RING OWNS

governance contracts
├── authority and projection integrity
├── work-state semantics
├── discovery classification
├── ADR governance mechanics
├── formal-assurance governance
├── evidence governance
└── repository-validation semantics

reusable implementations
├── ADR validator and renderer
├── repository-integrity runner core
├── Git whitespace checker
├── normative-terminology checker
├── formal-assurance checker core
└── generated formal-mapping renderer

generic schemas/contracts
├── formal-assurance base contract
├── hostile-review evidence base contract
└── concrete verification-backend evidence contracts already demonstrated
```

Consumer repositories retain:

```text
product semantics
accepted decision history
authority bindings
repository coordinates
GitHub Project coordinates
local schema overlays
formal-assurance instances
claims
models
verification configurations
review evidence
verification evidence
historical migrations
generated projections
product-specific validators
```

## Recommended extraction order

The current recommended extraction sequence is:

```text
1. accept authority changes in Turnlock and Ruu that permit shared governance
   implementation without transferring repository semantic authority

2. extract Git whitespace validation and repository-integrity runner core

3. extract shared ADR tooling

4. extract projection-integrity, work-governance, and discovery contracts

5. extract normative-terminology tooling

6. extract formal-assurance contracts and generic schemas

7. split and extract the formal-assurance checker and hostile-review machinery

8. make Turnlock consume only the shared implementations for extracted concerns

9. make Ruu consume the same proto-ring version
```

The sequence is provisional implementation planning derived from the audited
state. It establishes no product semantics beyond the accepted proto-ring
contracts produced by later work.

## Non-decisions

This audit intentionally does not decide:

* any unrelated product intent;
* any unrelated product scope;
* a universal repository-governance model;
* a universal proto-ring profile format;
* whether the existing external shared agent skills move into proto-ring;
* whether the existing generalized OKF ADR schema moves into proto-ring;
* a universal formal-verification backend architecture;
* release governance;
* maintenance governance;
* incident governance;
* security governance;
* implementation governance beyond requirements already encountered;
* any requirement not demonstrated by the currently audited repositories.

Future shared governance must continue to be derived from concrete engineering
problems rather than invented speculatively.

## Bootstrap principle

Proto-ring exists at this stage only to eliminate duplicated governance that has
already emerged concretely in Turnlock and Ruu.

The intended development loop is:

```text
consumer encounters a real governance problem
        ↓
solve it concretely
        ↓
determine whether the solution is genuinely shared
        ↓
promote the generic rule or mechanism into proto-ring
        ↓
consumers adopt the new pinned proto-ring version
```

This document records the initial extraction evidence for that process.

It does not constitute a product specification.
