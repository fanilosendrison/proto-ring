---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-normative-terminology"
severity: "strict"
name: "Canonical Normative Terminology contract"
---

# Canonical Normative Terminology contract

## Purpose

Normative Terminology governance detects structural drift and definition-like
lexical occurrences around a consumer-owned terminology corpus.

Its core boundary is:

```text
structured registry
+ consumer document policy
+ conservative lexical discovery
+ reviewed occurrence inventory
→ terminology-governance diagnostics
```

This mechanism supports review. It does not establish product meaning.

## Non-proof boundary

Terminology governance is heuristic and structural analysis, not semantic proof.

A passing result does not prove:

- semantic equivalence between two passages;
- completeness of a terminology corpus;
- absence of every competing definition;
- correctness of a canonical definition;
- validity of a product requirement or decision; or
- correspondence between natural language and a formal representation.

The mechanism may identify definition-like occurrences, structural conflicts,
stale reviewed occurrences, and canonical-location violations. Human or
separately governed semantic review remains responsible for meaning.

## Consumer authority

Each consumer owns:

- product terminology and semantic authority;
- the normative document or corpus;
- registry boundary markers and displayed table headers;
- concept-key and canonical-anchor syntax;
- section or location labels;
- canonical-definition location policy;
- role vocabulary and role assignment;
- inventory serialization and top-level metadata;
- source and inventory paths;
- CLI presentation and exit behavior;
- repository-validation membership; and
- adoption and immutable provider pinning.

The shared mechanism consumes these bindings. It does not infer or amend them.

## Registry model

A terminology registry contains exactly one ordered pair of standalone boundary
marker lines around a restricted GFM-style table. The consumer supplies the
exact boundary markers and displayed headers for six semantic columns:

1. concept key;
2. canonical term;
3. canonical anchor;
4. accepted aliases;
5. deprecated wording; and
6. term structure.

Each valid entry has:

```text
one concept key
one non-empty canonical term
one non-empty canonical anchor destination
zero or more accepted aliases
zero or more deprecated expressions
one non-empty term-structure classification
```

The concept key and canonical destination conform to consumer-supplied patterns.
Concept keys and canonical destinations are unique within the registry.

Canonical, alias, and deprecated expressions within one entry are unique under
Unicode case folding. An expression may occur in different entries because
lexical overlap and compound terminology do not prove concept identity.

The admitted table grammar is intentionally narrow:

- after surrounding line-whitespace trimming, every table line begins and ends
  with exactly one outer pipe;
- cells are separated by literal pipes, with no escaped-pipe interpretation;
- canonical terms and keys may have one surrounding backtick pair;
- aliases and deprecated expressions are semicolon-separated, and every list
  item must decode to a non-empty expression;
- an empty cell or a sole undecorated em dash represents an empty expression
  list, while combining or code-formatting the sentinel is invalid; and
- the separator uses at least three hyphens with optional edge colons.

This is a terminology-registry format, not a claim to parse every valid GFM
table. Decoded canonical terms, aliases, deprecated expressions, and term
structures must remain non-empty where their semantic column requires a value.

The parser validates registry structure and returns accumulated diagnostics for
ordinary invalid content. It does not interpret term-structure classifications
or assign semantic relationships between concepts.

## Markdown structural model

The shared parser discovers:

- ATX headings with zero to three leading spaces, empty heading text, and
  optional whitespace-separated closing hash sequences;
- consumer-derived section labels;
- ordinary paragraph blocks;
- fenced code blocks beginning with triple backticks or tildes;
- full-line canonical anchors admitted by the consumer format; and
- registry boundaries that exclude the registry itself from occurrence
  discovery.

A block carries its first line, current section label, current heading, and
exact joined text. Section labels are opaque consumer values. Numeric section
semantics are not part of this contract.

A duplicate canonical anchor is an error. Every canonical-looking anchor in the
consumer-defined anchor namespace must be registered, and every registered
canonical destination must exist.

## Definition-like discovery

For each registry expression, discovery preserves every expression character
and performs boundary-aware matching with the same full Unicode case-fold
relation used for registry expression uniqueness. Surrounding backtick,
emphasis, strike, and quote presentation delimiters are admitted only in the
definition context; they are not deleted from expression content.

Straight and curly single or double quotes may delimit an expression. The
heuristic recognizes definition-like prose cues equivalent to:

```text
is
are
means
refer(s) to
is/are defined as/by
```

It also recognizes line-start definition equations using `:=` or a single `=`.
It excludes equality `==` from that equation cue.

