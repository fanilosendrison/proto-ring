---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-evidence-requirements"
severity: "strict"
name: "Canonical Evidence Requirements contract"
---

# Canonical Evidence Requirements contract

## Purpose

Evidence Requirements persistently declares which consumer-governed
responsibility or object requires evidence, which exact evidence classes are
admitted, and which consumer sources supply current subject identity, optional
current context identity, requirement membership, admitted classes, and
candidate evidence bindings.

This contract is the higher-level declaration boundary over Exact Evidence
Binding. It does not determine evidence success, proof validity, claim truth,
sufficiency, candidate ranking, aggregation, review readiness, qualification,
retention, replay, hashing, canonicalization, or a universal evidence format.

## Persistent registry

Model version 1 uses canonical structured frontmatter:

```yaml
evidence_requirements:
  model_version: 1
  authority:
    responsibility: <GovernedResponsibilityId>
    source: <GovernedSourceId>
  requirements:
    <EvidenceRequirementId>:
      responsibility: <GovernedResponsibilityId>
      instances:
        kind: single
      evidence_classes:
        kind: explicit
        classes:
          - <exact-consumer-class>
      subject:
        source: <GovernedSourceId>
      context:
        required: false
      candidates:
        source: <GovernedSourceId>
```

The registry carrier is the routed authoritative source for the declared
Evidence Requirement Registry responsibility. Governance Authority owns the
source and responsibility identities and their authority relation.

An `EvidenceRequirementId` is an opaque, non-empty, stable, consumer-owned
identity for the persistent declaration. Proto-ring infers no meaning from its
spelling. It is not a runtime artifact identity or a concrete consumer record
identity.

Every requirement declares an existing `GovernedResponsibilityId`. A requirement
may additionally declare one target:

```yaml
target:
  interface: <InterfaceId>
  object: <ObjectId>
```

The target must exist in the consumer's Governed Objects catalog and participate
in the requirement responsibility. Target omission is valid for
responsibility-level or derived subjects.

## Requirement instantiation

Model version 1 supports exactly two declarations.

A single declaration yields one logical current requirement:

```yaml
instances:
  kind: single
```

A source declaration binds concrete membership to one consumer-owned source:

```yaml
instances:
  kind: source
  source: <GovernedSourceId>
```

Source binding is not an extraction language. Proto-ring does not infer whether
membership is represented by an array, mapping, directory, database, YAML
artifact, Git object set, registration set, or another consumer representation.
Concrete instance identity remains consumer-owned. Model version 1 defines no
generic `RequirementInstanceId`.

## Evidence-class admission

Model version 1 supports exactly two forms.

Explicit admission declares a non-empty duplicate-free set of exact strings:

```yaml
evidence_classes:
  kind: explicit
  classes:
    - <exact-consumer-class>
```

Proto-ring performs no trimming, aliasing, hierarchy, wildcard expansion,
fallback, substitution, or equivalence inference.

Source-backed admission binds class membership to a consumer-owned source:

```yaml
evidence_classes:
  kind: source
  source: <GovernedSourceId>
```

Before Exact Evidence Binding evaluation, consumer-owned resolution must produce
an explicit non-empty admitted-class set for each concrete requirement. The
registry does not define that resolution algorithm.

## Subject and context binding

Every persistent requirement identifies one source supplying its current exact
opaque subject identity:

```yaml
subject:
  source: <GovernedSourceId>
```

The consumer owns identity construction, canonicalization, serialization,
hashing, and byte encoding. If the source cannot currently determine the
subject, consumer resolution constructs the existing concrete Exact Evidence
Binding requirement with unknown subject identity, which evaluates fail-closed
as `UNDETERMINED`.

Context has exactly two forms:

```yaml
context:
  required: false
```

and:

```yaml
context:
  required: true
  source: <GovernedSourceId>
```

`required: false` forbids a context source. `required: true` requires one. The
consumer owns which interpretation dimensions are encoded by its one opaque
context token. An unavailable required context resolves to the existing
context-required but unknown representation and therefore remains
`UNDETERMINED`.

## Mechanically significant embedded assertions

A mechanically significant assertion carried by an evidence or provenance
artifact may participate in an Evidence Requirement without the carrier's
structural validity establishing that assertion.

