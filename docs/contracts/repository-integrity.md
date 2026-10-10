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
```

These identities record derivation provenance.

They do not make either consumer a reference implementation and do not make
future consumer repository state ambient authority over this contract.

## Purpose

Repository Integrity defines whether one exact current repository state
satisfies all repository-governance obligations that its consumer declares
mandatory for that state.

Repository Integrity is a current-state coherence property.

It is not:

- product-semantic authority;
- qualification of behavioral or product claims;
- formal verification;
- historical evidence replay;
- proof that a repository is correct in every possible sense;
- synonymous with a clean Git worktree.

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

The shared local observation semantics are owned by
[Canonical RepositoryState](repository-state.md). Repository Integrity MUST
compose that responsibility rather than define a competing state-capture model.

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

- mandatory validation membership;
- validation ordering where ordering is meaningful;
- validation prerequisites where they exist;
- repository-specific paths and bindings;
- required generated or validated projections;
- repository-specific environment/toolchain requirements;
- repository-specific preserved-artifact custody requirements;
- any consumer-specific cleanliness requirement.

Neither `proto-ring` nor another consumer may infer, add, remove, reorder, or
redefine these facts.

Validation membership and ordering are mutable repository knowledge and MUST
have one canonical consumer-owned source.

Documentation and CI MUST reference that canonical source or be mechanically
validated against it rather than maintaining independent manual copies.

## Persistent consumer profile

Model version 1 serializes the consumer-owned profile as canonical structured
frontmatter under `repository_integrity`:

```yaml
repository_integrity:
  model_version: 1
  authority:
    responsibility: <GovernedResponsibilityId>
    source: <GovernedSourceId>
  environments:
    - <ValidationEnvironmentId>
  continue_after_non_satisfied: true
  validations:
    <ValidationId>:
      responsibility: <GovernedResponsibilityId>
      prerequisites: []
      instances:
        kind: single
      command:
        kind: command
        environment: <ValidationEnvironmentId>
        arguments: []
        undetermined_exit_codes: []
  order:
    - <ValidationId>
```

The carrier is the authoritative governed source for the profile's declared
responsibility. Loading composes with Governance Authority and does not execute
validators. A `ValidationId` is an opaque, non-empty consumer-owned identity for
a validation requirement, not a label or runtime-instance identity. Every
model-version-1 validation is mandatory.

`order` is a total order containing every declared `ValidationId` exactly once.
Mapping declaration order has no meaning. Each validation's duplicate-free
`prerequisites` identifies declared validations that precede it in `order`.
Order alone does not imply dependency.

A validation may declare one governed-object target:

```yaml
target:
  interface: <InterfaceId>
  object: <ObjectId>
