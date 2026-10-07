from __future__ import annotations

from dataclasses import fields, is_dataclass
from enum import Enum
import json
from pathlib import Path
import re
import subprocess

from jsonschema import Draft202012Validator

REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
CORPUS_ROOT = REPOSITORY_ROOT / "conformance" / "v1"
BASE_SHA = "662d86d445cefed4e17aa3cfa897ac36344cba89"
FROZEN_SHA = "dedb01a3a9b7a18930c9da75afa3773b5ad67f69"
PLACEHOLDER_KEYS = {
    "scenario", "semantic_vector", "placeholder", "description_only",
    "expected", "expected_result",
}
GENERIC_HISTORY = (
    "differs from or predates", "conforms to the contract-derived expectation",
    "matches corrected semantics", "python differed", "python agreed",
)
OWNER_GROUPS = {
    63: {
        "structured-data.document", "structured-data.frontmatter",
        "governance-bootstrap.root", "governance-routing.resolve",
        "repository-governance-model.load",
        "repository-governance-model.binding-compatibility",
    },
    64: {
        "adr-metadata.parse", "adr-metadata.validation",
        "adr-metadata.repository-path", "portable-pattern.full-match",
        "canonical-adr.resolve", "accepted-adr-body.immutability",
        "normative-terminology.registry", "normative-terminology.markdown",
        "normative-terminology.definition-discovery",
        "normative-terminology.fingerprint", "normative-terminology.inventory",
    },
    65: {"governance-authority.profile", "governed-objects.catalog"},
    66: {
        "governance-bindings.registry", "projection-registry.registry",
        "repository-integrity.profile", "evidence-requirements.registry",
        "exact-evidence-binding.evaluate",
    },
    67: {"repository-state.capture"},
    68: {"repository-governance-state.compose"},
    69: {
        "repository-integrity.evaluate", "git-whitespace.validate",
        "github-authoritative-ref-monotonicity.observe",
    },
}
OWNER_BY_ID = {
    responsibility_id: owner
    for owner, responsibility_ids in OWNER_GROUPS.items()
    for responsibility_id in responsibility_ids
}
CLASSIFICATIONS = {
    "required_by_existing_normative_semantics",
    "bootstrap_implementation_accident",
    "missing_contract_responsibility",
    "consumer_specific_behavior",
}


def _integer_decimal(value: int) -> str:
    if value == 0:
        return "0"
    sign = "-" if value < 0 else ""
    remaining = -value if value < 0 else value
    chunks = []
    while remaining:
        remaining, chunk = divmod(remaining, 1_000_000_000)
        chunks.append(chunk)
    return sign + str(chunks[-1]) + "".join(f"{chunk:09d}" for chunk in reversed(chunks[:-1]))

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

def authority_key(authority: dict[str, object]) -> str:
    if authority["source_kind"] == "repository":
        return (
            f"repo:{authority['repository']}@{authority['commit']}:"
            f"{authority['path']}#{authority['clause']}"
        )
    return (
        f"external:{authority['source_id']}@{authority['version']}:"
        f"{authority['locator']}#{authority['clause']}"
    )

def read_json(path: Path):
    data = path.read_bytes()
    if data.startswith(b"\xef\xbb\xbf") or b"\r" in data:
        raise AssertionError(f"noncanonical JSON bytes: {path}")
    decoded = data.decode("utf-8")
    value = json.loads(decoded)
    canonical = json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    if decoded != canonical:
        raise AssertionError(f"noncanonical JSON formatting: {path}")
    return value

def load_corpus():
    index = read_json(CORPUS_ROOT / "index.json")
    responsibilities = {
        value["responsibility_id"]: value
        for value in (read_json(CORPUS_ROOT / path) for path in index["responsibility_files"])
    }
    cases = {
        value["responsibility_id"]: value
        for value in (read_json(CORPUS_ROOT / path) for path in index["case_files"])
    }
    coverage = [read_json(CORPUS_ROOT / path) for path in index["coverage_files"]]
    return index, responsibilities, cases, coverage

