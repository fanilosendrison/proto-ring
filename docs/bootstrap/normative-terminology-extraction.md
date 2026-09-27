---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "extraction-analysis"
domain: "proto-ring-bootstrap"
severity: "strict"
name: "Normative Terminology extraction inventory"
---

# Normative Terminology extraction inventory

Status: non-normative extraction analysis.

This record preserves the factorization evidence for proto-ring Issue #15. The
canonical shared scope is established separately by
[`docs/contracts/normative-terminology.md`](../contracts/normative-terminology.md)
and the responsibility-split implementation in
`src/proto_ring/normative_terminology.py`,
`src/proto_ring/normative_terminology_matching.py`, and
`src/proto_ring/normative_terminology_inventory.py`.

## Evidence baselines

The primary extraction source was:

```text
fanilosendrison/turnlock-rust
ddb6b766bd9ca779ec95ee214996831cd86f68ef
scripts/check-normative-terminology.py
scripts/tests/test-normative-terminology.py
```

The principal confrontation consumer was:

```text
fanilosendrison/ruu
8ce798de15ee67cfd17cf55128eccd61f94ef0fa
```

The proto-ring authority reviewed for this extraction was:

```text
fanilosendrison/proto-ring
4b139e2858fcb26b694503386096eb3fcea80d31
```

Turnlock ADR-050 and Ruu ADR-083 authorize immutable shared governance
implementation while retaining consumer authority. This inventory is evidence,
not authority for either consumer.

## Extraction method

Every Turnlock responsibility was first tested for consumer independence. Ruu
then confronted the candidate to expose false Turnlock generalizations,
missing distinctions, or stronger consumer-independent properties. The result
was not derived as an intersection.

The absence of a Ruu Normative Terminology implementation was not evidence
against extraction. Conversely, the existence of a Turnlock behavior was not,
by itself, evidence that proto-ring should own it.

The dispositions are:

- `EXTRACT`: consumer-independent governance owned by the shared contract and
  implementation;
- `TURNLOCK`: Turnlock authority, policy, presentation, path, or integration;
- `EXTERNAL`: maintained language or library behavior already owned elsewhere,
  or an unused implementation detail with no governance responsibility;
- `UNRESOLVED`: insufficient authority to assign ownership.

No candidate remains `UNRESOLVED` in the resulting boundary.

## Complete source candidate inventory

The inventory follows the source file from imports through CLI termination.

### Imported capabilities

- `argparse`: `TURNLOCK` for the concrete checker CLI and its presentation.
- `collections.Counter`: `EXTRACT` as the multiset mechanism used for exact
  occurrence reconciliation; Python owns the collection primitive itself.
- `dataclasses.dataclass`: `EXTERNAL` language support used to realize extracted
  immutable representations.
- `hashlib`: `EXTRACT` for the governed SHA-256 fingerprint operation; Python
  owns the cryptographic primitive.
- `json`: `EXTERNAL`; the import is unused and has no terminology-governance
  responsibility.
- `pathlib.Path`: `TURNLOCK` where it binds repository paths or reads files;
  `EXTERNAL` as a filesystem primitive.
- `re`: `EXTRACT` where it realizes registry, Markdown, expression-boundary,
  definition-cue, and fingerprint checks; Python owns the regex engine.
- `sys`: `TURNLOCK` for CLI streams and process exit status.
- `typing.Any`: `EXTERNAL`; the shared API narrows untrusted inventory values
  from `object` and does not expose a weak generic public type.
- `yaml`: `TURNLOCK` for inventory-file loading and YAML diagnostics. The shared
  engine reconciles already loaded values and does not own a serialization
  format.

### Constants and bindings

- Repository `ROOT`: `TURNLOCK` path binding.
- `DEFAULT_SPEC`: `TURNLOCK` default specification path.
- `DEFAULT_INVENTORY`: `TURNLOCK` default inventory path.
- One start marker and one end marker as the registry-boundary mechanism:
  `EXTRACT`.
- The exact Turnlock marker strings: `TURNLOCK`, supplied through registry
  format policy.
- The six semantic registry columns for concept key, canonical term, canonical
  anchor, accepted aliases, deprecated wording, and term structure: `EXTRACT`.
- Exact displayed header strings: `TURNLOCK`, supplied through registry format
  policy rather than treated as universal wording.
