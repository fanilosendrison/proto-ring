"""Unicode-caseless expression matching with original-text ASCII boundaries."""

from __future__ import annotations

import re
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .normative_terminology import MarkdownBlock, RegistryEntry

_QUOTES = "\"'“”‘’"
_PRESENTATION_DELIMITERS = "`*_~"
_CUE = re.compile(
    r"(?i:(?:is\b|are\b|means?\b|refers?\s+to\b|"
    r"(?:is|are)\s+defined\s+(?:as|by)\b))"
)


def _is_ascii_alphanumeric(character: str) -> bool:
    return (
        "A" <= character <= "Z"
        or "a" <= character <= "z"
        or "0" <= character <= "9"
    )


def _casefold_spans(text: str, expression: str) -> list[tuple[int, int]]:
    needle = expression.casefold()
    if not needle:
        return []

    folded_parts: list[str] = []
    boundaries: dict[int, int] = {}
    offset = 0
    for index, character in enumerate(text):
        boundaries[offset] = index
        folded = character.casefold()
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


def _is_definition_context(text: str, start: int, end: int) -> bool:
    delimiters = re.escape(_QUOTES + _PRESENTATION_DELIMITERS)
    after = text[end:]
    prose_delimiters = re.match(rf"[{delimiters}]*\s+", after)
    if prose_delimiters and _CUE.match(after[prose_delimiters.end() :]):
        return True

    line_start = text.rfind("\n", 0, start) + 1
    before = text[line_start:start]
    equation_prefix = re.fullmatch(rf"\s*[{delimiters}]*", before)
    equation_suffix = re.match(rf"[{delimiters}]*\s*(?::=|=(?!=))", after)
    return equation_prefix is not None and equation_suffix is not None


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