def validate_schemas(index, responsibilities, cases, coverage):
    schemas = {
        name: read_json(CORPUS_ROOT / "schemas" / name)
        for name in (
            "index.schema.json", "responsibility.schema.json",
            "case.schema.json", "coverage.schema.json",
        )
    }
    for schema in schemas.values():
        Draft202012Validator.check_schema(schema)
    Draft202012Validator(schemas["index.schema.json"]).validate(index)
    for value in responsibilities.values():
        Draft202012Validator(schemas["responsibility.schema.json"]).validate(value)
    for value in cases.values():
        Draft202012Validator(schemas["case.schema.json"]).validate(value)
    for value in coverage:
        Draft202012Validator(schemas["coverage.schema.json"]).validate(value)

def walk(value):
    yield value
    if isinstance(value, dict):
        for nested in value.values():
            yield from walk(nested)
    elif isinstance(value, list):
        for nested in value:
            yield from walk(nested)

def validate_no_placeholders(cases):
    count = 0
    for case in cases.values():
        for vector in case["vectors"]:
            fixture = vector["fixture"]
            for node in walk(fixture):
                if isinstance(node, dict) and PLACEHOLDER_KEYS & node.keys():
                    count += 1
            kind = fixture["kind"]
            if kind == "repository_plan" and not fixture["steps"]:
                count += 1
            if kind == "provider_observation" and not fixture["interactions"]:
                binding_bytes = bytes.fromhex(fixture["binding"]["bytes_hex"])
                if binding_bytes.startswith(b"---\n"):
                    count += 1
    if count:
        raise AssertionError(f"PLACEHOLDER_FIXTURES={count}")
    return count

def validate_vector_qualification(cases):
    historical_placeholders = 0
    matrix_classifications = 0
    vector_ids = set()
    for case in cases.values():
        if "behavior_classification" in case or "authorities" in case:
            matrix_classifications += 1
        for vector in case["vectors"]:
            vector_id = vector["vector_id"]
            if vector_id in vector_ids:
                raise AssertionError(f"duplicate vector ID: {vector_id}")
            vector_ids.add(vector_id)
            if vector["behavior_classification"] not in CLASSIFICATIONS:
                raise AssertionError(vector_id)
            keys = {authority_key(item) for item in vector["authorities"]}
            if set(vector["expected_observation"]["derivation"]["authority_keys"]) - keys:
                raise AssertionError(f"unbound derivation authority: {vector_id}")
            for field in ("frozen_python_observation", "later_python_observation"):
                record = vector[field]
                if record is None:
                    continue
                text = json.dumps(record["observation"], ensure_ascii=False).lower()
                historical_placeholders += sum(phrase in text for phrase in GENERIC_HISTORY)
            classification = vector["behavior_classification"]
            if classification == "missing_contract_responsibility":
                resolution = vector["resolution"]
                if resolution is None or resolution["status"] != "resolved":
                    raise AssertionError(f"unresolved historical vector: {vector_id}")
    if matrix_classifications:
        raise AssertionError(f"MATRIX_LEVEL_BEHAVIOR_CLASSIFICATIONS={matrix_classifications}")
    if historical_placeholders:
        raise AssertionError(
            f"HISTORICAL_GENERIC_OBSERVATION_PLACEHOLDERS={historical_placeholders}"
        )
    return len(vector_ids)

def compare(actual: dict[str, object], expected: dict[str, object], rule: dict[str, object]):
    if rule["kind"] != "exact":
        raise AssertionError(f"unsupported comparison kind: {rule['kind']}")
    target = {"kind": expected["kind"], "value": expected["value"]}
    if actual != target:
        raise AssertionError(
            "observation mismatch\nactual=" + json.dumps(actual, ensure_ascii=False, sort_keys=True)
            + "\nexpected=" + json.dumps(target, ensure_ascii=False, sort_keys=True)
        )
    return "MATCH"

def git_show(commit: str, path: str) -> bytes:
    return subprocess.check_output(
        ["git", "-C", str(REPOSITORY_ROOT), "show", f"{commit}:{path}"]
    )

def _heading_paths(markdown: str) -> set[str]:
    paths = set()
    parent = None
    for line in markdown.splitlines():
        if line.startswith("## "):
            parent = line[3:].strip()
            paths.add(parent)
        elif line.startswith("### "):
            paths.add(f"{parent} / {line[4:].strip()}")
    return paths

