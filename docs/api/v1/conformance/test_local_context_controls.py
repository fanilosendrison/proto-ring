"""Independent exact-ID controls, not implementation or host qualification."""
import copy
import json
from pathlib import Path
import sys
import unittest

sys.dont_write_bytecode = True
from local_context_controls import validate_unique_realizations

ROOT = Path(__file__).resolve().parents[1]


class LocalContextControls(unittest.TestCase):
    def test_fixed_collision_dataset(self):
        for case in json.loads((ROOT / 'conformance/local-context-cases.json').read_text()):
            with self.subTest(case=case['id']):
                data = json.loads((ROOT / case['document']).read_text())
                if case['expect'] == 'accept':
                    validate_unique_realizations(data)
                else:
                    with self.assertRaisesRegex(ValueError, 'local_context_invalid'):
                        validate_unique_realizations(data)

    def test_exact_string_equality_without_normalization(self):
        entries = [{'environment_id': name} for name in ['rust', 'Rust', 'rust ', 'rúst', 'ru\u0301st']]
        validate_unique_realizations({'realizations': entries})
        for item in entries:
            duplicate = copy.deepcopy(entries) + [copy.deepcopy(item)]
            with self.assertRaises(ValueError):
                validate_unique_realizations({'realizations': duplicate})


if __name__ == '__main__':
    unittest.main()
