"""Validate the Flatpak icon and its canonical artwork provenance."""

import hashlib
from pathlib import Path
import re
import struct
import unittest


ROOT = Path(__file__).resolve().parents[2]


class FlatpakIconTests(unittest.TestCase):
    def test_icon_has_a_standard_hicolor_size(self):
        manifest = (ROOT / "packaging/flatpak/aetherpak.yaml").read_text()
        match = re.search(r"^\s+icons: (\S+)\s*$", manifest, re.MULTILINE)
        self.assertIsNotNone(match)
        icon = (ROOT / match.group(1)).read_bytes()
        self.assertEqual(icon[:8], b"\x89PNG\r\n\x1a\n")
        self.assertEqual(icon[12:16], b"IHDR")
        self.assertEqual(struct.unpack(">II", icon[16:24]), (256, 256))

    def test_derivative_tracks_the_canonical_artwork(self):
        manifest = (ROOT / "packaging/flatpak/aetherpak.yaml").read_text()
        match = re.search(r"source-sha256: ([0-9a-f]{64})", manifest)
        self.assertIsNotNone(match)
        source = ROOT / "assets/app-icons/con-raccoon-tail-A6.png"
        self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), match.group(1))


if __name__ == "__main__":
    unittest.main()
