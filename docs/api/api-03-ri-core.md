---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "implementation-guide"
domain: "proto-ring-repository-integrity"
severity: "guideline"
name: "API-03 guarded Repository Integrity core"
version: "0.1.0"
---

# API-03 guarded Repository Integrity core

## Authority and evidence boundary

This component implements the shared realization/evaluation boundaries of
[API-01 sections 7–8](v1/proto-ring-api-01-v1-spec.md), accepted at
`a1b27ca1e21423973a5557415488e4fcd535d01a`. The existing
[RI](../contracts/repository-integrity.md) and
[RepositoryState](../contracts/repository-state.md) contracts remain unchanged.
Consumer policy, host authorization and engine classification remain separate.

This is component evidence, not a qualified host, public check/output service,
receipt implementation, consumer adoption, or complete API-03 qualification.
The accepted semantic cases retain `qualified:false`. The
[API-02 surface](api-02-implementation.md) remains unchanged.

## Modules and dependency direction

All module paths are under `crates/proto-ring-engine/src/repository_integrity/`.

- `realization.rs` owns the single canonical expansion, extracted from the former
  evaluator: exact-root/context validation, literal scope, one discovery,
  before/after discovery observations, selectors, arguments and consumer order.
- `evaluate.rs` owns the single evaluation loop, required observation sequencing,
  prerequisite/continuation policy, aggregate results and completion boundary.
- `backend.rs` defines one mutable `Backend`, its associated interruption/process
  types, capture phases and original validation/instance loop positions. Its
  `Ordinary` adapter supplies synchronous compatibility observations.
- `observations.rs` defines technical `Launch` and known `Completion` facts.
  Neither supplies an RI status. `NotStarted` is not itself host evidence.
- `execution.rs` retains the sole native command construction and owns the shared
  process-fact classification. No second runner is introduced.
- `control.rs` owns opaque 32-byte attempt identities, the exact eight API-01
  checkpoints, abort reasons and caller-retained monotone `AttemptControl`.
- `model.rs`, `selectors.rs`, `identity.rs` and `load.rs` retain their existing
  responsibilities. `mod.rs` exports preserved entries and implementation hooks.

The two cores depend on the backend contract, control, canonical models and
existing RepositoryState/selector mechanisms. The ordinary adapter depends on
the existing native executor. The engine never depends on `proto-ring-api`.
Tests do not supply a second evaluator or aggregate. Handwritten sources/tests
remain at most 400 physical lines each, with tests divided by responsibility.

## Preserved entries and implementation hooks

`realize_consumer_profile()` and `evaluate()` retain their signatures and result
shapes. Both use the same cores as guarded evaluation. Existing conformance
capture, discovery, ambient-environment and execution-observer hooks also use
these cores. Impossibility for `Infallible` uses an exhaustive empty match.

The doc-hidden `backend`, `control`, `observations`, `realize_guarded()` and
`evaluate_guarded()` exports are engine implementation boundaries for subsequent
components, not new API-01 public services. The core exists without features;
only legacy conformance hooks require `conformance-internal`.

`realize_guarded()` returns
`Result<Result<RealizedIntegrityProfile, RepositoryIntegrityError>, Interruption>`.
`evaluate_guarded()` returns `GuardedEvaluation::Completed { result }` or
`Interrupted { reason, effects_possible }`, never a partial result plus a flag.
No capture index, reference, resource reservation or closure certificate is
fabricated to satisfy future interfaces.

## Two observation error channels

Backend capture and discovery return respectively:

```text
Result<Result<RepositoryState, StateCaptureError>, Abort>
Result<Result<Vec<PathBuf>, StateCaptureError>, Abort>
```

The inner error remains ordinary RI uncertainty under an active guard. The
outer interruption immediately leaves the core, preserving its exact reason,
start knowledge and attempt. It is never converted to discovery failure,
StateCaptureError, ProcessOutcome or a completed IntegrityResult.

During realization, ordinary unavailable discovery or differing observed states
preserves failed resolution. The outer channel is extracted before this mapping.
Single/empty profiles do not invent discovery or extra state observations.
Successful empty `for_each`, failed resolution, zero-path `append_all`, missing
realization, selector ordering and opaque context identity retain their original
meanings. Context identity still excludes effective program/environment values;
it is not admission evidence.

## Process facts and monotone effects

The core passes the original validation/instance loop positions into launch and
completion. It does not reconstruct an obligation list to find positions.
`Started(process)` sets effects possible **before** completion can wait or abort.
A completion consists only of known success and optional canonical numeric exit
text. Guarded backends must establish principal completion and effect closure
before returning it, followed by the effect-lifetime checkpoint.

The engine classifies facts once:

- success: SATISFIED;
- configured unsuccessful undetermined exit: UNDETERMINED;
- other known unsuccessful termination, including absent numeric exit: VIOLATED;
- reported ordinary or established guarded non-start: UNDETERMINED, not executed;
- guarded uncertain start/completion/closure: HostAbort, not process classification.

