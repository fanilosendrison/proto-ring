#!/usr/bin/env python3
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path
from unittest import mock

from proto_ring.repository_integrity import (
    CommandBinding,
    CommandObligation,
    ConsumerIntegrityProfile,
    EvaluationContext,
    IntegrityVerdict,
    ObligationStatus,
    ProfileAuthority,
    RepositoryPathSelector,
    ValidationDefinition,
    ValidationEnvironmentRealization,
    ValidationInstances,
    evaluate_consumer_profile,
)
from proto_ring.repository_state import StateCaptureError
from repository_integrity_test_support import RepositoryFixtureTestCase


class RuntimeEvaluationTests(RepositoryFixtureTestCase):
    def test_evaluation_module_is_directly_importable_in_fresh_interpreter(
        self,
    ) -> None:
        environment = dict(os.environ)
        environment["PYTHONDONTWRITEBYTECODE"] = "1"
        completed = subprocess.run(
            [
                sys.executable,
                "-c",
                "import proto_ring.repository_integrity_evaluation; "
                "print('DIRECT_IMPORT_OK')",
            ],
            capture_output=True,
            text=True,
            env=environment,
            check=False,
        )
        self.assertEqual(0, completed.returncode, completed.stderr)
        self.assertEqual("DIRECT_IMPORT_OK", completed.stdout.strip())
        self.assertNotIn("ImportError", completed.stderr)

    def test_basic_exit_status_mapping(self) -> None:
        cases = (
            (0, frozenset(), ObligationStatus.SATISFIED, IntegrityVerdict.PASS),
            (7, frozenset(), ObligationStatus.VIOLATED, IntegrityVerdict.NON_PASS),
            (42, frozenset({42}), ObligationStatus.UNDETERMINED, IntegrityVerdict.NON_PASS),
        )
        for code, undetermined, status, verdict in cases:
            with self.subTest(code=code):
                result = self.run_integrity(
                    [CommandObligation("probe", self.exit_code(code), undetermined)]
                )
                self.assertEqual(verdict, result.verdict)
                self.assertEqual(status, result.obligations[0].status)
                self.assertEqual("test-runtime-context", result.evaluation_context_identity)

    def test_missing_executable_is_undetermined(self) -> None:
        result = self.run_integrity(
            [CommandObligation("missing", ("proto-ring-missing-executable",))]
        )
        self.assertEqual(ObligationStatus.UNDETERMINED, result.obligations[0].status)
        self.assertIn("command could not be started", result.obligations[0].detail)

    def test_continuation_policy_and_order(self) -> None:
        marker = self.root / "marker"
        stopped = self.run_integrity(
            [
                CommandObligation("first", self.exit_code(7)),
                CommandObligation("second", self.marker_code(marker)),
            ],
            continue_after=False,
        )
        self.assertFalse(marker.exists())
        self.assertEqual(ObligationStatus.UNDETERMINED, stopped.obligations[1].status)

        log = self.root / "order.log"
        continued = self.run_integrity(
            [
                CommandObligation("one", self.append_code(log, "1")),
                CommandObligation("two", self.append_code(log, "2")),
            ]
        )
        self.assertEqual("12", log.read_text(encoding="utf-8"))
        self.assertEqual(["one", "two"], [item.name for item in continued.obligations])

    def test_explicit_environment_reaches_command_without_ambient_values(self) -> None:
        probe = self.python(
            "import os\n"
            "assert 'RI_AMBIENT' not in os.environ\n"
            "assert os.environ.get('RI_EXPLICIT') == 'present'\n"
        )
        with mock.patch.dict(os.environ, {"RI_AMBIENT": "ambient"}):
            result = self.run_integrity(
                [CommandObligation("env", probe)], env={"RI_EXPLICIT": "present"}
            )
        self.assertEqual(IntegrityVerdict.PASS, result.verdict)

    def _definition(
        self,
        validation_id: str,
        *,
        instances: ValidationInstances | None = None,
        prerequisites: tuple[str, ...] = (),
        code: str = "pass",
        environment: str = "python",
        undetermined: frozenset[int] = frozenset({42}),
    ) -> ValidationDefinition:
        return ValidationDefinition(
            validation_id,
            "repository-integrity-validation",
            prerequisites,
            None,
            instances or ValidationInstances("single"),
            CommandBinding(environment, ("-c", code), undetermined),
        )

    def _profile(
        self,
        definitions: list[ValidationDefinition],
        *,
        continue_after: bool = True,
    ) -> ConsumerIntegrityProfile:
        return ConsumerIntegrityProfile(
            self.repo.resolve(),
            self.repo / "AGENTS.md",
            1,
            ProfileAuthority("repository-integrity-profile", "profile-source"),
            frozenset({item.command.environment for item in definitions}),
            continue_after,
            {item.validation_id: item for item in definitions},
            tuple(item.validation_id for item in definitions),
            "persistent-profile-identity",
        )

    def _context(
        self,
        *,
        identity: str = "python-test-runtime",
        prefix: tuple[str, ...] | None = None,
        process_environment: dict[str, str] | None = None,
        extras: dict[str, ValidationEnvironmentRealization] | None = None,
    ) -> EvaluationContext:
        realization = ValidationEnvironmentRealization(
            "python",
            identity,
            prefix or (sys.executable,),
            self.environment() if process_environment is None else process_environment,
        )
        values = {"python": realization}
        values.update(extras or {})
        return EvaluationContext(values)

    def _for_each(self, *selectors: RepositoryPathSelector) -> ValidationInstances:
        return ValidationInstances("repository_paths", "for_each", selectors)

    def test_for_each_zero_instances_is_satisfied_without_presence_inference(self) -> None:
        validation = self._definition(
            "syntax",
            instances=self._for_each(RepositoryPathSelector("glob", "missing/*.py")),
        )
        result = evaluate_consumer_profile(
            self.repo, self._profile([validation]), self._context()
        )
        self.assertEqual(IntegrityVerdict.PASS, result.verdict)
        self.assertEqual(ObligationStatus.SATISFIED, result.validations[0].status)
        self.assertEqual((), result.validations[0].obligations)
        self.assertEqual((), result.obligations)

    def test_zero_instance_prerequisite_allows_dependent(self) -> None:
        marker = self.root / "dependent-ran"
        first = self._definition(
            "empty",
            instances=self._for_each(RepositoryPathSelector("glob", "missing/*.json")),
        )
        second = self._definition(
            "dependent",
            prerequisites=("empty",),
            code=(
                "from pathlib import Path\n"
                f"Path({os.fspath(marker)!r}).write_text('ran')"
            ),
        )
        result = evaluate_consumer_profile(
            self.repo, self._profile([first, second]), self._context()
        )
        self.assertTrue(marker.exists())
        self.assertEqual(
            [ObligationStatus.SATISFIED, ObligationStatus.SATISFIED],
            [item.status for item in result.validations],
        )

    def test_failed_resolution_is_undetermined_and_blocks_dependent(self) -> None:
        marker = self.root / "must-not-run"
        first = self._definition(
            "unresolved",
            instances=self._for_each(RepositoryPathSelector("glob", "*.py")),
        )
        second = self._definition(
            "dependent",
            prerequisites=("unresolved",),
            code=f"from pathlib import Path; Path({os.fspath(marker)!r}).touch()",
        )
        with mock.patch(
            "proto_ring.repository_integrity_evaluation.discover_repository_paths",
            side_effect=StateCaptureError("discovery unavailable"),
        ):
            result = evaluate_consumer_profile(
                self.repo, self._profile([first, second]), self._context()
            )
        self.assertFalse(marker.exists())
        self.assertEqual(
            [ObligationStatus.UNDETERMINED, ObligationStatus.UNDETERMINED],
            [item.status for item in result.validations],
        )

    def _aggregate_status(self, codes: tuple[int, ...]) -> ObligationStatus:
        for index, code in enumerate(codes):
            self.write(f"inputs/{index}.txt", str(code))
        command = (
            "import pathlib, sys\n"
            "code = int(pathlib.Path(sys.argv[1]).read_text())\n"
            "sys.exit(code)"
        )
        selectors = tuple(
            RepositoryPathSelector("path", f"inputs/{index}.txt")
            for index in range(len(codes))
        )
        validation = self._definition(
            "aggregate", instances=self._for_each(*selectors), code=command
        )
        result = evaluate_consumer_profile(
            self.repo, self._profile([validation]), self._context()
        )
        return result.validations[0].status

    def test_multiple_instance_aggregation_precedence_is_order_independent(self) -> None:
        cases = (
            ((0, 0), ObligationStatus.SATISFIED),
            ((0, 42), ObligationStatus.UNDETERMINED),
            ((0, 7), ObligationStatus.VIOLATED),
            ((7, 42), ObligationStatus.VIOLATED),
            ((42, 7), ObligationStatus.VIOLATED),
        )
        for codes, expected in cases:
            with self.subTest(codes=codes):
                self.assertEqual(expected, self._aggregate_status(codes))

    def test_selector_order_glob_sorting_absent_literals_and_component_scope(self) -> None:
        self.write("paths/z.py", "pass\n")
        self.write("paths/a.py", "pass\n")
        self.write("paths/nested/ignored.py", "pass\n")
        log = self.root / "selected-paths"
        validation = self._definition(
            "selection",
            instances=self._for_each(
                RepositoryPathSelector("glob", "paths/*.py"),
                RepositoryPathSelector("path", "absent.py"),
            ),
            code=(
                "from pathlib import Path\nimport sys\n"
                f"path = Path({os.fspath(log)!r})\n"
                "path.write_text((path.read_text() if path.exists() else '') + sys.argv[1] + '\\n')"
            ),
        )
        result = evaluate_consumer_profile(
            self.repo, self._profile([validation]), self._context()
        )
        self.assertEqual(
            "paths/a.py\npaths/z.py\nabsent.py\n",
            log.read_text(encoding="utf-8"),
        )
        self.assertEqual(3, len(result.obligations))

    def test_duplicate_selected_path_fails_closed_without_partial_success(self) -> None:
        self.write("duplicate.py", "pass\n")
        validation = self._definition(
            "duplicates",
            instances=self._for_each(
                RepositoryPathSelector("path", "duplicate.py"),
                RepositoryPathSelector("glob", "*.py"),
            ),
        )
        result = evaluate_consumer_profile(
            self.repo, self._profile([validation]), self._context()
        )
        self.assertEqual(ObligationStatus.UNDETERMINED, result.validations[0].status)
        self.assertEqual((), result.obligations)
        self.assertIn("duplicate selected repository path", result.validations[0].detail)

    def test_append_all_zero_selection_executes_one_base_only_instance(self) -> None:
        marker = self.root / "argv"
        validation = self._definition(
            "append",
            instances=ValidationInstances(
                "repository_paths",
                "append_all",
                (RepositoryPathSelector("glob", "missing/*.py"),),
            ),
            code=(
                "import pathlib, sys\n"
                f"pathlib.Path({os.fspath(marker)!r}).write_text(repr(sys.argv[1:]))"
            ),
        )
        result = evaluate_consumer_profile(
            self.repo, self._profile([validation]), self._context()
        )
        self.assertEqual("[]", marker.read_text(encoding="utf-8"))
        self.assertEqual(1, len(result.obligations))
        self.assertEqual(ObligationStatus.SATISFIED, result.validations[0].status)

    def test_missing_environment_is_undetermined(self) -> None:
        validation = self._definition("missing-environment")
        result = evaluate_consumer_profile(
            self.repo, self._profile([validation]), EvaluationContext({})
        )
        self.assertEqual(ObligationStatus.UNDETERMINED, result.validations[0].status)
        self.assertIn("environment is unavailable", result.obligations[0].detail)

    def test_context_identity_uses_only_required_opaque_realization_identity(self) -> None:
        validation = self._definition("context")
        profile = self._profile([validation])
        unrelated = ValidationEnvironmentRealization(
            "unrelated", "ignored", ("missing",), {"SECRET": "ignored"}
        )
        first = evaluate_consumer_profile(
            self.repo,
            profile,
            self._context(process_environment={"VALUE": "one"}),
        )
        second = evaluate_consumer_profile(
            self.repo,
            profile,
            self._context(
                prefix=(sys.executable, "-I"),
                process_environment={"VALUE": "two", "SECRET": "hidden"},
                extras={"unrelated": unrelated},
            ),
        )
        changed = evaluate_consumer_profile(
            self.repo, profile, self._context(identity="different-runtime")
        )
        self.assertEqual(first.evaluation_context_identity, second.evaluation_context_identity)
        self.assertNotEqual(first.evaluation_context_identity, changed.evaluation_context_identity)
        self.assertEqual(profile.identity, first.profile_identity)


if __name__ == "__main__":
    import unittest

    unittest.main()