The consumer owns:

- whether the responsibility or object requires that evidence;
- the expected semantic identity;
- the source that resolves that identity;
- the candidate assertion or binding; and
- every higher-level consequence.

Artifact registration, hashing, replay success, manifest membership, Git
metadata, wall-clock metadata, or file existence is not a substitute for the
consumer-owned current identity required by the declaration.

When the required current subject or required current context cannot be
determined by the consumer-owned source, resolution MUST preserve the existing
Exact Evidence Binding unknown representation, and evaluation MUST remain
`UNDETERMINED`.

Known equality remains `MATCH`. Known inequality remains `MISMATCH`. Neither
`MATCH` nor carrier structural validity establishes claim truth, qualification
truth, evidence success, or semantic authority.

An Evidence Requirement states that evidence is required for its declared
consumer responsibility or object. Proto-ring does not add a separate generic
`mandatory` flag and does not define how the consumer aggregates that
requirement into a higher-level qualification or claim verdict.

## Candidate binding

Every persistent requirement identifies one source supplying zero, one, or many
candidate evidence bindings:

```yaml
candidates:
  source: <GovernedSourceId>
```

The registry defines no any-match, all-match, latest-wins, ranking, majority,
substitution, readiness, or qualification consequence. Consumers compose
individual Exact Evidence Binding statuses into their own higher-level logic.

## Source-role boundary

Every referenced source must exist in Canonical Governance Authority. Only the
registry authority source is required by this contract to have role `authority`
for the registry responsibility.

Instance, class, subject, context, and candidate sources have no universal
`SourceRole`. They may be authorities, secondary representations,
non-authoritative sources, or logical sources according to consumer-owned
semantics. Evidence Requirements neither acquires nor reinterprets their
roles.

## Identity separation

The identities are distinct:

```text
GovernedObjectRef
!=
Exact Evidence Binding subject identity
!=
Exact Evidence Binding context identity
```

A target states which governed object the requirement concerns. Proto-ring does
not encode `interface_id/object_id` into subject or context tokens and does not
make opaque exact-evidence identities into governed-object identities.

## Composition with Exact Evidence Binding

The complete generic composition is:

```text
persistent Evidence Requirement Registry
-> consumer-owned resolution
-> concrete EvidenceRequirement + candidate EvidenceBinding values
-> Exact Evidence Binding
-> MATCH | MISMATCH | UNDETERMINED
-> consumer-owned higher-level decision
```

Exact Evidence Binding remains the sole lower-level comparator. Evidence
Requirements does not alter its statuses or comparison semantics, duplicate its
implementation, or aggregate candidate results.

## Consumer confrontation

The model represents the demonstrated Turnlock Gate A declaration pattern with
two distinct responsibility-scoped, single requirements that admit
`assurance-decomposition`, use the same consumer-owned subject and candidate
sources, and differ only because one does not require context while the other
requires the consumer-owned current protocol-bundle context.

That expressibility does not move Gate A sequencing, materiality, finding
lifecycle, adjudication, challenge policy, reviewer qualification, or readiness
into this contract.

The model represents demonstrated Ruu post-baseline declarations through
source-driven membership, source-backed registration-owned artifact classes,
consumer-owned expected-subject sources, consumer-owned observed-candidate
sources, and a distinct explicit `recorded_output` declaration without context.

That expressibility does not move Ruu metadata, path safety, discovery, SHA-256
construction, registration, replay, expected exit codes, stdout comparison, or
qualification truth into this contract.

## Explicit exclusions

Evidence Requirements does not:

- modify consumer qualification or review truth;
- define a universal proof or evidence artifact schema;
- run evidence producers or replay;
- select or rank candidates;
- aggregate candidate statuses;
- invent generic evidence instance identities;
- infer hashes, canonical forms, byte encodings, or evidence classes;
- collapse governed-object, subject, and context identities;
- require Repository Integrity, `ValidationId`, `ProjectionId`, or
  per-requirement Governance Binding identities;
- define DevelopmentValidationEvidence or DevelopmentValidationDemand; or
- migrate consumer repositories.

Loading the registry is read-only and does not resolve consumer sources or
execute evidence mechanisms.