def validate_authorities(cases):
    for case in cases.values():
        for vector in case["vectors"]:
            for authority in vector["authorities"]:
                if authority["source_kind"] == "repository":
                    if not re.fullmatch(r"[0-9a-f]{40}", authority["commit"]):
                        raise AssertionError(authority)
                    content = git_show(authority["commit"], authority["path"])
                    if authority["path"].endswith(".md"):
                        if authority["clause"] not in _heading_paths(content.decode("utf-8")):
                            raise AssertionError(
                                f"missing authority clause: {authority_key(authority)}"
                            )
                elif authority["sha256"] is not None and not re.fullmatch(
                    r"[0-9a-f]{64}", authority["sha256"]
                ):
                    raise AssertionError(authority)

def validate_coverage(coverage, responsibilities, cases):
    vectors = {
        vector["vector_id"]: (case["responsibility_id"], vector)
        for case in cases.values() for vector in case["vectors"]
    }
    for record in coverage:
        contract = record["contract"]
        contract_key_prefix = (
            f"repo:{contract['repository']}@{contract['commit']}:"
            f"{contract['path']}#"
        )
        for heading in record["headings"]:
            if heading["disposition"] != "covered":
                continue
            if not heading["coverage_assertion"]:
                raise AssertionError(heading)
            exact_key = contract_key_prefix + heading["heading_path"]
            for vector_id in heading["vector_refs"]:
                responsibility_id, vector = vectors[vector_id]
                if responsibility_id not in heading["responsibility_ids"]:
                    raise AssertionError(f"coverage responsibility mismatch: {vector_id}")
                if exact_key not in {authority_key(item) for item in vector["authorities"]}:
                    raise AssertionError(f"coverage authority mismatch: {vector_id}")
                if exact_key not in vector["expected_observation"]["derivation"]["authority_keys"]:
                    raise AssertionError(f"coverage derivation mismatch: {vector_id}")


def _walk_identity_values(value, path=()):
    if isinstance(value, dict):
        for key, item in value.items(): yield from _walk_identity_values(item, (*path, key))
    elif isinstance(value, list):
        for index, item in enumerate(value): yield from _walk_identity_values(item, (*path, index))
    else: yield path, value