- The allowed-role set: `TURNLOCK`; fail-closed role validity against a
  consumer-supplied immutable set is `EXTRACT`.
- The concept-key regex mechanism: `EXTRACT`; the exact lowercase kebab pattern
  is a consumer-supplied format binding.
- The full-line HTML anchor regex mechanism: `EXTRACT`; the exact `term-`
  namespace and identifier pattern are consumer-supplied format bindings.
- The numbered-section regex and its accepted suffix grammar: `TURNLOCK`; the
  shared parser accepts a consumer section-label function.

### Immutable representations

- `RegistryEntry`: `EXTRACT` as the canonical representation of a concept key,
  preferred term, canonical destination, aliases, and deprecated expressions.
- `RegistryEntry.expressions`: `EXTRACT`; canonical, alias, and deprecated
  expressions all participate in heuristic discovery.
- `Block`: `EXTRACT` as a Markdown block with line, consumer-defined section
  label, heading, and exact text.
- `Occurrence`: `EXTRACT` as concept keys, section label, heading, fingerprint,
  line, and canonical/non-canonical classification.
- `Occurrence.signature`: `EXTRACT`; reconciliation identity is the sorted
  concept tuple, section label, heading, and fingerprint. Line and role are not
  signature fields.

### Text normalization and fingerprinting

- `normalize_block`: `EXTRACT`; all whitespace runs collapse to one ASCII space
  after trimming through split/join behavior.
- `fingerprint_block`: `EXTRACT`; SHA-256 is computed over UTF-8 bytes of the
  normalized block and rendered as lowercase hexadecimal.
- The declared inventory fingerprint metadata values are `TURNLOCK` top-level
  inventory policy. The underlying normalization and digest are shared.

### Registry table helpers and format rules

- Pipe-cell splitting after outer-pipe stripping: `EXTRACT`.
- Trimming every cell: `EXTRACT`.
- Allowing zero or one surrounding backtick pair and rejecting unmatched or
  additional backtick delimiters: `EXTRACT`, strengthening the source's
  permissive plain-cell decoding.
- Treating empty text or a sole undecorated em dash as an empty term list and
  rejecting combined or code-formatted sentinels: `EXTRACT`, strengthening the
  source so the sentinel cannot become an expression.
- Splitting aliases and deprecated wording on semicolons: `EXTRACT`.
- Rejecting empty semicolon items and non-sentinel list cells that decode no
  expression: `EXTRACT`, strengthening the source's ignored-empty behavior so
  no third empty-list encoding exists.
- Requiring every non-empty line inside the registry span to have exactly one
  outer pipe at each edge after surrounding line-whitespace trimming and before
  literal cell splitting: `EXTRACT`, strengthening the source parser so
  malformed edge syntax cannot be silently normalized.
- Requiring exactly one standalone start-marker line and one standalone
  end-marker line in start-before-end order: `EXTRACT`, strengthening the source
  implementation's substring-count check so malformed boundaries cannot define
  an ambiguous registry span.
- Requiring a header, separator, and at least one data row: `EXTRACT`.
- Requiring exact consumer-supplied header order: `EXTRACT`.
- Accepting GFM separator cells with at least three hyphens and optional edge
  colons: `EXTRACT`.
- Requiring exactly six cells in every data row: `EXTRACT`.
- Requiring a non-empty canonical term after code-span decoding and whitespace
  trimming: `EXTRACT`, strengthening the source's pre-decoding check.
- Parsing a non-empty canonical destination from a code-formatted local
  Markdown link: `EXTRACT`; an empty or nonparticipating capture fails closed,
  while the admitted anchor pattern remains format policy.
- Validating concept keys with the supplied pattern: `EXTRACT`.
- Validating canonical anchor links with the supplied pattern: `EXTRACT`.
- Parsing aliases and deprecated expressions into ordered tuples and rejecting
  decoded-empty list items: `EXTRACT`, strengthening the source so an empty
  expression cannot participate in lexical matching.
- Rejecting case-insensitive repetition among one entry's canonical term,
  aliases, and deprecated wording: `EXTRACT`.
- Allowing one expression to appear in distinct concepts, including overloaded
  compound terminology: `EXTRACT`; semantic disambiguation remains human work.
