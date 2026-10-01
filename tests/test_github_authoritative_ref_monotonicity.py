from __future__ import annotations

import io
import json
import os
from pathlib import Path
import socket
import ssl
import tempfile
import unittest
from unittest import mock
import urllib.error

import truststore
import yaml

from proto_ring import github_authoritative_ref_monotonicity as conformance


RULESET_ID = 12345


class FakeResponse:
    def __init__(self, payload: bytes, *, status: int = 200) -> None:
        self.payload = payload
        self.status = status

    def __enter__(self) -> FakeResponse:
        return self

    def __exit__(self, *_args: object) -> None:
        return None

    def read(self) -> bytes:
        return self.payload


class GitHubAuthoritativeRefMonotonicityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="github-arm-")
        self.binding_path = Path(self.temporary.name) / "binding.md"
        self.write_binding()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def write_binding(
        self,
        *,
        provider: object = "github",
        owner: object = "example-owner",
        repository: object = "example-repository",
        authoritative_ref: object = "refs/heads/trunk",
        mechanism: object = "github-repository-ruleset",
        ruleset_id: object = RULESET_ID,
    ) -> None:
        metadata = {
            "authoritative_ref_monotonicity": {
                "repository": {
                    "provider": provider,
                    "owner": owner,
                    "name": repository,
                },
                "authoritative_ref": authoritative_ref,
                "protection": {
                    "mechanism": mechanism,
                    "ruleset_id": ruleset_id,
                },
            }
        }
        self.binding_path.write_text(
            "---\n" + yaml.safe_dump(metadata, sort_keys=False) + "---\n\n# Binding\n",
            encoding="utf-8",
        )
    @staticmethod
    def response(items: object) -> FakeResponse:
        return FakeResponse(json.dumps(items).encode("utf-8"))
    def evaluate(self, items: object) -> conformance.ConformanceResult:
        with mock.patch(
            "urllib.request.urlopen", return_value=self.response(items)
        ):
            return conformance.check(self.binding_path)
    def test_required_rules_under_bound_ruleset_are_satisfied(self) -> None:
        result = self.evaluate(
            [
                {"type": "deletion", "ruleset_id": RULESET_ID},
                {"type": "non_fast_forward", "ruleset_id": RULESET_ID},
            ]
        )
        self.assertEqual(conformance.ConformanceStatus.SATISFIED, result.status)
    def test_additional_bound_rule_is_satisfied(self) -> None:
        result = self.evaluate(
            [
                {"type": "deletion", "ruleset_id": RULESET_ID},
                {"type": "non_fast_forward", "ruleset_id": RULESET_ID},
                {"type": "required_status_checks", "ruleset_id": RULESET_ID},
            ]
        )
        self.assertEqual(conformance.ConformanceStatus.SATISFIED, result.status)
    def test_missing_deletion_is_violated(self) -> None:
        result = self.evaluate(
            [{"type": "non_fast_forward", "ruleset_id": RULESET_ID}]
        )
        self.assertEqual(conformance.ConformanceStatus.VIOLATED, result.status)
        self.assertIn("deletion", result.diagnostics[0])
    def test_missing_non_fast_forward_is_violated(self) -> None:
        result = self.evaluate([{"type": "deletion", "ruleset_id": RULESET_ID}])
        self.assertEqual(conformance.ConformanceStatus.VIOLATED, result.status)
        self.assertIn("non_fast_forward", result.diagnostics[0])
    def test_rules_only_under_another_ruleset_are_violated(self) -> None:
        result = self.evaluate(
            [
                {"type": "deletion", "ruleset_id": RULESET_ID + 1},
                {"type": "non_fast_forward", "ruleset_id": RULESET_ID + 1},
            ]
        )
        self.assertEqual(conformance.ConformanceStatus.VIOLATED, result.status)
    def test_bound_ruleset_absent_is_violated(self) -> None:
        result = self.evaluate([])
        self.assertEqual(conformance.ConformanceStatus.VIOLATED, result.status)
        self.assertIn("deletion", result.diagnostics[0])
        self.assertIn("non_fast_forward", result.diagnostics[0])
    def test_binding_without_frontmatter_is_violated(self) -> None:
        self.binding_path.write_text("# Binding\n", encoding="utf-8")
        result = conformance.check(self.binding_path)
        self.assertEqual(conformance.ConformanceStatus.VIOLATED, result.status)
    def test_wrong_provider_is_violated(self) -> None:
        self.write_binding(provider="other")
        self.assertEqual(
            conformance.ConformanceStatus.VIOLATED,
            conformance.check(self.binding_path).status,
        )
    def test_wrong_mechanism_is_violated(self) -> None:
        self.write_binding(mechanism="other")
        self.assertEqual(
            conformance.ConformanceStatus.VIOLATED,
            conformance.check(self.binding_path).status,
        )
    def test_tag_ref_is_violated(self) -> None:
        self.write_binding(authoritative_ref="refs/tags/trunk")
        self.assertEqual(
            conformance.ConformanceStatus.VIOLATED,
            conformance.check(self.binding_path).status,
        )
    def test_empty_branch_suffix_is_violated(self) -> None:
        self.write_binding(authoritative_ref="refs/heads/")
        self.assertEqual(
            conformance.ConformanceStatus.VIOLATED,
            conformance.check(self.binding_path).status,
        )
    def test_zero_ruleset_id_is_violated(self) -> None:
        self.write_binding(ruleset_id=0)
        self.assertEqual(
            conformance.ConformanceStatus.VIOLATED,
            conformance.check(self.binding_path).status,
        )
    def test_boolean_ruleset_id_is_violated(self) -> None:
        self.write_binding(ruleset_id=True)
        self.assertEqual(
            conformance.ConformanceStatus.VIOLATED,
            conformance.check(self.binding_path).status,
        )
    def test_canonical_frontmatter_strengthening_is_violated(self) -> None:
        base = self.binding_path.read_bytes()
        cases = {
            "BOM": b"\xef\xbb\xbf" + base,
            "CRLF": base.replace(b"\n", b"\r\n"),
            "duplicate": base.replace(
                b"---\n", b"---\nauthoritative_ref_monotonicity: {}\n", 1
            ),
            "anchor": base.replace(
                b"---\n", b"---\nignored: &anchor value\n", 1
            ),
            "noncanonical integer": base.replace(
                f"ruleset_id: {RULESET_ID}".encode(), b"ruleset_id: 01"
            ),
        }
        for label, data in cases.items():
            with self.subTest(label=label):
                self.binding_path.write_bytes(data)
                result = conformance.check(self.binding_path)
                self.assertEqual(
                    conformance.ConformanceStatus.VIOLATED, result.status
                )
    def test_empty_owner_or_repository_is_violated(self) -> None:
        for field in ("owner", "repository"):
            with self.subTest(field=field):
                arguments = {field: ""}
                self.write_binding(**arguments)
                self.assertEqual(
                    conformance.ConformanceStatus.VIOLATED,
                    conformance.check(self.binding_path).status,
                )
    def test_http_errors_are_undetermined(self) -> None:
        for status in (403, 429, 500):
            with self.subTest(status=status):
                error = urllib.error.HTTPError(
                    "https://example.invalid", status, "failure", {}, None
                )
                with mock.patch("urllib.request.urlopen", side_effect=error):
                    result = conformance.check(self.binding_path)
                self.assertEqual(
                    conformance.ConformanceStatus.UNDETERMINED, result.status
                )
    def test_url_error_is_undetermined(self) -> None:
        with mock.patch(
            "urllib.request.urlopen",
            side_effect=urllib.error.URLError("unavailable"),
        ):
            result = conformance.check(self.binding_path)
        self.assertEqual(conformance.ConformanceStatus.UNDETERMINED, result.status)
    def test_direct_tls_certificate_verification_error_is_undetermined(self) -> None:
        error = ssl.SSLCertVerificationError(1, "certificate verify failed")
        with mock.patch("urllib.request.urlopen", side_effect=error):
            result = conformance.check(self.binding_path)
        self.assertEqual(conformance.ConformanceStatus.UNDETERMINED, result.status)
        self.assertEqual(
            ("provider request failed: TLS certificate verification error",),
            result.diagnostics,
        )
    def test_wrapped_tls_certificate_verification_error_is_undetermined(self) -> None:
        certificate_error = ssl.SSLCertVerificationError(
            1,
            "certificate verify failed",
        )
        error = urllib.error.URLError(certificate_error)
        with mock.patch("urllib.request.urlopen", side_effect=error):
            result = conformance.check(self.binding_path)
        self.assertEqual(conformance.ConformanceStatus.UNDETERMINED, result.status)
        self.assertEqual(
            ("provider request failed: TLS certificate verification error",),
            result.diagnostics,
        )
    def test_timeout_is_undetermined(self) -> None:
        for error in (TimeoutError(), socket.timeout()):
            with self.subTest(error=type(error).__name__):
                with mock.patch("urllib.request.urlopen", side_effect=error):
                    result = conformance.check(self.binding_path)
                self.assertEqual(
                    conformance.ConformanceStatus.UNDETERMINED, result.status
                )
    def test_malformed_utf8_is_undetermined(self) -> None:
        with mock.patch(
            "urllib.request.urlopen", return_value=FakeResponse(b"\xff")
        ):
            result = conformance.check(self.binding_path)
        self.assertEqual(conformance.ConformanceStatus.UNDETERMINED, result.status)
    def test_malformed_json_is_undetermined(self) -> None:
        with mock.patch(
            "urllib.request.urlopen", return_value=FakeResponse(b"not-json")
        ):
            result = conformance.check(self.binding_path)
        self.assertEqual(conformance.ConformanceStatus.UNDETERMINED, result.status)
    def test_wrong_json_top_level_is_undetermined(self) -> None:
        result = self.evaluate({"type": "deletion", "ruleset_id": RULESET_ID})
        self.assertEqual(conformance.ConformanceStatus.UNDETERMINED, result.status)
    def test_malformed_list_item_is_undetermined(self) -> None:
        for item in (
            "not-a-mapping",
            {"type": "deletion"},
            {"ruleset_id": 1},
            {"type": "deletion", "ruleset_id": True},
        ):
            with self.subTest(item=item):
                result = self.evaluate([item])
                self.assertEqual(
                    conformance.ConformanceStatus.UNDETERMINED, result.status
                )
    def test_non_success_response_status_is_undetermined(self) -> None:
        with mock.patch(
            "urllib.request.urlopen",
            return_value=FakeResponse(b"[]", status=301),
        ):
            result = conformance.check(self.binding_path)
        self.assertEqual(conformance.ConformanceStatus.UNDETERMINED, result.status)
    def test_pagination_aggregates_required_rules(self) -> None:
        first_page = [
            {"type": "deletion", "ruleset_id": RULESET_ID},
            *(
                {"type": f"extra-{index}", "ruleset_id": RULESET_ID + 1}
                for index in range(99)
            ),
        ]
        second_page = [{"type": "non_fast_forward", "ruleset_id": RULESET_ID}]
        with mock.patch(
            "urllib.request.urlopen",
            side_effect=[self.response(first_page), self.response(second_page)],
        ) as urlopen:
            result = conformance.check(self.binding_path)
        self.assertEqual(conformance.ConformanceStatus.SATISFIED, result.status)
        self.assertEqual(2, urlopen.call_count)
    def test_pagination_hard_cap_is_undetermined(self) -> None:
        full_page = [
            {"type": f"extra-{index}", "ruleset_id": RULESET_ID + 1}
            for index in range(100)
        ]
        with mock.patch(
            "urllib.request.urlopen", return_value=self.response(full_page)
        ) as urlopen:
            result = conformance.check(self.binding_path)
        self.assertEqual(conformance.ConformanceStatus.UNDETERMINED, result.status)
        self.assertEqual(100, urlopen.call_count)
    def test_branch_is_stripped_then_encoded_as_one_path_segment(self) -> None:
        self.write_binding(authoritative_ref="refs/heads/release/v1")
        with mock.patch(
            "urllib.request.urlopen", return_value=self.response([])
        ) as urlopen:
            conformance.check(self.binding_path)
        url = urlopen.call_args.args[0].full_url
        self.assertIn("/rules/branches/release%2Fv1", url)
        self.assertNotIn("/rules/branches/refs", url)
    def test_owner_and_repository_are_encoded_as_path_segments(self) -> None:
        self.write_binding(owner="owner/name", repository="repository name")
        with mock.patch(
            "urllib.request.urlopen", return_value=self.response([])
        ) as urlopen:
            conformance.check(self.binding_path)
        url = urlopen.call_args.args[0].full_url
        self.assertIn("/repos/owner%2Fname/repository%20name/", url)
    def test_request_has_exact_public_headers_and_no_authorization(self) -> None:
        with mock.patch(
            "urllib.request.urlopen", return_value=self.response([])
        ) as urlopen:
            conformance.check(self.binding_path)
        request = urlopen.call_args.args[0]
        headers = dict(request.header_items())
        self.assertEqual("application/vnd.github+json", headers["Accept"])
        self.assertEqual("2026-03-10", headers["X-github-api-version"])
        self.assertEqual(
            "proto-ring-github-arm-conformance", headers["User-agent"]
        )
        self.assertEqual(10, urlopen.call_args.kwargs["timeout"])
        self.assertIn("context", urlopen.call_args.kwargs)
        self.assertIsInstance(
            urlopen.call_args.kwargs["context"], truststore.SSLContext
        )
        self.assertIsNone(request.get_header("Authorization"))
    def test_ambient_tokens_do_not_add_authorization_header(self) -> None:
        with mock.patch.dict(
            os.environ,
            {"GITHUB_TOKEN": "ignored-one", "GH_TOKEN": "ignored-two"},
        ):
            with mock.patch(
                "urllib.request.urlopen", return_value=self.response([])
            ) as urlopen:
                result = conformance.check(self.binding_path)
        self.assertEqual(conformance.ConformanceStatus.VIOLATED, result.status)
        request = urlopen.call_args.args[0]
        self.assertIsNone(request.get_header("Authorization"))
    def test_implementation_has_no_credential_discovery_logic(self) -> None:
        source = Path(conformance.__file__).read_text(encoding="utf-8")
        for forbidden in ("GITHUB_TOKEN", "GH_TOKEN", "Authorization", "import os"):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)
    def test_implementation_does_not_mutate_global_ssl_state(self) -> None:
        source = Path(conformance.__file__).read_text(encoding="utf-8")
        for forbidden in (
            "inject_into_ssl",
            "_create_unverified_context",
            "CERT_NONE",
            "check_hostname = False",
            "SSL_CERT_FILE",
            "REQUESTS_CA_BUNDLE",
            "CURL_CA_BUNDLE",
            "import certifi", "from certifi", "certifi.",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)
    def test_implementation_contains_no_consumer_binding(self) -> None:
        source = Path(conformance.__file__).read_text(encoding="utf-8")
        for forbidden in (
            "turnlock-rust",
            "fanilosendrison",
            "24106121",
            "24106347",
            "refs/heads/main",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)
    def test_run_maps_status_to_exit_code(self) -> None:
        for status, expected in (
            (conformance.ConformanceStatus.SATISFIED, 0),
            (conformance.ConformanceStatus.VIOLATED, 1),
            (conformance.ConformanceStatus.UNDETERMINED, 2),
        ):
            with self.subTest(status=status):
                result = conformance.ConformanceResult(status, ("diagnostic",))
                with mock.patch.object(conformance, "check", return_value=result):
                    with mock.patch("sys.stdout", new_callable=io.StringIO) as output:
                        exit_code = conformance.run(self.binding_path)
                self.assertEqual(expected, exit_code)
                self.assertEqual(
                    "github authoritative ref monotonicity effective rules: "
                    f"{status.value}",
                    output.getvalue().splitlines()[0],
                )


if __name__ == "__main__":
    unittest.main()
