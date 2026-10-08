from __future__ import annotations

from conformance_corpus_test_support import authority_key

ISSUE66_RESPONSIBILITIES = frozenset(
    {
        "governance-bindings.registry",
        "projection-registry.registry",
        "repository-integrity.profile",
        "evidence-requirements.registry",
        "exact-evidence-binding.evaluate",
    }
)
GB_READ_ONLY_FINITE_WITNESS_VECTOR_IDS = frozenset(
    {"governance-bindings.registry.valid-registry"}
)
GB_READ_ONLY_AUTHORITY_KEY = (
    "repo:fanilosendrison/proto-ring@662d86d445cefed4e17aa3cfa897ac36344cba89:"
    "docs/contracts/shared-governance-provider.md#Governance Binding Registry / "
    "Read-only provider and installation boundary"
)
ISSUE66_PURE_CONTEXT_AUTHORITY_KEYS = frozenset(
    {
        "repo:fanilosendrison/proto-ring@662d86d445cefed4e17aa3cfa897ac36344cba89:docs/contracts/projection-integrity.md#Mechanically significant embedded assertions",
        "repo:fanilosendrison/proto-ring@662d86d445cefed4e17aa3cfa897ac36344cba89:docs/contracts/projection-integrity.md#Authority and evidence boundaries",
        "repo:fanilosendrison/proto-ring@662d86d445cefed4e17aa3cfa897ac36344cba89:docs/contracts/projection-integrity.md#Governed identity boundary",
        "repo:fanilosendrison/proto-ring@662d86d445cefed4e17aa3cfa897ac36344cba89:docs/contracts/evidence-requirements.md#Mechanically significant embedded assertions",
        "repo:fanilosendrison/proto-ring@5ba3ef457786b09fd51430418b3b62ed4dc991b9:docs/contracts/repository-integrity.md#Derivation baselines",
        "repo:fanilosendrison/proto-ring@662d86d445cefed4e17aa3cfa897ac36344cba89:docs/contracts/exact-evidence-binding.md#Fail-closed use of mandatory binding",
    }
)


def all_vector_map(
    cases: dict[str, dict[str, object]],
) -> dict[str, tuple[str, dict[str, object]]]:
    return {
        vector["vector_id"]: (case["responsibility_id"], vector)
        for case in cases.values()
        for vector in case["vectors"]
    }


def heading_is_issue66_relevant(
    heading: dict[str, object],
    all_vectors: dict[str, tuple[str, dict[str, object]]],
) -> bool:
    declared = bool(
        set(heading["responsibility_ids"]) & ISSUE66_RESPONSIBILITIES
    )
    referenced = any(
        vector_id in all_vectors
        and all_vectors[vector_id][0] in ISSUE66_RESPONSIBILITIES
        for vector_id in heading["vector_refs"]
    )
    return declared or referenced


def _contract_key_prefix(contract: dict[str, object]) -> str:
    return (
        f"repo:{contract['repository']}@{contract['commit']}:"
        f"{contract['path']}#"
    )


def coverage_authority_binding_violations(
    coverage: list[dict[str, object]],
    cases: dict[str, dict[str, object]],
) -> list[dict[str, object]]:
    vectors = all_vector_map(cases)
    violations = []
    for record in coverage:
        prefix = _contract_key_prefix(record["contract"])
        for heading in record["headings"]:
            if heading["disposition"] != "covered":
                continue
            if not heading_is_issue66_relevant(heading, vectors):
                continue
            exact_key = prefix + heading["heading_path"]
            for vector_id in heading["vector_refs"]:
                resolved = vectors.get(vector_id)
                if resolved is None:
                    violations.append(
                        {
                            "kind": "dangling_vector_ref",
                            "vector_id": vector_id,
                            "heading": heading["heading_path"],
                        }
                    )
                    continue
                responsibility_id, vector = resolved
                if responsibility_id not in heading["responsibility_ids"]:
                    violations.append(
                        {
                            "kind": "responsibility_mismatch",
                            "vector_id": vector_id,
                            "responsibility_id": responsibility_id,
                            "heading": heading["heading_path"],
                        }
                    )
                authorities = {
                    authority_key(item) for item in vector["authorities"]
                }
                if exact_key not in authorities:
                    violations.append(
                        {
                            "kind": "authority_missing",
                            "vector_id": vector_id,
                            "heading": heading["heading_path"],
                        }
                    )
                derivation_keys = set(
                    vector["expected_observation"]["derivation"]["authority_keys"]
                )
                if exact_key not in derivation_keys:
                    violations.append(
                        {
                            "kind": "derivation_authority_missing",
                            "vector_id": vector_id,
                            "heading": heading["heading_path"],
                        }
                    )
    return violations


def context_only_runtime_overclaims(
    coverage: list[dict[str, object]],
    cases: dict[str, dict[str, object]],
) -> int:
    del coverage
    return sum(
        key in ISSUE66_PURE_CONTEXT_AUTHORITY_KEYS
        for responsibility_id in ISSUE66_RESPONSIBILITIES
        for vector in cases.get(responsibility_id, {}).get("vectors", [])
        for key in vector["expected_observation"]["derivation"]["authority_keys"]
    )


