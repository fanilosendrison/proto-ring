---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Git Whitespace Validation extraction evidence"
---

# Git Whitespace Validation extraction evidence

Status: non-normative extraction analysis.

## Exact evidence

Turnlock evidence:

```text
repository: fanilosendrison/turnlock-rust
revision: 7534a4667ec9190aec4021f90bfae2fa8a046dae
historical surfaces:
- scripts/check-git-whitespace.py
- scripts/tests/test-git-whitespace.py
```

The Turnlock extraction audit already classified both surfaces as `PROTO-RING`.
It records generic behavior for local unstaged and staged changes, GitHub push
and pull-request ranges, root pushes, missing or malformed GitHub event payloads,
and fail-closed behavior.

Proto-ring extraction and confrontation state:

```text
repository: fanilosendrison/proto-ring
revision: 7ccde07f4d9801383d6a2410e3d12ce6ee716f34
surfaces:
- src/proto_ring/git_whitespace.py
- tests/test_git_whitespace.py
```

These implementations and tests are evidence. They are not normative authority.
No Ruu-specific evidence is asserted. The absence of a Ruu-local equivalent is
not a rejection reason under the current factorization doctrine.

## Candidate dispositions

| Candidate | Disposition |
| --- | --- |
| local unstaged Git `diff --check` | `EXTRACT` |
| local staged Git `diff --cached --check` | `EXTRACT` |
| GitHub push `before..after` range | `EXTRACT` |
| GitHub root push from the empty tree | `EXTRACT` |
| GitHub pull-request `base...head` range | `EXTRACT` |
| fail closed when required event or range state is unavailable | `EXTRACT` |
| explicit supplied environment as event-selection input | `EXTRACT` |
| Git whitespace grammar | `EXTERNAL` / Git-owned |
| exact diagnostic strings | `IMPLEMENTATION` / `NON-NORMATIVE` |
| exact subprocess implementation or order | `IMPLEMENTATION` / `NON-NORMATIVE` |
| consumer CI membership or order | `CONSUMER` |
| product semantics | `CONSUMER` |

## Resulting boundary

The maximum justified consumer-independent responsibility is read-only Git
whitespace validation over mandatory local unstaged and staged domains plus the
applicable GitHub committed range. Git retains ownership of whitespace parsing.
Consumers retain CI membership, validation ordering, branch policy, and product
semantics.
