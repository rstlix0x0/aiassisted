"""File manifest handling with SHA256 checksums"""

import hashlib
from pathlib import Path
from typing import Dict, Optional
from rich.console import Console

console = Console()


class Manifest:
    """Handle FILES.txt manifest with SHA256 checksums"""

    def __init__(self, verbosity: int = 1):
        self.verbosity = verbosity
        self.files: Dict[str, str] = {}  # filepath -> sha256 hash

    def log_debug(self, message: str):
        """Log debug message"""
        if self.verbosity >= 2:
            console.print(f"[blue][DEBUG][/blue] {message}")

    @staticmethod
    def calculate_sha256(file_path: Path) -> str:
        """Calculate SHA256 checksum of file

        Args:
            file_path: Path to file

        Returns:
            SHA256 hash as hex string
        """
        sha256_hash = hashlib.sha256()
        with open(file_path, "rb") as f:
            for byte_block in iter(lambda: f.read(4096), b""):
                sha256_hash.update(byte_block)
        return sha256_hash.hexdigest()

    def verify_checksum(self, file_path: Path, expected_hash: str) -> bool:
        """Verify file checksum matches expected hash

        Args:
            file_path: Path to file
            expected_hash: Expected SHA256 hash

        Returns:
            True if match, False otherwise
        """
        if not file_path.exists():
            return False

        actual_hash = self.calculate_sha256(file_path)

        if actual_hash == expected_hash:
            return True
        else:
            self.log_debug(f"Checksum mismatch for {file_path}")
            self.log_debug(f"  Expected: {expected_hash}")
            self.log_debug(f"  Actual:   {actual_hash}")
            return False

    def load_from_file(self, manifest_path: Path) -> bool:
        """Load manifest from FILES.txt

        Args:
            manifest_path: Path to FILES.txt

        Returns:
            True on success, False on failure
        """
        if not manifest_path.exists():
            return False

        try:
            with open(manifest_path, "r") as f:
                for line in f:
                    line = line.strip()
                    # Skip empty lines and comments
                    if not line or line.startswith("#"):
                        continue

                    # Parse filepath:hash format
                    if ":" in line:
                        filepath, hash_value = line.split(":", 1)
                        self.files[filepath] = hash_value

            return True
        except Exception as e:
            console.print(
                f"[red][ERROR][/red] Failed to load manifest: {e}", file=sys.stderr
            )
            return False

    def load_from_content(self, content: str) -> bool:
        """Load manifest from string content

        Args:
            content: Manifest content

        Returns:
            True on success, False on failure
        """
        try:
            for line in content.split("\n"):
                line = line.strip()
                # Skip empty lines and comments
                if not line or line.startswith("#"):
                    continue

                # Parse filepath:hash format
                if ":" in line:
                    filepath, hash_value = line.split(":", 1)
                    self.files[filepath] = hash_value

            return True
        except Exception as e:
            console.print(
                f"[red][ERROR][/red] Failed to parse manifest: {e}", file=sys.stderr
            )
            return False

    def compare_with(self, other: "Manifest") -> tuple[list[str], list[str]]:
        """Compare with another manifest

        Args:
            other: Other manifest to compare with

        Returns:
            Tuple of (changed_files, unchanged_files)
        """
        changed = []
        unchanged = []

        for filepath, remote_hash in other.files.items():
            local_hash = self.files.get(filepath)

            if local_hash is None or local_hash != remote_hash:
                changed.append(filepath)
            else:
                unchanged.append(filepath)

        return changed, unchanged


import sys