def validate_opaque_identity(responsibilities, cases):
    affected = {"repository-state.capture", "repository-governance-state.compose", "repository-integrity.profile", "repository-integrity.evaluate"}
    lengths = digests = without_authority = current_compatibility = 0
    expected = {}
    for responsibility_id in affected:
        responsibility = responsibilities[responsibility_id]
        required_text = " ".join(responsibility["required_state_distinctions"]).lower()
        if any(token in required_text for token in ("identity_length", "identity length", "64-character identity", "sha-256 identity")): lengths += 1
        freedom = " ".join(responsibility["allowed_implementation_freedom"]).lower()
        if not all(token in freedom for token in ("implementation-defined", "algorithm", "framing")): raise AssertionError(f"opaque identity freedom missing: {responsibility_id}")
        for vector in cases[responsibility_id]["vectors"]:
            valid_authority = any(item.get("role") == "compatibility_obligation" and item.get("source_kind") == "repository" and re.fullmatch(r"[0-9a-f]{40}", str(item.get("commit", ""))) and "representation" in str(item.get("clause", "")).lower() for item in vector["authorities"])
            current_compatibility += int(valid_authority)
            value = decode_transport(vector["expected_observation"].get("value")) if "value" in vector["expected_observation"] else None
            expected[vector["vector_id"]] = value
            assertions = 0
            for path, item in _walk_identity_values(value):
                key = str(path[-1]).lower() if path else ""
                if "identity" in key and "length" in key: lengths += 1; assertions += 1
                if "identity" in key and isinstance(item, str) and re.fullmatch(r"[0-9a-f]{64}", item): digests += 1; assertions += 1
            text = " ".join([*vector["state_distinctions"], vector["expected_observation"]["derivation"]["reason"]]).lower()
            textual = int(any(token in text for token in ("identity_length", "identity length", "64-character identity", "sha-256 identity"))) + len(re.findall(r"(?<![0-9a-f])[0-9a-f]{64}(?![0-9a-f])", text))
            lengths += int(textual > 0); without_authority += (assertions + textual) * int(not valid_authority)
    state_presence = {"repository-state.capture.absent", "repository-state.capture.absent-explicit", "repository-state.capture.directory", "repository-state.capture.exact-root", "repository-state.capture.nested-repository", "repository-state.capture.regular-file", "repository-state.capture.scope-canonical", "repository-state.capture.scope-order-duplicate", "repository-state.capture.symlink"}
    rgs_presence = {"repository-governance-state.compose.evidence", "repository-governance-state.compose.ignored-routed-file", "repository-governance-state.compose.integrity-absent", "repository-governance-state.compose.integrity-present", "repository-governance-state.compose.minimal-v2", "repository-governance-state.compose.no-adr-scan", "repository-governance-state.compose.no-integrity-execution", "repository-governance-state.compose.no-provider-contact", "repository-governance-state.compose.objects-absent", "repository-governance-state.compose.objects-present", "repository-governance-state.compose.projection", "repository-governance-state.compose.pure-construction", "repository-governance-state.compose.routed-symlink", "repository-governance-state.compose.routing-expression-not-scope"}
    profile_presence = {"repository-integrity.profile.authoritative-carrier", "repository-integrity.profile.glob-selector", "repository-integrity.profile.optional-target", "repository-integrity.profile.path-selector"}
    if any(expected[key].get("identity_present") is not True for key in state_presence | rgs_presence | profile_presence): raise AssertionError("opaque identity presence projection missing")
    profile_relations = {key: expected[key]["identity_relation"]["relation"] for key in ("repository-integrity.profile.valid-profile", "repository-integrity.profile.profile-order-identity", "repository-integrity.profile.profile-glob-identity")}
    if profile_relations != {"repository-integrity.profile.valid-profile": "NOT_EQUAL", "repository-integrity.profile.profile-order-identity": "EQUAL", "repository-integrity.profile.profile-glob-identity": "EQUAL"}: raise AssertionError("profile identity relations incomplete")
    context = expected["repository-integrity.evaluate.context-identity"]
    if context["identity_relations"] != [{"left": "one", "right": "two", "relation": "EQUAL"}, {"left": "one", "right": "three", "relation": "NOT_EQUAL"}]: raise AssertionError("context identity relations incomplete")
    if (lengths, digests, without_authority, current_compatibility) != (0, 0, 0, 0): raise AssertionError("unsupported exact identity representation remains")
    return {"identity_lengths": lengths, "identity_digests": digests, "identity_without_authority": without_authority, "identity_compatibility": current_compatibility, "opaque_identity_relations": "PASS"}