`AttemptControl` starts false and exposes no reset. Started and any
PotentiallyOccurred abort make it permanently true. NotStarted and
DefinitivelyExcluded aborts cannot clear prior effects. The caller borrows the
same object across realization/evaluation and retains it after Completed as well
as Interrupted, for subsequent finalization/persistence decisions. Attempt ID
creation/randomness and plan/permit/generation binding are not implemented here.

The ordinary adapter deliberately retains `Command::status()`. Its error can
come from **waiting**, not just spawning; the historical nonexecuted fallback
is not contractual proof of non-start. Successful synchronous completion is
wrapped in the technical Started boundary without changing existing results.
This adapter cannot satisfy `Backend<Abort = HostAbort>` and must not become a
protected production host. Environment clearing, explicit environment, prefix
tail, expanded arguments, root cwd and null standard streams are unchanged.
No stdout/stderr collection or actual descendant control is provided.

## Checkpoint order and early completion

Realization checks BeforeRealization, performs canonical work, then checks
AfterRealization on every ordinary success/error. An outer abort stops
immediately, with no later AfterRealization, discovery or capture.

Evaluation checks BeforeRequiredRiCapture before Baseline, PreObligation and
PostObligation captures. Each executable candidate checks BeforeEachLaunch;
Started then completes its process/effect lifetime and checks
AfterEachEffectLifetime **before** post-capture or any next obligation.
Missing/known-unlaunchable commands do not invent launches or lifetime closure,
but retain required ordinary pre/post observations. A reported NotStarted also
retains post-observation, without inventing a started lifetime.

All ordinary RI returns converge on AfterFinalRiObservation. This includes
unavailable baseline, A/B/B, empty profiles, zero instances, coherence halts and
consumer policy stops. There is no extra final capture. Failure of the final
checkpoint returns interruption, including on a vacuous otherwise-PASS result.
Aborts never retry or continue normal calculation/finalization. Host resource
cleanup remains the host's responsibility, not proof of contained effects.

## Component criteria and tests

Unit test paths below are relative to the mechanism's `tests/` directory.
The scripted backend declares a complete event queue and checks exact operation
order, phase, loop position and exhaustion. It claims no actual protection.
Expected statuses and full results are authored independently of production
classification/aggregation, then exercised separately through both adaptations.

| Criterion | Test evidence | Nature |
| --------- | ------------- | ------ |
| Shared canonical realization/aggregation | `realization.rs`, `policy.rs`, `processes.rs`, structural #69 controls | Independent expectations on both adaptations; source boundary checks |
| Abort never ordinary process outcome | `processes.rs::unknown_start_wait_and_uncontained_effects_never_become_results` | Deterministic failure facts; no completed obligation/result |
| Interruption never completed RI | `interruptions.rs::every_checkpoint_capture_launch_and_wait_abort_is_exact_and_terminal` | Every evaluation boundary cut, both start-knowledge values, exact abort/trace |
| Ordinary capture consequences | `evaluation.rs::ordinary_capture_errors_preserve_complete_results_at_every_phase`, `capture_errors_after_a_prior_execution_retain_every_obligation_fact` | Full results and execution/exit/identity fields, before/after prior effects |
| Consumer order/prerequisites/continuation | `policy.rs`, published #69 vectors | Multiple instances, violated dominance, post-validation stop, zero-instance prerequisite |
| Empty resolution versus failure | `realization.rs`, `policy.rs::satisfied_zero_instances_enable_dependents_but_resolution_failure_does_not` | Actual canonical expansion and direct aggregates |
| A/B/B | `evaluation.rs::realization_a_baseline_b_final_b_is_not_equal_and_never_launches` | Direct baseline B/final B, NOT_EQUAL, halt and absence of launch |
| RI statuses/relation/halt preserved | `evaluation.rs`, `policy.rs`, `processes.rs`, unchanged 34 published expectations | Full direct fields plus canonical real-process conformance |
| Independent expectations | `tests/script.rs` expected-value constructors, ordinary adapter tests, #69 corpus | Literal semantic expectations, never adapter-derived or production-aggregated |
| Monotone effects and retained control | `interruptions.rs`, `processes.rs`, integration `api03-ri-core.rs` | Before wait, uncertain first start, later non-start, excluded later abort, Completed and cross-core retention |
| All early completion checkpoints | `interruptions.rs::final_checkpoint_is_required_on_every_ordinary_early_exit` | Nine exit scenarios, exact final-checkpoint loss and retained effects |
| No execution at load/RGS | unchanged #133 tests and #69 structural controls | Existing real sentinels and canonical composition boundary |
| Portable integration boundary | `crates/proto-ring-conformance/tests/api03-ri-core.rs` | Both cores and retained control through normal compiled exports; deterministic host double |

