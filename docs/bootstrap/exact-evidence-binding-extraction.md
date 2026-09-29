---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Exact Evidence Binding extraction evidence"
---

# Exact Evidence Binding extraction evidence

Status: non-normative extraction analysis

## Exact baselines and provenance

This extraction is bounded to the following exact repository states and work
records:

```text
proto-ring source:
1b767eb4e81a83e778c90649c632694c22131595

Turnlock:
0c2fd68974466df37105682d9aa8d034ed22dad2

Ruu:
e27adc9290b4781396cb5c9745014ca5427e98ac

source adjudication:
fanilosendrison/proto-ring#19

extraction work:
fanilosendrison/proto-ring#20
```

The source adjudication identified Exact Evidence Binding as the sole residual
candidate ready for extraction. This record implements that fixed disposition;
it does not re-adjudicate the residual candidate matrix.

## Extracted responsibility

The consumer-independent responsibility is:

```text
consumer defines a governed evidence requirement
        ↓
consumer supplies the exact current subject identity
        ↓
consumer explicitly declares which evidence classes are admissible
        ↓
consumer declares whether interpretation-relevant context is mandatory
        ↓
consumer supplies the exact current context identity when known
        ↓
candidate evidence supplies its declared class
and exact subject/context binding identities
        ↓
proto-ring compares only those bindings
        ↓
MATCH OR MISMATCH OR UNDETERMINED
```

The shared mechanism consumes opaque exact `bytes` tokens. It does not construct,
hash, canonicalize, decode, normalize, or interpret an identity. The consumer
owns the conversion from each domain identity to the exact bytes supplied for
evaluation.

`MATCH` means only that the candidate class is explicitly admitted and every
consumer-required exact identity matches. It says nothing about the candidate
result, trustworthiness, sufficiency, governed obligation, underlying claim, or
product safety.

## Turnlock evidence distinctions

### ADR-041

ADR-041 separates hostile semantic-review evidence from mechanical checker
evidence. It also separates semantic correspondence, formal semantics,
verification execution, and evidence lifting. The extracted boundary preserves
that evidence classes do not substitute for one another and does not import the
formal-assurance graph or readiness gates.

### ADR-043

ADR-043 demonstrates exact packet-to-subject binding and exact
refutation-to-challenge binding. It separately preserves historical packet
validity and currentness: a self-contained historical packet can remain valid
for its declared historical subject without satisfying a current campaign.

The packet serialization, subject construction, canonicalization, hashing,
challenge payload, and repository path policy remain Turnlock-owned.

### ADR-044

ADR-044 forbids ambiguous subject selection. It rejects first, last,
deduplicated, packet-selected, and current-selected fallback among candidate
subjects. The extractable property is that an exact required subject binding
cannot be replaced by precedence or heuristic selection. Subject construction
and cardinality policy remain Turnlock-owned.

### ADR-045

ADR-045 establishes that semantic subject identity and hostile-review protocol
identity are distinct and that a current campaign binds both. Protocol authority
remains evidence/protocol authority and does not become product authority.

The extracted context token can carry consumer-declared interpretation-relevant
context, but proto-ring does not define protocol identity, reviewer profiles,
adjudication, or readiness.

### ADR-046

ADR-046 binds each challenge execution to its exact canonical input packet and
separates input identity from output interpretation and retry admissibility.
Challenge construction, deterministic validation, protocol-attempt semantics,
and append-only protocol evolution remain Turnlock-owned.

### ADR-048

ADR-048 demonstrates that an interpretation contract identity participates in
evidence meaning. Content-addressed meta-schema history and protocol evolution
remain Turnlock-owned. The generic extraction represents any required
interpretation context only as one opaque consumer-supplied token.

### ADR-049

ADR-049 keeps logical execution, protocol attempt, LLM call, and transport
attempt identities distinct. The extraction does not merge or define those
identities. A consumer may construct one exact subject or context token from the
dimensions relevant to its requirement.

### Mechanical evidence

`formal/tlc-result.schema.json` and `formal/verification.yaml` require mechanical
evidence to bind to the exact executed identity and relevant execution context.
The result vocabulary, model configuration, module, backend, tool version,
properties, invariants, bounds, assumptions, and evidence-lifting semantics
remain Turnlock-owned.

Turnlock has no executable canonical formal model and no completed hostile-review
campaign at this baseline. Exact binding nevertheless recurs across distinct
accepted evidence contracts and does not depend on treating the complete
assurance system as mature or shared.

## Ruu confrontation

### Retained exact-state rule

