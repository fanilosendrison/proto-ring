---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-canonical-governance-routing"
severity: "strict"
name: "Canonical Governance Routing contract"
---

# Canonical Governance Routing contract

## Purpose

Canonical Governance Routing resolves one consumer-declared repository target
through one exact route in a consumer-supplied structured routing mapping.

The contract owns exact-route traversal, repository containment, target
availability, and the explicit binding of the route to the resolved target. It
does not own the carrier, routing vocabulary, or routed artifact.

## Inputs

Resolution consumes exactly:

- a repository root;
- one already-parsed structured routing mapping supplied by the consumer; and
- one exact route supplied by the consumer.

The route must be non-empty. Every route segment must be a non-empty string.
Route segments are consumed exactly as supplied without trimming or
normalization.

## Exact-route traversal

Traversal begins at the supplied routing mapping and consumes each route segment
in order.

Every value traversed before the leaf must be a mapping. Every segment must
exist exactly in its current mapping. The leaf must exist exactly and must be a
non-empty string containing the declared repository-relative target path.

Missing segments, non-mapping intermediate values, a missing leaf, and a leaf
that is not a non-empty string fail closed.

Resolution follows only the supplied route. It performs no sibling search,
fuzzy matching, default-key lookup, alias lookup, candidate selection, or
fallback. An invalid or unavailable target does not trigger discovery by
convention or selection of another source.

## Repository target

The declared target path must be non-empty and repository-relative. The exact
declared string is retained as part of the resolved binding.

Target availability MUST be established by resolving the declared path itself,
anchored at the supplied repository root. Resolution MUST preserve the path
semantics that apply to the declaration while traversing filesystem components
and symbolic links.

An implementation MUST NOT lexically or representationally simplify away a
resolution failure merely because a simplified spelling would identify an
existing target. In particular, resolution MUST NOT erase an unavailable
intermediate component, traversal through a non-directory component, a broken or
cyclic symbolic link, or a platform-significant path suffix before availability
has been established.

`.` and `..` components are not forbidden generically. They are admissible only
when the declared path can actually be resolved with their applicable filesystem
semantics. Symbolic links are resolved as part of that traversal; an
implementation MUST NOT lexically cancel `symlink/..` before resolving the
symbolic link.

After faithful declared-path resolution succeeds, the final resolved target MUST
remain contained by the resolved repository root. This contract constrains final
resolved-target containment. It does not require every intermediate component
used during resolution to remain inside the repository root.

The contained final target must exist. The contract does not require the target
to be a file or a directory; target kind and target semantics remain
consumer-owned.

Failure to establish declared-path resolution or target availability MUST fail
closed through the controlled routing error boundary. The routing operation does
not read or validate target content, does not reserve the target against later
mutation, and does not create RepositoryGovernanceState observation semantics.

## Resolved binding

Successful resolution returns a result that explicitly retains:

- the exact route used;
- the exact declared path string; and
- the resolved repository-contained target.

The result binds only the supplied route and declared path to that target. It
does not grant proto-ring semantic authority over the target.

## Consumer authority boundary

The consumer retains authority over:

- carrier identity;
- carrier parsing and serialization;
- routing vocabulary;
- route selection and the responsibility assigned to that route;
- repository-specific configuration;
- target format and artifact type;
- target semantics; and
- target authority.

Representation-level ambiguity in the consumer carrier, including duplicate-key
handling in a serialization format, belongs to the consumer-owned carrier
parser. Canonical Governance Routing receives an already-parsed mapping and
performs no candidate discovery.

## Explicit exclusions

Canonical Governance Routing does not define:

- a universal repository configuration format;
- a universal carrier or carrier parser;
- universal routing keys or vocabulary;
- discovery by filesystem or naming convention;
- fallback or precedence among candidate routes;
- target content validation;
- target artifact semantics; or
- target semantic authority.
