"""Parse and expose the Canonical Portable Pattern operation."""
from __future__ import annotations

from dataclasses import dataclass, field
from types import MappingProxyType
from typing import Mapping, TypeAlias

__all__ = ["PortablePattern", "PortablePatternError", "PortablePatternMatch", "full_match"]

_METASYNTAX = frozenset(r"\^$.[]()|?*+{}")
_CLASS_ESCAPES = frozenset("]\\-^")


class PortablePatternError(ValueError):
    """Report malformed patterns and values outside the scalar-string domain."""


@dataclass(frozen=True)
class _Empty:
    pass

@dataclass(frozen=True)
class _Literal:
    value: str

@dataclass(frozen=True)
class _CharacterClass:
    negated: bool
    literals: tuple[str, ...]
    ranges: tuple[tuple[int, int], ...]

@dataclass(frozen=True)
class _StartAnchor:
    pass

@dataclass(frozen=True)
class _EndAnchor:
    pass

@dataclass(frozen=True)
class _Sequence:
    parts: tuple[_Node, ...]

@dataclass(frozen=True)
class _Alternation:
    alternatives: tuple[_Node, ...]

@dataclass(frozen=True)
class _Group:
    child: _Node
    capture_index: int | None
    capture_name: str | None

@dataclass(frozen=True)
class _Repeat:
    child: _Node
    minimum: int
    maximum: int | None

_Node: TypeAlias = (
    _Empty
    | _Literal
    | _CharacterClass
    | _StartAnchor
    | _EndAnchor
    | _Sequence
    | _Alternation
    | _Group
    | _Repeat
)
_CaptureSpan: TypeAlias = tuple[int, int] | None


@dataclass
class _Frame:
    alternatives: list[_Node] = field(default_factory=list)
    sequence: list[_Node] = field(default_factory=list)
    capture_index: int | None = None
    capture_name: str | None = None


def _validate_scalar_string(value: object, label: str) -> str:
    if not isinstance(value, str):
        raise PortablePatternError(f"{label} must be a string")
    if any(0xD800 <= ord(character) <= 0xDFFF for character in value):
        raise PortablePatternError(f"{label} contains a surrogate code point")
    return value


def _sequence(parts: list[_Node]) -> _Node:
    if not parts:
        return _Empty()
    if len(parts) == 1:
        return parts[0]
    return _Sequence(tuple(parts))


def _finish_frame(frame: _Frame) -> _Node:
    alternatives = (*frame.alternatives, _sequence(frame.sequence))
    if len(alternatives) == 1:
        return alternatives[0]
    return _Alternation(alternatives)


def _nullable(node: _Node) -> bool:
    if isinstance(node, (_Empty, _StartAnchor, _EndAnchor)):
        return True
    if isinstance(node, (_Literal, _CharacterClass)):
        return False
    if isinstance(node, _Sequence):
        return all(_nullable(part) for part in node.parts)
    if isinstance(node, _Alternation):
        return any(_nullable(part) for part in node.alternatives)
    if isinstance(node, _Group):
        return _nullable(node.child)
    return node.minimum == 0 or _nullable(node.child)


def _minimum_consumption(node: _Node) -> int:
    if isinstance(node, (_Empty, _StartAnchor, _EndAnchor)):
        return 0
    if isinstance(node, (_Literal, _CharacterClass)):
        return 1
    if isinstance(node, _Sequence):
        return sum(_minimum_consumption(part) for part in node.parts)
    if isinstance(node, _Alternation):
        return min(_minimum_consumption(part) for part in node.alternatives)
    if isinstance(node, _Group):
        return _minimum_consumption(node.child)
    return node.minimum * _minimum_consumption(node.child)


def _valid_capture_name(name: str) -> bool:
    if not name or not (name[0].isascii() and (name[0].isalpha() or name[0] == "_")):
        return False
    return all(
        character.isascii() and (character.isalnum() or character == "_")
        for character in name[1:]
    )