ADR-030 retains the broader rule that exact state plus relevant policy/context
governs safe evidence reuse. Evidence for state `X` cannot authorize changed
state `X'`. Evidence classes remain distinct and may require different context
bindings.

Later Ruu amendments supersede the generic `DevelopmentValidationEvidence` and
`DevelopmentValidationDemand` mechanism and its internal fixed-point execution
semantics. This extraction does not resurrect those superseded semantics. It
retains only the broader exact-state and time-of-check/time-of-use binding
principle for evidence or external facts that still genuinely exist.

### Qualification lineage and manifests

Ruu's retained lineage binds exact artifact identity, source provenance,
original path, current path, status, and SHA-256. The immutable snapshot and
current manifest distinguish retained historical evidence from active repository
state and detect missing, extra, or changed artifacts.

Those concrete hashes, schemas, snapshot paths, artifact kinds, manifests,
version ranges, and retention rules remain Ruu-owned. They demonstrate the need
for exact binding without establishing a universal identity or evidence schema.

### Historical, post-baseline, and Git-smoke replay

Historical replay binds admitted executable and recorded-output hashes to an
exact historical source package when that package is required. Missing or
mismatched baselines are explicit non-success outcomes.

Post-baseline replay binds registered executable identity, expected exit code,
and recorded output to one registered qualification. Git-smoke replay binds the
admitted retained scripts to a supported Git environment and their declared
behavior.

Replay, output comparison, environment qualification, state-space versioning,
Git policy, and result interpretation remain Ruu-owned.

### Recorded output is not current proof

`qualification/README.md` explicitly rejects inferring validity from a recorded
`PASS` string. Artifact presence and recorded text do not establish a current
claim. Current Repository Integrity and Ruu-owned replay/qualification remain
separate responsibilities.

## Consumer-independence analysis

Turnlock demonstrates exact subject and interpretation-context binding across
review and mechanical evidence classes. Ruu concretely demonstrates exact-state,
policy/context, artifact, provenance, executable, output, and replay binding.
Their representations differ materially, so no shared schema, hash function,
identity algorithm, backend, evidence store, replay procedure, or assurance
architecture is justified.

The reusable boundary is only the exact relation among:

```text
consumer-owned admitted class identifiers
consumer-owned current subject token
consumer-owned context participation decision
consumer-owned current context token when required and known
candidate-declared class and exact binding tokens
```

Proto-ring compares those supplied values without acquiring authority over any
of them.

## EXTRACT

The following properties belong to Exact Evidence Binding:

- consumer explicitly identifies admitted evidence classes;
- consumer supplies exact current subject binding token;
- consumer declares whether exact context binding participates;
- consumer supplies exact current context token when required and known;
- candidate evidence declares class and exact binding tokens;
- no implicit class substitution;
- known class, subject, or context mismatch does not authorize current evidence;
- missing or unknown mandatory current binding is not success;
- missing or unknown mandatory evidence binding is not success;
- `MATCH` is only binding equivalence;
- historical binding and current binding are requirement-relative; and
- evidence binding transfers no semantic authority.

## CONSUMER

The following responsibilities remain consumer-owned:

- subject identity construction;
- context identity construction;
- hashing and canonicalization;
- evidence-class vocabulary;
- which classes satisfy which obligations;
- which context dimensions matter;
- evidence result semantics;
- claim semantics;
- proof sufficiency;
- evidence generation;
- storage;
- retention;
- replay;
- review policy;
- qualification policy;
- formal backend; and
- product semantics.

## Rejected over-generalization

The following proposed shared responsibilities are rejected:

- universal evidence schema;
- universal proof format;
- universal context schema;
- universal hashing algorithm;
- universal identity algorithm;
- universal evidence time-to-live policy;
- universal evidence cache;
- universal reviewer protocol;
- universal formal backend; and
- evidence `MATCH` implying proof or claim success.

No candidate remains unresolved.

## Relationship to Repository Integrity

Exact Evidence Binding determines only whether one candidate evidence binding
matches one consumer-supplied evidence requirement.

Repository Integrity may execute a consumer-owned obligation that uses Exact
Evidence Binding. Repository Integrity does not thereby own evidence semantics,
and Exact Evidence Binding does not own validation membership or the overall
integrity verdict.

Consumer adoption and integration remain separate follow-up work.

## Conclusion

The exact baselines justify one minimal domain-neutral binding relation with
three results: `MATCH`, `MISMATCH`, and `UNDETERMINED`. This extraction does not
transfer Turnlock assurance authority, Ruu qualification authority, or consumer
identity construction into proto-ring.
