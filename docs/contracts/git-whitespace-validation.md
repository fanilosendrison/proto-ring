---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-git-whitespace-validation"
severity: "strict"
name: "Canonical Git Whitespace Validation contract"
---

# Canonical Git Whitespace Validation contract

## Purpose

Canonical Git Whitespace Validation determines whether required Git whitespace
domains are clean.

The caller supplies a repository working directory and an explicit environment
mapping. The mechanism is read-only. It does not repair files, stage files,
commit, mutate refs, select CI membership, define consumer validation ordering,
define branch policy, or define product semantics.

## Git whitespace authority

Proto-ring does not define an independent whitespace grammar. Whitespace
acceptance and rejection are delegated to Git `diff --check` semantics.

A conforming implementation may invoke Git directly or use an observationally
equivalent Git operation over the same domain.

## Mandatory local domains

Every invocation evaluates both local domains regardless of the GitHub event
name:

```text
unstaged local working-tree domain
→ equivalent to git diff --check

staged/index domain
→ equivalent to git diff --cached --check
```

## Explicit environment authority

The supplied environment mapping is the complete authority for event selection.
Only these keys are interpreted:

```text
GITHUB_EVENT_NAME
GITHUB_EVENT_PATH
```

An implementation must not consult the ambient process environment for an
absent key. This is an operation-input boundary.

## Events without a committed range

When `GITHUB_EVENT_NAME` is absent or is anything other than exactly `push` or
`pull_request`, no committed-range whitespace check is required. The unstaged
and staged checks remain required.

This includes `workflow_dispatch`, `schedule`, and an absent or empty event
name. Such events do not require `GITHUB_EVENT_PATH` for this mechanism.

## Push range

For the exact `push` event, a readable event payload is required. Its top-level
object must contain non-empty string fields `before` and `after`.

When `before` is not:

```text
0000000000000000000000000000000000000000
```

the committed Git domain is exactly:

```text
before..after
```

Git `diff --check` semantics apply to that two-dot range.

## Root push

When `before` is exactly forty zero characters, the prior state is the
repository's Git empty tree. The required committed comparison is the Git empty
tree to `after` under Git `diff --check` semantics.

The implementation must obtain or otherwise use the correct empty-tree identity
for the repository's Git object format. No hard-coded SHA-1 empty-tree hash is
normative. Invoking `git hash-object -t tree --stdin` is one valid realization,
not the semantic definition.

## Pull-request range

For the exact `pull_request` event, a readable event payload is required. Its
top-level object must contain non-empty string fields at:

```text
pull_request.base.sha
pull_request.head.sha
```

The committed Git domain is exactly:

```text
base_sha...head_sha
```

Git `diff --check` semantics apply to that triple-dot range. This is distinct
from two-dot semantics.

## Event payload failure boundary

For `push` and `pull_request`, all of these conditions are required:

- `GITHUB_EVENT_PATH` exists in the supplied environment;
- the referenced file is readable;
- exact UTF-8 decoding succeeds;
- JSON parsing succeeds;
- the top-level JSON value is an object; and
- required fields exist with the required string shape.

If a required range fact cannot be determined, the result is `NON_PASS`.

This contract does not prescribe Python exception classes, `json.loads`, Python
mapping types, or exact parse-error text. It does not create a general canonical
JSON model.

## Aggregate result

The standalone language-neutral results are exactly:

```text
PASS
NON_PASS
```

`PASS` requires every required domain to be successfully determined and clean.
It therefore requires clean unstaged and staged domains and, when applicable, a
successfully determined and clean committed GitHub range.

Any required whitespace failure, Git operation failure, process-start failure,
missing event payload, unreadable event payload, invalid UTF-8, malformed JSON,
non-object event payload, missing required range field, or unusable committed
comparison produces `NON_PASS`.

`SATISFIED`, `VIOLATED`, and `UNDETERMINED` are not standalone statuses of this
contract. Repository Integrity may separately map a process outcome according
to its own contract and binding.

## Diagnostic boundary

Controlled diagnostics must identify at least the failed domain:

- unstaged local check;
- staged local check;
- GitHub push committed-range check or range determination; or
- GitHub pull-request committed-range check or range determination.

Exact diagnostic text, Git stderr bytes, line formatting, diagnostic count, and
diagnostic order are non-normative. Multiple truthful diagnostics may be
returned.

## Process-start failure

When a required Git operation cannot be started, the overall result is
`NON_PASS` and the failure remains controlled. A process-creation exception is
not a language-neutral result.

This applies to local and staged Git observations and to root-push empty-tree
discovery.

## Non-goals

This contract does not define or perform:

- an independent whitespace parser;
- automatic repair;
- staging;
- commit creation;
- repository mutation;
- ref protection;
- CI workflow membership;
- consumer validation membership or order;
- universal GitHub event semantics;
- a #28 public API; or
- Rust implementation architecture.
