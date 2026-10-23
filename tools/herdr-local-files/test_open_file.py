import tempfile
import unittest
from pathlib import Path

from open_file import reveal_command


class RevealCommandTests(unittest.TestCase):
    def test_documents_and_archives_are_revealed_as_argv(self):
        with tempfile.TemporaryDirectory() as directory:
            for name in ["research résumé.md", "report.zip", "$(touch unexpected).md", "script.command"]:
                path = Path(directory, name)
                path.write_text("test")
                self.assertEqual(
                    reveal_command(path.as_uri() + "#L12C3"),
                    ["/usr/bin/open", "-R", str(path.resolve())],
                )

    def test_directories_open_in_finder(self):
        with tempfile.TemporaryDirectory() as directory:
            self.assertEqual(
                reveal_command(Path(directory).as_uri()),
                ["/usr/bin/open", str(Path(directory).resolve())],
            )

    def test_nonlocal_and_malformed_urls_are_rejected(self):
        for url in [
            None,
            "https://example.com/report.md",
            "file://remote-server/tmp/report.md",
            "file:relative.md",
            "file:///tmp/report%00.md",
            "file:///tmp/report%1b.md",
            "file:///tmp/report.md?command=open",
            "file:///tmp/report\n.md",
        ]:
            with self.subTest(url=url), self.assertRaises(ValueError):
                reveal_command(url)

    def test_missing_files_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(FileNotFoundError):
                reveal_command(Path(directory, "missing.md").as_uri())


if __name__ == "__main__":
    unittest.main()