- Requiring a non-empty term-structure classification after whole-cell
  code-span decoding and whitespace trimming: `EXTRACT` as a registry
  completeness rule. This strengthens the source's pre-decoding check. The
  classification content remains consumer-owned, allows inline code and local
  prose, and is not parsed as a term expression.
- Preserving only governed fields in `RegistryEntry` rather than making term
  structure semantic input to discovery: `EXTRACT`.
- Detecting duplicate concept keys: `EXTRACT`.
- Detecting duplicate canonical destinations: `EXTRACT`.
- Returning accumulated diagnostics rather than raising on ordinary invalid
  registry content: `EXTRACT`.
- Preventing canonical occurrence discovery when registry validation fails or
  caller-supplied entries differ from the document registry: `EXTRACT`, making
  fail-closed composition explicit rather than relying on every caller to
  preserve an error tuple.

### Markdown block and anchor parsing

- Excluding registry content from occurrence discovery: `EXTRACT`.
- Tracking the current heading and a consumer-derived section label:
  `EXTRACT`; Turnlock numbering is not shared.
- Flushing paragraphs on blank lines, headings, anchors, registry boundaries,
  fences, and full-line comment starts: `EXTRACT`.
- Preserving paragraph line number and newline-joined exact text: `EXTRACT`.
- Treating fenced code beginning with triple backticks or tildes as one block,
  including fence lines: `EXTRACT`.
- Closing a fence when a trimmed line starts with the opening fence token:
  `EXTRACT`.
- Leaving an unterminated final fence undiscovered: `EXTRACT` as preserved
  current heuristic behavior, not a claim that the Markdown is valid.
- Recognizing ATX headings of levels one through six with zero to three leading
  spaces, empty heading text, and optional whitespace-separated closing hashes:
  `EXTRACT`, strengthening the source's narrower heading regex.
- Deriving section labels from heading text: `TURNLOCK` policy, supplied as a
  callback to the shared parser.
- Recognizing only full-line anchors admitted by the consumer anchor pattern:
  `EXTRACT`; exact HTML syntax and namespace remain format policy.
- Preserving every recognized anchor position and rejecting a canonical
  destination with more than one position: `EXTRACT`. This strengthens the
  source's duplicate sentinel so a duplicated intervening anchor cannot vanish
  from another destination's immediate-binding check.
- Ignoring a line whose left-trimmed text begins with `<!--`: `EXTRACT` as the
  current block heuristic; this is not a complete Markdown parser claim.
- Flushing the final ordinary paragraph at end of input: `EXTRACT`.

### Definition-like heuristic

- Preserving punctuation-bearing expression content while admitting surrounding
  backtick, emphasis, strike, and quote presentation delimiters in definition
  context: `EXTRACT`, strengthening the source's character-deletion heuristic.
- Searching canonical terms, aliases, and deprecated wording: `EXTRACT`.
- Escaping expressions before regex composition: `EXTRACT`.
- Requiring ASCII-alphanumeric expression boundaries that remain ASCII-only
  under case-insensitive expression matching: `EXTRACT`, strengthening the
  source so Unicode case-fold equivalents do not alter boundary membership.
- Applying the same full Unicode case-fold relation to expression uniqueness
  and discovery while mapping complete folded spans back to original-text ASCII
  boundaries: `EXTRACT`, strengthening the source's simple regex folding.
- Allowing optional straight or curly single or double quotes around the
  expression: `EXTRACT`.
- Recognizing `is`, `are`, `mean(s)`, `refer(s) to`, and `is/are defined as/by`
  prose cues: `EXTRACT`.
- Recognizing line-start `:=` and single `=` equation cues while excluding
  `==`: `EXTRACT`.
- Reporting each concept at most once per block even when several expressions
  match: `EXTRACT`.
- Treating all lexical discovery as conservative review support rather than
  semantic proof: `EXTRACT` and mandatory contract boundary.

### Anchor binding and occurrence discovery

- Rejecting every recognized canonical-looking anchor absent from the registry:
  `EXTRACT`.
- Requiring every registry destination to exist: `EXTRACT`.
- Rejecting duplicate canonical anchors: `EXTRACT`.
- Requiring canonical anchors to satisfy a boolean consumer
  canonical-location predicate: `EXTRACT`; non-boolean policy output fails
  closed, while Turnlock's Section 2 predicate is `TURNLOCK`.
