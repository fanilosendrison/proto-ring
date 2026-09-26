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

Existing proto-ring contracts and mechanisms retain the authority already
established for their stated scope.
