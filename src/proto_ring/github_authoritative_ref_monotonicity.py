"""Provider-specific public live observation of GitHub effective branch rules for
consumer-bound Authoritative Ref Monotonicity realization.
"""

from __future__ import annotations

import argparse
from collections.abc import Mapping
from dataclasses import dataclass
from enum import Enum
import json
from pathlib import Path
import socket
import ssl
import urllib.error
import urllib.parse
import urllib.request

import truststore

from proto_ring import structured_data

__all__ = [
    "ConformanceResult",
    "ConformanceStatus",
    "check",
    "run",
]

_API_BASE = "https://api.github.com"
_API_VERSION = "2026-03-10"
_BRANCH_PREFIX = "refs/heads/"
_MECHANISM = "github-repository-ruleset"
_PAGE_SIZE = 100
_MAX_PAGES = 100
_REQUIRED_TYPES = frozenset({"deletion", "non_fast_forward"})
_TIMEOUT_SECONDS = 10


class ConformanceStatus(str, Enum):
    SATISFIED = "SATISFIED"
    VIOLATED = "VIOLATED"
    UNDETERMINED = "UNDETERMINED"


@dataclass(frozen=True)
class ConformanceResult:
    status: ConformanceStatus
    diagnostics: tuple[str, ...]


@dataclass(frozen=True)
class _Binding:
    owner: str
    repository: str
    branch: str
    ruleset_id: int


class _BindingError(ValueError):
    """Report a malformed consumer-owned binding."""


class _ProviderError(RuntimeError):
    """Report a provider observation that cannot be interpreted."""


def _required_mapping(parent: Mapping[object, object], key: str) -> Mapping[object, object]:
    value = parent.get(key)
    if not isinstance(value, Mapping):
        raise _BindingError(f"{key} must be a mapping")
    return value


def _required_nonempty_string(parent: Mapping[object, object], key: str) -> str:
    value = parent.get(key)
    if not isinstance(value, str) or not value:
        raise _BindingError(f"{key} must be a nonempty string")
    return value


def _load_binding(path: Path) -> _Binding:
    try:
        data = path.read_bytes()
    except OSError as error:
        raise _BindingError(f"cannot read binding: {error}") from error
    try:
        parsed = structured_data.parse_frontmatter_bytes(data)
    except structured_data.StructuredDataError as error:
        raise _BindingError(f"cannot parse binding frontmatter: {error}") from error

    configuration = _required_mapping(
        parsed.metadata, "authoritative_ref_monotonicity"
    )
    repository = _required_mapping(configuration, "repository")
    protection = _required_mapping(configuration, "protection")

    provider = _required_nonempty_string(repository, "provider")
    if provider != "github":
        raise _BindingError("repository.provider must be github")
    owner = _required_nonempty_string(repository, "owner")
    repository_name = _required_nonempty_string(repository, "name")

    authoritative_ref = _required_nonempty_string(configuration, "authoritative_ref")
    if not authoritative_ref.startswith(_BRANCH_PREFIX):
        raise _BindingError("authoritative_ref must begin with refs/heads/")
    branch = authoritative_ref[len(_BRANCH_PREFIX) :]
    if not branch:
        raise _BindingError("authoritative_ref branch must be nonempty")

    mechanism = _required_nonempty_string(protection, "mechanism")
    if mechanism != _MECHANISM:
        raise _BindingError(f"protection.mechanism must be {_MECHANISM}")
    ruleset_id = protection.get("ruleset_id")
    if type(ruleset_id) is not int or ruleset_id <= 0:
        raise _BindingError("protection.ruleset_id must be a positive integer")

    return _Binding(owner, repository_name, branch, ruleset_id)


def _request_url(binding: _Binding, page: int) -> str:
    owner = urllib.parse.quote(binding.owner, safe="")
    repository = urllib.parse.quote(binding.repository, safe="")
    branch = urllib.parse.quote(binding.branch, safe="")
    query = urllib.parse.urlencode((("per_page", _PAGE_SIZE), ("page", page)))
    return f"{_API_BASE}/repos/{owner}/{repository}/rules/branches/{branch}?{query}"


