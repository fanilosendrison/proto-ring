"""Regression controls for candidate tooling, not a public API implementation."""
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
from jsonschema import Draft202012Validator
from referencing import Registry, Resource
from schema_catalog import discovery_response, verify_discovery

ROOT = Path(__file__).resolve().parents[1]
API = json.loads((ROOT / 'schemas/api.schema.json').read_text())


def load(name):
    return json.loads((ROOT / name).read_text())


class SchemaControls(unittest.TestCase):
    def test_subject_inventory_and_single_declaration_definitions(self):
        definitions = API['$defs']
        variants = definitions['subject_projection']['oneOf']
        self.assertEqual([v['properties']['kind']['const'] for v in variants],
                         ['capability', 'validation', 'binding', 'governed_object',
                          'projection', 'evidence_requirement'])
        for owner, field, declaration in [('projections_projection', 'projections', 'projection_decl'),
                                           ('evidence_projection', 'requirements', 'evidence_requirement_decl')]:
            self.assertEqual(definitions[owner]['properties'][field]['additionalProperties'],
                             {'$ref': '#/$defs/' + declaration})
            self.assertEqual(variants[4 if declaration == 'projection_decl' else 5]['properties']['declaration'],
                             {'$ref': '#/$defs/' + declaration})
        self.assertEqual([v['properties']['kind']['const'] for v in definitions['inspect_selector']['oneOf'][1:]],
                         ['capability', 'validation', 'binding', 'governed_object'])

    def test_response_controls_isolate_original_boundaries(self):
        registry = Registry().with_resource(API['$id'], Resource.from_contents(API))
        validator = Draft202012Validator(load('schemas/response.schema.json'), registry=registry)
        for suffix in ['wrong-profile-ref', 'outcome-code-mismatch']:
            with self.subTest(boundary=suffix):
                self.assertTrue(validator.is_valid(load('examples/response-control-' + suffix + '.json')))
                self.assertFalse(validator.is_valid(load('examples/negative-' + suffix + '.json')))
        cases = load('conformance/schema-cases.json')
        self.assertTrue(all(c['target'] == 'response' for c in cases if c['id'] in {
            'reject-negative-wrong-profile-ref', 'reject-negative-outcome-code-mismatch'}))

    def test_discovery_rejects_inventory_and_schema_drift(self):
        fixture = load('examples/response-discover.json')
        for label in ['missing', 'duplicate', 'sixth', 'empty-schema', 'external-ref', 'effects', 'preconditions']:
            with self.subTest(mutation=label):
                bad = copy.deepcopy(fixture)
                entries = bad['value']['operations']
                if label == 'missing':
                    entries.pop()
                elif label == 'duplicate':
                    entries[1] = copy.deepcopy(entries[0])
                elif label == 'sixth':
                    entries.append(copy.deepcopy(entries[0]))
                elif label == 'empty-schema':
                    entries[0]['request_schema'] = {}
                elif label == 'external-ref':
                    entries[0]['request_schema']['$ref'] = 'https://invalid.example/schema'
                elif label == 'effects':
                    entries[1]['effects'] = ['consumer_command_execution']
                else:
                    entries[3]['preconditions'].remove('coherent_workspace')
                with self.assertRaises(AssertionError):
                    verify_discovery(API, bad, [])

    def test_real_cli_detects_removal_of_intended_response_constraints(self):
        # Real subprocess and real fixture copies. Regenerate discovery after each
        # mutation so stale-artifact detection cannot mask the intended oracle.
        target = ROOT.parents[2] / 'target'
        target.mkdir(exist_ok=True)
        for boundary in ['wrong-profile-ref', 'outcome-code-mismatch']:
            with self.subTest(boundary=boundary), tempfile.TemporaryDirectory(prefix='schema-control-', dir=target) as tmp:
                fixture = Path(tmp) / 'candidate'
                shutil.copytree(ROOT, fixture, ignore=shutil.ignore_patterns('__pycache__'))
                api = copy.deepcopy(API)
                if boundary == 'wrong-profile-ref':
                    api['$defs']['profile_ref']['properties']['format'] = {'enum': ['ri-profile-v1', 'ri-context-v1']}
                else:
                    api['$defs']['error_codes_inapplicable']['enum'].append('invalid_request')
                (fixture / 'schemas/api.schema.json').write_text(json.dumps(api))
                (fixture / 'examples/response-discover.json').write_text(json.dumps(discovery_response(api)))
                environment = {**os.environ, 'PYTHONDONTWRITEBYTECODE': '1'}
                run = subprocess.run([sys.executable, str(fixture / 'conformance/validate-wire-schemas.py')],
                                     cwd=fixture, env=environment, capture_output=True, text=True)
                self.assertNotEqual(run.returncode, 0)
                self.assertIn('reject-negative-' + boundary, run.stderr + run.stdout)

    def test_all_shape_fixtures_have_explicit_targets(self):
        cases = load('conformance/schema-cases.json')
        self.assertEqual(len({c['id'] for c in cases}), len(cases))
        self.assertTrue(all(c['target'] in {'request', 'response', 'local'} for c in cases))
        fixtures = {c['document'] for c in cases}
        self.assertEqual(fixtures, {str(p.relative_to(ROOT)) for p in (ROOT / 'examples').glob('*.json')})


if __name__ == '__main__':
    unittest.main()