Expression matching requires boundaries outside ASCII letters and digits so a
registered expression is not matched inside a longer ASCII alphanumeric word.
Unicode case-fold expansion must not make Unicode characters adjacent in the
original text part of the ASCII boundary class. Matching therefore preserves a
mapping from folded spans to complete original characters before evaluating
ASCII boundaries.
The heuristic searches canonical terms, accepted aliases, and deprecated
wording because any of them may introduce an independent lexical assignment.

These cues are intentionally conservative and incomplete. Their discovery has
no semantic-proof status.

## Canonical occurrence binding

The consumer supplies a predicate that decides whether a candidate block is in
an allowed canonical-definition location. The predicate must return a boolean;
other output is invalid policy and fails closed.

A registered canonical anchor is valid only when:

1. it occurs exactly once;
2. its first following parsed block has the same consumer section label;
3. no other recognized canonical anchor intervenes;
4. the consumer canonical-location predicate accepts the entry and block; and
5. the block heuristically defines that entry's concept.

A valid binding produces one canonical occurrence for that registry entry.
Other definition-like blocks produce non-canonical occurrences. A block may
contain more than one concept, and its concept keys are represented in sorted
order.

Canonical-location policy is consumer authority. The shared engine defines the
binding procedure, not an expected section number, heading, path, or document
organization.

## Fingerprints and signatures

Block normalization is exactly:

```text
split on Unicode whitespace
→ remove empty runs
→ join tokens with one ASCII space
```

The occurrence fingerprint is the lowercase hexadecimal SHA-256 digest of the
normalized text encoded as UTF-8.

Every occurrence normalizes its concept tuple into sorted order. An occurrence
signature is:

```text
(sorted concept keys, section label, heading, fingerprint)
```

The source line and canonical flag are diagnostics and discovery facts. They do
not participate in the maintained signature.

The fingerprint detects reviewed-text drift. It does not authenticate semantic
meaning and is not a universal governed-object identity.

## Consumer role policy

The consumer supplies:

- a non-empty set, represented immutably by the engine API, of allowed role
  strings; and
- a role resolver for discovered occurrences.

The resolver may use the canonical flag, section label, heading, concepts, or
other occurrence properties. Every resolved role must belong to the supplied
allowed set.

The shared engine does not define canonical-definition, intent, obligation,
implication, reference, synopsis, or another consumer vocabulary.

## Reviewed occurrence inventory

The shared reconciler receives an already loaded occurrence collection. The
consumer retains ownership of YAML, JSON, or another serialization format and
of all top-level schema, authority, source, and fingerprint-policy declarations.

Every maintained occurrence contains:

- a non-empty list of concept-key strings;
- the consumer section label as a string;
- the heading as a string;
- one consumer-allowed role; and
- one lowercase 64-character SHA-256 fingerprint.

The reconciler validates:

- occurrence collection and record shape;
- unknown concept references;
- invalid roles;
- malformed fingerprints;
- maintained occurrence copies in excess of discovered signature multiplicity;
- disagreement with the consumer role resolver, with expected and maintained
  roles reconciled as multisets when occurrences share a signature;
- discovered occurrences absent from the inventory; and
- inventory occurrences absent from discovery.

Actual and maintained signatures are compared as multisets. Expected and
maintained roles are also compared as multisets within each signature, because
canonical status and other role inputs are intentionally absent from signature
identity. Duplicate copies cannot be hidden through set deduplication.

An unmatched discovered copy is unreviewed. An unmatched maintained copy is
stale. Both are failures requiring consumer review or correction.

## Failure behavior

Invalid registry structure prevents trustworthy canonical discovery but returns
controlled diagnostics. Structural anchor failures and inventory failures also
return controlled diagnostics rather than being treated as success.

A consumer policy that cannot produce an allowed role is invalid. Unknown or
malformed mandatory information fails closed; it is not interpreted as an empty
or successful inventory.

The engine has no file, network, environment, Git, rendering, repair, or
repository-mutation responsibility.

## Adoption boundary

A consumer adopts the mechanism through an immutable executable provider
identity and explicit local bindings.

Adoption must preserve or intentionally strengthen every existing consumer
guarantee. It must not transfer product terminology, semantic authority,
document organization, inventory content, paths, roles, CLI behavior, or
validation membership to proto-ring.

After adoption, each generic production rule has one executable owner in the
shared mechanism. Independent consumer tests may restate expected behavior as
verification oracles without becoming a second production authority.