- Binding an anchor to the first later parsed block in the same consumer section
  label: `EXTRACT`.
- Rejecting an anchor with no following same-section block: `EXTRACT`.
- Rejecting an intervening recognized anchor before the bound block: `EXTRACT`.
- Requiring the bound block to heuristically define that entry's own canonical
  concept: `EXTRACT`.
- Emitting one canonical occurrence per successfully bound registry entry:
  `EXTRACT`.
- Excluding canonical concepts from non-canonical discovery in their bound
  block while still allowing other concepts in that block: `EXTRACT`.
- Sorting concept keys in every occurrence representation, including
  consumer-constructed occurrences: `EXTRACT`.
- Fingerprinting exact block text through the shared normalization rule:
  `EXTRACT`.
- Preserving source line only as a diagnostic location, not signature identity:
  `EXTRACT`.

### Roles and occurrence records

- Mapping canonical occurrences to `canonical-definition`: `TURNLOCK` role
  policy, not universal vocabulary.
- Mapping sections beginning 0, 3, 5, and 8 to intent, obligation, implication,
  and synopsis: `TURNLOCK`.
- Mapping every other section to reference: `TURNLOCK`.
- Resolving a role through consumer policy: `EXTRACT`.
- Serializing concepts, section, heading, resolved role, and fingerprint into an
  inventory occurrence mapping: `EXTRACT`.
- Omitting line and canonical flags from the maintained occurrence mapping:
  `EXTRACT`; those properties are represented through role policy and
  discovery diagnostics rather than occurrence signature.

### Inventory document policy and reconciliation

- Requiring a YAML mapping: `TURNLOCK` serialization and top-level document
  policy. The shared reconciler accepts an occurrence collection as `object`
  and validates it structurally.
- Requiring `schema_version: 1`: `TURNLOCK`.
- Requiring the exact non-authoritative authority declaration: `TURNLOCK`.
- Requiring the Turnlock specification source path: `TURNLOCK`.
- Requiring the exact top-level fingerprint policy declaration: `TURNLOCK`.
- Requiring the `occurrences` value to be a list: `EXTRACT` once that value is
  supplied to the reconciler.
- Requiring each occurrence to be a mapping: `EXTRACT`.
- Requiring a non-empty string concept list: `EXTRACT`.
- Sorting concepts before signature construction: `EXTRACT`.
- Rejecting references to unknown registry concept keys: `EXTRACT`.
- Validating the role against consumer-supplied allowed roles: `EXTRACT`.
- Validating a lowercase 64-character SHA-256 fingerprint: `EXTRACT`.
- Requiring present string section and heading values before signature
  comparison: `EXTRACT`, strengthening the source's coercion behavior so
  malformed mandatory location fields fail closed.
- Counting actual and recorded signatures as multisets: `EXTRACT`.
- Reconciling recorded roles against consumer-resolved roles as an
  order-independent multiset for each equal-multiplicity occurrence signature:
  `EXTRACT`, strengthening the source's first-match behavior when role inputs
  intentionally excluded from signature differ.
- Detecting maintained occurrence copies that exceed actual signature
  multiplicity: `EXTRACT`; this makes the requested duplicate-inventory
  responsibility direct without rejecting valid discovered signature
  collisions.
- Reporting every unmatched actual copy as unreviewed: `EXTRACT`.
- Reporting every unmatched maintained copy as stale: `EXTRACT`.
- Including concepts, section, heading, and discovered lines in diagnostics:
  `EXTRACT`; exact consumer CLI framing remains local.

### Path and CLI orchestration

- Reading the specification and inventory from default repository paths:
  `TURNLOCK`.
- Treating a missing inventory as `None` before validation: `TURNLOCK`.
- Safe YAML loading and YAML parse diagnostics: `TURNLOCK` for the current
  checker; PyYAML owns parsing semantics.
- `--spec`, `--inventory`, and `--list-occurrences`: `TURNLOCK` CLI.
- Printing YAML occurrence suggestions: `TURNLOCK` presentation.
- Printing errors to stderr in list mode and stdout in check mode: `TURNLOCK`.
- Exit codes zero and one: `TURNLOCK` CLI contract.
- Success and failure banners, including the semantic-proof disclaimer:
  `TURNLOCK` presentation. The underlying non-proof rule is `EXTRACT`.
