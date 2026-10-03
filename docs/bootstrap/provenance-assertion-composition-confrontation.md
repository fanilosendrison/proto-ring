---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Mechanically significant embedded assertion confrontation"
---

# Mechanically significant embedded assertion confrontation

Status: non-normative extraction/confrontation analysis.

The canonical normative results remain the existing proto-ring contracts:
Governance Authority, Projection Integrity, Evidence Requirements,
Exact Evidence Binding, and Repository Integrity.

## Evidence baseline

```text
proto-ring:
890ed560e61e205067bdf3628e419302613ef06e

Ruu confrontation:
4f8ffb12520cdbe40e010ed933f035830f8aab47

consumer finding:
fanilosendrison/ruu#55
```

The concrete occurrences used for confrontation are:

- the ADR-081 heading in
  `docs/design/decision-integration-log.md`;
- the `Date:` field in
  `qualification/state-space/post-baseline/v046/state-space-audit-v46.md`;
- the `Date:` field in
  `qualification/audits/hostile/hostile-audit-adr081.md`; and
- the `Date:` field in
  `qualification/package-verification/package-verification-adr081.md`.

This analysis does not adjudicate any of those occurrences.

## Finding

```text
carrier structural validity
!=
embedded assertion semantic validity
```

Byte authentication, artifact registration, replay, manifest membership,
lineage, or Repository Integrity of a carrier can establish structural
properties of that carrier. None of those properties alone establishes the
semantic truth, authority, or currentness of an assertion embedded in it.

## Generic shape A — known consumer authority

```text
consumer-owned canonical fact
        ↓
secondary representation containing duplicated assertion
        ↓
Projection Integrity / Projection Registry
        ↓
consumer-owned ValidationId / Repository Integrity
```

Projection Integrity applies only when consumer-owned authority already exists.
The canonical source must be declared `authority` for the governed
responsibility, and the carrying source must be declared
`secondary_representation`. The projection must remain a direct-source
relationship. The consumer continues to own the validation and currentness
obligation represented by its `ValidationId` and Repository Integrity profile.
Hash or replay validity is irrelevant to the truth of the duplicated field.

The Ruu Decision Integration Log supplies conditional confrontation evidence
for this shape:

```text
IF Ruu determines that the heading date represents canonical ADR date metadata,
THEN it is an ordinary Projection Integrity case.
```

This analysis does not state that Ruu has made that determination.

## Generic shape B — required binding with unavailable current identity

```text
consumer-owned Evidence Requirement
        ↓
subject/context source resolution
        ↓
Exact Evidence Binding
        ↓
MATCH | MISMATCH | UNDETERMINED
        ↓
consumer-owned higher-level consequence
```

The registry requirement declares that its consumer-owned responsibility or
object requires evidence. Proto-ring does not need a new generic `mandatory`
boolean. Whether and how the binding result gates qualification, a claim, or
another consumer verdict remains consumer-owned.

A known equal required identity produces `MATCH`. A known unequal identity
produces `MISMATCH`. An unavailable mandatory current identity produces
`UNDETERMINED`. An absent authority declaration remains Governance Authority
`UNKNOWN_RESPONSIBILITY` or `UNDECLARED`; authority must not be synthesized to
complete the comparison.

No Git author time, Git committer time, filesystem time, artifact creation time,
or other wall clock is promoted to semantic authority.

The three Ruu qualification and audit `Date:` fields demonstrate that a
structurally valid report can carry an assertion for which the consumer has not
yet declared a canonical semantic source. This analysis does not define what
those fields mean.

## Composition decision

No new generic proto-ring primitive is justified.

The existing contracts are sufficient when composed according to their existing
responsibility boundaries.

```text
authority existence / role
-> Canonical Governance Authority

known canonical fact -> secondary assertion
-> Projection Integrity / Projection Registry

persistent declaration that evidence is required
-> Evidence Requirements

known / mismatched / unavailable exact required identity
-> Exact Evidence Binding

mandatory current-state enforcement
-> Repository Integrity

consumer qualification / claim truth
-> consumer-owned
```

## Rejected alternatives

The confrontation rejects:

- `ProvenanceRegistry`;
- `TimestampRegistry`;
- a universal `ProvenanceAssertion` type;
- universal Date semantics;
- Git timestamps as semantic authority;
- artifact hash as assertion truth;
- replay success as assertion truth;
- broadening Exact Evidence Binding into claim or evidence truth evaluation;
- a new Repository Governance Model capability; and
- a new registry model version.

## Ruu handoff boundary

proto-ring#41 does not correct Ruu.

ruu#55 must still determine, independently for each of the four occurrences:

- exact semantic claim;
- consumer-owned authority, if one exists;
- binding;
- `ESTABLISHED`, `MISMATCH`, `UNKNOWN`, or an equivalent consumer result; and
- authorized correction mechanism.
