---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Canonical Governance Routing extraction evidence"
---

# Canonical Governance Routing extraction evidence

Status: non-normative extraction analysis.

This record preserves the consumer evidence and complete candidate disposition
for Canonical Governance Routing. The canonical shared scope is defined by
[`docs/contracts/canonical-governance-routing.md`](../contracts/canonical-governance-routing.md)
and implemented by `src/proto_ring/governance_routing.py`.

## Evidence baselines

The exact evidence states are:

```text
proto-ring
fanilosendrison/proto-ring
c298343efa07d71790b53493ff15494f2fcac671

Turnlock
fanilosendrison/turnlock-rust
0c2fd68974466df37105682d9aa8d034ed22dad2

Ruu
fanilosendrison/ruu
e27adc9290b4781396cb5c9745014ca5427e98ac
```

The consumer repositories remain authoritative for their own carriers, routing
vocabulary, configuration, routed artifacts, and product or evidence semantics.
This record creates no consumer authority.

## Concrete Turnlock evidence

At the recorded baseline, Turnlock's root `AGENTS.md` frontmatter demonstrates
these exact routes and declared targets:

```text
repository_governance.architecture_decisions.profile_path
→ docs/adr/adr-profile.yaml

repository_governance.shared_governance_provider.binding_path
→ docs/repository-governance/turnlock-rust-shared-governance-provider.md
```

The routed ADR profile owns Turnlock's ADR corpus and representation rules. The
routed Shared Governance Provider binding preserves Turnlock authority while
binding the applicable shared-provider contract. Turnlock's Projection
Integrity binding identifies the corresponding root routing fields as local
canonical owners of repository-operational routing.

## Concrete Ruu evidence

At the recorded baseline, Ruu's root `AGENTS.md` frontmatter independently
demonstrates these exact routes and declared targets:

```text
repository_governance.architecture_decisions.profile_path
→ docs/adr/adr-profile.yaml

repository_governance.shared_governance_provider.binding_path
→ docs/repository-governance/ruu-shared-governance-provider.md
```

The routed ADR profile owns Ruu's ADR corpus and representation rules. The
routed Shared Governance Provider binding preserves Ruu authority, qualification
evidence, manifests, lineage, and snapshots while binding the applicable
shared-provider contract. Ruu's Projection Integrity binding identifies the
corresponding root routing fields as local canonical owners of
repository-operational routing.

## Existing proto-ring dependency

At the pre-extraction proto-ring baseline,
`src/proto_ring/canonical_adr.py` embeds ADR-profile routing. Its current adapter
parses root `AGENTS.md` frontmatter, traverses
`repository_governance.architecture_decisions.profile_path`, and resolves the
configured path through the existing repository-containment primitive.

This is concrete evidence that explicit canonical routing is already a current
dependency. It does not make that carrier, its serialization, or its key
vocabulary generic.

## Resulting boundary

The extracted responsibility is:

```text
consumer supplies:
    one authoritative structured routing mapping
    +
    one exact route through that mapping

proto-ring:
    traverses only that exact route
    resolves its declared repository-relative target
    requires repository containment
    requires target availability
    returns a result explicitly bound to that route
    fails closed on invalid routing
    never discovers an alternative route
    never falls back to convention

consumer retains:
    carrier identity
    carrier serialization/parsing
    routing vocabulary
    route selection
    target semantics
    target authority
```

## Candidate disposition table