- Python script entry-point termination through `sys.exit`: `TURNLOCK`.

## Review-derived correction records

These records classify material pre-publication findings from adversarial review.
They alter no consumer product semantics and require no separate work item
because Issue #15 owns their contract, implementation, and regression boundary.

### Ordered standalone registry boundaries

- **Statement:** Substring counts alone permit reversed or embedded boundary
  markers to select an ambiguous registry span.
- **Source and evidence:** Adversarial review of the Issue #15 implementation on
  its unpublished task worktree; direct reversed and embedded-marker fixtures.
- **Existing authority:** Issue #15 requires structured registry parsing and
  missing or duplicated boundary coverage; the consumer source names distinct
  start and end boundaries.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  `verification-or-qualification-evidence`, and
  `repository-governance-or-documentation`.
- **Related discoveries or consequences:** None.
- **Required authority:** None beyond the current extraction acceptance boundary.
- **Next action:** Require one ordered pair of standalone marker lines and retain
  regression coverage.

### Non-empty decoded registry values

- **Statement:** Pre-decoding non-empty checks can admit an empty canonical term,
  alias, deprecated expression, or term structure after backtick removal.
- **Source and evidence:** Adversarial review plus decoded-empty registry
  fixtures on the unpublished Issue #15 implementation.
- **Existing authority:** The extracted registry model requires non-empty values
  for mandatory fields and no empty governed expression.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** None.
- **Required authority:** None.
- **Next action:** Validate decoded values, exclude empty expressions, and retain
  positive valid-registry coverage.

### Mandatory inventory location shape

- **Statement:** Coercing absent or non-string section and heading values can
  hide malformed maintained occurrence records.
- **Source and evidence:** Adversarial review plus missing and non-string field
  fixtures on the unpublished Issue #15 implementation.
- **Existing authority:** The shared occurrence record requires explicit string
  location fields and fail-closed malformed mandatory information.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** None.
- **Required authority:** None.
- **Next action:** Reject malformed location fields before signature matching.

### Canonical concept ordering

- **Statement:** Consumer-constructed occurrences can retain unsorted concept
  tuples even though signatures require sorted concept keys.
- **Source and evidence:** Adversarial review plus opposite-order occurrence
  fixtures on the unpublished Issue #15 implementation.
- **Existing authority:** The occurrence signature contract defines sorted
  concept keys.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `architecture-or-implementation` and
  `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** None.
- **Required authority:** None.
- **Next action:** Normalize concept tuples in the immutable occurrence model.

### Complete anchor-position retention

- **Statement:** Replacing duplicated anchor positions with one sentinel can
  hide that anchor from another destination's intervening-anchor check.
- **Source and evidence:** Adversarial review plus a duplicated intervening
  anchor fixture on the unpublished Issue #15 implementation.
- **Existing authority:** Canonical binding forbids every recognized intervening
  anchor and separately rejects duplicate destinations.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** None.
- **Required authority:** None.
- **Next action:** Preserve all positions and use all of them for binding checks.

### Restricted table grammar clarity

- **Statement:** Calling the parser a GFM-table parser without qualification
  overstates a deliberately restricted literal-pipe grammar.
- **Source and evidence:** Adversarial comparison of contract wording with the
  extracted parser behavior.
- **Existing authority:** The source mechanism requires outer pipes, literal
  splitting, code-span stripping, semicolon lists, an em-dash empty sentinel,
  and a GFM separator row; it does not implement complete Markdown table
  parsing.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract` and
  `repository-governance-or-documentation`.
- **Related discoveries or consequences:** None.
- **Required authority:** None.
- **Next action:** Specify the restricted grammar exactly rather than inventing
  broader parser behavior or a new dependency.

### Exact outer-pipe enforcement

- **Statement:** Stripping arbitrary edge pipes can accept missing or repeated
  outer delimiters despite the restricted grammar.
- **Source and evidence:** Second adversarial review plus missing-edge and
  repeated-edge fixtures on the unpublished Issue #15 implementation.
- **Existing authority:** The clarified registry grammar requires exactly one
  outer pipe at each edge of every non-empty registry line.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** Restricted table grammar clarity.
- **Required authority:** None.
- **Next action:** Validate edge delimiters before splitting cells.

