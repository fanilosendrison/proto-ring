---
okf_version: "1.0"
kind: "KnowledgeAsset"
asset_type: "governance-contract"
domain: "proto-ring-portable-pattern"
severity: "strict"
name: "Canonical Portable Pattern contract"
---

# Canonical Portable Pattern contract

## Purpose

Canonical Portable Pattern defines the bounded regular-pattern language used
when a proto-ring contract consumes a consumer-owned textual pattern but the
governed behavior must remain reproducible across implementation languages.

It exists to prevent an implementation's native regular-expression engine from
becoming hidden semantic authority.

The consumer owns the pattern value and the domain meaning assigned to a match.
This contract owns only the admitted pattern syntax, exact full-match semantics,
and capture behavior.

## Text domain

Patterns and candidate inputs are Unicode scalar-value strings.

Matching is exact by Unicode scalar value. The pattern mechanism performs no
Unicode normalization, case folding, locale transformation, trimming, path
normalization, or encoding conversion.

A consuming contract that needs a separate caseless relation must define that
relation independently rather than enabling a pattern-engine flag.

## Matching operation

Canonical Portable Pattern exposes one semantic matching operation:

```text
full_match(pattern, input)
```

Success requires the pattern to cover the complete input string.

There is no generic search, find-first, replacement, substitution, split, or
implicit multiline mode.

The start anchor `^` denotes the exact beginning of the input. The end anchor
`$` denotes the exact end of the input. There is no special before-final-LF
interpretation.

## Admitted grammar

The model-version-1 grammar admits only the following constructs.

### Literal characters and escaping

Outside a character class, pattern metasyntax is exactly:

```text
\ ^ $ . [ ] ( ) | ? * + { }
```

Every other Unicode scalar matches itself exactly.

A backslash may escape exactly one of those metacharacters so that the escaped
character is matched literally. A backslash before any other character is
invalid; Canonical Portable Pattern has no implementation-defined escape
vocabulary.

The dot `.` is reserved metasyntax in model version 1 but has no wildcard
meaning. An unescaped dot is therefore invalid. A literal dot is written
`\.`.

Likewise, a metacharacter that is not valid in its grammatical position is an
error rather than an implementation-specific literal fallback.

There are no shorthand character classes such as `\d`, `\w`, or `\s`,
no Unicode-property escapes, and no implementation-defined escape vocabulary.

### Character classes

A character class is delimited by `[` and `]`.

It may contain exact literal characters and inclusive ASCII ranges such as:

```text
[a-z]
[A-Z]
[0-9]
```

A leading `^` immediately after `[` negates the class. Elsewhere inside the
class, `^` is a literal character.

Inside a class:

- `]` terminates the class and may appear literally only when escaped;
- `\` begins an escape and a literal backslash is written `\\`;
- `-` denotes a range only when it occurs between two class literals and is
  neither the first nor final class item;
- a first or final unescaped `-` is literal; and
- `]`, `\`, `-`, and `^` may be escaped to force literal meaning.

Outside-class metacharacters such as `.`, `(`, `)`, `{`, `}`, `|`,
`?`, `*`, and `+` have no special meaning inside a class unless covered by
the rules above; they are literal class members.

Class ranges are allowed only when both endpoints are ASCII scalar values and
the start code point is not greater than the end code point.

POSIX character classes, Unicode property classes, collating elements, locale
classes, class subtraction/intersection, and implementation-specific extensions
are forbidden.

### Concatenation and alternation

Adjacent pattern atoms concatenate.

`|` denotes alternation.

Alternation precedence is lower than concatenation.

Alternatives are ordered. When more than one alternative can participate in a
successful full match, alternatives are attempted from left to right in source
order.

### Groups and captures

The admitted group forms are exactly:

```text
(...)
(?:...)
(?P<name>...)
```

The first form is an ordered capturing group.

The second form is non-capturing.

The third form is an ordered named capturing group. A name must match:

```regex
^[A-Za-z_][A-Za-z0-9_]*$
```

Capture numbering is assigned by the left-to-right order of opening capturing
groups. Named captures participate in that same ordered capture sequence and are
also addressable by exact name.

Duplicate named-capture names are invalid.

Lookahead, lookbehind, backreferences, conditionals, atomic groups, branch-reset
groups, inline flags, comments, recursion, subroutines, and implementation
extensions are forbidden.

### Quantifiers

The admitted greedy quantifiers are exactly:

```text
?
*
+
{m}
{m,}
{m,n}
```

where `m` and `n` are canonical non-negative ASCII decimal integers with
grammar:

```regex
^(0|[1-9][0-9]*)$
```

and, for `{m,n}`, `m <= n`.

Quantifier bounds are mathematical integers and have no contract-defined
machine-width or runtime digit limit.

Lazy, possessive, conditional, or implementation-specific quantifiers are
forbidden.

A quantifier applies to the immediately preceding consuming atom or group.
Anchors are not quantifiable.

An unbounded quantifier (`*`, `+`, or `{m,}`) applied to an expression
that can match the empty string is invalid. Otherwise, unbounded repetition has
a finite maximum for a finite input because every successful repetition consumes
at least one Unicode scalar.

Bounded quantifiers may apply to an empty-matching group because their maximum
repetition count is finite.

### Anchors

`^` and `$` are admitted with the exact semantics defined by this contract.

No other zero-width assertion is admitted.

## Forbidden engine behavior

A conforming implementation MUST NOT silently accept unsupported pattern syntax
by delegating it to a more expressive native regex engine.

Unsupported syntax or malformed pattern structure fails closed through the
consuming proto-ring operation's controlled error boundary.

Native engine differences in Unicode version, locale, line mode, dot behavior,
capture extensions, or escape interpretation must not affect Canonical Portable
Pattern results.

## Match selection and captures

Matching is deterministic under these priorities:

1. the complete input must be consumed;
2. ordered alternatives are attempted left to right;
3. at each quantified atom/group, larger permitted repetition counts are
   attempted before smaller counts;
4. when a larger greedy choice prevents a successful full match, the matcher
   backtracks to the next smaller permitted count and continues under the same
   priorities; and
5. the first successful full-match derivation under those priorities is the
   result.

A successful full match returns the exact input substring captured by each
participating capture group.

When a capturing group participates more than once because it is quantified or
is inside a quantified group, its reported capture is the substring from its
last successful participation in the selected derivation.

A capture that never participates is absent. It is distinct from a participating
capture of the empty string.

A named capture and its positional capture refer to the same participation and
must report the same substring/absence state.

A consuming contract may require a particular named or positional capture and
must fail closed when that required capture is absent or empty.

## Consumer authority boundary

Canonical Portable Pattern does not own:

- the pattern string chosen by a consumer;
- filename, identity, anchor, or key vocabulary;
- semantic interpretation of captures;
- product meaning;
- path meaning;
- terminology meaning; or
- policy about which consumer pattern is required.

It owns only portable pattern interpretation for contracts that explicitly
compose it.
