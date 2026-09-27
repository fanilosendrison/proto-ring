---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Canonical ADR Identity extraction evidence"
---

# Canonical ADR Identity extraction evidence

Status: non-normative extraction analysis.

This record preserves the consumer evidence and factorization boundary for
Canonical ADR Identity. The canonical shared scope is defined by
[`docs/contracts/canonical-adr-identity.md`](../contracts/canonical-adr-identity.md)
and implemented by `src/proto_ring/canonical_adr.py`.

## Evidence baselines

Turnlock was the primary evidence source at:

```text
fanilosendrison/turnlock-rust
b4d0325b8130b8130cdfb2e92e620adde9408b9d
AGENTS.md
docs/adr/adr-profile.yaml
docs/adr/
scripts/adr-metadata.py
```

Ruu was the principal confrontation consumer at:

```text
fanilosendrison/ruu
8585b704d1a5799ef394f97a33b4e9c67c490fd3
AGENTS.md
docs/adr/adr-profile.yaml
docs/adr/
tools/adr-metadata.py
```

The consumer repositories and their accepted ADRs remain authoritative for
their own decisions. This evidence record creates no Turnlock or Ruu authority.

## Turnlock evidence

Turnlock declares an ADR directory, filename pattern, identity pattern, and
identity width in a consumer-owned profile. Its structured ADR files form the
identity-bearing corpus. The generated index and maintained annotated history
are projections over that corpus rather than selectors of canonical identity.

Turnlock additionally owns lifecycle transitions, schema provenance, a local
overlay, migration evidence, annotated-history grammar, rendering policy, and
repository validation. None of those responsibilities is required to answer
which corpus artifact represents a supplied logical ADR identity.

## Ruu confrontation evidence

Ruu independently declares the same four resolution inputs in its own
consumer-owned profile and treats its structured ADR directory as the canonical
corpus. Its generated index is a projection, not resolution authority.

Ruu falsifies any proposal that would absorb one consumer's complete ADR engine.
It has different heading policy, legacy boundaries, future-date checks,
migration evidence, rendering details, and qualification integration. Those
differences remain local while the identity-to-corpus-artifact relation remains
consumer-independent.

## Common mechanism

The demonstrated generic relation is:

```text
generic:
logical ADR ID + consumer-owned ADR profile/corpus
-> unique canonical structured ADR artifact
```

The mechanism reads the consumer's configured profile route, consumes only the
profile's corpus and identity representation fields, validates the requested ADR
identity, requires exactly one direct corpus candidate, parses that structured
candidate, and verifies its metadata identity.

Zero candidates and multiple candidates fail closed. A file outside the
declared corpus cannot participate merely by claiming the same identity.

## Differences remaining consumer-local

The following responsibilities remain consumer-local:

```text
consumer-local:
lifecycle
legacy policy
schema ownership
overlays
migration evidence
annotated history
rendering
qualification
product semantics
```

Profile contents, corpus paths, filename and ID patterns, ID width, domain,
validation membership, and accepted decision history also remain
consumer-owned.

## Rejected false generalizations

The extraction explicitly rejects:

- a universal ADR profile;
- a universal ADR engine;
- inference of accepted status from identity resolution;
- corpus-wide schema, relation, lifecycle, or migration validation;
- index or annotated-history authority over identity;
- historical accepted-ADR byte sealing;
- universal decision identity;
- universal governed identity; and
- universal serialization.

These rejected generalizations are not required by the common mechanism and
would transfer consumer-owned semantics or evidence into proto-ring.

## Extraction boundary

Canonical ADR Identity answers only which unique structured artifact in the
consumer-declared corpus represents a requested logical ADR identity. Shared
Governance Provider composes accepted-status, decision-body hash, and contract
identity checks after resolution.

This extraction is deliberately narrower than general governed identity. It
does not resolve `fanilosendrison/turnlock-rust#35` or
`fanilosendrison/ruu#49`. Accepted Decision Immutability also remains outside
this extraction and unresolved.
