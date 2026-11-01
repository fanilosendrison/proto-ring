"""Deterministic provider interaction sequence for conformance fixtures."""
from __future__ import annotations

import socket
import ssl
import urllib.error


class ProviderSequenceError(ValueError):
    pass


class _HttpResponse:
    def __init__(self, status: int, body: bytes) -> None:
        self.status = status
        self._body = body

    def __enter__(self):
        return self

    def __exit__(self, *_args: object) -> None:
        return None

    def read(self) -> bytes:
        return self._body


class ProviderSequence:
    def __init__(self, interactions: list[dict[str, object]]) -> None:
        self._interactions = interactions
        self._position = 0

    @property
    def consumed(self) -> int:
        return self._position

    def urlopen(self, *_args: object, **_kwargs: object):
        if self._position >= len(self._interactions):
            raise ProviderSequenceError("implementation requested an unsupplied interaction")
        interaction = self._interactions[self._position]
        self._position += 1
        kind = interaction["kind"]
        if kind == "http_response":
            return _HttpResponse(
                int(interaction["status"]), bytes.fromhex(str(interaction["body_hex"]))
            )
        if kind == "http_error":
            status = int(interaction["status"])
            raise urllib.error.HTTPError(
                "https://example.invalid", status, "fixture HTTP error", {}, None
            )
        if kind == "network_error":
            raise urllib.error.URLError(str(interaction["message"]))
        if kind == "tls_verification_error":
            raise ssl.SSLCertVerificationError(1, str(interaction["message"]))
        if kind == "timeout":
            raise socket.timeout(str(interaction["message"]))
        if kind == "io_error":
            raise OSError(str(interaction["message"]))
        raise ProviderSequenceError(f"unsupported provider interaction: {kind}")

    def require_consumed(self) -> None:
        if self._position != len(self._interactions):
            raise ProviderSequenceError(
                f"unused provider interactions: {len(self._interactions) - self._position}"
            )
