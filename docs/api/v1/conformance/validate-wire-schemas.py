#!/usr/bin/env python3
"""Candidate syntax controls only; no engine, admission or security qualification."""
import json
import sys
from pathlib import Path
from jsonschema import Draft202012Validator
from referencing import Registry, Resource
sys.dont_write_bytecode = True
from schema_catalog import verify_discovery
from local_context_controls import validate_unique_realizations

ROOT = Path(__file__).resolve().parents[1]


def strict_object_pairs(items):
    result = {}
    for key, value in items:
        if key in result:
            raise ValueError('duplicate JSON key: ' + key)
        result[key] = value
    return result


def reject_constant(value):
    raise ValueError('non-JSON numeric constant: ' + value)


def load(path):
    return json.loads(path.read_bytes(), object_pairs_hook=strict_object_pairs,
                      parse_constant=reject_constant)


def offline_retrieval(uri):
    raise ValueError('Network schema retrieval forbidden: ' + uri)


def main():
    registry = Registry(retrieve=offline_retrieval)
    documents = [load(path) for path in sorted((ROOT / 'schemas').glob('*.json'))]
    for document in documents:
        Draft202012Validator.check_schema(document)
        registry = registry.with_resource(document['$id'], Resource.from_contents(document))
    def check_references(value, resolver):
        if isinstance(value, dict):
            if '$ref' in value:
                resolver.lookup(value['$ref'])
            for child in value.values():
                check_references(child, resolver)
        elif isinstance(value, list):
            for child in value:
                check_references(child, resolver)
    for document in documents:
        check_references(document, registry.resolver(document['$id']))
    by_id = {document['$id']: document for document in documents}
    api = by_id['urn:proto-ring:api-01:v1:api']
    validators = {target: Draft202012Validator(
        load(ROOT / 'schemas' / filename), registry=registry)
        for target, filename in [('request', 'request.schema.json'),
                                 ('response', 'response.schema.json'),
                                 ('local', 'local-context.schema.json')]}
    cases = load(ROOT / 'conformance/schema-cases.json')
    assert len({case['id'] for case in cases}) == len(cases), 'Duplicate schema case ID'
    failures = []
    accepted = []
    for case in cases:
        target = case['target']  # Never derive the target from a filename.
        validator = validators[target]
        try:
            data = load(ROOT / case['document'])
            valid = validator.is_valid(data)
            if case['expect'] == 'accept' and valid:
                accepted.append((target, data))
        except (ValueError, UnicodeError):
            valid = False
        if valid != (case['expect'] == 'accept'):
            failures.append(case['id'])
        if 'control_document' in case:
            control = load(ROOT / case['control_document'])
            assert validator.is_valid(control), case['id'] + ': positive control rejected'
            # Controls differ only at the named boundary, proving rejection is not
            # caused by routing a response to the request schema or missing provenance.
            pointer = case['boundary_pointer'].strip('/').split('/')
            left, right = data, control
            for key in pointer[:-1]:
                left, right = left[key], right[key]
            repaired = json.loads(json.dumps(data))
            cursor = repaired
            for key in pointer[:-1]:
                cursor = cursor[key]
            cursor[pointer[-1]] = right[pointer[-1]]
            assert repaired == control and left[pointer[-1]] != right[pointer[-1]], case['id']
    assert not failures, 'Schema cases failed: ' + ', '.join(failures)
    verify_discovery(api, load(ROOT / 'examples/response-discover.json'), accepted)
    ranges = load(ROOT / 'conformance/output-range-cases.json')
    for case in ranges:
        assert case['qualified'] is False
        validators['request'].validate(case['request'])
        validators['response'].validate(load(ROOT / case['response_document']))
    semantic = load(ROOT / 'conformance/semantic-cases.json')
    assert [row['id'] for row in semantic] == [f'A{i:02}' for i in range(1, len(semantic) + 1)]
    assert all(row['qualified'] is False and row['kind'] == 'semantic_review_required' for row in semantic)
    assert all(set(row) == {'id', 'kind', 'scenario', 'expected_boundary', 'qualified'} for row in semantic)
    local_cases = load(ROOT / 'conformance/local-context-cases.json')
    for case in local_cases:
        document = load(ROOT / case['document'])
        validators['local'].validate(document)  # Duplicate IDs are shape-valid.
        try:
            validate_unique_realizations(document)
            unique = True
        except ValueError:
            unique = False
        assert unique == (case['expect'] == 'accept'), case['id']
    print(f'LOCAL_REALIZATION_UNIQUENESS_CASES_PASS={len(local_cases)}; NO_API_RUNTIME_ATTESTATION')
    print(f'SCHEMA_VALIDATION_PASS={len(cases)}; DISCOVERY_DERIVATION_PASS=5')
    print(f'OUTPUT_RANGE_FIXTURE_SHAPES_PASS={len(ranges)}; NO_OUTPUT_RUNTIME_ATTESTATION')
    print(f'SEMANTIC_RUNTIME_CASES_NOT_QUALIFIED={len(semantic)}')


if __name__ == '__main__':
    main()
