from __future__ import annotations

from dataclasses import fields, is_dataclass
from enum import Enum
from pathlib import Path


def _integer_decimal(value: int) -> str:
    if value == 0:
        return "0"
    sign = "-" if value < 0 else ""
    remaining = -value if value < 0 else value
    chunks = []
    while remaining:
        remaining, chunk = divmod(remaining, 1_000_000_000)
        chunks.append(chunk)
    return sign + str(chunks[-1]) + "".join(
        f"{chunk:09d}" for chunk in reversed(chunks[:-1])
    )


def transport(value: object):
    if isinstance(value, dict) and set(value) == {"type", "value"} and value["type"] in {
        "null", "boolean", "integer", "string", "bytes", "sequence", "record"
    }:
        return value
    if value is None:
        return {"type": "null", "value": None}
    if isinstance(value, bool):
        return {"type": "boolean", "value": value}
    if isinstance(value, int):
        return {"type": "integer", "value": _integer_decimal(value)}
    if isinstance(value, bytes):
        return {"type": "bytes", "value": value.hex()}
    if isinstance(value, str):
        return {"type": "string", "value": value}
    if isinstance(value, Enum):
        return transport(value.value)
    if isinstance(value, Path):
        return transport(value.as_posix())
    if is_dataclass(value):
        return transport({field.name: getattr(value, field.name) for field in fields(value)})
    if isinstance(value, (list, tuple)):
        return {"type": "sequence", "value": [transport(item) for item in value]}
    if isinstance(value, (set, frozenset)):
        ordered = sorted(value, key=lambda item: repr(item))
        return {"type": "sequence", "value": [transport(item) for item in ordered]}
    if isinstance(value, dict):
        return {
            "type": "record",
            "value": [
                {"name": str(key), "value": transport(item)}
                for key, item in sorted(value.items(), key=lambda pair: str(pair[0]))
            ],
        }
    raise TypeError(f"cannot encode transport value: {type(value).__name__}")


def decode_transport(value: dict[str, object]):
    kind = value["type"]
    payload = value["value"]
    if kind == "null":
        return None
    if kind == "boolean":
        return payload
    if kind == "integer":
        return int(payload)
    if kind == "string":
        return payload
    if kind == "bytes":
        return bytes.fromhex(payload)
    if kind == "sequence":
        return [decode_transport(item) for item in payload]
    if kind == "record":
        return {entry["name"]: decode_transport(entry["value"]) for entry in payload}
    raise ValueError(f"unknown transport type: {kind}")


def observation(kind: str, value: object):
    return {"kind": kind, "value": transport(value)}
