# Canonical Repository Integrity Contract

## Derivation baselines

This contract was derived by comparing concrete governance already demonstrated
by these exact consumer states:

```text
Turnlock
fanilosendrison/turnlock-rust
7f0526cf6883a323b056cd833785cd863a3d4497

Ruu
fanilosendrison/ruu
3aead6b0058d6ad41abda13012ac955941dd3a9d

proto-ring authority at derivation
fanilosendrison/proto-ring
b183bed5e04b93fa5c9c337af9f0830fe75c9537
````

These identities record derivation provenance.

They do not make either consumer a reference implementation and do not make
future consumer repository state ambient authority over this contract.

## Purpose

Repository Integrity defines whether one exact current repository state
satisfies all repository-governance obligations that its consumer declares
mandatory for that state.

Repository Integrity is a current-state coherence property.

It is not:

* product-semantic authority;
* qualification of behavioral or product claims;
* formal verification;
* historical evidence replay;
* proof that a repository is correct in every possible sense;
* synonymous with a clean Git worktree.

A consumer owns what Repository Integrity requires.

`proto-ring` owns only the consumer-independent semantics of evaluating those
requirements.

## Core model

Repository Integrity is evaluated over:

```text
RepositoryState S
+
ConsumerIntegrityProfile P
+
EvaluationContext C
→
IntegrityVerdict
```

### RepositoryState

`RepositoryState` is the exact governed repository state to which the verdict
applies.

It includes every repository-resident artifact whose content, identity,
presence, absence, metadata, or relationship can affect a mandatory integrity
obligation.

Consumer-local or environment-specific paths may be excluded only through
explicit consumer-owned policy.

A repository state need not be Git-clean.

Pre-existing tracked modifications, staged modifications, or untracked files
are not generic integrity failures merely because they exist.

A consumer MAY declare additional cleanliness requirements as consumer-owned
obligations.

### ConsumerIntegrityProfile

Each consumer owns one canonical Repository Integrity profile.

That profile owns:

* mandatory validation membership;
* validation ordering where ordering is meaningful;
* validation prerequisites where they exist;
* repository-specific paths and bindings;
* required generated or validated projections;
* repository-specific environment/toolchain requirements;
* repository-specific preserved-artifact custody requirements;
* any consumer-specific cleanliness requirement.

Neither `proto-ring` nor another consumer may infer, add, remove, reorder, or
redefine these facts.

Validation membership and ordering are mutable repository knowledge and MUST
have one canonical consumer-owned source.

Documentation and CI MUST reference that canonical source or be mechanically
validated against it rather than maintaining independent manual copies.

## Integrity obligation result

Every mandatory applicable obligation has one of three semantic outcomes:

```text
SATISFIED
VIOLATED
UNDETERMINED
```

`UNDETERMINED` includes cases where required state, environment, tooling,
evidence, or other prerequisite information cannot be established sufficiently
to evaluate the obligation.

Failure to determine a mandatory obligation MUST NOT be interpreted as success.

## Repository Integrity verdict

Repository Integrity is `PASS` if and only if, for the exact governed repository
state:

1. every mandatory applicable obligation is determined; and
2. every mandatory applicable obligation is satisfied; and
3. every required current projection is current with respect to its canonical
   source; and
4. every applicable current custody/preservation obligation is satisfied; and
5. integrity evaluation has not converted a different repository state into a
   passing state.

Formally:

```text
Integrity(S, P, C) = PASS

iff

for every mandatory applicable obligation O in P:
    result(O, S, C) = SATISFIED

and