def validate_migration_accounting(index, responsibilities, cases, coverage):
    retired_id = "shared-governance-provider.check"; rust_ids = {key for key, value in responsibilities.items() if value["migration_disposition"] == "rust_port"}; retired = {key for key, value in responsibilities.items() if value["migration_disposition"] == "retire_without_rust_port"}
    if (len(responsibilities), len(rust_ids), retired) != (30, 29, {retired_id}) or set(cases) != rust_ids or set(OWNER_BY_ID) != rust_ids or len(index["case_files"]) != 29: raise AssertionError("invalid migration disposition partition")
    provider = responsibilities[retired_id]; ledger = provider.get("source_test_accounting", [])
    if (provider["rust_owner_issue"], provider["case_file"], provider["python_retirement_owner"]) != (None, None, 73) or len(ledger) != 1 or ledger[0]["disposition"] != "bootstrap_evidence_only": raise AssertionError("invalid legacy provider retirement accounting")
    encoded = json.dumps([cases, coverage], sort_keys=True)
    if "shared-governance-provider.check." in encoded or "974ca31ff12630a90da6371cc27c1f5ef0cc590e" in encoded: raise AssertionError("unresolved legacy provider conformance authority")
    rgs = responsibilities["repository-governance-state.compose"]; expected_sha = "5ba3ef457786b09fd51430418b3b62ed4dc991b9"; rgs_authorities = [item for item in rgs["authorities"] if item.get("path") == "docs/contracts/repository-governance-state.md"]
    if not rgs_authorities or {item["commit"] for item in rgs_authorities} != {expected_sha} or any(item["commit"] != expected_sha for vector in cases["repository-governance-state.compose"]["vectors"] for item in vector["authorities"] if item.get("path") == "docs/contracts/repository-governance-state.md"): raise AssertionError("RGS canonical authority conflict")
    fenced = responsibilities["normative-terminology.markdown"]["required_state_distinctions"]
    if "fenced code blocks beginning with triple backticks or tildes" not in " ".join(fenced) or "exclud" in " ".join(fenced).lower(): raise AssertionError("fenced responsibility/vector conflict")
    vectors = {vector["vector_id"]: vector for case in cases.values() for vector in case["vectors"]}; scalar = vectors["adr-metadata.parse.canonical-scalars"]; actual_input = bytes.fromhex(decode_transport(scalar["fixture"]["input"])["bytes_hex"]).decode(); expected_scalars = {"true_value": True, "integer_value": 42, "null_value": None, "yes_value": "yes", "capital_null_value": "Null", "leading_zero_value": "01", "float_like_value": "1.0", "date_like_value": "2025-01-02"}
    if decode_transport(scalar["expected_observation"]["value"]) != expected_scalars or any(f"{key}: {'null' if value is None else str(value).lower() if isinstance(value, bool) else value}" not in actual_input for key, value in expected_scalars.items()): raise AssertionError("canonical scalar declaration/input mismatch")
    declarations = set(responsibilities["structured-data.document"]["required_state_distinctions"]); declaration_tokens = {token.strip(".,;:()") for declaration in declarations for token in declaration.split()}; literal_mismatches = len(declaration_tokens & {"123", "-123", "9223372036854775808"})
    for vector_id, literal, declaration in (("structured-data.document.positive-integer", "42", "canonical positive decimal integer 42 constructs the exact mathematical integer 42"), ("structured-data.document.negative-integer", "-42", "canonical negative decimal integer -42 constructs the exact mathematical integer -42"), ("structured-data.document.beyond-i64", "18446744073709551616", "the canonical decimal scalar 18446744073709551616 constructs the exact mathematical integer 18446744073709551616 without rejection, truncation, rounding, float conversion, or string fallback caused by a 64-bit machine-width boundary")):
        vector = vectors[vector_id]; fixture = bytes.fromhex(decode_transport(vector["fixture"]["input"])["bytes_hex"]).decode().strip(); expected = str(decode_transport(vector["expected_observation"]["value"])); literal_mismatches += declaration not in declarations
        if fixture != literal or expected != literal or literal not in " ".join(vector["state_distinctions"]): raise AssertionError(f"vector declaration/input mismatch: {vector_id}")
    module_dispositions = {}
    for value in responsibilities.values():
        for module in value["frozen_baseline_modules"]: module_dispositions.setdefault(module, set()).add(value["migration_disposition"])
    expected_modules = {name for name in subprocess.check_output(["git", "-C", str(REPOSITORY_ROOT), "ls-tree", "--name-only", f"{FROZEN_SHA}:src/proto_ring"], text=True).splitlines() if name.endswith(".py") and name != "__init__.py"}
    if set(module_dispositions) != expected_modules or any(len(value) != 1 for value in module_dispositions.values()): raise AssertionError("unexplained frozen module migration disposition")
    carriers = {item for value in responsibilities.values() for item in value["post_baseline_implementation_carriers"]}; fixture_files = {path.relative_to(CORPUS_ROOT).as_posix() for path in (CORPUS_ROOT / "fixtures").rglob("*") if path.is_file() and path.name != "README.md"}
    if carriers != {"portable_pattern.py", "portable_pattern_matching.py", "_normative_terminology_unicode.py"} or any(path not in encoded for path in fixture_files): raise AssertionError("carrier or fixture accounting mismatch")
    return {"rust": len(rust_ids), "retired": len(retired), "matrices": len(cases), "modules": len(module_dispositions), "carriers": len(carriers), "literal_mismatches": literal_mismatches, **validate_opaque_identity(responsibilities, cases)}


def validate_repository_boundaries():
    if (CORPUS_ROOT / "manifest.json").exists():
        raise AssertionError("superseded manifest exists")
    original = git_show(BASE_SHA, "pyproject.toml")
    if (REPOSITORY_ROOT / "pyproject.toml").read_bytes() != original:
        raise AssertionError("pyproject.toml changed")
    if b'version = "0.17.0"' not in original:
        raise AssertionError("package version changed")
