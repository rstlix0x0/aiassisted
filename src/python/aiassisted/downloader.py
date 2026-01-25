"""Download utilities for fetching files from GitHub"""

import tempfile
from pathlib import Path
from typing import Optional
import httpx
from rich.console import Console

console = Console()


class Downloader:
    """Handle file downloads with error handling"""

    def __init__(self, verbosity: int = 1):
        self.verbosity = verbosity

    def log_debug(self, message: str):
        """Log debug message"""
        if self.verbosity >= 2:
            console.print(f"[blue][DEBUG][/blue] {message}")

    def download_file(self, url: str, output_path: Path) -> bool:
        """Download file from URL to output path

        Args:
            url: URL to download from
            output_path: Path to save file to

        Returns:
            True on success, False on failure
        """
        self.log_debug(f"Downloading {url} to {output_path}")

        try:
            with httpx.Client(follow_redirects=True, timeout=30.0) as client:
                response = client.get(url)
                response.raise_for_status()

                output_path.parent.mkdir(parents=True, exist_ok=True)
                output_path.write_bytes(response.content)

                return True
        except httpx.HTTPError as e:
            console.print(
                f"[red][ERROR][/red] Failed to download {url}: {e}", file=sys.stderr
            )
            return False
        except Exception as e:
            console.print(
                f"[red][ERROR][/red] Unexpected error downloading {url}: {e}",
                file=sys.stderr,
            )
            return False

    def download_text(self, url: str) -> Optional[str]:
        """Download text content from URL

        Args:
            url: URL to download from

        Returns:
            Text content on success, None on failure
        """
        self.log_debug(f"Downloading {url}")

        try:
            with httpx.Client(follow_redirects=True, timeout=30.0) as client:
                response = client.get(url)
                response.raise_for_status()
                return response.text
        except httpx.HTTPError as e:
            console.print(
                f"[red][ERROR][/red] Failed to download {url}: {e}", file=sys.stderr
            )
            return None
        except Exception as e:
            console.print(
                f"[red][ERROR][/red] Unexpected error downloading {url}: {e}",
                file=sys.stderr,
            )
            return None


import sys
