"""Independent fixed owner matrix oracle; candidate conformance, not API execution."""
import copy
import json
from pathlib import Path
import sys
import unittest

sys.dont_write_bytecode = True
from jsonschema import Draft202012Validator
from schema_catalog import bundle, operation_schemas

ROOT = Path(__file__).resolve().parents[1]
PROTOCOL = {'invalid_request', 'unsupported_version'}
OBSERVATION = {
    'inapplicable': {'component_not_declared'},
    'not_found': {'subject_not_found'},
    'admission_refused': {'repository_not_authorized'},
    'controlled_failure': {'governance_load_failed', 'response_too_large', 'internal_failure'},
    'protocol_error': PROTOCOL,
}
# Deliberately independent of api.schema.json and the generator: verification oracle.
EXPECTED = {
    'discover': {'protocol_error': PROTOCOL, 'controlled_failure': {'internal_failure'}},
    'inspect': OBSERVATION,
    'explain': OBSERVATION,
    'check_repository_integrity': {
        'inapplicable': {'repository_integrity_not_declared'},
        'admission_refused': {'repository_not_authorized', 'profile_not_authorized',
                             'execution_plan_not_authorized', 'execution_guarantee_unavailable'},
        'stale_observation': {'stale_observation'},
        'incomparable_observation': {'incomparable_observation'},
        'controlled_failure': {'governance_load_failed', 'local_context_invalid',
                               'response_too_large', 'receipt_persistence_failed', 'internal_failure'},
        'execution_incomplete': {'execution_incomplete', 'host_guard_lost_after_start',
                                 'receipt_persistence_failed'},
        'protocol_error': PROTOCOL,
    },
    'read_execution_output': {
        'not_found': {'subject_not_found'},
        'controlled_failure': {'output_not_started', 'output_expired', 'output_access_refused',
                               'output_unavailable', 'range_not_retained', 'invalid_output_range',
                               'internal_failure'},
        'protocol_error': PROTOCOL,
    },
    None: {'protocol_error': PROTOCOL},
}


def error_response(operation, outcome, code):
    incomplete = outcome == 'execution_incomplete'
    return {'api_version': 1, 'operation': operation, 'outcome': outcome,
            'error': {'code': code, 'stage': None, 'detail': None,
                      'attempt_id': '1' * 64 if incomplete else None,
                      'effects_possible': incomplete}}


class ErrorAttribution(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.api = json.loads((ROOT / 'schemas/api.schema.json').read_text())
        cls.global_validator = Draft202012Validator(bundle(cls.api, cls.api['$defs']['response']))
        cls.operation_validators = {op: Draft202012Validator(operation_schemas(cls.api, op)[1])
                                    for op in EXPECTED if op is not None}

    def test_exhaustive_independent_operation_outcome_code_matrix(self):
        codes = set().union(*(codes for families in EXPECTED.values() for codes in families.values()))
        outcomes = set().union(*(set(families) for families in EXPECTED.values()))
        definitions = self.api['$defs']
        code_definitions = {k: v for k, v in definitions.items() if k.startswith('error_code_')}
        self.assertEqual(code_definitions, {'error_code_' + code: {'const': code} for code in codes})
        for outcome in outcomes:
            expected_codes = set().union(*(families.get(outcome, set()) for families in EXPECTED.values()))
            self.assertEqual(set(definitions['error_codes_' + outcome]), {'anyOf'})
            actual_codes = {item['$ref'].removeprefix('#/$defs/error_code_')
                            for item in definitions['error_codes_' + outcome]['anyOf']}
            self.assertEqual(actual_codes, expected_codes)
        for op, families in EXPECTED.items():
            for outcome in sorted(outcomes):
                for code in sorted(codes | {'unknown_error'}):
                    with self.subTest(operation=op, outcome=outcome, code=code):
                        data = error_response(op, outcome, code)
                        expected = code in families.get(outcome, set())
                        self.assertEqual(self.global_validator.is_valid(data), expected)
                        for advertised_op, validator in self.operation_validators.items():
                            self.assertEqual(validator.is_valid(data), expected and op == advertised_op)

    def test_original_successes_remain_valid(self):
        for op, name in [('discover', 'response-discover'), ('inspect', 'response-inspect-all'),
                         ('explain', 'response-explain-role'),
                         ('check_repository_integrity', 'response-check-nonpass'),
                         ('read_execution_output', 'response-output-complete-eof')]:
            with self.subTest(operation=op):
                value = json.loads((ROOT / ('examples/' + name + '.json')).read_text())
                self.global_validator.validate(value)
                self.operation_validators[op].validate(value)

    def test_incomplete_metadata_and_persistence_boundaries(self):
        for code in EXPECTED['check_repository_integrity']['execution_incomplete']:
            data = error_response('check_repository_integrity', 'execution_incomplete', code)
            self.global_validator.validate(data)
            for key, values in [('attempt_id', [None]), ('effects_possible', [None, False])]:
                for value in values:
                    bad = copy.deepcopy(data)
                    bad['error'][key] = value
                    self.assertFalse(self.global_validator.is_valid(bad), (code, key, value))
        for code in ['receipt_persistence_failed', 'local_context_invalid']:
            data = error_response('check_repository_integrity', 'controlled_failure', code)
            self.global_validator.validate(data)
            for value in [None, True]:
                data['error']['effects_possible'] = value
                self.assertFalse(self.global_validator.is_valid(data))


if __name__ == '__main__':
    unittest.main()