class _Parser:
    def __init__(self, source: str) -> None:
        self.source = source
        self.position = 0
        self.capture_count = 0
        self.capture_names: list[str] = []
        self.name_indexes: dict[str, int] = {}
        self.frames = [_Frame()]

    def parse(self) -> _Node:
        while self.position < len(self.source):
            character = self.source[self.position]
            if character == "|":
                self._alternate()
            elif character == "(":
                self._open_group()
            elif character == ")":
                self._close_group()
            elif character == "[":
                self._append(self._character_class())
            elif character == "\\":
                self._append(self._escaped_literal())
            elif character in "?*+{":
                self._quantify(character)
            elif character == "^":
                self._append(_StartAnchor())
                self.position += 1
            elif character == "$":
                self._append(_EndAnchor())
                self.position += 1
            elif character in ".]}":
                raise PortablePatternError("metacharacter is invalid in atom position")
            else:
                self._append(_Literal(character))
                self.position += 1
        if len(self.frames) != 1:
            raise PortablePatternError("group is not closed")
        return _finish_frame(self.frames[0])

    def _append(self, node: _Node) -> None:
        self.frames[-1].sequence.append(node)

    def _alternate(self) -> None:
        frame = self.frames[-1]
        frame.alternatives.append(_sequence(frame.sequence))
        frame.sequence = []
        self.position += 1

    def _open_group(self) -> None:
        capture_index: int | None = None
        capture_name: str | None = None
        if self.source.startswith("(?:", self.position):
            self.position += 3
        elif self.source.startswith("(?P<", self.position):
            end = self.source.find(">", self.position + 4)
            if end < 0:
                raise PortablePatternError("named capture is not closed")
            capture_name = self.source[self.position + 4 : end]
            if not _valid_capture_name(capture_name):
                raise PortablePatternError("named capture has an invalid name")
            if capture_name in self.name_indexes:
                raise PortablePatternError("named capture name is duplicated")
            capture_index = self._new_capture(capture_name)
            self.position = end + 1
        elif self.source.startswith("(?", self.position):
            raise PortablePatternError("unsupported group form")
        else:
            capture_index = self._new_capture(None)
            self.position += 1
        self.frames.append(
            _Frame(capture_index=capture_index, capture_name=capture_name)
        )

    def _new_capture(self, name: str | None) -> int:
        self.capture_count += 1
        if name is not None:
            self.capture_names.append(name)
            self.name_indexes[name] = self.capture_count
        return self.capture_count

    def _close_group(self) -> None:
        if len(self.frames) == 1:
            raise PortablePatternError("closing parenthesis has no opening group")
        frame = self.frames.pop()
        child = _finish_frame(frame)
        self._append(_Group(child, frame.capture_index, frame.capture_name))
        self.position += 1

    def _escaped_literal(self) -> _Literal:
        if self.position + 1 >= len(self.source):
            raise PortablePatternError("trailing escape")
        escaped = self.source[self.position + 1]
        if escaped not in _METASYNTAX:
            raise PortablePatternError("unsupported escape")
        self.position += 2
        return _Literal(escaped)

    def _class_literal(self, position: int) -> tuple[str, int, bool]:
        character = self.source[position]
        if character != "\\":
            return character, position + 1, character == "-"
        if position + 1 >= len(self.source):
            raise PortablePatternError("character-class escape is incomplete")
        escaped = self.source[position + 1]
        if escaped not in _CLASS_ESCAPES:
            raise PortablePatternError("unsupported character-class escape")
        return escaped, position + 2, False

    def _character_class(self) -> _CharacterClass:
        position = self.position + 1
        negated = position < len(self.source) and self.source[position] == "^"
        if negated:
            position += 1
        literals: list[str] = []
        ranges: list[tuple[int, int]] = []
        item_count = 0
        while position < len(self.source) and self.source[position] != "]":
            literal, after_literal, raw_hyphen = self._class_literal(position)
            if raw_hyphen:
                if item_count == 0 or (
                    after_literal < len(self.source)
                    and self.source[after_literal] == "]"
                ):
                    literals.append("-")
                    item_count += 1
                    position = after_literal
                    continue
                raise PortablePatternError("middle class hyphen is invalid")
            range_follows = (
                after_literal + 1 < len(self.source)
                and self.source[after_literal] == "-"
                and self.source[after_literal + 1] != "]"
            )
            if range_follows:
                endpoint, after_endpoint, _raw = self._class_literal(after_literal + 1)
                if not literal.isascii() or not endpoint.isascii():
                    raise PortablePatternError("class range endpoints must be ASCII")
                if ord(literal) > ord(endpoint):
                    raise PortablePatternError("class range is reversed")
                ranges.append((ord(literal), ord(endpoint)))
                position = after_endpoint
            else:
                literals.append(literal)
                position = after_literal
            item_count += 1
        if position >= len(self.source):
            raise PortablePatternError("character class is not closed")
        if item_count == 0:
            raise PortablePatternError("character class is empty")
        self.position = position + 1
        return _CharacterClass(negated, tuple(literals), tuple(ranges))

    def _decimal(self, position: int) -> tuple[int, int]:
        if position >= len(self.source) or not self.source[position].isascii() or not self.source[position].isdigit():
            raise PortablePatternError("quantifier bound is missing")
        start = position
        value = 0
        while position < len(self.source):
            character = self.source[position]
            if not character.isascii() or not character.isdigit():
                break
            value = value * 10 + ord(character) - ord("0")
            position += 1
        if self.source[start] == "0" and position - start > 1:
            raise PortablePatternError("quantifier bound is not canonical")
        return value, position

    def _quantify(self, marker: str) -> None:
        sequence = self.frames[-1].sequence
        if not sequence or not isinstance(sequence[-1], (_Literal, _CharacterClass, _Group)):
            raise PortablePatternError("quantifier has no quantifiable atom")
        child = sequence[-1]
        if marker == "?":
            minimum, maximum, end = 0, 1, self.position + 1
        elif marker == "*":
            minimum, maximum, end = 0, None, self.position + 1
        elif marker == "+":
            minimum, maximum, end = 1, None, self.position + 1
        else:
            minimum, end = self._decimal(self.position + 1)
            if end < len(self.source) and self.source[end] == "}":
                maximum, end = minimum, end + 1
            elif end < len(self.source) and self.source[end] == ",":
                end += 1
                if end < len(self.source) and self.source[end] == "}":
                    maximum, end = None, end + 1
                else:
                    maximum, end = self._decimal(end)
                    if end >= len(self.source) or self.source[end] != "}":
                        raise PortablePatternError("quantifier is not closed")
                    end += 1
                    if minimum > maximum:
                        raise PortablePatternError("quantifier range is reversed")
            else:
                raise PortablePatternError("quantifier is malformed")
        if maximum is None and _nullable(child):
            raise PortablePatternError("unbounded repeat has a nullable child")
        sequence[-1] = _Repeat(child, minimum, maximum)
        self.position = end


