---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "GitHub Authoritative Ref Monotonicity live conformance extraction evidence"
---

# GitHub Authoritative Ref Monotonicity live conformance extraction evidence

Status: non-normative extraction analysis.

This record preserves the consumer evidence, composition boundary, provider
mapping, and residual observability boundary for GitHub Authoritative Ref
Monotonicity effective-rule live conformance. It creates no governance contract
and changes no existing contract.

## Consumer evidence baselines

The Turnlock preparation baseline is:

```text
fanilosendrison/turnlock-rust
918654957a0ab3eee9886c9223b90bab7e529645
docs/repository-governance/turnlock-rust-authoritative-ref-monotonicity.md
```

The Ruu preparation baseline is:

```text
fanilosendrison/ruu
954f85c551b308f8ead07379da24806bb5664f69
docs/repository-governance/ruu-authoritative-ref-monotonicity.md
```

Both consumers demonstrate the same binding shape:

```text
provider = github
repository owner and name
authoritative_ref
mechanism = github-repository-ruleset
positive integer ruleset_id
```

This evidence does not derive the mechanism as a Turnlock and Ruu intersection.
Each consumer independently demonstrates the same missing reusable GitHub
realization check while retaining ownership of its repository coordinates,
authoritative ref, and provider-specific protection identity.

## Observed gap

The repository-local bindings are mechanically validated. Current Repository
Integrity does not, however, observe whether the external effective rules
identified by those bindings remain active at the validation occurrence.

No semantic contract is missing:

```text
Authoritative Ref Monotonicity
    -> owns required authoritative-ref protection semantics

Projection Integrity
    -> owns direct external-source currentness
    -> owns fail-closed currentness synchronization

Repository Integrity
    -> owns mandatory SATISFIED / VIOLATED / UNDETERMINED evaluation
```

The missing reusable artifact is a GitHub-specific executable mapping from the
consumer binding to a live effective-branch-rules observation.

## Public observation mechanism

GitHub's effective branch rules endpoint supports unauthenticated observation
for public repositories. This first mechanism version deliberately uses no
GitHub credential. It performs only public GET requests and has no credential
fallback.

This boundary prevents Repository Integrity from acquiring GitHub write or
administration capability merely to evaluate external currentness. Public
unauthenticated rate limiting, refusal, or provider unavailability produces
`UNDETERMINED` and therefore a non-passing Repository Integrity occurrence.
That fail-closed availability tradeoff is intentional.

The observation can establish that both required effective rule types are
currently active on the exact authoritative branch and are supplied by the
exact bound ruleset identity:

```text
deletion
non_fast_forward
```

Additional effective rules do not weaken those minimum guarantees.

## Occurrence-scoped currentness

Every result is a live observation for one validation occurrence. A previous
`SATISFIED` observation cannot be reused as proof that the provider realization
remains current during a later validation occurrence.

The mechanism therefore creates no durable cache, committed observation,
timestamp projection, or currentness registry. Repository Integrity reexecutes
the observation for every occurrence.

## Bypass and control-plane boundary

The effective branch rules observation does not establish live
`bypass_actors` currentness or complete provider control-plane currentness. The
fact that initial ruleset creation established an empty bypass list is distinct
from the effective-rule currentness observed here.

No privileged credential is introduced to expand this scope. The mechanism
proves only that the bound ruleset's required rule types are currently effective
on the exact branch. Live bypass and complete control-plane currentness remain
explicit future observability work outside this checker; that residual is not
an Authoritative Ref Monotonicity violation established by this mechanism.

## Ruu product boundary

This repository-governance mechanism observes only the external realization of
a consumer repository's own Authoritative Ref Monotonicity binding. It does not
implement or qualify Ruu product behavior for repositories managed by Ruu.

In particular, it does not duplicate Ruu Issue #29 responsibility for product
provider-adapter authorization observations, execution identity, provider
currentness, protection drift, API outage, webhook ordering, provider
rejection, or managed target interactions. It also creates no qualification
evidence for that Issue.