### Fail-closed discovery composition

- **Statement:** Returning partial entries with registry errors allows a caller
  to invoke discovery unless discovery independently validates its registry
  input.
- **Source and evidence:** Second adversarial review plus an invalid-registry
  discovery fixture on the unpublished Issue #15 implementation.
- **Existing authority:** Invalid registry structure prevents trustworthy
  canonical discovery under the shared failure contract.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** Non-empty decoded registry values.
- **Required authority:** None.
- **Next action:** Revalidate registry content at discovery and reject mismatched
  caller-supplied entries.

### Boolean canonical-location policy

- **Statement:** General truthiness permits a non-boolean consumer callback
  value to authorize canonical location accidentally.
- **Source and evidence:** Second adversarial review plus a non-boolean policy
  fixture on the unpublished Issue #15 implementation.
- **Existing authority:** Canonical location is a consumer predicate, not a
  coercible value.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `architecture-or-implementation`,
  `integration-or-conformance`, and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** None.
- **Required authority:** None.
- **Next action:** Require an actual boolean callback result and fail otherwise.

### ASCII boundary isolation from Unicode case folding

- **Statement:** Applying case-insensitive regex flags to ASCII boundary classes
  makes selected Unicode letters behave as ASCII boundary members.
- **Source and evidence:** Final adversarial review plus a Kelvin-sign boundary
  fixture on the unpublished Issue #15 implementation.