def governance_bindings_read_only_unrelated_derivations(
    cases: dict[str, dict[str, object]],
) -> list[str]:
    return [
        vector["vector_id"]
        for vector in cases.get("governance-bindings.registry", {}).get(
            "vectors", []
        )
        if GB_READ_ONLY_AUTHORITY_KEY
        in vector["expected_observation"]["derivation"]["authority_keys"]
        and vector["vector_id"] not in GB_READ_ONLY_FINITE_WITNESS_VECTOR_IDS
    ]


def validate_governance_bindings_read_only_witness(
    cases: dict[str, dict[str, object]],
) -> bool:
    matrix = cases["governance-bindings.registry"]
    vectors = {
        vector["vector_id"]: vector for vector in matrix["vectors"]
    }
    if matrix["responsibility_id"] != "governance-bindings.registry":
        return False
    if set(vectors) & GB_READ_ONLY_FINITE_WITNESS_VECTOR_IDS != set(
        GB_READ_ONLY_FINITE_WITNESS_VECTOR_IDS
    ):
        return False
    vector = vectors["governance-bindings.registry.valid-registry"]
    observation = vector["expected_observation"]
    value = observation["value"]
    repository_unchanged = any(
        entry["name"] == "repository_unchanged"
        and entry["value"] == {"type": "boolean", "value": True}
        for entry in value["value"]
    )
    return (
        observation["kind"] == "result"
        and repository_unchanged
        and vector["state_distinctions"]
        == ["valid model-version-1 registry loads without mutating the repository"]
        and GB_READ_ONLY_AUTHORITY_KEY
        in observation["derivation"]["authority_keys"]
    )


def derivation_mismatches(
    cases: dict[str, dict[str, object]],
) -> int:
    mismatches = 0
    for responsibility_id in ISSUE66_RESPONSIBILITIES:
        for vector in cases[responsibility_id]["vectors"]:
            derivation = vector["expected_observation"]["derivation"]
            expected_keys = [authority_key(item) for item in vector["authorities"]]
            if vector["vector_id"] not in derivation["reason"]:
                mismatches += 1
            if derivation["authority_keys"] != expected_keys:
                mismatches += 1
    return mismatches


def issue66_relevant_covered_heading_count(
    coverage: list[dict[str, object]],
    cases: dict[str, dict[str, object]],
) -> int:
    vectors = all_vector_map(cases)
    return sum(
        heading["disposition"] == "covered"
        and heading_is_issue66_relevant(heading, vectors)
        for record in coverage
        for heading in record["headings"]
    )


def open_domain_source_audit_required(
    coverage: list[dict[str, object]],
) -> bool:
    reasons = {
        (record["contract"]["path"], heading["heading_path"]): heading.get(
            "reason", ""
        )
        for record in coverage
        for heading in record["headings"]
    }
    binding_reason = reasons.get(
        (
            "docs/contracts/shared-governance-provider.md",
            "Governance Binding Registry / Read-only provider and installation boundary",
        ),
        "",
    )
    evidence_reason = reasons.get(
        ("docs/contracts/evidence-requirements.md", "Explicit exclusions"), ""
    )
    return (
        "mandatory #66 source audit" in binding_reason
        and "provider discovery" in binding_reason
        and "process execution" in binding_reason
        and "mandatory #66 source audit" in evidence_reason
        and "source-target reads" in evidence_reason
        and "evidence production" in evidence_reason
    )


def audit_review3_metadata(
    responsibilities: dict[str, dict[str, object]],
    coverage: list[dict[str, object]],
    cases: dict[str, dict[str, object]],
) -> dict[str, int | str]:
    violations = coverage_authority_binding_violations(coverage, cases)
    source_audit = open_domain_source_audit_required(coverage)
    return {
        "derivation_mismatches": derivation_mismatches(cases),
        "blocking_gaps_nonempty": sum(
            bool(responsibilities[item]["blocking_gaps"])
            for item in ISSUE66_RESPONSIBILITIES
        ),
        "context_runtime_overclaims": context_only_runtime_overclaims(
            coverage, cases
        ),
        "gb_read_only_unrelated_derivation_violations": len(
            governance_bindings_read_only_unrelated_derivations(cases)
        ),
        "gb_read_only_finite_witness_count": len(
            GB_READ_ONLY_FINITE_WITNESS_VECTOR_IDS
        ),
        "gb_read_only_witness_classification": (
            "PASS"
            if validate_governance_bindings_read_only_witness(cases)
            else "FAIL"
        ),
        "gb_read_only_heading_classification": "MIXED_CONTEXT",
        "coverage_authority_violations": len(violations),
        "coverage_authority": "PASS" if not violations else "FAIL",
        "coverage_full_vector_map_size": len(all_vector_map(cases)),
        "coverage_relevant_headings": issue66_relevant_covered_heading_count(
            coverage, cases
        ),
        "open_domain_source_audit_required": "yes" if source_audit else "no",
    }