| Candidate | Disposition | Result |
| --------- | ----------- | ------ |
| Routing is explicit rather than discovered by convention. | `EXTRACT` | Resolution consumes only an explicit caller-supplied route. |
| One responsibility resolves through one canonical declared route. | `EXTRACT` | The caller supplies one exact route for the responsibility being resolved. |
| The route is non-empty and every segment is a non-empty string. | `EXTRACT` | Empty routes and empty or non-string segments fail closed without normalization. |
| Traversal follows only the exact supplied route. | `EXTRACT` | No sibling, alias, fuzzy, default, discovery, or fallback traversal is permitted. |
| Every intermediate traversed value is a mapping. | `EXTRACT` | A malformed intermediate value fails closed. |
| Missing route segments fail closed. | `EXTRACT` | Every segment must exist exactly in its current mapping. |
| Malformed routing state fails closed. | `EXTRACT` | Invalid input mappings, intermediate values, and leaf values are controlled failures. |
| The final declared target is a non-empty string. | `EXTRACT` | The exact leaf string becomes the declared path. |
| The declared target path is repository-relative. | `EXTRACT` | Absolute paths fail closed. |
| The resolved target remains contained by the repository root. | `EXTRACT` | Existing symlink-aware repository containment is reused. |
| The resolved target exists. | `EXTRACT` | An unavailable target fails during routing resolution. |
| Invalid, unavailable, or missing targets do not trigger another source. | `EXTRACT` | Convention lookup, sibling lookup, first-match selection, and fallback are prohibited. |
| Successful resolution is bound to the exact declared route. | `EXTRACT` | The result retains the route, exact declared path string, and contained target. |
| Routing transfers no target authority to proto-ring. | `EXTRACT` | The consumer retains target semantics and authority. |
| Ambiguous routing fails closed. | `NARROW` | The generic mechanism performs no candidate discovery because the caller supplies one exact route. Missing or malformed exact-route traversal fails closed, while representation-level ambiguity belongs to the consumer-owned carrier parser. |
| Root `AGENTS.md`. | `CONSUMER` | It is the demonstrated carrier, not a universal generic carrier. |
| YAML frontmatter parsing. | `CONSUMER` | Carrier serialization and parsing remain outside the generic contract. |
| `repository_governance`. | `CONSUMER` | Routing vocabulary remains consumer-owned. |
| `architecture_decisions`. | `CONSUMER` | Routing vocabulary remains consumer-owned. |
| `profile_path`. | `CONSUMER` | Routing vocabulary remains consumer-owned. |
| `shared_governance_provider`. | `CONSUMER` | Routing vocabulary remains consumer-owned. |
| `binding_path`. | `CONSUMER` | Routing vocabulary remains consumer-owned. |
| Choice of route for a governance responsibility. | `CONSUMER` | Route selection and responsibility meaning remain consumer-owned. |
| ADR-profile semantics. | `CONSUMER` | Canonical ADR Identity and consumer ADR authority remain unchanged. |
| Shared Governance Provider binding semantics. | `CONSUMER` | Provider selection and immutable binding remain owned by their existing contract and consumer binding. |
| Target file format. | `CONSUMER` | Routing requires existence but does not interpret target content. |
| Target artifact type beyond existence. | `CONSUMER` | The generic resolver imposes no file-versus-directory rule. |
| Target semantic authority. | `CONSUMER` | Resolution does not transfer authority to proto-ring. |

No candidate remains unresolved after this extraction.

## Narrowed representation ambiguity

Canonical Governance Routing performs no candidate discovery at all. The caller
supplies one exact route. The generic contract therefore rejects missing or
malformed exact-route traversal and prohibits fallback or candidate selection.

Representation-level ambiguity inside the consumer carrier, such as YAML
duplicate-key semantics, belongs to the consumer-owned carrier parser because
carrier serialization is outside this contract. YAML duplicate-key policy is
not part of Canonical Governance Routing.

## Composition and intentional strengthening

Canonical ADR Identity retains its existing adapter and ADR-resolution public
API. The adapter continues to parse the current consumer carrier and supply its
consumer-owned route, while Canonical Governance Routing owns only exact path
resolution.

The generic contract requires the routed target to exist. Composition therefore
intentionally strengthens `configured_profile_path()` so a missing configured
ADR profile fails immediately rather than only when later profile loading is
attempted. No other Canonical ADR candidate, corpus, identity, metadata, or
body behavior changes.

Shared Governance Provider continues to govern provider selection and immutable
provider binding. ADR Metadata Primitives continues to own repository
containment. Repository Integrity continues to evaluate consumer-declared
mandatory obligations. Projection Integrity continues to govern canonical
ownership and projections. These responsibilities remain non-overlapping.