required current-state coherence holds for S
```

Any `VIOLATED` or `UNDETERMINED` mandatory obligation makes the overall verdict
non-PASS.

An obligation that was skipped because execution stopped after another failure
is not thereby considered satisfied.

## Exact-state binding

An integrity verdict applies only to the exact repository state and integrity
profile that were evaluated.

A change to any governed repository state or to the consumer-owned integrity
profile invalidates a previous current-state verdict unless an explicit
mechanism proves that the previous evidence remains valid for the new state.

Persisted integrity evidence MUST therefore bind to an exact state identity
sufficient for its claimed scope.

The contract does not require one universal state-identity mechanism.

A consumer may use, for example:

* direct evaluation of the current state;
* Git object identity;
* a content-sensitive transient fingerprint;
* a committed content manifest;
* another mechanism that establishes the required exact-state binding.

## Evaluation purity

Repository Integrity is evaluation, not repair.

Integrity evaluation MUST NOT report `PASS` by modifying the governed repository
into compliance during the evaluation.

Pre-existing repository state is the evaluation baseline.

If evaluation itself leaves the governed repository state different from that
baseline, the evaluation MUST be non-PASS.

This comparison MUST be sensitive to changes in the content and identity of the
governed state. A comparison that observes only Git status classes or path
dirtiness is insufficient when content can change without changing those
classes.

This requirement intentionally preserves the useful Turnlock property:

```text
pre-existing dirty state is admissible
+
new validation-caused drift is not
```

while strengthening state comparison so that mutation of an already-dirty or
already-untracked artifact cannot become invisible merely because its Git status
classification remains unchanged.

Evaluation mechanisms SHOULD be observational.

A consumer may invoke a deterministic generator as a freshness probe, but the
generator MUST NOT be able to mutate stale state into a successful integrity
result.

Repair, rendering, regeneration, and other state-producing operations are
outside the semantic responsibility of Repository Integrity.

## Generated and maintained projections

A required mutable projection participates in Repository Integrity when the
consumer declares it part of the current governed repository state.

Such a projection MUST correspond to its canonical source for the same
repository state.

A stale required projection makes Repository Integrity non-PASS.

A generated projection does not become semantic authority merely because
Repository Integrity validates it.

Repository Integrity does not prescribe one projection-validation mechanism.

Consumers may use:

* pure comparison;
* deterministic re-derivation in isolation;
* content hashes;
* schema validation;
* consumer-specific validators;
* another mechanism that establishes currentness without weakening this
  contract.

The complete generic ownership and projection taxonomy may be governed by a
separate proto-ring Projection Integrity contract.

Repository Integrity requires only the currentness properties necessary to
determine current repository coherence.

## Active-state manifests

A committed active-state manifest is NOT a universal Repository Integrity
requirement.

Ruu demonstrates that exhaustive content manifests are a strong mechanism for:

* content-sensitive state binding;
* active-file discovery;
* missing or unexpected artifact detection;
* projection-currentness enforcement.

Turnlock demonstrates that Repository Integrity can exist without requiring a
persisted whole-repository manifest.

Therefore the generic requirement is exact and sufficiently complete state
binding, not one mandated manifest representation.

A consumer MAY use an active-state manifest as one mechanism satisfying this
requirement.

The manifest remains a consumer-owned artifact and does not acquire independent
semantic authority.

## Failure collection and execution strategy

Repository Integrity requires complete logical coverage, not one universal
execution strategy.

A consumer or shared runtime MAY:

* continue after independent failures and aggregate diagnostics;
* stop after a failure when later obligations cannot or should not execute;
* model explicit validation prerequisites.

However:

* an observed failure MUST NOT be suppressed;
* an unevaluated mandatory obligation MUST NOT be represented as satisfied;
* execution ordering declared by the consumer MUST be preserved;
* execution strategy MUST NOT change the meaning of the consumer-owned
  obligation set.

Failure aggregation is therefore an admissible reusable capability, but not
itself the definition of Repository Integrity.

## Historical artifacts and evidence

Historical evidence has two distinct relationships to Repository Integrity.

### Current custody

When a consumer declares that historical artifacts must remain preserved,
Repository Integrity MAY include current-state obligations such as:

* required artifact presence;
* exact content hash;
* immutability or non-writability requirements;
* lineage consistency;
* registration completeness;
* path/provenance consistency;
* separation between retained and current evidence classes.

These are current repository-coherence obligations.

### Historical claim validity

Repository Integrity does NOT establish that historical evidence proves its
original claim.

Activities such as:

* replaying a historical executable;
* reproducing a historical result;
* validating a behavioral claim;
* establishing semantic correspondence;
* formal or state-space verification;

belong to Qualification, formal assurance, or another consumer-specific evidence
system.

A preserved `PASS` string is never sufficient merely because the containing
artifact has Repository Integrity.

## Repository Integrity and Qualification

Repository Integrity and Qualification are distinct.

```text
Repository Integrity
→ Is this exact current repository state internally coherent
  according to its repository-governance obligations?

