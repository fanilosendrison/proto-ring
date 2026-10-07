---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-canonical-adr-identity"
severity: "strict"
name: "Canonical ADR Identity contract"
---

# Canonical ADR Identity contract

## Purpose

Canonical ADR Identity resolves one logical Architecture Decision Record
identity to the unique canonical structured ADR artifact in a consumer-owned
corpus. It does not define a universal ADR profile or transfer authority over a
consumer's decisions to proto-ring.

## Consumer-owned configuration

The consumer owns its ADR profile. Canonical ADR Identity consumes the current
Repository Governance Model and uses Canonical Governance Routing through the
model-owned resolved route.

The current adapter requires:

```text
capability: architecture_decisions
route: profile
```

Repository Governance Model owns the serialized repository-governance carrier,
capability declaration, route lookup, and resulting ResolvedGovernanceRoute.
Canonical ADR Identity does not read a legacy `profile_path` key and does not
reconstruct a route from prose or filesystem convention.

The resolved profile target must satisfy the routing authority already applied
by Repository Governance Model. The consumer retains authority over the profile
and every field it contains.

## Consumed profile fields

The resolver consumes only these fields from the profile:

```text
repository.adr_directory
repository.filename_pattern
repository.id_pattern
repository.id_width
```

All other profile properties remain outside Canonical ADR Identity authority.

`repository.filename_pattern` and `repository.id_pattern` are consumer-owned
patterns interpreted through
[Canonical Portable Pattern](portable-pattern.md). Native Python, Rust, PCRE,
POSIX, or another regex dialect is not semantic authority.

The filename pattern MUST expose one non-empty named capture exactly named
`number`. The ID pattern has no generic capture requirement.

## Logical ADR identity

A requested logical identity must fully match `repository.id_pattern`, must have
the exact form `ADR-<digits>`, and must contain exactly
`repository.id_width` digits. This contract does not generalize beyond ADR
identities.

## Canonical corpus

The resolution corpus is exclusively `repository.adr_directory`. Files outside
that directory are irrelevant to Canonical ADR Identity, including files that
claim the requested identity in structured metadata.

Only direct files in the declared directory are inspected. Nested files do not
participate. Canonical candidates must be direct non-symlink regular-file entries
in the declared ADR directory. A matching symbolic-link entry is invalid and
resolution fails closed without following its target.

## Candidate resolution

Candidate filenames must fully match `repository.filename_pattern`. The pattern
must expose a named group called `number`. A candidate represents the requested
logical identity only when that group equals the requested ADR numeric suffix.

Resolution is fail-closed:

```text
0 candidates  -> fail
1 candidate   -> continue
>1 candidates -> fail ambiguous
```

There is no first-match, sorted-match, or lexicographic fallback.

## Structured record requirement

The unique candidate must remain contained by the repository root and must be
parseable through the exact ADR parsing operation owned by the Canonical shared
ADR metadata primitive contract. Its `metadata.id` must equal the requested
logical ADR identity exactly.

Successful resolution returns the logical identity, contained artifact path,
structured metadata, and the exact decision-body bytes produced by that
contract-owned parsing operation.

## Authority boundary

Canonical ADR Identity answers only:

```text
Which canonical corpus artifact represents this logical ADR ID?
```

It does not answer whether the ADR is accepted, whether historical content is
immutable, whether the whole corpus is valid, whether schemas or overlays are
valid, or whether lifecycle state is correct. Those concerns remain separate
consumer or composed-governance responsibilities.

The consumer continues to own profile contents, corpus location, filename and
identity representation, domain, schemas, overlays, lifecycle, legacy policy,
migration evidence, annotated history, rendering, qualification, validation
membership, and product semantics.

## Projection exclusion

Generated indexes, annotated histories, and other projections never participate
in canonical resolution. In particular, `docs/adr/index.md` and
`docs/adr/README.md` cannot select or redirect the canonical artifact.
