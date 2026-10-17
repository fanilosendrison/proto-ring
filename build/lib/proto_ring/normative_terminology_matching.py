"""Unicode-caseless expression matching with original-text ASCII boundaries."""

from __future__ import annotations

from typing import TYPE_CHECKING

from . import _normative_terminology_unicode as unicode14

if TYPE_CHECKING:
    from .normative_terminology import MarkdownBlock, RegistryEntry

_QUOTES = "\"'“”‘’"
_PRESENTATION_DELIMITERS = "`*_~"
_ALL_PRESENTATION_DELIMITERS = _QUOTES + _PRESENTATION_DELIMITERS
_CUES = (
    ("is",),
    ("are",),
    ("mean",),
    ("means",),
    ("refer", "to"),
    ("refers", "to"),
    ("is", "defined", "as"),
    ("is", "defined", "by"),
    ("are", "defined", "as"),
    ("are", "defined", "by"),
)


def _is_ascii_alphanumeric(character: str) -> bool:
    return (
        "A" <= character <= "Z"
        or "a" <= character <= "z"
        or "0" <= character <= "9"
    )


def _is_ascii_cue_word_character(character: str) -> bool:
    return _is_ascii_alphanumeric(character) or character == "_"


def _ascii_equal_at(text: str, position: int, expected: str) -> int | None:
    if position + len(expected) > len(text):
        return None
    for offset, expected_character in enumerate(expected):
        actual = text[position + offset]
        if "A" <= actual <= "Z":
            actual = chr(ord(actual) + 32)
        elif not "a" <= actual <= "z":
            return None
        if actual != expected_character:
            return None
    return position + len(expected)


def _cue_at(text: str, position: int) -> bool:
    for words in _CUES:
        current = position
        matched = True
        for index, word in enumerate(words):
            if index:
                separator_start = current
                while current < len(text) and unicode14.is_whitespace(text[current]):
                    current += 1
                if current == separator_start:
                    matched = False
                    break
            word_end = _ascii_equal_at(text, current, word)
            if word_end is None:
                matched = False
                break
            current = word_end
        if matched and (
            current == len(text)
            or not _is_ascii_cue_word_character(text[current])
        ):
            return True
    return False


def _casefold_spans(text: str, expression: str) -> list[tuple[int, int]]:
    needle = unicode14.casefold(expression)
    if not needle:
        return []

    folded_parts: list[str] = []
    boundaries: dict[int, int] = {}
    offset = 0
    for index, character in enumerate(text):
        boundaries[offset] = index
        folded = unicode14.casefold(character)
        folded_parts.append(folded)
        offset += len(folded)
    boundaries[offset] = len(text)
    folded_text = "".join(folded_parts)

    spans: list[tuple[int, int]] = []
    search_from = 0
    while True:
        folded_start = folded_text.find(needle, search_from)
        if folded_start < 0:
            return spans
        folded_end = folded_start + len(needle)
        if folded_start in boundaries and folded_end in boundaries:
            start, end = boundaries[folded_start], boundaries[folded_end]
            left_ok = start == 0 or not _is_ascii_alphanumeric(text[start - 1])
            right_ok = end == len(text) or not _is_ascii_alphanumeric(text[end])
            if left_ok and right_ok:
                spans.append((start, end))
        search_from = folded_start + 1


def _prose_definition_follows(text: str, end: int) -> bool:
    position = end
    while (
        position < len(text)
        and text[position] in _ALL_PRESENTATION_DELIMITERS
    ):
        position += 1
    whitespace_start = position
    while position < len(text) and unicode14.is_whitespace(text[position]):
        position += 1
    return position > whitespace_start and _cue_at(text, position)


def _equation_prefix_matches(text: str, line_start: int, start: int) -> bool:
    position = line_start
    while position < start and unicode14.is_whitespace(text[position]):
        position += 1
    while position < start and text[position] in _ALL_PRESENTATION_DELIMITERS:
        position += 1
    return position == start


def _equation_suffix_matches(text: str, end: int) -> bool:
    position = end
    while (
        position < len(text)
        and text[position] in _ALL_PRESENTATION_DELIMITERS
    ):
        position += 1
    while position < len(text) and unicode14.is_whitespace(text[position]):
        position += 1
    if text.startswith(":=", position):
        return True
    return (
        position < len(text)
        and text[position] == "="
        and not text.startswith("==", position)
    )


def _is_definition_context(text: str, start: int, end: int) -> bool:
    if _prose_definition_follows(text, end):
        return True
    line_start = text.rfind("\n", 0, start) + 1
    return _equation_prefix_matches(
        text, line_start, start
    ) and _equation_suffix_matches(text, end)


def definition_concepts(
    block: MarkdownBlock, entries: list[RegistryEntry]
) -> set[str]:
    text = block.text
    concepts: set[str] = set()
    for entry in entries:
        if any(
            _is_definition_context(text, start, end)
            for expression in entry.expressions
            for start, end in _casefold_spans(text, expression)
        ):
            concepts.add(entry.key)
    return concepts