Qualification
→ Do the consumer-specific evidence and replay procedures establish
  the claims they are intended to establish?
```

Repository Integrity is necessary but not sufficient for a current
repository-level Qualification verdict when that qualification depends on
artifacts from the current repository state.

Such a qualification MUST bind to the same exact repository state whose
Repository Integrity was established.

A qualification replay result may exist independently as evidence about its own
explicitly bound historical or external subject.

That replay result does not itself establish current Repository Integrity.

Therefore a consumer may have:

```text
Repository Integrity PASS
Qualification FAIL
```

but a current repository-level qualification process MUST NOT claim overall
success for a repository state whose required Repository Integrity is non-PASS
or undetermined.

## Consumer authority boundary

The canonical proto-ring contract owns the generic semantics above.

Each consumer continues to own:

* validation membership;
* validation ordering;
* validation prerequisites;
* canonical repository facts;
* repository paths;
* generated artifacts;
* projection bindings;
* product semantics;
* accepted decisions;
* formal-assurance policy;
* qualification claims;
* qualification evidence;
* historical snapshots;
* concrete manifests;
* lineage and provenance data;
* toolchain requirements;
* repository-specific output and CLI contracts.

Shared Repository Integrity implementation consumes these consumer-owned facts.

It does not create or infer them.

## Adoption and strengthening

A consumer may adopt shared Repository Integrity through:

```text
exact equivalence
OR
preservation of an existing guarantee
OR
explicitly justified intentional strengthening
```

Adoption MUST NOT weaken an existing demonstrated consumer guarantee.

When current implementation behavior is weaker than an already-declared
consumer guarantee, migration to the shared contract MAY correct that gap as an
intentional strengthening rather than preserving the weaker implementation
artifact.

## Derived cross-consumer decisions

The contract is derived from the current Turnlock and Ruu governance systems
without treating either as the reference implementation.

### From Turnlock

Turnlock demonstrates:

* one canonical repository-validation membership and ordering;
* explicit repository-integrity evaluation;
* consumer-owned membership separated from generic execution mechanics;
* fail-closed step failures;
* failure aggregation;
* relative purity checking;
* support for pre-existing dirty state;
* generated projections produced outside diagnostic validation;
* a distinction between repository integrity and deeper formal-assurance
  evidence.

These support the generic requirements for canonical consumer-owned membership,
fail-closed evaluation, relative baseline preservation, and separation of
integrity from higher evidence layers.

Turnlock's current status-based before/after purity comparison does not fully
identify content changes to artifacts that were already dirty or untracked.

The canonical contract therefore strengthens this mechanism to
content-sensitive governed-state comparison while preserving its intended
pre-existing-dirty-state behavior.

### From Ruu

Ruu demonstrates:

* deterministic generated-artifact freshness;
* exhaustive active-file registration through a content manifest;
* content-sensitive SHA-256 state authentication;
* explicit retained/current evidence separation;
* immutable historical snapshot custody;
* lineage and provenance integrity;
* fail-closed unsupported or unavailable replay prerequisites;
* deterministic qualification replay distinct from mere artifact presence.

These support the generic requirements for exact-state binding, projection
currentness, fail-closed undetermined states, and the distinction between
historical-artifact custody and historical-claim qualification.

Ruu currently combines current repository-coherence checks and qualification
replay in one top-level Qualification workflow.

The canonical contract separates those responsibilities:

```text
Ruu Repository Integrity
→ current repository coherence

Ruu Qualification
→ Repository Integrity for the same current state
  +
  Ruu-specific qualification/replay obligations
```

Ruu's committed active manifest remains a valid and strong local mechanism, but
the contract does not require Turnlock or another consumer to adopt that
specific representation.

## Resulting shared boundary

The resulting architecture is:

```text
                         proto-ring
                Repository Integrity contract
                           │
              generic evaluation substrate
                    /               \
                   /                 \
          Turnlock binding        Ruu binding
                │                     │
     Turnlock-owned profile    Ruu-owned profile
                │                     │
   Turnlock Repository       Ruu Repository
       Integrity                 Integrity
                │                     │
      formal assurance       Ruu Qualification
       where applicable       where applicable
```

The consumers converge on the meaning of Repository Integrity without converging
on their product semantics, validation membership, evidence systems, or
repository-specific governance facts.