- **Existing authority:** The extracted boundary contract is explicitly ASCII
  alphanumeric while expression matching is separately case-insensitive.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `architecture-or-implementation` and
  `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** None.
- **Required authority:** None.
- **Next action:** Scope case folding to expressions and lexical cues, not the
  boundary lookarounds.

### Immutable allowed-role collection shape

- **Statement:** A string can satisfy element checks intended for a role set and
  make substring membership look like allowed-role membership.
- **Source and evidence:** Final adversarial review plus a string-valued role-set
  fixture on the unpublished Issue #15 implementation.
- **Existing authority:** Consumer role policy requires a non-empty set of
  non-empty role strings.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `architecture-or-implementation`,
  `integration-or-conformance`, and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** None.
- **Required authority:** None.
- **Next action:** Require the immutable set representation declared by the
  public API before validating members.

### Non-empty anchor capture identity

- **Statement:** A syntactically matching regex may have an empty or
  nonparticipating first capture and therefore produce no canonical destination.
- **Source and evidence:** Final adversarial review plus empty and optional
  capture fixtures on the unpublished Issue #15 implementation.
- **Existing authority:** Every registry entry and recognized anchor requires a
  non-empty canonical destination.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** None.
- **Required authority:** None.
- **Next action:** Validate captured destination values at registry and Markdown
  parsing boundaries.

### Fail-closed backtick grammar

- **Statement:** Unmatched or repeated surrounding backticks can be admitted as
  literal expression content and then become unmatchable after Markdown
  emphasis stripping.
- **Source and evidence:** Adversarial review plus unmatched and repeated
  delimiter fixtures on the unpublished Issue #15 implementation.
- **Existing authority:** The restricted registry grammar allows at most one
  surrounding backtick pair and malformed mandatory input fails closed.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** Restricted table grammar clarity.
- **Required authority:** None.
- **Next action:** Validate delimiters before plain-cell decoding or discovery.

### ATX heading normalization

- **Statement:** A narrow unindented heading regex can misclassify valid ATX
  indentation, empty headings, and closing hash sequences as paragraph text or
  heading content.
- **Source and evidence:** Adversarial review plus indented, closed, and empty
  heading fixtures on the unpublished Issue #15 implementation.
- **Existing authority:** The shared Markdown structural model claims ATX
  heading discovery rather than a consumer-specific heading subset.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** None.
- **Required authority:** None.
- **Next action:** Normalize the demonstrated ATX forms before consumer section
  interpretation.

### Signature-collision role reconciliation

- **Statement:** Selecting the first occurrence for role validation is incorrect
  when multiple actual occurrences share a signature but have different
  consumer-resolved roles.
- **Source and evidence:** Adversarial review plus canonical/non-canonical
  signature-collision fixtures on the unpublished Issue #15 implementation.
- **Existing authority:** Signature excludes canonical status and line while
  role policy may use them; occurrence reconciliation is explicitly multiset
  based.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** Duplicate occurrence reconciliation.
- **Required authority:** None.
- **Next action:** Reconcile expected and maintained roles as order-independent
  multisets within each equal-multiplicity signature, route excess copies only
  through count diagnostics, and report all candidate lines for unmatched copies.

### Em-dash sentinel isolation

- **Statement:** Combining the em-dash empty-list sentinel with expressions or
  code-formatting it can turn the sentinel into a discoverable expression.
- **Source and evidence:** Adversarial review plus combined and code-formatted
  sentinel fixtures on the unpublished Issue #15 implementation.
- **Existing authority:** The restricted grammar reserves the em dash for one
  empty-list representation.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** Fail-closed backtick grammar.
- **Required authority:** None.
- **Next action:** Reject every non-sole or decorated sentinel and exclude it
  from decoded expressions.

### Controlled structural-anchor diagnostics

- **Statement:** A malformed consumer anchor capture can escape discovery as an
  exception rather than a structural diagnostic.
- **Source and evidence:** Adversarial review plus an empty-capture discovery
  fixture on the unpublished Issue #15 implementation.
- **Existing authority:** Structural anchor failures return controlled
  diagnostics from occurrence discovery.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `architecture-or-implementation`,
  `integration-or-conformance`, and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** Non-empty anchor capture identity.
- **Required authority:** None.
- **Next action:** Convert low-level structural parser policy failures into
  discovery diagnostics.

### Independent lexical and role multiplicity oracles

- **Statement:** Count-only lexical tests and one-of-each role tests can pass
  after compensating false positives or a set-based implementation loses
  multiplicity.
- **Source and evidence:** Adversarial review of boundary, equation, and role
  collision tests on the unpublished Issue #15 worktree.
- **Existing authority:** Independent regression coverage must detect expression
  boundaries, equation exclusions, and exact multiset reconciliation.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** ASCII boundary isolation and
  signature-collision role reconciliation.
- **Required authority:** None.
- **Next action:** Use fixed independent fingerprints for positive and negative
  lexical blocks and repeated-role collision vectors for role multiplicity.

### Unique empty-list encodings

- **Statement:** Semicolon-only or empty-item lists can create ungoverned empty
  list encodings beyond an empty cell or sole em dash.
- **Source and evidence:** Adversarial review plus semicolon-only, repeated,
  interior-empty, and trailing-empty fixtures on the unpublished Issue #15
  implementation.
- **Existing authority:** The restricted grammar defines exactly two empty-list
  representations and requires every semicolon item to be an expression.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** Em-dash sentinel isolation.
- **Required authority:** None.
- **Next action:** Reject every non-sentinel list cell containing an empty item
  or no decoded expression.

### Unified Unicode caseless relation

- **Statement:** Simple regex case-insensitivity does not perform the full
  Unicode case-fold expansions already used for registry uniqueness.
- **Source and evidence:** Adversarial review plus a `Straße`/`STRASSE`
  expansion fixture on the unpublished Issue #15 implementation.
- **Existing authority:** Registry uniqueness and lexical discovery govern the
  same expressions, while original-text boundaries remain ASCII-defined.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** ASCII boundary isolation from Unicode
  case folding.
- **Required authority:** None.
- **Next action:** Match folded complete-character spans and map them back to
  original characters before boundary evaluation.

### Complete curly-quote delimiter set

- **Statement:** Curly double quotes without curly single quotes make quote
  handling structurally incomplete for prose and equations.
- **Source and evidence:** Adversarial review plus fixed-fingerprint curly-single
  prose and equation fixtures on the unpublished Issue #15 implementation.
- **Existing authority:** Quote delimiters are lexical presentation, not
  consumer semantics, and both straight and curly forms are admitted.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `architecture-or-implementation` and
  `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** Independent lexical oracles.
- **Required authority:** None.
- **Next action:** Include curly single quotes in both prose and equation
  contexts.

### Punctuation-preserving expression matching

- **Statement:** Deleting Markdown punctuation from source text makes literal
  punctuation-bearing terms unmatchable and can create false punctuation-free
  matches.
- **Source and evidence:** Adversarial review plus fixed-fingerprint canonical,
  alias, deprecated, and false punctuation-free fixtures on the unpublished
  Issue #15 implementation.
