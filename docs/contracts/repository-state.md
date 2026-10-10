---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-repository-state"
severity: "strict"
name: "Canonical RepositoryState contract"
---

# Canonical RepositoryState contract

## Purpose

Canonical `RepositoryState` defines one shared, read-only exact local
repository-observation responsibility reusable by Repository Integrity and
`RepositoryGovernanceState`.

It owns:

```text
exact Git worktree-root establishment
explicit observation-scope semantics
local Git and filesystem observation dimensions
content-sensitive exact-state distinction
capture coherence / fail-closed torn-read handling
opaque observed-state identity semantics
```

It does not own Repository Integrity validation membership, obligation
execution, integrity verdicts, governance-component composition, provider
observation, qualification, evidence truth, or mutation.

## Exact repository root

Observation starts from one explicit repository path.

The path MUST resolve to the exact Git worktree root being observed. A plain
non-Git directory or a worktree subdirectory is not an admissible substitute
for that root.

Ambient Git redirection or configuration MUST NOT silently redirect observation
to another worktree, index, object database, namespace, or repository identity.

RepositoryState performs local observation only. It creates no remote-provider
or forge identity and performs no network access.

## Observation scope

RepositoryState observes the Git-visible worktree state plus an optional
explicit observation scope supplied by its caller.

The Git-visible path set includes:

- every tracked worktree path; and
- every untracked worktree path not excluded by repository-owned ignore policy.

Tracked paths remain governed even when an ignore rule would otherwise match
them.

User-global Git ignore configuration and `.git/info/exclude` MUST NOT silently
remove a path from the generic governed state. Repository-owned tracked
`.gitignore` policy may exclude untracked environment artifacts.

An explicit scope entry identifies one additional repository-contained
filesystem path identity that the caller requires to be observed even when
normal Git-visible discovery would omit it. It may identify an existing regular
file, symbolic link, directory, or an absent path.

Explicit observation-scope entries are path identities, not Canonical Governance
Routing declarations. RepositoryState does not parse, normalize, simplify, or
reinterpret routing syntax. A caller composing routing is responsible for
supplying the in-repository binding and resolved-target path observations
required by its own contract.

An explicit scope entry MUST NOT cause RepositoryState to observe a filesystem
object outside the exact worktree root.

Observation-scope order and duplicate spelling of the same exact path identity
have no generic semantic meaning. The returned explicit scope is therefore a
canonical duplicate-free set projection.

## Exact path identity

Governed path identity is exact within the host filesystem domain.

RepositoryState MUST NOT create path equivalence through trimming, case folding,
Unicode normalization, lossy decoding, prefix inference, or alias inference.

A language/runtime-specific path object or string representation is not itself
normative. Implementations must preserve the exact filesystem path distinctions
required by the host and Git observation.

## Required local state dimensions

The exact observation distinguishes at least:

1. symbolic HEAD identity versus an explicit detached-HEAD state;
2. HEAD object identity versus an explicit unborn-HEAD state;
3. exact Git index stage entries sufficient to distinguish mode, object
   identity, stage, and path;
4. governed path membership;
5. for each governed path:
   - exact relative path identity;
   - presence or absence;
   - filesystem object kind;
   - relevant filesystem permission/mode metadata;
   - exact regular-file bytes when the path is a regular file; or
   - exact symbolic-link target when the path is a symbolic link.

The `.git` implementation directory is not recursively hashed as worktree
content. Semantic HEAD and index state are observed explicitly instead.

For an explicitly observed directory, RepositoryState observes the directory
identity, kind, and relevant mode metadata. Explicit directory observation does
not recursively make ignored descendants part of the governed state.

A governed filesystem object whose exact state cannot be represented under this
contract MUST fail capture closed rather than be silently ignored or coerced
into another object kind.

## Content-sensitive distinction

RepositoryState must be sensitive to contract-relevant state changes even when
coarser Git status classification does not change.

In particular, mutating the bytes or relevant metadata of an already-dirty or
already-untracked governed path is still a state change.

Pre-existing dirtiness is not itself an invalid state. RepositoryState observes
state; Repository Integrity separately decides whether obligations are
satisfied for that state.

## Capture coherence

One returned RepositoryState describes one globally coexisting exact local
observation under an explicit mutation-isolation interval. Throughout that
interval, no uncoordinated actor may mutate the contract-relevant Git or
worktree state covered by the observation.

RepositoryState is a portable read-only observer. It neither establishes nor
proves exclusion from arbitrary uncoordinated writers, and a successful capture
MUST NOT be treated as disproving arbitrary mutation/restore or ABA scheduling.

The mutation-isolation precondition does not permit weak observation. Capture
MUST fail closed when required Git state cannot be established, a governed path
cannot be read deterministically, an unsupported governed object prevents exact
observation, or contract-relevant structural, path-membership, or content drift
is observed during capture.

When regular-file bytes or a symbolic-link target are read, concurrent mutation
that prevents those bytes or that target from belonging to the same observed
filesystem object state MUST be detected and fail closed. A content-only change
after an earlier governed-path observation that remains changed through a later
whole-capture validation MUST prevent successful return of the stale
observation, even when HEAD, the index, and governed path membership are
unchanged. Local per-path guards alone are insufficient to establish
whole-capture coherence.

An implementation may use multiple internal reads or structural checks. This
contract does not prescribe an exact number of passes or a particular command
sequence.

## Opaque exact identity

A successful observation exposes one opaque exact-state identity together with
its explicit observation scope. Identity equality means that successful
captures encode equal contract-relevant observed state; it does not establish
that a mutation-isolation interval existed, that no concurrent writer existed,
or that arbitrary ABA scheduling was absent.

The identity representation and digest algorithm are implementation-defined
unless a separate immutable interoperability authority explicitly requires a
particular representation.

The contract requires the identity mechanism to preserve these observable
properties:

```text
same exact governed state under the same effective observed path set
→ equal identity

contract-relevant governed-state difference
→ observably different state identity

capture whose exact state cannot be established
→ no successful identity
```

A change only in redundant scope metadata need not change the opaque identity
when the effective observed path set and all observed state are unchanged. The
explicit observation scope remains a separate returned dimension.

Historical Python digest vectors or platform-specific mode integers do not
become universal cross-language identity representations merely because an
implementation preserved them during an internal refactor.

Where an explicit compatibility obligation requires exact historical identity
bytes or text, that obligation must identify its immutable provenance and scope
separately from this generic contract.

## Read-only boundary

RepositoryState is observational.

It MUST NOT modify the worktree, index, HEAD, ignore policy, Git configuration,
governance artifacts, or observed filesystem objects.

## Composition boundaries

Repository Integrity uses RepositoryState to bind validation results to an exact
repository observation and to enforce state-preserving evaluation. Repository
Integrity remains the owner of `SATISFIED / VIOLATED / UNDETERMINED` and
`PASS / NON_PASS` semantics.

Canonical RepositoryGovernanceState uses RepositoryState to bind composed
governance declarations to one coherent exact observation and observation
scope. RepositoryGovernanceState remains the owner of component composition and
scope-coherence requirements.

Neither consumer may redefine RepositoryState observation semantics by creating
a competing exact-state primitive.
