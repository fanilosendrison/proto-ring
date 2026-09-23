import unittest

import proto_ring


class PackageTest(unittest.TestCase):
    def test_package_is_importable(self) -> None:
        self.assertIsNotNone(proto_ring)


if __name__ == "__main__":
    unittest.main()