- **Existing authority:** Registry expressions are exact lexical candidates;
  presentation delimiters may surround them but are not part of unrelated
  expression normalization.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** Unified Unicode caseless relation.
- **Required authority:** None.
- **Next action:** Match exact expression punctuation and tolerate presentation
  delimiters only in the surrounding definition context.

### Presentation-decorated em-dash rejection

- **Statement:** Emphasis or strike presentation can disguise the reserved em
  dash and admit it as an expression.
- **Source and evidence:** Adversarial review plus backtick, asterisk,
  underscore, tilde, straight-quote, and curly-quote sentinel fixtures in both
  alias and deprecated columns on the unpublished Issue #15 implementation.
- **Existing authority:** Only one undecorated em dash may represent an empty
  list; every decorated form is invalid.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** Em-dash sentinel isolation.
- **Required authority:** None.
- **Next action:** Detect and exclude the sentinel after removing every
  recognized presentation wrapper.

### Term-structure inline code

- **Statement:** Term structure must allow consumer inline code.
- **Source and evidence:** Turnlock adoption at
  `bfc64d8a1b8f219de22467d0f200c24ac97fa840` against proto-ring provider
  `11123355e1e36b64d11f05b9ee627af5e3ba12d1`; the current Turnlock registry
  produced 71 failures beginning at row 9.
- **Existing authority:** The shared contract requires non-empty term structure
  but states that its classification content is not interpreted. Consumers own
  that content.
- **Semantic disposition:** `no-normative-impact`.
- **Affected layers:** `normative-contract`, `architecture-or-implementation`,
  `integration-or-conformance`, and `verification-or-qualification-evidence`.
- **Related discoveries or consequences:** None.
- **Required authority:** None.
- **Next action:** Allow inline code while retaining whole-cell decoding and the
  non-empty guard.

## Ruu confrontation

Ruu has no canonical terminology registry, `term-*` anchor namespace,
terminology inventory, or equivalent checker at the confronted revision. That
absence did not reject consumer-independent Turnlock mechanics.

Ruu's specification demonstrates a different organization:

- Section 2 is a product mental model rather than an independently declared
  universal terminology-registry location.
- Definitions occur in product-owned prose without Turnlock's registry table or
  review-role taxonomy.
- Accepted decisions such as ADR-025 and ADR-034 deliberately rename concepts
  and preserve supersession history. That semantic decision history cannot be
  inferred or proved by lexical heuristics.
- Ruu uses numbered headings, but this superficial similarity does not justify
  universal numeric section parsing or Turnlock's role mapping.

The confrontation therefore requires consumer-supplied marker/header patterns,
section-label extraction, canonical-location predicates, allowed roles, and role
resolution. It falsifies any generic contract that mandates Turnlock paths,
Section 2, Turnlock numbering, Turnlock roles, Turnlock registry contents, or a
specific product-terminology organization.

Ruu demonstrates no stronger consumer-independent lexical mechanism in this
scope. Its terminology-renaming decisions strengthen the authority boundary:
semantic renaming and supersession remain consumer decisions outside heuristic
governance.

## Resulting shared boundary

Proto-ring owns:

- structured registry parsing under an explicit consumer format;
- immutable canonical terminology representations;
- generic Markdown blocks and canonical-anchor binding;
- conservative definition-like prose and equation discovery;
- boundary-aware canonical, alias, and deprecated-expression matching;
- normalized SHA-256 fingerprints and occurrence signatures;
- generic occurrence-record construction under consumer role policy;
- fail-closed occurrence inventory reconciliation; and
- the mandatory statement that these mechanics are heuristic governance, not
  semantic proof.

Turnlock retains:

- exact files and paths;
- exact registry markers, displayed headers, key/anchor patterns, and section
  label grammar;
- Section 2 canonical-location policy;
- role vocabulary and the Section 0, 3, 5, 8 mapping;
- registry and inventory content;
- top-level inventory schema, authority, source, and fingerprint declarations;
- YAML file loading;
- CLI, diagnostics framing, and Repository Integrity membership; and
- every product-semantic or formal meaning.

Ruu retains all corresponding product terminology, specification organization,
accepted renaming decisions, authority, qualification, and any future adoption
policy. Ruu was not modified, and no Ruu adoption Issue was created.
