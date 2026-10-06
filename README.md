# proto-ring

`proto-ring` is the bootstrap and factorization repository for generic
software-repository governance mechanisms empirically demonstrated by consumer
repositories such as Turnlock and Ruu.

Its current bootstrap direction is:

```text
observe Turnlock governance
→ identify the maximum consumer-independent extraction candidate
→ confront the candidate with Ruu and other concrete consumers
→ narrow false generalizations
→ strengthen with better consumer-independent guarantees
→ establish the proto-ring contract or mechanism
→ validate it independently
→ adopt it back into consumers without weakening their guarantees
```

Turnlock is the primary extraction source for this bootstrap direction. That
makes it the starting point for maximum justified extraction, not automatic
proto-ring authority or an unquestioned reference implementation. A Turnlock
governance property is an extraction candidate when analysis establishes that it
is consumer-independent repository methodology rather than Turnlock product
semantics, authority, local configuration, history, topology, or another
consumer-local concern.

Ruu is the principal current confrontation consumer. It can falsify a proposed
generalization, expose a missing distinction, demonstrate a stronger
generic guarantee, or contribute another consumer-independent property that
Turnlock does not demonstrate. Ruu's absence of an equivalent mechanism is not
evidence against extracting a justified Turnlock candidate.

`proto-ring` does not invent governance speculatively. It does not copy Turnlock
blindly, and it is not derived as `Turnlock ∩ Ruu`. Common implementation is
evidence for factorization, not a prerequisite. The result is the strongest
justified consumer-independent governance, while consumer authority and
consumer-specific behavior remain with their respective consumers.

The historical extraction analysis lives at:

[docs/bootstrap/turnlock-extraction-audit.md](docs/bootstrap/turnlock-extraction-audit.md)

The baseline- and maturity-bounded residual adjudication lives at:

[docs/bootstrap/turnlock-residual-governance-reaudit.md](docs/bootstrap/turnlock-residual-governance-reaudit.md)

## Shared contracts

- [Shared Governance Provider](docs/contracts/shared-governance-provider.md)
- [Repository Integrity](docs/contracts/repository-integrity.md)
- [Git Whitespace Validation](docs/contracts/git-whitespace-validation.md)
- [ADR metadata primitives](docs/contracts/adr-metadata-primitives.md)
- [Canonical ADR Identity](docs/contracts/canonical-adr-identity.md)
- [Canonical Structured Data](docs/contracts/structured-data.md)
- [Canonical Portable Pattern](docs/contracts/portable-pattern.md)
- [Canonical Governance Bootstrap](docs/contracts/governance-bootstrap.md)
- [Canonical Governance Routing](docs/contracts/canonical-governance-routing.md)
- [Repository Governance Model](docs/contracts/repository-governance-model.md)
- [RepositoryState](docs/contracts/repository-state.md)
- [RepositoryGovernanceState](docs/contracts/repository-governance-state.md)
- [Governance Authority](docs/contracts/governance-authority.md)
- [Governed Objects](docs/contracts/governed-objects.md)
- [Accepted ADR Body Immutability](docs/contracts/accepted-adr-body-immutability.md)
- [Authoritative Ref Monotonicity](docs/contracts/authoritative-ref-monotonicity.md)
- [Projection Integrity](docs/contracts/projection-integrity.md)
- [Normative Terminology](docs/contracts/normative-terminology.md)
- [Exact Evidence Binding](docs/contracts/exact-evidence-binding.md)
- [Evidence Requirements](docs/contracts/evidence-requirements.md)

Git Whitespace Validation factorization evidence is preserved in the
[Git Whitespace Validation extraction record](docs/bootstrap/git-whitespace-validation-extraction.md).
Extraction evidence for the ADR metadata primitive boundary is preserved in the
[ADR metadata primitive extraction inventory](docs/bootstrap/adr-metadata-primitives-extraction.md).
Canonical ADR Identity factorization evidence is preserved in the
[Canonical ADR Identity extraction record](docs/bootstrap/canonical-adr-identity-extraction.md).
Canonical Governance Bootstrap factorization evidence is preserved in the
[Canonical Governance Bootstrap extraction record](docs/bootstrap/governance-bootstrap-extraction.md).
Canonical Governance Routing factorization evidence is preserved in the
[Canonical Governance Routing extraction record](docs/bootstrap/canonical-governance-routing-extraction.md).
Canonical Repository Governance Model factorization evidence is preserved in the
[Repository Governance Model extraction record](docs/bootstrap/repository-governance-model-extraction.md).
Canonical RepositoryGovernanceState factorization evidence is preserved in the
[RepositoryGovernanceState extraction record](docs/bootstrap/repository-governance-state-extraction.md).
Canonical Governance Authority factorization evidence is preserved in the
[Governance Authority extraction record](docs/bootstrap/governance-authority-extraction.md).
Canonical Governed Objects factorization evidence is preserved in the
[Governed Objects extraction record](docs/bootstrap/governed-objects-extraction.md).
Accepted ADR Body Immutability factorization evidence is preserved in the
[Accepted ADR Body Immutability extraction record](docs/bootstrap/accepted-adr-body-immutability-extraction.md).
Authoritative Ref Monotonicity factorization evidence is preserved in the
[Authoritative Ref Monotonicity extraction record](docs/bootstrap/authoritative-ref-monotonicity-extraction.md).
Normative Terminology factorization evidence is preserved in the
[Normative Terminology extraction inventory](docs/bootstrap/normative-terminology-extraction.md).
Exact Evidence Binding factorization evidence is preserved in the
[Exact Evidence Binding extraction record](docs/bootstrap/exact-evidence-binding-extraction.md).

Existing proto-ring contracts and mechanisms retain the authority already
established for their stated scope.

## Shared provider mechanisms

GitHub Authoritative Ref Monotonicity effective-rule live conformance is
provided by `proto_ring.github_authoritative_ref_monotonicity`. It is an
executable provider mechanism composed with existing contracts, not a new
contract.

Its extraction evidence and observability boundary are recorded in the
[GitHub Authoritative Ref Monotonicity live conformance extraction record](docs/bootstrap/github-authoritative-ref-monotonicity-live-conformance-extraction.md).