def _read_page(binding: _Binding, page: int) -> list[object]:
    request = urllib.request.Request(
        _request_url(binding, page),
        headers={
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": _API_VERSION,
            "User-Agent": "proto-ring-github-arm-conformance",
        },
        method="GET",
    )
    tls_context = truststore.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    try:
        with urllib.request.urlopen(
            request,
            timeout=_TIMEOUT_SECONDS,
            context=tls_context,
        ) as response:
            status = response.status
            payload = response.read()
    except urllib.error.HTTPError as error:
        raise _ProviderError(f"provider request failed: HTTP {error.code}") from error
    except ssl.SSLCertVerificationError as error:
        raise _ProviderError(
            "provider request failed: TLS certificate verification error"
        ) from error
    except urllib.error.URLError as error:
        if isinstance(error.reason, ssl.SSLCertVerificationError):
            raise _ProviderError(
                "provider request failed: TLS certificate verification error"
            ) from error
        raise _ProviderError(
            "provider request failed: network or timeout error"
        ) from error
    except (TimeoutError, socket.timeout) as error:
        raise _ProviderError(
            "provider request failed: network or timeout error"
        ) from error
    except OSError as error:
        raise _ProviderError("provider request failed: I/O error") from error

    if type(status) is not int or not 200 <= status < 300:
        raise _ProviderError(f"provider request returned non-success status {status!r}")
    if not isinstance(payload, bytes):
        raise _ProviderError("provider response body is not bytes")
    try:
        decoded = payload.decode("utf-8")
    except UnicodeDecodeError as error:
        raise _ProviderError("provider response is not valid UTF-8") from error
    try:
        document = json.loads(decoded)
    except json.JSONDecodeError as error:
        raise _ProviderError("provider response is not valid JSON") from error
    if not isinstance(document, list):
        raise _ProviderError("provider response top level must be a list")
    return document


def _observed_types(binding: _Binding) -> set[str]:
    observed: set[str] = set()
    for page in range(1, _MAX_PAGES + 1):
        items = _read_page(binding, page)
        for item in items:
            if not isinstance(item, Mapping):
                raise _ProviderError("provider rule entry must be a mapping")
            rule_type = item.get("type")
            ruleset_id = item.get("ruleset_id")
            if not isinstance(rule_type, str) or not rule_type:
                raise _ProviderError("provider rule type must be a nonempty string")
            if type(ruleset_id) is not int:
                raise _ProviderError("provider rule ruleset_id must be an integer")
            if ruleset_id == binding.ruleset_id:
                observed.add(rule_type)

        if len(items) < _PAGE_SIZE:
            return observed

    raise _ProviderError("provider pagination exceeds the 100-page hard cap")


def check(binding_path: Path) -> ConformanceResult:
    try:
        binding = _load_binding(binding_path)
    except _BindingError as error:
        return ConformanceResult(ConformanceStatus.VIOLATED, (str(error),))

    try:
        observed = _observed_types(binding)
    except _ProviderError as error:
        return ConformanceResult(ConformanceStatus.UNDETERMINED, (str(error),))

    missing = sorted(_REQUIRED_TYPES - observed)
    if missing:
        return ConformanceResult(
            ConformanceStatus.VIOLATED,
            (f"bound ruleset is missing effective rule types: {', '.join(missing)}",),
        )
    return ConformanceResult(ConformanceStatus.SATISFIED, ())


def run(binding_path: Path) -> int:
    result = check(binding_path)
    print(
        "github authoritative ref monotonicity effective rules: "
        f"{result.status.value}"
    )
    for diagnostic in result.diagnostics:
        print(diagnostic)
    return {
        ConformanceStatus.SATISFIED: 0,
        ConformanceStatus.VIOLATED: 1,
        ConformanceStatus.UNDETERMINED: 2,
    }[result.status]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binding", required=True, type=Path)
    arguments = parser.parse_args()
    return run(arguments.binding)


if __name__ == "__main__":
    raise SystemExit(main())
