"""Pinned Unicode operations for the Normative Terminology contract."""

from __future__ import annotations

from functools import lru_cache
import hashlib
from importlib import resources
from types import MappingProxyType
from typing import Mapping


UNICODE_VERSION = "14.0.0"
_CASE_FOLDING_SHA256 = (
    "a566cd48687b2cd897e02501118b2413c14ae86d318f9abbbba97feb84189f0f"
)
_WHITESPACE_CODEPOINTS = frozenset(
    {
        0x0009,
        0x000A,
        0x000B,
        0x000C,
        0x000D,
        0x001C,
        0x001D,
        0x001E,
        0x001F,
        0x0020,
        0x0085,
        0x00A0,
        0x1680,
        0x2000,
        0x2001,
        0x2002,
        0x2003,
        0x2004,
        0x2005,
        0x2006,
        0x2007,
        0x2008,
        0x2009,
        0x200A,
        0x2028,
        0x2029,
        0x202F,
        0x205F,
        0x3000,
    }
)


def _case_folding_bytes() -> bytes:
    resource = resources.files("proto_ring").joinpath(
        "data", "unicode-14.0.0", "CaseFolding.txt"
    )
    try:
        data = resource.read_bytes()
    except (FileNotFoundError, OSError) as error:
        raise RuntimeError("Unicode 14.0.0 case-fold data is unavailable") from error
    if hashlib.sha256(data).hexdigest() != _CASE_FOLDING_SHA256:
        raise RuntimeError("Unicode 14.0.0 case-fold data has an invalid SHA-256")
    return data


def _decode_case_folding(data: bytes) -> str:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise RuntimeError("Unicode 14.0.0 case-fold data is not UTF-8") from error
    lines = text.splitlines()
    if not lines or lines[0] != "# CaseFolding-14.0.0.txt":
        raise RuntimeError("Unicode case-fold data has an invalid version header")
    return text


@lru_cache(maxsize=1)
def _case_fold_map() -> Mapping[str, str]:
    mapping: dict[str, str] = {}
    text = _decode_case_folding(_case_folding_bytes())
    for line_number, line in enumerate(text.splitlines(), start=1):
        record = line.split("#", 1)[0]
        if not record.strip(" \t"):
            continue
        fields = [field.strip(" \t") for field in record.split(";")]
        if len(fields) != 4 or fields[3] or not all(fields[:3]):
            raise RuntimeError(f"malformed case-fold record at line {line_number}")
        source_text, status, mapping_text = fields[:3]
        try:
            source = chr(int(source_text, 16))
            mapped_values = mapping_text.split(" ")
            if not mapped_values or any(not value for value in mapped_values):
                raise ValueError
            mapped = "".join(chr(int(value, 16)) for value in mapped_values)
        except (ValueError, OverflowError) as error:
            raise RuntimeError(
                f"malformed case-fold code point at line {line_number}"
            ) from error
        if status in {"S", "T"}:
            continue
        if status not in {"C", "F"}:
            raise RuntimeError(f"unknown case-fold status at line {line_number}")
        if source in mapping:
            raise RuntimeError(f"duplicate C/F case-fold mapping at line {line_number}")
        mapping[source] = mapped
    return MappingProxyType(mapping)


def casefold(text: str) -> str:
    mapping = _case_fold_map()
    return "".join(mapping.get(character, character) for character in text)


def is_whitespace(character: str) -> bool:
    if not isinstance(character, str):
        raise TypeError("character must be a string")
    if len(character) != 1:
        raise ValueError("character must contain exactly one Unicode scalar")
    return ord(character) in _WHITESPACE_CODEPOINTS


def lstrip(text: str) -> str:
    start = 0
    while start < len(text) and is_whitespace(text[start]):
        start += 1
    return text[start:]


def strip(text: str) -> str:
    start = 0
    end = len(text)
    while start < end and is_whitespace(text[start]):
        start += 1
    while end > start and is_whitespace(text[end - 1]):
        end -= 1
    return text[start:end]


def split_whitespace(text: str) -> list[str]:
    tokens: list[str] = []
    position = 0
    while position < len(text):
        while position < len(text) and is_whitespace(text[position]):
            position += 1
        start = position
        while position < len(text) and not is_whitespace(text[position]):
            position += 1
        if start < position:
            tokens.append(text[start:position])
    return tokens
