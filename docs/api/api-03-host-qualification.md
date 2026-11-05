---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "qualification-guide"
domain: "proto-ring-api03-host"
severity: "guideline"
name: "Bounded API03-A host qualification"
version: "0.1.0"
---

# API03-A bounded host qualification

## Status and authority

**NOT_YET_QUALIFIED.** This document freezes an experimental protocol for
[Issue #143](https://github.com/fanilosendrison/proto-ring/issues/143).
No campaign result, API-03 service, public CheckHost, public HostAbort,
CheckReceipt or host-established API observation is supplied by this document.
The accepted [API-01 contract](v1/proto-ring-api-01-v1-spec.md), especially
sections 7–9, and the generic RepositoryState, RepositoryGovernanceState and
Repository Integrity contracts retain authority. API-02's actual exports and
missing execution services are described in [its guide](api-02-implementation.md).
The Rust-shaped host signatures in API-01 are future boundaries, not existing
exports. The harness uses existing canonical loaders/captures and pure RI
realization, never consumer validator evaluation.

## Fixed candidate and trust boundary

The candidate is one GitHub-hosted `ubuntu-24.04` job, trusted Rust controller,
existing Docker daemon and kernel, and nonprivileged Docker probe processes.
The examined coherence class is `exclusive_workspace`. The subject is a new,
standalone disposable Git repository, with a real `.git`, HEAD, index, tracked
files and minimal loadable governance. It is not the development checkout, a
linked development worktree, or a consumer repository.

The VM administrator, Docker daemon, kernel, reviewed controller, canonical
Proto-Ring observation tools and fixed campaign source/policy are trusted.
Untrusted actors are the admitted fixture and an unadmitted confined writer.
Neither receives the controller's host shell, credentials, Docker socket or
control directories. The claim excludes a compromised administrator/daemon
and any untrusted actor already possessing controller-equivalent host access.
It is not an absolute prohibition of root writes or universal Docker/Linux
qualification. Ordinary local same-user checkout use has no such automatic
basis: API-01 requires `admission_refused/execution_guarantee_unavailable`
before validators when the necessary basis is missing. This harness does not
implement that public refusal service.

All evidence, control files and compilation caches are outside the subject.
The admitted process sees the same real writable subject as the canonical
observer; mutations are not hidden by a snapshot. Root correspondence is
established through inspected mounts, device/inode identities and controlled
witnesses, not path-string equality alone. Canonical API-02 references remain
`ordinary_capture`; the experiment does not change their DTO qualification.

## Bounded reuse

Review Broker is inspected read-only at exact revision
`64ee8062c5016cdc5a862b49b8acea18eec0aab4`. Paths in the following table refer
to that revision of `fanilosendrison/review-broker`.

| Source | Useful property | Adaptation | Limit |
| ------ | --------------- | ---------- | ----- |
| `docs/architecture/component-boundaries.md` | Separate evaluated process, controller, verifier and publisher | Rust resource controller, inert Python fixtures, independent evidence checks, external publication | No broker runtime, journal, reconciler or queue |
| `src/review_broker/security_probe.py`, `tests/test_security_probe.py` | Nonsecret witnesses and refusal controls must really be constructible | Writable positive controls and protected sacrificial targets; controller checks bytes | No secret reads; no transfer of broker sandbox verdict; no Internet target |
| `docs/architecture/codespace-lifecycle.md` | Requested stop differs from observed terminal state | Inspect owned resources before deletion and re-list after deletion | No Codespace, remote lifecycle or validator timeout copied |
| `src/review_broker/pilot_private_retention.py`, `tests/testpilotretention.py` | Exact bytes, private staging, no overwrite on conflict, retain failures | Exclusive evidence creation and verified private artifact archive | Not ReceiptCustodian or production retention qualification |
| `src/review_broker/artifact_locator.py`, `tests/test_artifact_locator.py` | Exact run/artifact identity, safe paths and inventory | Retrieve exact artifact ID; verify archive and every member before extraction | A digest proves byte identity, not process truth |
| `docs/contracts/oidc-arm-authority.md` | Launch authority is not workspace coherence | Keep authorization, source binding and actual protection separate | No OIDC, journal or distributed single-use system imported |

The additional precedent is
[Proto-Scope #55](https://github.com/fanilosendrison/proto-scope/issues/55),
[run 40234216650](https://github.com/fanilosendrison/proto-scope/actions/runs/40234216650),
attempt 1, source `f9b53f07a27fb4489314786066951970bff50327`.
Its `.github/workflows/dt03e2-inert-containment.yml` and
`docs/research/derivtrace/falsification/fixtures/dt-03-e-actions-inert-probes.sh`
(script blob `e6fb73c2058058866730fcef13065f326ab94f32`, SHA-256
`f4d18c503043bf8729a0b605883ed2bb3c4486960ab60061dd1f60b96423cedf`)
provide a method precedent: review before experiments, separate probe/result
publication, sacrificial resources, discriminating positive controls, observed
cleanup and retention of failed attempts. No RP08 is run; its result or later
admission disposition does not qualify Proto-Ring.

## Envelope and ownership

The only candidate image is
`docker.io/library/python@sha256:8a00ecb83ed9d0864dfd102a1bdb2067fda907ceb30bb86d09da9ee558b19590`.
Expected image config is
`sha256:141fe9b385f3bb6d618c07ace1836c0b3c72bf525566ac3eb13cd1f405069877`.
The inspected predecessor establishes provenance of this candidate, not its
current availability or protection. Pull/inspect must establish `linux/amd64`,
repository digest and exact config. No mutable tag or alternative image is
permitted. Unavailability is retained, never repaired by installing a daemon.

Ordinary probe containers require network none, read-only rootfs, all
capabilities dropped, no-new-privileges, numeric nonroot UID:GID, private
PID/IPC/network namespaces, no additional devices or Docker socket, pids 64,
512 MiB memory, CPU 1, and at most a 16 MiB scratch tmpfs. Mounts include only
owned resources: subject writable when admitted, program read-only, and
sacrificial writable/read-only control targets. No global permission widening,
privileged container, extra capability, disabled seccomp, host home, credential,
real checkout or host root mount is permitted. Image-defined environment is
compared with effective process configuration; no runner credentials are passed.
Image identity includes the interpreter/dependency trust assumption; a script
hash alone does not establish them.

Only H08 network controls use a campaign-owned internal Docker bridge and inert
local listener, with no published port. The negative client remains network
none. Positive clients use the same numerical endpoint before and after denial.
No Internet or authentication endpoint is a probe target. Downloads happen on
the controller, outside probe containers.

Every container/network receives a campaign label and unique name; IDs are
retained after creation. No persistent working container is reused and no
`--rm` hides observations. Cleanup targets only recorded campaign resources,
never global prune. Git fixture commands use isolated HOME/configuration and
no ambient repository redirection. Positive Git administrative resources and
objects are contained; external git-dir/common-dir/aliases appear only in
negative fixtures.

## Frozen H01–H08 protocol

Each subcase starts `NOT_RUN`; allowed outcomes are `PASS`, `FAIL`,
`UNAVAILABLE`, `INDETERMINATE`, `NOT_RUN`. These are experimental evidence terms,
not public Proto-Ring verdicts. PASS requires all mandatory observations and
controls, not a controller or Docker exit zero.

### H01 — Effective environment and policy

Capture runner OS, architecture, image/build, kernel, Docker client/server,
runtime/cgroup/security mechanisms, image/config identity and fixture source
inventory. Inspect each actual container, including UID, mounts, environment,
limits, namespace modes and privilege/security settings. Exercise process-side
UID/capabilities/no-new-privileges/seccomp/rootfs checks with the inert fixture.
Requested-but-unapplied restrictions fail. A required unavailable mechanism is
not a skipped success. Cgroup observations are reported only when accessible
without elevation; Docker namespace/runtime observations remain distinct.

### H02 — Admitted mutation and canonical visibility

Establish subject/control separation and coverage before initial RGS access.
Load canonical governance and capture canonical state. An admitted container
changes a governed authority file while preserving loadability. Only after
container effect closure, reload and require changed canonical public state
reference, unchanged root/scope domain and expected controller-read bytes.
Also create a nonignored untracked witness and require a distinct canonical
RepositoryState identity. Never emit raw opaque identity bytes. No RI evaluator
or validator is invoked. Pure canonical profile realization may expose RI scope.

### H03 — Unadmitted writer

Run equivalent writes against an owned writable positive target and an existing
read-only protected subject target. Require the positive effect, explicit
program-start evidence, exact EROFS denial, and unchanged controller-observed
negative target bytes. Missing target, wrong path, launch failure and transport
failure do not count as protection. This checks only the declared confined
actor, not a same-user host shell.

### H04 — Git and path coverage

For the positive subject inspect canonical root, real git-dir/common-dir,
RepositoryState dimensions, governance paths and canonical RI scope. Require
contained administrative objects and paths, no external object dependency,
and covered linkage identities. On distinct disposable fixtures introduce an
external git-dir, external common-dir, escaping governed alias and replacement
of a controlled path binding. Require actual detection of insufficient coverage
or canonical rejection; never run validation on these fixtures. The harness's
coverage guard does not replace the engine's canonical path resolver.

### H05 — Program and control protection

Bind launch to exact image, fixture bytes/hash, image interpreter, argv, cwd,
effective environment and inspected mounts. An equivalent writable target must
accept the write. Attempts to alter the read-only program and a sacrificial
control target must fail with exact controller-checked unchanged bytes.
The probe reports only its finite allowed environment and checks absence of
credential names; the runner's complete environment is never recorded.

### H06 — Principal, descendants and effect closure

Run a normal finite process. Separately run parents with live descendants,
including an inherited stream and an already-open write descriptor. Use bounded
explicit ready/release rendezvous, not a sleep followed by assumed readiness.
Capture parent/child start markers, same-container process observations before
principal termination, principal exit and stopped-container/runtime observations.
Do not use recyclable host PID equality as process attribution. Namespace/runtime
closure and pipe collection are separate facts. A forced shutdown is retained
as forced effect closure, never a normal successful validator completion.
No subsequent canonical capture occurs until the owned effect perimeter is
observed closed. Finite absence of a late write is not the proof of closure.

### H07 — Output and evidence boundary

An inert process emits independently specified distinct stdout/stderr, non-UTF8
bytes and exit 23. Compare retained byte files exactly; retain Docker transport,
wait output and inspected process exit separately. Positive writes must work;
an attempt to alter an existing read-only sacrificial evidence target must fail
and leave its bytes unchanged. Real campaign receipts are never mounted as a
writable attack target. Process data, controller observations and verifier
conclusions stay separate. This does not qualify #146's 64 MiB tail retention
or #147's ReceiptCustodian.

### H08 — Actual guard loss and bounded network

Replace an owned linkage/resource after recording its filesystem identity.
Require a real observed mismatch, latch guard loss and refuse subsequent
protected observation under that generation. A mocked false predicate is
insufficient. Preserve before/after identities and the refused checkpoint.
For network controls require an actually ready owned listener and positive
connection/expected response before and after network-none refusal. Only
connect failure classified as route denial counts; successful connect followed
by read failure is not isolation. Docker network-none inspection is mandatory.
This network policy applies to these fixtures, not all RI profiles.

## Evidence, failure and cleanup rules

Fixtures have finite announced outputs, each stream at most 64 KiB; this is not
a load test. Rendezvous and controller transport have experiment guards; a
trigger produces an incomplete/nonqualified result, not a universal RI timeout.
Unknown start never permits automatic retry. A later campaign requires a retained
diagnosis, explicit correction/hypothesis, adapted review, a new campaign identity
and established control of prior resources. No rerun-until-PASS is permitted.

Cleanup is attempted after success or failure. Before removal the controller
retains resource IDs and observations. After removal it re-lists exact owned
containers/networks and checks no live probe perimeter remains. Deletion exit
zero alone is insufficient. Cancellation preventing re-observation yields
`CLEANUP=UNVERIFIED`; VM ephemerality is not a substitute.

Evidence inventory binds repository, run ID/attempt, workflow/source SHA/tree,
source blobs/hashes, runner/runtime/image, resource identities, effective launch,
canonical comparisons, raw finite stream bytes, transport/process statuses,
rendezvous, per-subcase results, guard loss and cleanup. Exclusive evidence
creation refuses replacement. The verifier rejects missing mandatory evidence,
contradictory bytes, absent positive controls, bad hashes, inconsistent source or
run attribution, unexpected image/mounts, nonstart, unclosed effects, unknown
cleanup, oversized data and any non-PASS required subcase.

The manual workflow executes only on main with a required `expected_sha` used
solely for comparison. Checkout source is `github.sha`, with credentials disabled.
Before build/probes it checks input syntax, GitHub SHA, checked-out HEAD and source
blob identity. Official actions are pinned to verified full SHAs. Permissions
are contents-read only, with no OIDC or write permission. Artifact retention is
14 days (the inspected precedent's explicit retention); useful evidence must be
archived privately outside Git before expiry. Artifact retrieval must bind exact
run/job/attempt and artifact ID, verify archive size/digest, reject unsafe,
duplicate/case-colliding paths and aliases before extraction, then verify the
complete member inventory. Review traces and raw campaign outputs are not source
fixtures and are not committed.

## Corrective evidence architecture and R1 traceability

The historical R1 **NO_GO** on unpublished tree
`5409df5966b9fc1cd631018628321be3c5f7c74b` remains valid. Its report SHA-256 is
`e09d76d213d19f6827daf6ed7d6744c1187c4fad5e67741c8dab0f6fdd142a58`;
the [append-only handoff](https://github.com/fanilosendrison/proto-ring/issues/143#issuecomment-6661957486)
records the findings. No runtime campaign produced that NO_GO.
The corrective base is `22165fef9596c892f0e9e0729f1e67739559d1ac`, incorporating
#144 without changing its code. The preserved canonical entry signatures still
support this experiment; its [guarded core](api-03-ri-core.md) is not a host.

Paths below are relative to the isolated harness. The existing verifier now
separates four responsibilities: frozen role authorization in `launch-policy.rs`;
controller allocation in `campaign-context.rs`; retained observations; and the
one acceptance kernel in `verification::verify`. Launcher expectations are
subordinate correspondence diagnostics. They cannot authorize root, host
networking, another program/argument, an unowned mount or a writable forbidden
mount. Mandatory case obligations do not come from the producer's check list.

The standalone command and campaign completion invoke that same kernel:

```text
api03-host-qualification verify EVIDENCE_ROOT SOURCE_SHA RUN_ID RUN_ATTEMPT SOURCE_GIT_ROOT
```

`SOURCE_GIT_ROOT` is the verifier's authorized local Git object store, not an
arbitrary source selected by the evidence. `source.rs` obtains the exact required
file set, modes, blobs and bytes from that immutable commit through official Git
commands. Inventory roots include the workflow, harness/fixtures, workspace and
isolated manifests/locks, toolchain, API/engine path dependencies (including data)
and accepted API schemas. Required files cannot be omitted from a self-declared
inventory. Executable source directories are also checked for untracked/ignored
additions. No file contains its future SHA or its inventory's hash.

The execution source path retained in controller context and a later verifier's
local Git path may differ. Git source identity is replayed; the subject's actual
root, program mount and resource identities are not silently relocated.
Source checks before/after probes and the invalidation latch are mandatory.
A detected drift or persistence failure never resets to valid, including after
restoration of original bytes. This does not prove absence of unobserved ABA or
resist an administrator rewriting the controller's protected evidence.

H02 retains exact before/after bytes, presence, target, campaign/container and
ordered action/capture bindings, alongside canonical public provenance. H04
retains separately emitted canonical governance, RI and Git-discovered scopes,
their complete path coverage, resolved paths, mode/link identities and actual
mount domain. Neither module resolves selectors or reconstructs RepositoryState.

H06 retains creation/start/live/terminal identities, host and namespace PID
relationships, namespace links, birth observations and the actual owned child's
open descriptor identity. These controller observations must agree with the
atomically published child record. The publisher closes/fsyncs a unique sibling
temporary, uses no-replace hard-link publication, removes the temporary and syncs
the directory. The reader opens a bounded regular file with no-follow/nonblocking
flags and checks attribution; existence alone is not readiness. The only added
direct auxiliary dependency is the already workspace-locked `libc=0.2.190` for
those safe OS flag constants; no unsafe code or product dependency is introduced.
Unavailable unprivileged process observations prevent qualification rather than
triggering elevation or a substitute mechanism.

`tests/support/offline-bundle.rs` and `process-fixture.rs` independently specify
one complete **synthetic test dossier**. Neutral JSON/hash libraries and owned
Git fixture primitives are used; neither launcher, collector nor verifier creates
its expected policy or verdict. Its source exemplar and process facts are not an
executed build or Docker run. The complete dossier is accepted through both the
library entry and the standalone binary. Semantic negative derivatives re-inventory
the modified bytes; integrity-corruption cases deliberately retain a stale hash.
A second individually valid synthetic campaign is also used to test mixed evidence.
There is no production "accept synthetic" switch. These tests qualify acceptance
logic only and must never be published as runtime host evidence.

| Finding | Correction | Independent test | Review disposition |
| ------- | ---------- | ---------------- | ------------------ |
| API03-R1-001 | Independent role policy and controller-owned path/resource bindings, before request/inspection correspondence | `complete-policy-regressions.rs`; matching forbidden user/network/program; arguments and mounts; role exception reuse | Pending R1_CORRECTIVE |
| API03-R1-002 | Mandatory `mutation-evidence.rs` bytes, targets, canonical provenance and temporal chain | `complete-observation-regressions.rs`; omitted/unchanged/wrong bytes, target, subject/campaign and missing canonical piece | Pending R1_CORRECTIVE |
| API03-R1-003 | Git-derived complete source authority and persistent `source-state.rs` invalidation | `complete-source-regressions.rs`; wrong/omitted/blob/duplicate source, missing control and actual drift followed by restoration | Pending R1_CORRECTIVE |
| API03-R1-004 | Retained canonical vectors and complete per-path `scope-evidence.rs` verification | `complete-observation-regressions.rs`; omitted/reduced scope, external Git/alias, duplicate and foreign protection | Pending R1_CORRECTIVE |
| API03-R1-005 | Same-campaign/container creation through live capability and effect closure in `process-evidence.rs` | `complete-observation-regressions.rs`, `cross-campaign-regression.rs`; separately valid campaigns, container, readiness, lifecycle and order substitutions | Pending R1_CORRECTIVE |
| API03-R1-006 | Complete independently specified positive oracle for the full kernel and CLI | `complete-verifier.rs`; positive library/CLI, individually targeted negative derivatives and exact rejection classes | Pending R1_CORRECTIVE |
| API03-R1-007 | Atomic no-replace publication plus bounded attributed readiness reader | `atomic-rendezvous.rs`; partial temporary/final, complete, duplicate, foreign identity and symlink cases without sleep-based races | Pending R1_CORRECTIVE |

The initial new oracle executed against the unchanged rejected verifier on the
reconciled base: both complete positive paths passed, while targeted counterexamples
exposed false acceptance. The missing historical positive oracle was a proof gap,
not an invented failed historical test. Initial construction errors and all RED
outputs remain in the private corrective archive. New-source results do not replace
the original NO_GO or qualify unexecuted runtime mechanisms.

### First corrective review and additional mandatory observations

R1_CORRECTIVE on tree `fd92bd58b2bcfc1756a3920c7a0fb5439ca966e3` returned
**NO_GO** (report SHA-256
`f1ce818cdb7a64352677af969a00982febe8a5a2ee5d244a854023a68e494010`).
It resolved 001/002/003/007, retained 004/005/006 and identified API03-R1-008.
That report and source remain preserved, not replaced by subsequent corrections.
The matrix above describes the initial corrective snapshot; dispositions require
the new exact-source receipt, rather than self-certification by passing tests.

The additional correction requires `subject-root-identity.json` and
`positive-git-identities.json`: complete, attributed filesystem witnesses for the
subject and Git administration (directory, HEAD, index, configuration, object/ref
parents and the controlled branch ref). External alternates/common-directory
linkage is explicitly excluded. Typed metadata includes device, inode, mode and
link count; missing/empty values, contradictory roots, aliases and omitted
administrative records fail acceptance. This is resource coverage, not another
RepositoryState encoding or recursive Git-directory hash.

Standalone H06 verification now uses the same typed readiness requirements as
runtime reading. Namespace links must be present and valid. Descriptor and target
identities must both be complete regular-file witnesses before equality can be
considered; two missing namespace values or two empty objects are not evidence.

API03-R1-008 is addressed by mandatory structured kernel and Docker client/server,
runtime/security/cgroup inventories, with kernel/version correspondence. Every
emitted process inventory field is required: executable/path/version/fingerprint,
allowed environment names, cwd, namespaces, cgroup and mount observations. Removing
producer diagnostics cannot disable these checks. A trusted controller copies only
`/usr/local/bin/python3` from the already owned, pinned, stopped policy container;
its bounded public-image bytes, hash, size and container/image binding independently
check the process-reported interpreter fingerprint. No consumer or credential file
is read, no executable is installed, and no additional container is started for
this copy. The acquisition bound is 4 MiB, distinct from fixture stream limits and
from future RI output retention.

`mandatory-observation-regressions.rs` reproduces the seven source-review
counterexamples before correction through the full verifier. The expanded positive
fixture supplies the missing observations; that file and
`typed-inventory-regressions.rs` check root/Git contradictions, jointly absent
namespace/descriptor data, omitted inventory fields, bad mount/environment data,
interpreter substitution and kernel/runtime mismatch. Semantic derivatives retain
coherent inventories so rejection reaches the intended obligation.

The adapted cycle-02 review on tree
`7f48eb39d811033e85197ff19b9aa38f1c917603` also returned **NO_GO** (report
SHA-256 `0f31b85f189aac8e6085733f8cce247df3beaab02d5cdb8fe28b53bb425acfde`).
It resolved 001/002/003/005/006/007/008 but retained 004: administrative and
canonical-path identity sets were disconnected. The subsequent correction carries
one identity registry through allocation, subject, Git administration and every
canonical scope file, and applies the complete typed regular-file contract to each
scope identity. `cross-inventory-identity.rs` reproduces the cross-registry alias
and zero-inode acceptances before correction, then requires rejection through both
the full kernel and CLI while the complete positive remains accepted. The prior
NO_GO and its exact source are retained; this text does not self-certify a GO.

Cycle-03 review on tree `827a4d1fb41c22b132825d2cdcb505d32ad458b6` resolved
001–008 but returned **NO_GO** for new API03-R1-009 (report SHA-256
`53b8d6de0b75df0f0d6da1c3350bd7ab16f128877c4326a8bf5e786d35b8d7c4`).
An empty replacement object had been treated as unequal to a numeric prior inode.
The correction validates complete before/after identities of the prescribed kind,
requires actual object-identity replacement in the allocated filesystem, and binds
invalidation to the campaign and owned resource. Guard loss must concern the same
subject observed initially. Negative Git/alias fixtures also require their actual
allocated paths, not arbitrary strings that merely look outside the subject.
`invalidation-observation.rs` reproduces the H04 and H08 false acceptances in an
isolated archive of the rejected tree, then checks complete-gate library/CLI
rejection for empty, missing, invalid, unchanged and wrongly attributed witnesses.
The witness worktree and every prior archive/report remain unchanged.

## Checkpoint mapping and downstream handoff

The proposed mechanisms map to API-01 checkpoints: initial whole-subject boundary
before RGS; canonical scope coverage before/after pure realization and admission;
resource/program/mount identity before each launch; runtime effect closure after
each process and before required capture; guard checks through final observation.
The experiment does not integrate these checkpoints into a future CheckHost,
issue permits or execute an actual admitted RI service.

- #145 receives coverage, root/program/mount binding and admission checkpoint
  mechanisms, subject to actual campaign evidence and integration with completed #144.
- #146 receives process/descendant/stream distinctions and demonstrated effect
  closure limits, not a qualified general-purpose capture implementation.
- #147 receives access-separation observations only, not production retention,
  durability or finalized-receipt qualification.
- #148 receives a recipe for an eventual integrated admitted-path exercise after
  its components exist. No integrated API qualification is anticipated here.

## Campaign results and invalidation

The first real attempt is
[run 42097959515](https://github.com/fanilosendrison/proto-ring/actions/runs/42097959515),
attempt 1, job `127492884826`, source
`283e7ee5a096ab313b26714fcf0ee0ac7440b441` / tree
`a9126b09200b3c081e2eb28a1b60eb09859c8cea`, published through
[PR150](https://github.com/fanilosendrison/proto-ring/pull/150). It is **NOT_QUALIFIED**:
H07's byte check failed, and remaining H07/H08 cases were not run. The controller
recorded preceding H01–H06 subcases as PASS; those observations are retained for
independent evidence review, not promoted to a completed host basis here.

Artifact `13449350358` was recovered through the exact run metadata, SHA-256
`5176add901769d625f80b3ab4c55c27bbbf1855ac6bd6334d2ebebca466592f9`.
All 647 ZIP members and both complete inventories were verified; the campaign
inventory covers 625 entries. Runner image `20261101.360.1`, Docker `28.0.4` and
the exact candidate/config digests were observed. Source checks remained valid.
The program's observed exit was 23, but `docker logs` returned UTF-8 replacement
bytes `ef bf bd` instead of the fixture's `ff`/`fe`. The actual container logging
configuration was `json-file`. Thus the exercised capture path did not preserve
arbitrary bytes. This is a measured instrument/transport limitation, not a
universal Docker/Linux failure. Cleanup and absence of owned containers/networks
were observed; no automatic campaign retry occurred. The attempt and its original
conclusion remain preserved.

### Bounded capture correction requiring fresh review and campaign

The revised instrument selects Docker's maintained `local` logging driver **per
owned container**, with `max-size=1m` and `max-file=1`. No daemon configuration,
installation, privilege, image or host change is authorized. Runtime inventory
must show that facility available, and actual inspected `LogConfig` must match.
All announced fixture streams are far below the rotation boundary; equality
checks still reject lost or altered bytes. This is not the 64 MiB RI retention
qualification or a policy for real consumer validators.

The method reference is Docker's local-driver documentation at
`docker/docs@16e452c8eaea26cec842cbf1065037fb2015001f`,
`content/manuals/engine/logging/drivers/local.md`. Its per-container configuration
is used; the daemon's private log files are never read directly. The inspected
implementation reference is `moby/moby@6430e49a55babd9b8f4d08e70ecb2b68900770fe`
(tag `v28.0.4`), `daemon/logger/local/local.go` and
`api/types/plugins/logdriver/entry.proto`: lines are stored as protobuf bytes and
copied without a string conversion. That source evidence supports a new bounded
hypothesis, not runtime success. H07's original stdout/stderr and exit expectations
are unchanged; `binary-logging-policy.rs` rejects the old/replacement-byte path,
missing facility and absent bounds.

A new exact-source adapted R1 GO and separately attributed campaign are required
before relying on this correction. Prior results cannot be transplanted onto it.
R2 must judge the real evidence and frozen report candidate before publication B.
A GO on an insufficient report does not qualify the host.

Final conclusions are restricted to `QUALIFIED_SCOPED_HOST_BASIS`, `PARTIAL`,
`UNAVAILABLE` or `NOT_QUALIFIED`. The positive conclusion requires every mandatory
control, coverage, effect closure, attributed verified evidence and verified
cleanup. It never qualifies all Linux/Docker, administrator resistance, API-03,
ReceiptCustodian, the user's local checkout or absence of ABA from equal hashes.

Any changed probe/policy/image, uncovered Git/path dependency, replaced resource,
uncontrolled external writer, unavailable lifecycle observation or weakened
controller/evidence boundary invalidates reliance until adapted review and an
attributed new campaign establish it. Editorial report-only changes require
verified diffs, not automatic probe reruns. The final report must distinguish
`PROBE_SOURCE_SHA`, `REPORT_SOURCE_SHA` and R2's `REVIEWED_SOURCE_SHA` without
requiring artificial equality.
