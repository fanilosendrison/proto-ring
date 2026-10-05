---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-exact-evidence-binding"
severity: "strict"
name: "Exact Evidence Binding contract"
---

# Exact Evidence Binding contract

## Purpose

Exact Evidence Binding determines whether one candidate evidence binding matches
one consumer-supplied evidence requirement.

The contract owns only exact comparison of:

```text
explicitly admitted evidence class
+
exact opaque subject identity
+
exact opaque context identity when the consumer requires context
→
MATCH OR MISMATCH OR UNDETERMINED
```

The contract does not determine whether evidence is positive, trustworthy,
sufficient, semantically correct, or authoritative for a claim.

## Requirement authority

The consumer defines each governed evidence requirement. The consumer supplies:

- the explicit set of admitted evidence-class identifiers;
- the exact current subject identity when known;
- whether interpretation-relevant context participates in the binding; and
- the exact current context identity when required and known.

The requirement is authoritative for the comparison performed. Proto-ring does
not discover a requirement, choose a subject, select relevant context, or infer
an admissible evidence class.

A requirement whose current subject identity is unknown is structurally valid
but currently undetermined. A context-required requirement whose current context
identity is unknown is also structurally valid but currently undetermined.

## Opaque identity tokens

Subject and context identities are opaque non-empty octet sequences. Proto-ring
compares them only by exact length-and-octet equality.

The consumer owns how a domain identity becomes the exact octet sequence supplied
to the mechanism. A consumer may use raw canonical octets, an encoded object
identity, content-digest octets, a consumer-defined canonical serialization, or
another consumer-owned representation.

The concrete in-memory representation of an identity token is
implementation-defined and has no semantic significance.

The contract defines no identity algorithm, hash function, serialization,
canonical form, encoding, namespace, prefix, case rule, or semantic-equivalence
relation.

Exact comparison performs:

```text
no decoding
no hashing
no normalization
no trimming
no case folding
no prefix matching
no semantic equivalence
```

## Explicit evidence-class admission

Evidence-class vocabulary remains consumer-owned. A requirement supplies a
non-empty duplicate-free set of explicitly admitted class identifiers. Every
identifier must be a non-empty string and is consumed exactly as supplied.

The concrete in-memory set representation is implementation-defined and has no
semantic significance.

A requirement may admit multiple classes. That is an explicit consumer-policy
decision; it is not an equivalence inferred by proto-ring.

A candidate supplies one declared evidence-class identifier. A known candidate
class outside the requirement's admitted set produces `MISMATCH` after mandatory
binding information has been established.

## Exact subject binding

A requirement supplies the exact current subject token when known. Candidate
evidence supplies its declared exact subject token.

When either mandatory subject token is unavailable or the observed token is
malformed, current binding is `UNDETERMINED`. When both valid exact tokens are
known but differ, current binding is `MISMATCH`.

Subject equality alone does not establish evidence result truth, evidence
sufficiency, or semantic authority.

## Optional-but-explicit context binding

The consumer explicitly declares whether interpretation-relevant context
participates in a requirement.

```text
context not required
≠
context required but currently unknown
```

When context is required, the requirement and candidate evidence must each
supply a valid non-empty opaque context token. Unknown or malformed mandatory
context produces `UNDETERMINED`. Known unequal context tokens produce
`MISMATCH`.

When context is not required, a requirement must not carry a context identity.
Any candidate context value is ignored completely for that evaluation. The
consumer requirement, not the candidate, controls whether context participates.

The one opaque context token may represent every interpretation dimension that
the consumer declares relevant. Proto-ring does not define, enumerate, split,
combine, or interpret those dimensions.

## MATCH / MISMATCH / UNDETERMINED

The mechanism returns exactly one `BindingStatus`.

### MATCH

`MATCH` means only:

```text
the candidate evidence class is explicitly admitted
AND
the candidate exact subject token equals the required current subject token
AND
every consumer-required exact context token equals the required current token
```

Explicitly:

```text
MATCH is not evidence success
MATCH is not proof validity
MATCH is not claim truth
MATCH is not obligation satisfaction
```

