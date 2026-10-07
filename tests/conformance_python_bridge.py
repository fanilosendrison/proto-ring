from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

import conformance_python_adapter_63_64 as adapter_63_64
import conformance_python_adapter_65_66 as adapter_65_66
import conformance_python_adapter_67_69 as adapter_67_69
from conformance_corpus_test_support import OWNER_BY_ID
from conformance_fixture_runner import materialize


ADAPTER_BY_OWNER = {
    63: adapter_63_64,
    64: adapter_63_64,
    65: adapter_65_66,
    66: adapter_65_66,
    67: adapter_67_69,
    68: adapter_67_69,
    69: adapter_67_69,
}
REQUEST_KEYS = {"fixture", "responsibility_id", "vector_id"}


def _arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--request", required=True)
    parser.add_argument("--output", required=True)
    return parser.parse_args()


def _requests(path: str) -> list[dict[str, object]]:
    document = json.loads(Path(path).read_bytes().decode("utf-8"))
    if not isinstance(document, list):
        raise ValueError("request root must be an array")
    validated = []
    identities = set()
    for position, request in enumerate(document):
        if not isinstance(request, dict) or set(request) != REQUEST_KEYS:
            raise ValueError(f"request {position} must contain exactly the execution fields")
        responsibility_id = request["responsibility_id"]
        vector_id = request["vector_id"]
        fixture = request["fixture"]
        if not isinstance(responsibility_id, str) or not responsibility_id:
            raise ValueError(f"request {position} has an invalid responsibility_id")
        if not isinstance(vector_id, str) or not vector_id:
            raise ValueError(f"request {position} has an invalid vector_id")
        if not isinstance(fixture, dict):
            raise ValueError(f"request {position} has an invalid fixture")
        if responsibility_id not in OWNER_BY_ID:
            raise ValueError(f"request {position} has an unknown responsibility_id")
        identity = (responsibility_id, vector_id)
        if identity in identities:
            raise ValueError(f"request {position} duplicates an execution identity")
        identities.add(identity)
        validated.append(request)
    return validated


def _execute(request: dict[str, object]) -> dict[str, object]:
    responsibility_id = request["responsibility_id"]
    owner = OWNER_BY_ID[responsibility_id]
    adapter = ADAPTER_BY_OWNER[owner]
    with materialize(request["fixture"]) as realized:
        actual = adapter.execute(responsibility_id, realized)
    if not isinstance(actual, dict) or set(actual) != {"kind", "value"}:
        raise ValueError("adapter returned an invalid observation")
    return {
        "responsibility_id": responsibility_id,
        "vector_id": request["vector_id"],
        "observation": actual,
    }


def main() -> int:
    arguments = _arguments()
    try:
        requests = _requests(arguments.request)
    except (OSError, UnicodeError, ValueError, TypeError) as error:
        sys.stderr.write(f"error={error}\n")
        return 1
    records = []
    for request in requests:
        try:
            records.append(_execute(request))
        except Exception as error:
            sys.stderr.write(
                f"responsibility_id={request['responsibility_id']} "
                f"vector_id={request['vector_id']} error={error}\n"
            )
            return 1
    records.sort(key=lambda record: (record["responsibility_id"], record["vector_id"]))
    serialized = json.dumps(records, ensure_ascii=False, indent=2, sort_keys=True)
    Path(arguments.output).write_text(serialized + "\n", encoding="utf-8", newline="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
