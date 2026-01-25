"""Installation and update logic"""

import shutil
import tempfile
import subprocess
from pathlib import Path
from typing import Optional
from rich.console import Console
from rich.prompt import Confirm
from .downloader import Downloader
from .manifest import Manifest

console = Console()


class Installer:
    """Handle installation and updates of .aiassisted directory"""

    def __init__(self, github_repo: str, verbosity: int = 1):
        self.github_repo = github_repo
        self.github_raw_url = f"https://raw.githubusercontent.com/{github_repo}/main"
        self.verbosity = verbosity
        self.downloader = Downloader(verbosity=verbosity)

    def log_error(self, message: str):
        """Log error message"""
        console.print(f"[bold red][ERROR][/bold red] {message}", file=sys.stderr)

    def log_success(self, message: str):
        """Log success message"""
        if self.verbosity >= 1:
            console.print(f"[bold green][SUCCESS][/bold green] {message}")

    def log_info(self, message: str):
        """Log info message"""
        if self.verbosity >= 1:
            console.print(f"[bold blue][INFO][/bold blue] {message}")

    def log_warn(self, message: str):
        """Log warning message"""
        if self.verbosity >= 1:
            console.print(f"[bold yellow][WARN][/bold yellow] {message}")

    def log_debug(self, message: str):
        """Log debug message"""
        if self.verbosity >= 2:
            console.print(f"[blue][DEBUG][/blue] {message}")

    @staticmethod
    def parse_version_file(version_file: Path, key: str) -> Optional[str]:
        """Parse version file for specific key

        Args:
            version_file: Path to .version file
            key: Key to extract (e.g., COMMIT_HASH)

        Returns:
            Value if found, None otherwise
        """
        if not version_file.exists():
            return None

        try:
            with open(version_file, "r") as f:
                for line in f:
                    line = line.strip()
                    if line.startswith(f"{key}="):
                        return line.split("=", 1)[1]
            return None
        except Exception:
            return None

    def get_remote_version(self) -> Optional[str]:
        """Fetch remote version hash

        Returns:
            Remote COMMIT_HASH or None on failure
        """
        url = f"{self.github_raw_url}/.aiassisted/.version"
        content = self.downloader.download_text(url)

        if content is None:
            return None

        # Parse COMMIT_HASH from content
        for line in content.split("\n"):
            line = line.strip()
            if line.startswith("COMMIT_HASH="):
                return line.split("=", 1)[1]

        return None

    def download_aiassisted(self, temp_dir: Path) -> bool:
        """Download complete .aiassisted directory to temp location

        Args:
            temp_dir: Temporary directory to download to

        Returns:
            True on success, False on failure
        """
        target_dir = temp_dir / ".aiassisted"
        target_dir.mkdir(parents=True, exist_ok=True)

        self.log_info("Downloading .aiassisted directory...")

        # Download .version file first
        version_url = f"{self.github_raw_url}/.aiassisted/.version"
        if not self.downloader.download_file(version_url, target_dir / ".version"):
            return False

        # Download FILES.txt manifest
        manifest_url = f"{self.github_raw_url}/.aiassisted/FILES.txt"
        manifest_file = temp_dir / "FILES.txt"
        if not self.downloader.download_file(manifest_url, manifest_file):
            self.log_error("Failed to download FILES.txt manifest")
            return False

        # Load manifest
        manifest = Manifest(verbosity=self.verbosity)
        if not manifest.load_from_file(manifest_file):
            return False

        # Download each file with checksum verification
        for filepath, expected_hash in manifest.files.items():
            file_url = f"{self.github_raw_url}/.aiassisted/{filepath}"
            file_path = target_dir / filepath

            self.log_debug(f"Downloading {filepath}...")
            if not self.downloader.download_file(file_url, file_path):
                self.log_error(f"Failed to download {filepath}")
                return False

            # Verify checksum
            if not manifest.verify_checksum(file_path, expected_hash):
                self.log_error(f"Checksum verification failed for {filepath}")
                return False

            self.log_debug(f"Verified checksum for {filepath}")

        # Copy FILES.txt to target
        shutil.copy(manifest_file, target_dir / "FILES.txt")

        self.log_success(f"Downloaded .aiassisted directory to {temp_dir}")
        return True

    def download_changed_files(
        self, target_path: Path, remote_manifest: Manifest
    ) -> Optional[Path]:
        """Download only changed files based on checksums

        Args:
            target_path: Target installation path
            remote_manifest: Remote manifest to compare with

        Returns:
            Path to temp directory with changed files, or None on failure
        """
        local_manifest_file = target_path / ".aiassisted" / "FILES.txt"
        local_manifest = Manifest(verbosity=self.verbosity)

        if local_manifest_file.exists():
            local_manifest.load_from_file(local_manifest_file)

        # Compare manifests
        changed, unchanged = local_manifest.compare_with(remote_manifest)

        self.log_info(
            f"Changed: {len(changed)} file(s), Unchanged: {len(unchanged)} file(s)"
        )

        # Create temp directory
        temp_dir = Path(tempfile.mkdtemp())
        target_dir = temp_dir / ".aiassisted"
        target_dir.mkdir(parents=True, exist_ok=True)

        # Download only changed files
        for filepath in changed:
            expected_hash = remote_manifest.files[filepath]
            file_url = f"{self.github_raw_url}/.aiassisted/{filepath}"
            file_path = target_dir / filepath

            self.log_debug(f"Downloading changed file: {filepath}...")
            if not self.downloader.download_file(file_url, file_path):
                self.log_error(f"Failed to download {filepath}")
                shutil.rmtree(temp_dir, ignore_errors=True)
                return None

            # Verify checksum
            if not remote_manifest.verify_checksum(file_path, expected_hash):
                self.log_error(f"Checksum verification failed for {filepath}")
                shutil.rmtree(temp_dir, ignore_errors=True)
                return None

        return temp_dir

    def install(self, target_path: Path) -> bool:
        """Install .aiassisted directory

        Args:
            target_path: Directory to install to

        Returns:
            True on success, False on failure
        """
        # Create temp directory
        temp_dir = Path(tempfile.mkdtemp())

        try:
            # Download .aiassisted
            if not self.download_aiassisted(temp_dir):
                return False

            # Copy to target
            target_dir = target_path / ".aiassisted"
            if target_dir.exists():
                shutil.rmtree(target_dir)

            shutil.copytree(temp_dir / ".aiassisted", target_dir)

            self.log_success(f"Installed .aiassisted to {target_path}")
            return True
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)

    def update(self, target_path: Path, force: bool = False) -> bool:
        """Update existing .aiassisted installation

        Args:
            target_path: Directory containing .aiassisted
            force: Skip confirmation prompts

        Returns:
            True on success, False on failure
        """
        # Download remote manifest
        manifest_url = f"{self.github_raw_url}/.aiassisted/FILES.txt"
        manifest_content = self.downloader.download_text(manifest_url)

        if manifest_content is None:
            self.log_error("Failed to download remote manifest")
            return False

        remote_manifest = Manifest(verbosity=self.verbosity)
        if not remote_manifest.load_from_content(manifest_content):
            return False

        # Download only changed files
        temp_dir = self.download_changed_files(target_path, remote_manifest)
        if temp_dir is None:
            return False

        try:
            # Download .version file
            version_url = f"{self.github_raw_url}/.aiassisted/.version"
            if not self.downloader.download_file(
                version_url, temp_dir / ".aiassisted" / ".version"
            ):
                self.log_error("Failed to download .version file")
                return False

            # Show diff unless --force
            if not force:
                has_changes = any((temp_dir / ".aiassisted").iterdir())

                if has_changes:
                    console.print(
                        "\n[bold yellow]Changes to be applied:[/bold yellow]\n"
                    )

                    # Generate diff
                    try:
                        result = subprocess.run(
                            [
                                "diff",
                                "-ru",
                                str(target_path / ".aiassisted"),
                                str(temp_dir / ".aiassisted"),
                            ],
                            capture_output=True,
                            text=True,
                        )

                        diff_output = result.stdout
                        if diff_output:
                            lines = diff_output.split("\n")
                            for line in lines[:100]:
                                console.print(line)

                            if len(lines) > 100:
                                console.print(
                                    f"\n[blue]... ({len(lines) - 100} more lines)[/blue]\n"
                                )
                            else:
                                console.print()
                    except Exception:
                        self.log_warn("Could not generate diff")
                else:
                    console.print(
                        "\n[bold blue]No file content changes (version metadata update only)[/bold blue]\n"
                    )

                # Prompt for confirmation
                if not Confirm.ask("Apply these changes?"):
                    self.log_info("Update cancelled")
                    return False

            # Apply selective updates
            if not self.apply_selective_update(temp_dir, target_path, remote_manifest):
                return False

            return True
        finally:
            shutil.rmtree(temp_dir, ignore_errors=True)

    def apply_selective_update(
        self, source_dir: Path, target_path: Path, remote_manifest: Manifest
    ) -> bool:
        """Apply selective updates (only changed files)

        Args:
            source_dir: Temp directory with downloaded changes
            target_path: Target installation path
            remote_manifest: Remote manifest

        Returns:
            True on success, False on failure
        """
        self.log_info("Applying selective updates...")

        target_aiassisted = target_path / ".aiassisted"
        source_aiassisted = source_dir / ".aiassisted"

        updated = 0
        skipped = 0

        # Update .version file
        version_source = source_aiassisted / ".version"
        version_target = target_aiassisted / ".version"
        if version_source.exists():
            shutil.copy(version_source, version_target)

        # Update FILES.txt manifest - write from remote_manifest
        manifest_target = target_aiassisted / "FILES.txt"
        with open(manifest_target, "w") as f:
            for filepath, hash_value in remote_manifest.files.items():
                f.write(f"{filepath}:{hash_value}\n")

        # Copy only changed files
        for filepath in remote_manifest.files.keys():
            source_file = source_aiassisted / filepath
            target_file = target_aiassisted / filepath

            # If file exists in source (was downloaded), copy it
            if source_file.exists():
                target_file.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy(source_file, target_file)
                self.log_debug(f"Updated: {filepath}")
                updated += 1
            else:
                skipped += 1

        self.log_success(
            f"Updated {updated} file(s), skipped {skipped} unchanged file(s)"
        )
        return True


import sys
