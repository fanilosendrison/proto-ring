---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-governance-bootstrap"
severity: "strict"
name: "Canonical Governance Bootstrap contract"
---

# Canonical Governance Bootstrap contract

## Purpose

Canonical Governance Bootstrap defines the deterministic entry from a repository
root to the machine-readable repository-governance configuration in root
`AGENTS.md`.

The contract owns the canonical carrier identity, byte and frontmatter-envelope
requirements, admitted structured-data representation, extraction of the
`repository_governance` mapping, and controlled fail-closed bootstrap behavior.
It does not assign semantic meaning to consumer-owned configuration.

## Canonical carrier

The canonical agent-entry governance carrier MUST be exactly:

```text
<repository-root>/AGENTS.md
```

Bootstrap MUST begin from an explicit repository root. It MUST NOT perform
recursive discovery, nearest-file discovery, alternate filename lookup,
fallback lookup, candidate ranking, filesystem heuristics, or prose
reconstruction.

## Byte-level carrier requirements

The complete carrier MUST be read as bytes. It MUST NOT contain a UTF-8 byte
order mark or a carriage-return byte anywhere, and it MUST decode completely as
valid UTF-8. Admitted line endings are therefore LF only.

These requirements apply to the complete carrier, including the agent-facing
body after the frontmatter envelope.

## Exact frontmatter envelope

The first four carrier bytes MUST be exactly:

```text
---\n
```

No whitespace, prose, comment, or other byte may precede the opening delimiter.

The closing delimiter MUST be the first subsequent line exactly equal to `---`
and MUST have LF termination. At the byte level, the closing boundary is the
first subsequent occurrence of:

```text
\n---\n
```

Absence of that exact boundary MUST fail closed. The bytes between the opening
and closing delimiters form exactly one structured-data payload governed by
this contract. The envelope delimiters are not YAML multi-document syntax.

The remainder after the closing delimiter is agent-facing body or prose. The
bootstrap MUST NOT interpret that remainder as machine-governing configuration.

## Structured-data model

The admitted structured-data tree MUST contain only:

```text
mapping<string, value>
sequence<value>
string
boolean
integer
null
```

No floating-point type or timestamp/date runtime type exists in this model. No
representation feature may construct an object outside this fixed data model.

Mappings and sequences MAY use ordinary YAML block or flow form. Comments are
allowed.

## Scalar interpretation

Scalars MUST be constructed in this exact order:

```text
exact lowercase true / false
→ boolean

exact lowercase null
→ null

canonical decimal integer grammar
→ integer

every other admitted ordinary scalar
→ string
```

Implicit YAML scalar resolution MUST NOT alter this order or introduce another
runtime type.

### Strings

Quoted YAML strings MUST construct strings. A quoted empty string is allowed.
Quoted `"null"` and `'null'` MUST construct strings.

Any admitted plain scalar that does not match the exact boolean, null, or
integer rules MUST construct a string. This includes dates, versions, hashes,
identifiers, refs, paths, ordinary words, floating-looking text,
timestamp-looking text, `yes`, `no`, `on`, `off`, `True`, `FALSE`, `Null`,
`NULL`, and `~`.

Block scalar forms `|` and `>` MUST NOT be admitted.

### Booleans

Only the exact unquoted lowercase tokens `true` and `false` MUST construct
booleans.

Alternative spellings, including `True`, `FALSE`, `yes`, `no`, `on`, and `off`,
MUST construct strings.

### Integers

Only an unquoted plain scalar matching this exact grammar MUST construct an
integer:

```regex
^(0|-?[1-9][0-9]*)$
```

The tokens `01`, `-0`, `+1`, `1_000`, `0x10`, `0o10`, `0b10`, `1.0`, and `1e3`
MUST construct strings. Implicit floating-point conversion MUST NOT occur.

### Null

Only the exact unquoted lowercase token `null` MUST construct the null value.