API-01 component-property contributions: A03/A04 (empty/zero), A06 (missing
permitted realization), A56 (A/B/B), A65 (realization abort), A67 (ordinary
capture failure), A75 (zero versus failed resolution). A09/A47 and A78/A79/A80
are **partial** contributions: abort/effect-state and ordering rules are tested,
but public execution_incomplete projection, real admission, real descendant
containment and qualified host protection are absent. All other full public
scenarios are outside this component unless separately evidenced by unchanged
regressions. No complete public semantic scenario is qualified by this double.

## Validation and sensitivity

The task used Rust/Cargo 1.99.0 and the pinned `requirements-test.txt` dependencies
in a worktree-local `target/dev-venv`, with PROTO_RING_PYTHON pointing to its
lexical interpreter and PYTHONDONTWRITEBYTECODE=1. Baseline targeted #69/#133
passed at `e88f79a02b8919e27013bc9e3962a06e0a4d3d89`; exact-source hosted baseline
[CI](https://github.com/fanilosendrison/proto-ring/actions/runs/42001771930)
also passed.

Executed component commands (all passed on the restored candidate):

```bash
cargo test -p proto-ring-engine --lib repository_integrity::tests --locked
cargo test -p proto-ring-conformance --locked \
  --test issue69_repository_integrity_evaluate \
  --test issue133_repository_integrity_observation --test api03-ri-core
```

Results: 19 new unit tests, 34/34 unchanged published RI observations (#69's two
tests), seven #133 tests and one integration test. The #69 process regressions
use the existing canonical native runner, not a test replica. Deterministic
boundary tests and real ordinary processes are distinct evidence classes.

Five one-at-a-time temporary production mutations each compiled and made the
targeted test fail with exit 101: absorbed discovery abort; evaluation abort
converted to Completed; ignored A/B/B mismatch; effects reset to false; final
checkpoint omitted on empty profile. Original production hashes and the complete
tracked diff were compared between mutations. Restored targeted tests passed
19/19. These intentional RED controls were not published; their logs are
retained with task evidence. An initial test-construction compiler typo was
corrected before the passing validation, not classified as a baseline defect.

Final local validation also passed the following commands, in sequence:

```bash
git diff --check
cargo fmt --all -- --check
cargo clippy -p proto-ring-engine --lib --locked -- -D warnings
cargo clippy -p proto-ring-engine --all-targets --all-features --locked -- -D warnings
cargo test -p proto-ring-engine --all-targets --no-default-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo test -p proto-ring-api --all-targets --no-default-features --locked
cargo build -p proto-ring-api --lib --bin proto-ring --no-default-features --locked
cargo clippy -p proto-ring-api --all-targets --no-deps --locked -- -D warnings
cargo test --workspace --doc --locked
target/dev-venv/bin/python -I -B tests/test_package.py
target/dev-venv/bin/python tools/run_python_tests.py
target/dev-venv/bin/python -B docs/api/v1/conformance/validate-wire-schemas.py
target/dev-venv/bin/python -B docs/api/v1/conformance/check-identity-vectors.py
target/dev-venv/bin/python -B -m unittest discover \
  -s docs/api/v1/conformance -p 'test_*.py' -v
```

The normal engine passed 70 tests; the workspace passed 352 tests and two
doctests; the normal API passed 118 tests. Neutral Python passed 176 tests.
API-01 controls passed 197 schema cases, five discovery derivations, 13 public
identity vectors, 11 path codec cases and ten Python tests. These controls do
not qualify the 87 runtime scenarios. No full local gate failed. Exact-source
hosted CI and independent review are attributed in the linked change and
delivery receipt, separately from local evidence; Windows is not executed on
the local macOS machine. Linux CI automatically discovers the new tests through
workspace commands. Its isolated normal-engine test and the Windows component integration
step explicitly cover configurations absent from the prior command inventory.

## Subsequent integration boundaries

- **#145:** real invocation identity, BeforeRgs/BeforePlanAdmission, workspace
  acquisition/coverage, complete immutable plan and admission, permit/attempt/
  generation/executable binding and dispatch consumption. Reuse these cores and
  original loop positions; do not invent another expander or evaluator.
- **#146:** extend the existing native execution pathway with actual guarded
  start/wait, output capture and established descendant-effect containment.
  Technical facts/checkpoint success in a double are not real closure evidence.
- **#147:** reserve budgets/resources before effects; use the same retained
  control after Completed or Interrupted for truthful persistence outcomes.
  This component supplies no receipt, capture index or custody certificate.
- **#148:** orchestrate the protected RGS/RI invocation and reuse API-02 syntax,
  projections and dispatch. Translate genuine completion/interruption only at
  that boundary; no public JSON or CLI check/output service is added here.

Full assembled execution and independent fresh-agent qualification remain
separate obligations; no parent acceptance follows solely from component tests.
