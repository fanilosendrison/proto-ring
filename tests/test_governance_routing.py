from __future__ import annotations

from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from proto_ring.governance_routing import GovernanceRoutingError, resolve_path


ROUTE = ("governance", "responsibility", "path")


class GovernanceRoutingTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repository = self.root / "repository"
        self.repository.mkdir()
        self.target = self.repository / "config" / "target.yaml"
        self.target.parent.mkdir()
        self.target.write_text("target: true\n", encoding="utf-8")

    def routing(self, declared_path: object = "config/target.yaml") -> object:
        return {"governance": {"responsibility": {"path": declared_path}}}

    def assert_routing_error(self, routing: object, route: object = ROUTE) -> None:
        with self.assertRaises(GovernanceRoutingError):
            resolve_path(self.repository, routing, route)  # type: ignore[arg-type]

    def test_valid_exact_nested_route_resolves(self) -> None:
        result = resolve_path(self.repository, self.routing(), ROUTE)  # type: ignore[arg-type]

        self.assertEqual(result.target, self.target.resolve())

    def test_returned_route_is_exact_supplied_tuple(self) -> None:
        supplied_route = ("governance", "responsibility", "path")

        result = resolve_path(
            self.repository, self.routing(), supplied_route  # type: ignore[arg-type]
        )

        self.assertEqual(result.route, supplied_route)

    def test_returned_declared_path_is_exact_configured_string(self) -> None:
        declared_path = "config/../config/target.yaml"

        result = resolve_path(
            self.repository, self.routing(declared_path), ROUTE  # type: ignore[arg-type]
        )

        self.assertEqual(result.declared_path, declared_path)

    def test_returned_target_is_contained_resolved_target(self) -> None:
        result = resolve_path(self.repository, self.routing(), ROUTE)  # type: ignore[arg-type]

        self.assertEqual(result.target, (self.repository / "config/target.yaml").resolve())

    def test_empty_route_fails(self) -> None:
        self.assert_routing_error(self.routing(), ())

    def test_string_route_fails(self) -> None:
        self.assert_routing_error(self.routing(), "governance")

    def test_bytes_route_fails(self) -> None:
        self.assert_routing_error(self.routing(), b"governance")

    def test_non_string_route_segment_fails(self) -> None:
        self.assert_routing_error(self.routing(), ("governance", 1, "path"))

    def test_empty_route_segment_fails(self) -> None:
        self.assert_routing_error(self.routing(), ("governance", "", "path"))

    def test_non_mapping_routing_input_fails(self) -> None:
        self.assert_routing_error(["governance"])

    def test_missing_first_segment_fails(self) -> None:
        self.assert_routing_error({"other": self.routing()})

    def test_missing_intermediate_segment_fails(self) -> None:
        self.assert_routing_error({"governance": {"other": {"path": "config/target.yaml"}}})

    def test_non_mapping_intermediate_value_fails(self) -> None:
        self.assert_routing_error({"governance": "config/target.yaml"})

    def test_missing_leaf_fails(self) -> None:
        self.assert_routing_error({"governance": {"responsibility": {}}})

    def test_non_string_leaf_fails(self) -> None:
        self.assert_routing_error(self.routing(1))

    def test_empty_string_leaf_fails(self) -> None:
        self.assert_routing_error(self.routing(""))

    def test_absolute_declared_path_fails(self) -> None:
        self.assert_routing_error(self.routing(str(self.target.resolve())))

    def test_repository_escape_fails(self) -> None:
        outside = self.root / "outside.yaml"
        outside.write_text("outside: true\n", encoding="utf-8")

        self.assert_routing_error(self.routing("../outside.yaml"))

    def test_missing_unavailable_target_fails(self) -> None:
        self.assert_routing_error(self.routing("config/missing.yaml"))

    def test_plausible_sibling_route_does_not_act_as_fallback(self) -> None:
        routing = {
            "governance": {
                "requested": {},
                "plausible": {"path": "config/target.yaml"},
            }
        }

        self.assert_routing_error(routing, ("governance", "requested", "path"))

    def test_two_valid_routes_resolve_only_the_explicitly_requested_one(self) -> None:
        other = self.repository / "config" / "other.yaml"
        other.write_text("other: true\n", encoding="utf-8")
        routing = {
            "governance": {
                "first": {"path": "config/target.yaml"},
                "second": {"path": "config/other.yaml"},
            }
        }

        result = resolve_path(
            self.repository, routing, ("governance", "second", "path")
        )

        self.assertEqual(result.route, ("governance", "second", "path"))
        self.assertEqual(result.declared_path, "config/other.yaml")
        self.assertEqual(result.target, other.resolve())

    def test_symlink_escape_outside_repository_fails(self) -> None:
        outside = self.root / "outside.yaml"
        outside.write_text("outside: true\n", encoding="utf-8")
        link = self.repository / "config" / "outside-link.yaml"
        link.symlink_to(outside)

        self.assert_routing_error(self.routing("config/outside-link.yaml"))


if __name__ == "__main__":
    unittest.main()
