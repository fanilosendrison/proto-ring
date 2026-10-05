---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-structured-data"
severity: "strict"
name: "Canonical Structured Data contract"
---

# Canonical Structured Data contract

## Purpose

Canonical Structured Data defines one deterministic, language-neutral structured
value model and one restricted YAML representation for proto-ring machine-readable
governance data.

It owns representation and construction only. It does not assign semantic meaning
to consumer fields, choose carrier paths, define frontmatter envelopes, define
routing, or own a consumer schema.

## Structured value model

The canonical value domain contains exactly:

```text
mapping<string, value>
sequence<value>
string
boolean
integer
null
```

No floating-point value, timestamp/date runtime value, binary value, tagged
object, alias identity, or implementation-specific object exists in this model.

Integers are mathematical integers. The contract defines no machine-width,
digit-count, serializer, Python-runtime, Rust-runtime, or other implementation
limit.

A consuming contract MAY require a particular root type, such as a mapping.
Canonical Structured Data itself permits any value from the model as the root.

## Canonical document bytes

A canonical structured document is read as exact bytes.

The document:

- MUST be valid UTF-8;
- MUST NOT contain a UTF-8 byte-order mark;
- MUST NOT contain a carriage-return byte;
- therefore uses LF when line termination is present; and
- MUST contain exactly one YAML document.

A single YAML document MAY use an explicit document-start marker, an explicit
document-end marker, both, or neither. Those markers do not create additional
documents.

Residual non-comment content after an explicit document-end marker is invalid.
A second document-start marker or otherwise representable second YAML document
is invalid.

YAML directives are forbidden.

A consuming carrier MAY extract the canonical-document bytes from a larger
envelope. Carrier bytes outside the extracted document remain owned by that
carrier contract.

## Collection representation

Mappings and sequences may use ordinary YAML block or flow form. Comments are
allowed.

Every mapping key MUST construct to a string.

Duplicate constructed string keys fail closed at every mapping depth. Quoting
does not create a distinct key, so:

```yaml
foo: 1
"foo": 2
```

is a duplicate.

A mapping key whose constructed string is exactly `<<` is forbidden whether
quoted or unquoted.

## Scalar interpretation

Scalars are constructed in this exact order:

```text
exact unquoted lowercase true / false
→ boolean

exact unquoted lowercase null
→ null

canonical unquoted decimal integer grammar
→ integer

every other admitted ordinary scalar
→ string
```

The canonical decimal integer grammar is:

```regex
^(0|-?[1-9][0-9]*)$
```

No implicit YAML scalar resolution may introduce another type.

### Strings

Quoted YAML scalars construct strings exactly as represented by the YAML string
value, including a quoted empty string.

Quoted `"null"`, `'null'`, `"true"`, and `'false'` remain strings.

Any admitted plain scalar outside the exact boolean/null/integer rules constructs
a string. This includes dates, versions, hashes, identifiers, refs, paths,
floating-looking text, timestamp-looking text, `yes`, `no`, `on`, `off`,
`True`, `FALSE`, `Null`, `NULL`, `~`, `01`, `-0`, `+1`,
`1_000`, `0x10`, `0o10`, `0b10`, `1.0`, and `1e3`.

Block scalar forms `|` and `>` are forbidden.

### Booleans

Only exact unquoted lowercase `true` and `false` construct booleans.

### Integers

Only an unquoted scalar matching the canonical decimal grammar constructs an
integer. Its exact mathematical value must be preserved regardless of size.

### Null

Only exact unquoted lowercase `null` constructs null.

Omitted-value null syntax is forbidden:

```yaml
key:
```

Explicit null is:

```yaml
key: null
```

## Forbidden representation features

Canonical Structured Data forbids:

- anchors;
- aliases;
- YAML merge semantics;
- any mapping key whose constructed string is `<<`;
- explicit YAML tags;
- YAML directives;
- multiple YAML documents;
- arbitrary object constructors or other unsafe construction;
- duplicate mapping keys at any depth;
- non-string mapping keys;
- block scalar forms `|` and `>`; and
- omitted-value null syntax.

Malformed or forbidden representation fails closed and is not normalized,
repaired, merged, or reinterpreted into accepted data.

## Determinism boundary

The canonical constructed value is independent of a language runtime's native
YAML implicit typing.

A conforming implementation may use any parser internally, but parser defaults
must not change the canonical structured value or admit a forbidden
representation.

Container class names, map/hash implementation, integer storage type, exception
class names, and parser-library object types are not normative.

## Frontmatter composition

A frontmatter-owning contract defines its own exact envelope and extracts one
canonical structured document from it.

Canonical Structured Data does not define an opening delimiter, closing
delimiter, body semantics, carrier filename, or repository location.

The extracted document is then constructed under this contract.

## Standalone-document composition

A standalone machine-readable YAML artifact may use its complete file bytes as
one Canonical Structured Data document.

The consumer of that artifact owns any additional root-shape, field, schema,
authority, or path requirements.

## Failure boundary

Unreadable carrier bytes are owned by the calling carrier/file mechanism.
Once document bytes are supplied, invalid UTF-8, forbidden representation,
ambiguous construction, malformed YAML, or non-deterministic structured value
must fail closed through the consuming proto-ring operation's controlled error
boundary.

## Authority boundary

Canonical Structured Data owns representation only.

It does not own:

- consumer field vocabulary;
- product semantics;
- governance responsibility semantics;
- schema ownership;
- repository paths or routing;
- authority relationships;
- evidence truth;
- validation truth; or
- mutation.
