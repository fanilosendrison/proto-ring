"""Interpret canonical structured frontmatter without consumer semantics."""

from __future__ import annotations

from dataclasses import dataclass
import re
from typing import TypeAlias

import yaml
from yaml.nodes import MappingNode, Node, ScalarNode, SequenceNode
from yaml.tokens import (
    AliasToken,
    AnchorToken,
    DirectiveToken,
    ScalarToken,
    TagToken,
)


__all__ = [
    "ParsedFrontmatter",
    "StructuredDataError",
    "StructuredValue",
    "parse_frontmatter_bytes",
]

_BOM = b"\xef\xbb\xbf"
_OPENING_DELIMITER = b"---\n"
_CLOSING_BOUNDARY = b"\n---\n"
_INTEGER_PATTERN = re.compile(r"^(0|-?[1-9][0-9]*)$")
_FORBIDDEN_TOKEN_TYPES = (
    AnchorToken,
    AliasToken,
    TagToken,
    DirectiveToken,
)


class StructuredDataError(ValueError):
    """Report controlled failure to interpret canonical structured frontmatter."""


StructuredValue: TypeAlias = (
    str
    | bool
    | int
    | None
    | list["StructuredValue"]
    | dict[str, "StructuredValue"]
)


@dataclass(frozen=True)
class ParsedFrontmatter:
    metadata: dict[str, StructuredValue]
    body: bytes


def _validate_tokens(payload_text: str) -> None:
    try:
        tokens = yaml.scan(payload_text, Loader=yaml.BaseLoader)
        for token in tokens:
            if isinstance(token, _FORBIDDEN_TOKEN_TYPES):
                raise StructuredDataError(
                    f"forbidden YAML token: {type(token).__name__}"
                )
            if isinstance(token, ScalarToken) and token.style in ("|", ">"):
                raise StructuredDataError("block scalar values are forbidden")
    except yaml.YAMLError as error:
        raise StructuredDataError(
            f"invalid structured frontmatter: {error}"
        ) from error


def _compose_single_document(payload_text: str) -> Node:
    try:
        documents = list(yaml.compose_all(payload_text, Loader=yaml.BaseLoader))
    except yaml.YAMLError as error:
        raise StructuredDataError(
            f"invalid structured frontmatter: {error}"
        ) from error
    if not documents or documents[0] is None:
        raise StructuredDataError("structured frontmatter must not be empty")
    if len(documents) != 1:
        raise StructuredDataError("structured frontmatter must contain one document")
    return documents[0]


def _construct_canonical_integer(text: str) -> int:
    negative = text.startswith("-")
    digits = text[1:] if negative else text
    result = 0
    for character in digits:
        result = result * 10 + (ord(character) - ord("0"))
    return -result if negative else result


def _construct_scalar(node: ScalarNode) -> str | bool | int | None:
    if node.style in ("'", '"'):
        return node.value
    if node.style is not None:
        raise StructuredDataError(f"unsupported scalar style: {node.style}")
    if node.value == "":
        raise StructuredDataError("omitted scalar values are forbidden")
    if node.value == "true":
        return True
    if node.value == "false":
        return False
    if node.value == "null":
        return None
    if _INTEGER_PATTERN.fullmatch(node.value):
        return _construct_canonical_integer(node.value)
    return node.value


def _construct_sequence(node: SequenceNode) -> list[StructuredValue]:
    return [_construct(child) for child in node.value]


def _construct_mapping(node: MappingNode) -> dict[str, StructuredValue]:
    result: dict[str, StructuredValue] = {}
    for key_node, value_node in node.value:
        key = _construct(key_node)
        if not isinstance(key, str):
            raise StructuredDataError("mapping keys must construct to strings")
        if key == "<<":
            raise StructuredDataError("mapping key '<<' is forbidden")
        if key in result:
            raise StructuredDataError(f"duplicate mapping key: {key}")
        result[key] = _construct(value_node)
    return result


def _construct(node: Node) -> StructuredValue:
    if isinstance(node, ScalarNode):
        return _construct_scalar(node)
    if isinstance(node, SequenceNode):
        return _construct_sequence(node)
    if isinstance(node, MappingNode):
        return _construct_mapping(node)
    raise StructuredDataError(f"unsupported YAML node: {type(node).__name__}")


def parse_frontmatter_bytes(data: bytes) -> ParsedFrontmatter:
    """Parse one exact frontmatter envelope into deterministic structured data."""

    if _BOM in data:
        raise StructuredDataError("UTF-8 BOM is forbidden")
    if b"\r" in data:
        raise StructuredDataError("carriage-return bytes are forbidden")
    try:
        data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise StructuredDataError("carrier is not valid UTF-8") from error

    if not data.startswith(_OPENING_DELIMITER):
        raise StructuredDataError("carrier has no exact opening delimiter")
    closing = data.find(_CLOSING_BOUNDARY, len(_OPENING_DELIMITER) - 1)
    if closing < 0:
        raise StructuredDataError("carrier has no exact closing delimiter")

    payload_bytes = data[len(_OPENING_DELIMITER) : closing]
    body = data[closing + len(_CLOSING_BOUNDARY) :]
    payload_text = payload_bytes.decode("utf-8")
    _validate_tokens(payload_text)
    root = _compose_single_document(payload_text)
    if not isinstance(root, MappingNode):
        raise StructuredDataError("structured frontmatter must be a mapping")

    return ParsedFrontmatter(metadata=_construct_mapping(root), body=body)