```

The object must exist and participate in the validation's governed
responsibility. This association does not make the validation operation or an
executable path a governed object.

### Concrete instance resolution

Model version 1 supports exactly `single` and `repository_paths` instances.
`single` resolves to one concrete command obligation. Repository paths support
exactly `append_all` and `for_each`:

```yaml
instances:
  kind: repository_paths
  mode: append_all
  selectors:
    - kind: path
      path: pyproject.toml
    - kind: glob
      glob: src/*/*.py
```

A literal `path` remains selected when absent. A `glob` uses `/` as separator
and only `*` as syntax. `*` matches zero or more characters within one component
and never crosses `/`. Absolute paths, `..`, `**`, `?`, character classes,
brace expansion, shell expansion, regular expressions, and Git pathspec
semantics are forbidden.

Selectors run in declaration order. Matches from each glob are sorted lexically
by repository-relative path. Duplicate paths across selectors fail resolution;
they are not deduplicated. Zero glob matches are valid and create no generic
presence obligation.

`append_all` always resolves to one obligation with selected paths appended to
the base arguments. With zero selected paths, that one obligation contains only
the base arguments. `for_each` resolves to one obligation per selected path and
to zero obligations when selection succeeds with zero paths. Expanded path
membership is runtime repository state, not persistent profile identity.

### Command and environment binding

Model version 1 supports only `command` bindings. A command declares one
profile-owned `ValidationEnvironmentId`, ordered base arguments, and explicit
nonzero `undetermined_exit_codes`. Machine-specific executable paths are not
persistent profile data.

An `EvaluationContext` supplies each available environment realization:

```text
ValidationEnvironmentId
+
opaque realization identity
+
runtime command prefix
+
explicit process environment
```

The runtime command is the realization's prefix followed by the persistent base
arguments and any selected repository paths. A missing required realization or
a command that cannot execute is `UNDETERMINED`. Repository Integrity does not
provision environments, create virtual environments, install dependencies,
modify `PATH`, or repair tooling.

Profile identity derives from canonical persistent profile semantics. It
excludes runtime command prefixes, executable paths, expanded glob results,
process environments, and ambient variables. Evaluation-context identity
depends only on the opaque identities of realizations required by the profile.
It excludes unrelated realizations, command prefixes, process environment
values, ambient secrets, `HOME`, and `PATH`.

A result binds all three exact dimensions:

```text
repository state identity
profile identity
evaluation context identity
```

## Validation-level aggregation and prerequisites

Instance resolution and command execution are separate. If complete instance
resolution cannot be determined, the aggregate `ValidationId` status is
`UNDETERMINED`; a partial selection is not successful resolution.

After successful complete resolution, aggregate exactly:

```text
zero concrete instances
→ SATISFIED

one or more instances and any VIOLATED
→ VIOLATED

one or more instances, none VIOLATED, and any UNDETERMINED
→ UNDETERMINED

one or more instances and all SATISFIED
→ SATISFIED
```

Thus `VIOLATED` has precedence over `UNDETERMINED`, which has precedence over
`SATISFIED`, independent of concrete execution order. Model version 1 has no
fourth `NOT_APPLICABLE`, `SKIPPED`, or `EMPTY` status.

A successfully resolved zero-instance `for_each` validation is deliberately
`SATISFIED` for that exact repository state. This means only that no concrete
obligation applies under the declared membership rule. It does not prove that a
path exists and does not imply that a selector should normally match. A
consumer that requires presence must declare a separate consumer-owned
validation; proto-ring never invents that requirement.

Prerequisites refer to aggregate `ValidationId` status. A dependent executes
only when every prerequisite is `SATISFIED`. A zero-instance satisfied
prerequisite therefore permits its dependent. A `VIOLATED` or `UNDETERMINED`
prerequisite makes the dependent `UNDETERMINED` and not executed.

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

- direct evaluation of the current state;
- Git object identity;
- a content-sensitive transient fingerprint;
- a committed content manifest;
- another mechanism that establishes the required exact-state binding.

Repository Integrity inherits RepositoryState's mutation-isolation requirement
with respect to uncoordinated external writers across exact-state observation
and evaluation. Before/after RepositoryState identity equality is mandatory
drift detection; it is not proof that mutation isolation existed.

The currently evaluated obligation remains observable as a possible mutator; it
is not hidden by the external-writer isolation precondition. Its mutation MUST
continue to be detected through the existing before/after RepositoryState
comparison and the existing `SATISFIED`, `VIOLATED`, `UNDETERMINED`, `PASS`, and
`NON_PASS` semantics.

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

### Exact purity-status attribution

Repository-state coherence and command-result classification remain distinct.

The exact model-version-1 attribution is:

```text
required state cannot be established before an obligation
→ that obligation is UNDETERMINED
→ it is not executed
→ evaluation halts for state-coherence reasons

required pre-obligation state is known and differs from the evaluation baseline
→ that obligation is UNDETERMINED
→ it is not executed
→ evaluation halts for state-coherence reasons

the obligation executes but required post-obligation state cannot be established
→ that obligation is UNDETERMINED
→ evaluation halts for state-coherence reasons

the obligation executes and the required post-obligation state is known to
differ from the evaluation baseline
→ that obligation is VIOLATED
→ evaluation records a purity/state-coherence failure
→ evaluation halts for state-coherence reasons
```

Known post-execution drift is a demonstrated violation of the evaluation-purity
requirement, even when the command exit status alone would otherwise have mapped
to `SATISFIED` or `UNDETERMINED`.

Failure to establish the required state is instead `UNDETERMINED`; inability to
observe a comparison MUST NOT be converted into a known mismatch.

Every not-yet-executed mandatory obligation after a state-coherence halt is
`UNDETERMINED`.

The consumer's `continue_after_non_satisfied` policy does not override a
state-coherence halt. Continuation applies only to non-satisfied validation
results observed while repository state remains the exact evaluation baseline.

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

- pure comparison;
- deterministic re-derivation in isolation;
- content hashes;
- schema validation;
- consumer-specific validators;
- another mechanism that establishes currentness without weakening this
  contract.

The complete generic ownership and projection taxonomy may be governed by a
separate proto-ring Projection Integrity contract.

Repository Integrity requires only the currentness properties necessary to
determine current repository coherence.

## Active-state manifests

A committed active-state manifest is NOT a universal Repository Integrity
requirement.

Ruu demonstrates that exhaustive content manifests are a strong mechanism for:

- content-sensitive state binding;
- active-file discovery;
- missing or unexpected artifact detection;
- projection-currentness enforcement.

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

- continue after independent failures and aggregate diagnostics;
- stop after a failure when later obligations cannot or should not execute;
- model explicit validation prerequisites.

However:

- an observed failure MUST NOT be suppressed;
- an unevaluated mandatory obligation MUST NOT be represented as satisfied;
- execution ordering declared by the consumer MUST be preserved;
- execution strategy MUST NOT change the meaning of the consumer-owned
  obligation set.

Failure aggregation is therefore an admissible reusable capability, but not
itself the definition of Repository Integrity.

## Historical artifacts and evidence

Historical evidence has two distinct relationships to Repository Integrity.

### Current custody

When a consumer declares that historical artifacts must remain preserved,
Repository Integrity MAY include current-state obligations such as:

- required artifact presence;
- exact content hash;
- immutability or non-writability requirements;
- lineage consistency;
- registration completeness;
- path/provenance consistency;
- separation between retained and current evidence classes.

These are current repository-coherence obligations.

### Historical claim validity

Repository Integrity does NOT establish that historical evidence proves its
original claim.

Activities such as:

- replaying a historical executable;
- reproducing a historical result;
- validating a behavioral claim;
- establishing semantic correspondence;
- formal or state-space verification;

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

- validation membership;
- validation ordering;
- validation prerequisites;
- canonical repository facts;
- repository paths;
- generated artifacts;
- projection bindings;
- product semantics;
- accepted decisions;
- formal-assurance policy;
- qualification claims;
- qualification evidence;
- historical snapshots;
- concrete manifests;
- lineage and provenance data;
- toolchain requirements;
- repository-specific output and CLI contracts.

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

- one canonical repository-validation membership and ordering;
- explicit repository-integrity evaluation;
- consumer-owned membership separated from generic execution mechanics;
- fail-closed step failures;
- failure aggregation;
- relative purity checking;
- support for pre-existing dirty state;
- generated projections produced outside diagnostic validation;
- a distinction between repository integrity and deeper formal-assurance
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

- deterministic generated-artifact freshness;
- exhaustive active-file registration through a content manifest;
- content-sensitive SHA-256 state authentication;
- explicit retained/current evidence separation;
- immutable historical snapshot custody;
- lineage and provenance integrity;
- fail-closed unsupported or unavailable replay prerequisites;
- deterministic qualification replay distinct from mere artifact presence.

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