@dataclass(frozen=True)
class PortablePatternMatch:
    _text: str
    _spans: tuple[_CaptureSpan, ...]
    _name_indexes: Mapping[str, int]

    def group(self, key: int | str) -> str | None:
        if isinstance(key, bool) or not isinstance(key, (int, str)):
            raise TypeError("capture key must be an integer or string")
        if isinstance(key, str):
            if key not in self._name_indexes:
                raise KeyError(key)
            index = self._name_indexes[key]
        else:
            index = key
        if index < 1 or index > len(self._spans):
            raise IndexError("capture index is out of range")
        span = self._spans[index - 1]
        return None if span is None else self._text[span[0] : span[1]]

class PortablePattern:
    __slots__ = ("_source", "_root", "_capture_count", "_capture_names", "_name_indexes")

    def __init__(self, source: str) -> None:
        self._source = _validate_scalar_string(source, "pattern")
        parser = _Parser(self._source)
        self._root = parser.parse()
        self._capture_count = parser.capture_count
        self._capture_names = tuple(parser.capture_names)
        self._name_indexes = MappingProxyType(dict(parser.name_indexes))

    @property
    def source(self) -> str:
        return self._source

    @property
    def capture_count(self) -> int:
        return self._capture_count

    @property
    def capture_names(self) -> tuple[str, ...]:
        return self._capture_names

    def full_match(self, text: str) -> PortablePatternMatch | None:
        candidate = _validate_scalar_string(text, "input")
        from .portable_pattern_matching import match_full

        spans = match_full(self._root, candidate, self._capture_count)
        if spans is None:
            return None
        return PortablePatternMatch(candidate, spans, self._name_indexes)


def full_match(pattern: str, text: str) -> PortablePatternMatch | None:
    """Apply one Canonical Portable Pattern to a complete input string."""

    return PortablePattern(pattern).full_match(text)
