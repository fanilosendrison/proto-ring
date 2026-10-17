"""Execute Canonical Portable Pattern ASTs in deterministic derivation order."""

from __future__ import annotations

from collections.abc import Iterator
from dataclasses import dataclass

from .portable_pattern import (
    _Alternation,
    _CharacterClass,
    _Empty,
    _EndAnchor,
    _Group,
    _Literal,
    _minimum_consumption,
    _Node,
    _Repeat,
    _Sequence,
    _StartAnchor,
)

_CaptureSpan = tuple[int, int] | None
_CaptureState = tuple[_CaptureSpan, ...]
_Result = tuple[int, _CaptureState]


@dataclass
class _BoundedRepeatFrame:
    count: int
    position: int
    captures: _CaptureState
    child_results: Iterator[_Result] | None


def _class_contains(node: _CharacterClass, character: str) -> bool:
    code_point = ord(character)
    member = character in node.literals or any(
        start <= code_point <= end for start, end in node.ranges
    )
    return not member if node.negated else member


def _match_sequence(
    parts: tuple[_Node, ...],
    index: int,
    text: str,
    position: int,
    captures: _CaptureState,
) -> Iterator[_Result]:
    if index == len(parts):
        yield position, captures
        return
    for next_position, next_captures in _match(
        parts[index], text, position, captures
    ):
        yield from _match_sequence(
            parts, index + 1, text, next_position, next_captures
        )


def _match_bounded_nullable_repeat(
    node: _Repeat,
    text: str,
    position: int,
    captures: _CaptureState,
) -> Iterator[_Result]:
    maximum = node.maximum
    if maximum is None:
        raise RuntimeError("bounded nullable repeat requires a finite maximum")
    stack = [_BoundedRepeatFrame(0, position, captures, None)]
    while stack:
        frame = stack[-1]
        if frame.child_results is None:
            frame.child_results = (
                iter(_match(node.child, text, frame.position, frame.captures))
                if frame.count < maximum
                else iter(())
            )
        try:
            next_position, next_captures = next(frame.child_results)
        except StopIteration:
            completed = stack.pop()
            if completed.count >= node.minimum:
                yield completed.position, completed.captures
            continue
        next_count = frame.count + 1
        if next_count <= frame.count or next_count > maximum:
            raise RuntimeError("bounded repeat count invariant failed")
        stack.append(
            _BoundedRepeatFrame(
                next_count, next_position, next_captures, None
            )
        )


def _match_repeat(
    node: _Repeat,
    text: str,
    position: int,
    captures: _CaptureState,
    count: int,
    child_minimum: int,
) -> Iterator[_Result]:
    if child_minimum <= 0:
        raise RuntimeError("consuming repeat requires positive minimum consumption")
    remaining = len(text) - position
    if count + remaining // child_minimum < node.minimum:
        return
    can_repeat = node.maximum is None or count < node.maximum
    if can_repeat and child_minimum <= remaining:
        for next_position, next_captures in _match(
            node.child, text, position, captures
        ):
            if next_position <= position:
                raise RuntimeError("consuming repeat child did not advance")
            yield from _match_repeat(
                node,
                text,
                next_position,
                next_captures,
                count + 1,
                child_minimum,
            )
    if count >= node.minimum:
        yield position, captures


def _match(
    node: _Node,
    text: str,
    position: int,
    captures: _CaptureState,
) -> Iterator[_Result]:
    if isinstance(node, _Empty):
        yield position, captures
    elif isinstance(node, _Literal):
        if position < len(text) and text[position] == node.value:
            yield position + 1, captures
    elif isinstance(node, _CharacterClass):
        if position < len(text) and _class_contains(node, text[position]):
            yield position + 1, captures
    elif isinstance(node, _StartAnchor):
        if position == 0:
            yield position, captures
    elif isinstance(node, _EndAnchor):
        if position == len(text):
            yield position, captures
    elif isinstance(node, _Sequence):
        yield from _match_sequence(node.parts, 0, text, position, captures)
    elif isinstance(node, _Alternation):
        for alternative in node.alternatives:
            yield from _match(alternative, text, position, captures)
    elif isinstance(node, _Group):
        start = position
        for end, child_captures in _match(node.child, text, position, captures):
            if node.capture_index is None:
                yield end, child_captures
                continue
            updated = list(child_captures)
            updated[node.capture_index - 1] = (start, end)
            yield end, tuple(updated)
    else:
        child_minimum = _minimum_consumption(node.child)
        if node.maximum is not None and child_minimum == 0:
            yield from _match_bounded_nullable_repeat(
                node, text, position, captures
            )
        else:
            yield from _match_repeat(
                node, text, position, captures, 0, child_minimum
            )


def match_full(
    root: _Node, text: str, capture_count: int
) -> tuple[_CaptureSpan, ...] | None:
    """Return the first complete derivation's positional capture spans."""

    captures: _CaptureState = (None,) * capture_count
    for position, result_captures in _match(root, text, 0, captures):
        if position == len(text):
            return result_captures
    return None
