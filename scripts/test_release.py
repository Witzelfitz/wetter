import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
import release


class ReleaseTests(unittest.TestCase):
    def archives(self, folder):
        for target in release.TARGETS:
            path = folder / release.filename(target)
            path.write_bytes(("test archive " + target).encode())
            path.with_name(path.name + ".sha256").write_text(f"{release.sha256(path)}  {path.name}\n")

    def test_formula_uses_all_real_checksums(self):
        with tempfile.TemporaryDirectory() as temp:
            folder = Path(temp)
            self.archives(folder)
            output = folder / "wetter.rb"
            release.formula("Witzelfitz/wetter", folder, output)
            text = output.read_text()
            for target in release.TARGETS:
                self.assertIn(release.sha256(folder / release.filename(target)), text)
            self.assertNotIn("@VERSION@", text)
            self.assertIn("/releases/download/v" + release.version(), text)
            self.assertEqual(len((folder / "SHA256SUMS").read_text().splitlines()), 4)

    def test_tampered_or_missing_archive_fails(self):
        with tempfile.TemporaryDirectory() as temp:
            folder = Path(temp)
            self.archives(folder)
            path = folder / release.filename(release.TARGETS[0])
            path.write_bytes(b"changed after checksum was calculated")
            with self.assertRaisesRegex(ValueError, "Checksum mismatch"):
                release.formula("Witzelfitz/wetter", folder, folder / "wetter.rb")
            path.unlink()
            with self.assertRaises(FileNotFoundError):
                release.formula("Witzelfitz/wetter", folder, folder / "wetter.rb")

    def test_repository_validation(self):
        with self.assertRaisesRegex(ValueError, "OWNER/REPO"):
            release.formula('owner/repo"; system("bad")', Path("."), Path("wetter.rb"))

    def test_wrong_binary_version_fails_before_packaging(self):
        with patch("release.subprocess.check_output", return_value="wetter 99.0.0\n"):
            with self.assertRaisesRegex(ValueError, "version mismatch"):
                release.package(Path("unused"), release.TARGETS[0], Path("unused"))


if __name__ == "__main__":
    unittest.main()
