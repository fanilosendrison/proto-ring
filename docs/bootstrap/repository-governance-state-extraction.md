---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-record"
domain: "proto-ring-repository-governance-state"
severity: "strict"
name: "RepositoryGovernanceState extraction record"
---

# RepositoryGovernanceState extraction record

## Status

This record preserves the concrete factorization evidence for the canonical
`RepositoryGovernanceState` contract. It is not consumer product authority and
does not make either observed consumer a reference implementation.

## Observed duplicate composition

Turnlock-Rust currently performs generic structured-governance composition in:

```text
scripts/check-structured-governance.py
```

Ruu independently performs the same generic composition in:

```text
tools/check-structured-governance.py
```

Both wrappers load the same sequence:

```text
Repository Governance Model
→ Governance Authority
→ Governed Objects
→ Governance Binding Registry
→ binding/capability composition validation
→ Consumer Repository Integrity
→ Projection Registry
→ Evidence Requirements
```

Both wrappers obtain every component through a Repository Governance Model route
and return the loaded values without running validators or generators. The
consumer-local implementations differ only in their repository-owned routed
content and authority.

This duplicated orchestration demonstrates one consumer-independent composition
responsibility. It does not transfer the consumers' governed values or semantic
authority to proto-ring.

## Extracted responsibility

The maximum justified generic extraction is:

```text
compose already-owned governance values
according to Repository Governance Model version 2 declarations
bind the complete returned state to one stable exact repository observation
fail closed without returning partial coherent state
remain read-only and non-executing
```

The extracted contract makes optional component presence depend only on the
corresponding Repository Governance Model capability. It preserves existing
component loaders and the existing binding/capability composition validator as
the owners of their rules.

## Consumer confrontation

Turnlock-Rust and Ruu both declare the mandatory Governance Authority and Shared
Governance Provider capabilities and both currently declare Governed Objects,
Repository Integrity, Projection Integrity, and Evidence Requirements. Their
independent wrappers therefore exercise the same generic dependency direction.

The common sequence does not justify extracting either consumer's:

- product or specification semantics;
- accepted decision meaning;
- object taxonomy;
- formal-assurance model;
- validation membership;
- qualification policy;
- evidence interpretation;
- provider topology; or
- repository layout.

Those values remain consumer-owned inputs to generic composition.

Turnlock-Rust is the primary extraction source under the proto-ring
factorization doctrine. Its wrapper is not normative for implementation shape.
Ruu independently corroborates that the composition concern is not intrinsically
Turnlock-shaped, while retaining its distinct qualification, replay, evidence,
and External Control Plane boundaries.

## Exact-observation strengthening

A tuple of successful loads alone does not prove that the values came from one
repository observation. Both wrappers expose the need for a stronger generic
coherence guarantee when their generic sequence is centralized.

The resulting contract therefore requires one stable exact repository
observation and one stable observation scope to bind every returned component.
The scope includes the bootstrap carrier and routed governance targets actually
used by construction, without becoming a recursive snapshot of consumer
semantics.

This strengthening is consumer-independent: a torn governance view can falsely
present incompatible declarations as coherent in any repository. The contract
leaves the implementation mechanism and exact number of observation passes
unspecified.

## Retained model-only capabilities

`architecture_decisions` remains available through Repository Governance Model
configuration and routes. Neither consumer wrapper's generic sequence justifies
eager ADR enumeration.

`authoritative_ref_monotonicity` remains available through Repository Governance
Model configuration and routes. Generic state construction does not justify
provider observation, ruleset inspection, GitHub API access, or network access.

## Authority result

The canonical contract owns generic composition and exact-observation coherence.
Each component contract continues to own its schema and semantics. Turnlock-Rust
and Ruu continue to own their routed values, product semantics, formal or
qualification claims, evidence, policy, validation membership, and provider
facts.