The tokens `Null`, `NULL`, and `~` MUST construct strings. Quoted `"null"` and
`'null'` MUST construct strings.

An omitted mapping value, such as the following, MUST be invalid and MUST NOT
construct null:

```yaml
key:
```

Explicit null MUST be written as:

```yaml
key: null
```

## Mapping requirements

Every mapping key MUST construct to a string. Duplicate constructed string keys
MUST fail closed at every mapping depth. Quoting does not create a distinct key;
therefore this input contains a forbidden duplicate:

```yaml
foo: 1
"foo": 2
```

A mapping key whose constructed string is exactly `<<` MUST be forbidden,
whether quoted or unquoted.

The top-level structured value MUST be a mapping.

## Forbidden YAML representation features

The structured-data representation MUST forbid:

- anchors;
- aliases;
- merge semantics;
- any mapping key whose constructed string is `<<`;
- explicit YAML tags;
- YAML directives;
- multiple YAML documents;
- arbitrary object constructors or other unsafe construction;
- duplicate mapping keys at any depth;
- non-string mapping keys;
- block scalar forms `|` and `>`; and
- omitted-value null syntax.

Malformed or forbidden representation MUST NOT be normalized into accepted
input.

## Governance bootstrap root

The top-level mapping MUST contain the exactly addressable key
`repository_governance`, and its value MUST be a mapping. Other valid top-level
metadata is allowed.

Unknown valid descendants of `repository_governance` MUST be preserved. The
bootstrap MUST NOT discard an unknown descendant merely because the current
proto-ring scope does not interpret it, and it MUST NOT assign universal
semantic meaning to every descendant.

Machine-governing facts required by proto-ring MUST come from structured
configuration. Missing machine-governing state MUST NOT be reconstructed from
`AGENTS.md` prose.

## Failure behavior

Missing, unreadable, malformed, ambiguous, unsafe, or structurally invalid
bootstrap state MUST fail closed through a controlled proto-ring error boundary.
There MUST be no best-effort recovery from prose and no silent selection among
ambiguous interpretations.

## Composition with Canonical Governance Routing

The contracts compose as follows:

```text
repository root
    ↓
Canonical Governance Bootstrap
    - exact root AGENTS.md
    - exact frontmatter envelope
    - deterministic structured-data semantics
    - repository_governance mapping
    ↓
already-parsed structured mapping
    ↓
Canonical Governance Routing
    - caller supplies one exact route
    - exact route traversal
    - repository-relative path
    - symlink-aware containment
    - target existence
    ↓
consumer-owned target
```

Governance Bootstrap owns representation before routing. Canonical Governance
Routing continues to receive an already-parsed mapping and one exact
caller-supplied route. Canonical Governance Routing still does not own carrier
parsing.

The upstream Governance Bootstrap responsibility does not retroactively change
the historical extraction boundary of Canonical Governance Routing. Its
statement that carrier parsing is outside that contract's responsibility
remains true.

## Consumer authority boundary

The consumer retains authority over product semantics, Product Intent, decision
authority, ADR meaning, formal-verification semantics, qualification semantics,
consumer-local policy, the meaning of arbitrary `repository_governance`
descendants, routed target semantics, and routed target authority.

A route obtained from the bootstrap may identify a consumer-owned profile,
binding, registry, or other target. Existing repository-containment guarantees
remain applicable when a separate routing contract resolves that route.
Bootstrap and routing do not transfer semantic authority over the target to
proto-ring.

## Explicit exclusions

Canonical Governance Bootstrap does not define:

- consumer product semantics or Product Intent;
- consumer decision authority or ADR meaning;
- formal-verification or qualification semantics;
- consumer-local policy;
- universal meaning for arbitrary `repository_governance` descendants;
- routing vocabulary beyond contracts that separately own it;
- target artifact semantics or target authority;
- the full repository-governance model;
- a capability registry;
- projection, validation, or evidence registries;
- mutation semantics; or
- source-code layout.
