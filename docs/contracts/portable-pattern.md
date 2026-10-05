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

A character that is not pattern metasyntax matches itself exactly.

A backslash may escape a pattern metacharacter so that the escaped character is
matched literally.

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

A leading `^` immediately after `[` negates the class.

Class ranges are allowed only when both endpoints are ASCII scalar values and
the start code point is not greater than the end code point.

Characters that are syntactically significant inside a class may be escaped
literally.

POSIX character classes, Unicode property classes, collating elements, locale
classes, class subtraction/intersection, and implementation-specific extensions
are forbidden.

### Concatenation and alternation

Adjacent pattern atoms concatenate.

`|` denotes alternation.

Alternation precedence is lower than concatenation.

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

where `m` and `n` are canonical non-negative ASCII decimal integers and,
for `{m,n}`, `m <= n`.

Lazy, possessive, conditional, or implementation-specific quantifiers are
forbidden.

A quantifier applies to the immediately preceding atom or group.

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

## Captures

A successful full match returns the exact input substring captured by each
participating capture group.

An optional capture that does not participate is absent; it is distinct from a
participating capture of the empty string.

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
