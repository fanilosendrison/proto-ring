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

The consumer owns its ADR profile. The operational route to that profile is
stored in the root `AGENTS.md` frontmatter:

```yaml
repository_governance:
  architecture_decisions:
    profile_path: "<relative path>"
```

The configured path must be non-empty, repository-relative, and contained by
the repository root after resolution.

## Consumed profile fields

The resolver consumes only these fields from the profile:

```text
repository.adr_directory
repository.filename_pattern
repository.id_pattern
repository.id_width
```

All other profile properties remain outside Canonical ADR Identity authority.

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
parseable through `proto_ring.adr_metadata.parse_adr`. Its `metadata.id` must
equal the requested logical ADR identity exactly.

Successful resolution returns the logical identity, contained artifact path,
structured metadata, and exact decision-body bytes returned by `parse_adr`.

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