`MATCH` also does not establish evidence trustworthiness, product safety,
review correctness, test success, or sufficiency for any broader decision.

### MISMATCH

`MISMATCH` means:

```text
all binding information required to compare the relevant dimension is known,
but at least one exact binding does not match
```

A known unadmitted class, unequal exact subject token, or unequal required exact
context token produces `MISMATCH`. The mechanism performs no fallback,
substitution, alias lookup, or equivalence inference.

### UNDETERMINED

`UNDETERMINED` means:

```text
at least one mandatory current or observed binding identity required for
evaluation is unavailable or malformed
```

It includes an unknown current subject, unknown required current context, absent
candidate evidence, missing or malformed candidate class, missing or malformed
candidate subject, and missing or malformed required candidate context.

Current requirement determinacy is evaluated before candidate evidence. An
unknown current mandatory identity therefore produces `UNDETERMINED` even when
another observed field appears mismatched.

MISMATCH and UNDETERMINED are distinct. `MISMATCH` records a known failed exact
comparison; `UNDETERMINED` records that a mandatory comparison cannot safely be
completed.

## Current versus historical requirement

Currentness is requirement-relative. The mechanism evaluates the candidate only
against the exact requirement supplied for that call.

Evidence may match a historical requirement while failing to match a later
current requirement whose subject or required context changed. That later
mismatch does not rewrite or globally invalidate the historical evaluation.

The contract preserves that historically bound evidence may remain historically
inspectable even when it does not match a later current requirement.

Historical custody, interpretation, replay, retention, and claim validity remain
consumer-owned.

## No implicit evidence substitution

No implicit evidence substitution is permitted.

A candidate class not explicitly present in `admitted_classes` cannot fall back
to another class. Proto-ring does not infer aliases, class hierarchy,
interchangeability, substitutability, similarity, or cross-class proof value.

Explicit admission of multiple classes is the only way a requirement accepts
more than one evidence-class identifier.

## Fail-closed use of mandatory binding

When a consumer declares Exact Evidence Binding mandatory, the binding
prerequisite is interpreted as:

```text
MATCH          → binding prerequisite is satisfied
MISMATCH       → binding prerequisite is not satisfied
UNDETERMINED   → binding prerequisite is not satisfied
```

That mapping does not establish the consumer's overall verdict. The consumer
owns how the prerequisite composes with assurance, qualification, integrity,
review, validation, or product decisions.

The module itself returns only `BindingStatus`.

## Consumer authority boundary

Consumers retain authority over:

- subject identity construction;
- context identity construction;
- hashing and canonicalization;
- evidence-class vocabulary;
- which classes satisfy which obligations;
- which context dimensions matter;
- evidence result and claim semantics;
- proof sufficiency;
- evidence generation, storage, retention, and replay;
- review and qualification policy;
- formal backends and formal semantics;
- validation membership and ordering; and
- product semantics.

Exact Evidence Binding consumes consumer-owned values. It does not become their
canonical owner and transfers no semantic authority from the consumer or the
evidence subject to proto-ring.

## Relationship to Repository Integrity

Exact Evidence Binding determines only whether one candidate evidence binding
matches one consumer-supplied evidence requirement.

Repository Integrity may execute a consumer-owned obligation that uses Exact
Evidence Binding.

Repository Integrity does not become the owner of evidence semantics.

Exact Evidence Binding does not become the owner of repository validation
membership or overall integrity verdicts.

The generic mechanism is not directly integrated into the Repository Integrity
implementation. A consumer adoption may compose it through a consumer-owned
obligation and immutable provider binding.

## Explicit exclusions

Exact Evidence Binding does not:

- read files, Git state, GitHub state, or environment state;
- infer the current repository state;
- hash bytes or canonicalize structures;
- parse JSON or YAML;
- validate schemas;
- run evidence producers, tests, checkers, or replay;
- evaluate proof validity, evidence result truth, or claim truth;
- define review, qualification, or product policy;
- define evidence storage, retention, expiration, or caching;
- define evidence-class vocabulary;
- define subject identities or context dimensions;
- infer consumer authority;
- define a universal evidence or proof format; or
- authorize a broader assurance, integrity, or product verdict.
