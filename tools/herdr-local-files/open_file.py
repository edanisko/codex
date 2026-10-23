"""Reveal an explicit local hyperlink in Finder without executing the linked file."""

import argparse
import json
import os
from pathlib import Path
import subprocess
from urllib.parse import unquote, urlsplit


def reveal_command(url):
    if not url or any(ord(char) < 32 or ord(char) == 127 for char in url):
        raise ValueError("Expected a local file hyperlink")
    parts = urlsplit(url)
    if (
        parts.scheme != "file"
        or parts.netloc not in ("", "localhost")
        or parts.query
    ):
        raise ValueError("Only local file URLs are supported")
    path_text = unquote(parts.path)
    if any(ord(char) < 32 or ord(char) == 127 for char in path_text):
        raise ValueError("Control characters are not allowed in file paths")
    path = Path(path_text)
    if not path.is_absolute():
        raise ValueError("Expected an absolute file path")
    path = path.resolve(strict=True)
    if path.is_dir():
        return ["/usr/bin/open", str(path)]
    if not path.is_file():
        raise ValueError("Target must be a regular file or directory")
    return ["/usr/bin/open", "-R", str(path)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("url", nargs="?", default=os.environ.get("HERDR_PLUGIN_CLICKED_URL"))
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    try:
        command = reveal_command(args.url)
    except (ValueError, OSError) as error:
        parser.error(str(error))
    if args.dry_run:
        print(json.dumps(command))
        return 0
    return subprocess.run(command, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
