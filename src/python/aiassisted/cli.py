"""CLI command routing and interface"""

import sys
from pathlib import Path
from typing import Optional
from rich.console import Console
from .installer import Installer
from .manifest import Manifest

console = Console()

VERSION = "2.0.0"
GITHUB_REPO = "rstlix0x0/aiassisted"


class CLI:
    """Main CLI handler"""

    def __init__(self, verbosity: int = 1):
        self.verbosity = verbosity
        self.console = console
        self.installer = Installer(github_repo=GITHUB_REPO, verbosity=verbosity)

    def log_error(self, message: str):
        """Log error message to stderr"""
        self.console.print(f"[bold red][ERROR][/bold red] {message}", file=sys.stderr)

    def log_success(self, message: str):
        """Log success message"""
        if self.verbosity >= 1:
            self.console.print(f"[bold green][SUCCESS][/bold green] {message}")

    def log_info(self, message: str):
        """Log info message"""
        if self.verbosity >= 1:
            self.console.print(f"[bold blue][INFO][/bold blue] {message}")

    def log_warn(self, message: str):
        """Log warning message"""
        if self.verbosity >= 1:
            self.console.print(f"[bold yellow][WARN][/bold yellow] {message}")

    def log_debug(self, message: str):
        """Log debug message"""
        if self.verbosity >= 2:
            self.console.print(f"[blue][DEBUG][/blue] {message}")

    def cmd_install(self, path: str = "."):
        """Install .aiassisted directory"""
        target_path = Path(path).resolve()
        self.log_info(f"Installing .aiassisted to {target_path}")

        # Check if already exists
        aiassisted_dir = target_path / ".aiassisted"
        if aiassisted_dir.exists():
            version_file = aiassisted_dir / ".version"
            if version_file.exists():
                local_hash = self.installer.parse_version_file(
                    version_file, "COMMIT_HASH"
                )
                remote_hash = self.installer.get_remote_version()

                if local_hash == remote_hash:
                    self.log_success(
                        f".aiassisted is already up-to-date (version: {local_hash})"
                    )
                    return 0
                else:
                    self.log_warn(".aiassisted already exists but is outdated")
                    self.log_info(f"Current version: {local_hash}")
                    self.log_info(f"Latest version:  {remote_hash}")
                    self.log_info(
                        "Run 'aiassisted update' to update to the latest version"
                    )
                    return 0
            else:
                self.log_warn(".aiassisted exists but no version information found")
                self.log_info(
                    "Run 'aiassisted update --force' to overwrite with the latest version"
                )
                return 0

        # Download and install
        if not self.installer.install(target_path):
            self.log_error("Failed to install .aiassisted")
            return 1

        # Show success message
        version_file = aiassisted_dir / ".version"
        installed_hash = self.installer.parse_version_file(version_file, "COMMIT_HASH")
        self.log_success(
            f"Successfully installed .aiassisted (version: {installed_hash})"
        )

        # Show tips
        self.console.print("\n[bold green]Quick Tips:[/bold green]")
        self.console.print("  • Your .aiassisted directory is now ready to use")
        self.console.print(
            "  • Update to latest version: [bold]aiassisted update[/bold]"
        )
        self.console.print("  • Check for updates: [bold]aiassisted check[/bold]")
        self.console.print("  • View all commands: [bold]aiassisted help[/bold]\n")

        return 0

    def cmd_update(self, path: str = ".", force: bool = False):
        """Update existing .aiassisted installation"""
        target_path = Path(path).resolve()
        self.log_info(f"Checking for updates in {target_path}")

        # Check if .aiassisted exists
        aiassisted_dir = target_path / ".aiassisted"
        if not aiassisted_dir.exists():
            self.log_error(f".aiassisted not found in {target_path}")
            self.log_info("Run 'aiassisted install' first")
            return 1

        # Get local version
        version_file = aiassisted_dir / ".version"
        if not version_file.exists():
            self.log_warn("No version information found locally")
            local_hash = "unknown"
        else:
            local_hash = self.installer.parse_version_file(version_file, "COMMIT_HASH")

        # Get remote version
        remote_hash = self.installer.get_remote_version()

        self.log_debug(f"Local version:  {local_hash}")
        self.log_debug(f"Remote version: {remote_hash}")

        # Compare versions
        if local_hash != "unknown" and local_hash == remote_hash:
            self.log_success(
                f".aiassisted is already up-to-date (version: {local_hash})"
            )
            return 0

        self.log_info("Update available!")
        self.log_info(f"Current version: {local_hash}")
        self.log_info(f"Latest version:  {remote_hash}")

        # Update
        if not self.installer.update(target_path, force=force):
            self.log_error("Failed to update .aiassisted")
            return 1

        self.log_success(f"Successfully updated to version {remote_hash}")
        return 0

    def cmd_check(self, path: str = "."):
        """Check for updates"""
        target_path = Path(path).resolve()
        self.log_info(f"Checking version in {target_path}")

        # Check if .aiassisted exists
        aiassisted_dir = target_path / ".aiassisted"
        if not aiassisted_dir.exists():
            self.log_error(f".aiassisted not found in {target_path}")
            self.log_info("Run 'aiassisted install' to install")
            return 1

        # Get local version
        version_file = aiassisted_dir / ".version"
        if not version_file.exists():
            self.log_warn("No version information found locally")
            local_hash = "unknown"
        else:
            local_hash = self.installer.parse_version_file(version_file, "COMMIT_HASH")

        # Get remote version
        remote_hash = self.installer.get_remote_version()

        self.console.print(f"\n[bold]Current version:[/bold] {local_hash}")
        self.console.print(f"[bold]Latest version:[/bold]  {remote_hash}\n")

        if local_hash != "unknown" and local_hash == remote_hash:
            self.log_success("You are up-to-date!")
        else:
            self.log_warn("An update is available")
            self.log_info("Run 'aiassisted update' to update to the latest version")

        return 0

    def cmd_version(self):
        """Show version"""
        self.console.print(f"aiassisted version {VERSION} (python runtime)")
        return 0

    def cmd_help(self):
        """Show help message"""
        help_text = """aiassisted - AI-Assisted Engineering Guidelines Installer

Usage:
  aiassisted <command> [options]

Commands:
  install [--path=DIR]              Install .aiassisted to directory (default: current)
  update [--force] [--path=DIR]     Update existing .aiassisted installation
  check [--path=DIR]                Check if updates are available
  setup-skills [--tool=TOOL]        Setup AI agent skills (opencode, claude, or auto)
  templates <subcommand>            Manage templates (list, show, init, sync, path, diff)
  config <subcommand>               Manage configuration (show, get, edit, reset, path)
  runtime <subcommand>              Manage runtime backends (list, set, info, help)
  version                           Show CLI version
  self-update                       Update the aiassisted CLI itself
  help                              Show this help message

Options:
  --path=DIR                        Target directory (default: current directory)
  --force                           Skip confirmation prompts during update
  --tool=TOOL                       AI tool to setup (opencode, claude, auto)
  --runtime=RUNTIME                 Runtime backend to use (shell, python, bun)
  --dry-run                         Show what would be created without creating
  --verbose                         Show detailed output
  --quiet                           Show only errors

Examples:
  # Install to current directory
  aiassisted install

  # Install to specific project
  aiassisted install --path=/path/to/project

  # Check for updates
  aiassisted check

  # Update with confirmation
  aiassisted update

  # Force update without confirmation
  aiassisted update --force

  # Setup AI agent skills (auto-detect tools)
  aiassisted setup-skills

  # Setup for specific tool
  aiassisted setup-skills --tool=opencode
  aiassisted setup-skills --tool=claude

  # Preview what would be created
  aiassisted setup-skills --dry-run

  # Manage templates
  aiassisted templates list          # List all templates
  aiassisted templates init          # Copy to project for customization
  aiassisted templates sync          # Update from global

  # Manage configuration
  aiassisted config show                 # View current config
  aiassisted config get default_runtime  # Get specific value
  aiassisted config edit                 # Edit in $EDITOR

  # Manage runtime backends
  aiassisted runtime list                # List available runtimes
  aiassisted runtime set python          # Set preferred runtime
  aiassisted runtime help                # Show runtime help

  # Override runtime for single command
  aiassisted install --runtime=shell
  aiassisted update --runtime=python

  # Update CLI tool itself
  aiassisted self-update

For more information, visit:
  https://github.com/rstlix0x0/aiassisted
"""
        self.console.print(help_text)
        return 0


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(
        description="AI-Assisted Engineering Guidelines Installer", add_help=False
    )
    parser.add_argument("command", nargs="?", default="help")
    parser.add_argument("--path", default=".")
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("--quiet", action="store_true")
    parser.add_argument("--runtime", choices=["shell", "python", "bun"])

    args, unknown = parser.parse_known_args()

    # Determine verbosity
    verbosity = 1
    if args.verbose:
        verbosity = 2
    elif args.quiet:
        verbosity = 0

    # Create CLI instance
    cli = CLI(verbosity=verbosity)

    # Route to command
    try:
        if args.command == "install":
            sys.exit(cli.cmd_install(path=args.path))
        elif args.command == "update":
            sys.exit(cli.cmd_update(path=args.path, force=args.force))
        elif args.command == "check":
            sys.exit(cli.cmd_check(path=args.path))
        elif args.command == "version":
            sys.exit(cli.cmd_version())
        elif args.command == "help":
            sys.exit(cli.cmd_help())
        elif args.command == "self-update":
            cli.log_error("self-update is handled by the shell orchestrator")
            sys.exit(1)
        else:
            cli.log_error(f"Unknown command: {args.command}")
            console.print("Run 'aiassisted help' for usage information")
            sys.exit(1)
    except KeyboardInterrupt:
        console.print("\n[yellow]Cancelled by user[/yellow]")
        sys.exit(130)
    except Exception as e:
        cli.log_error(f"Unexpected error: {e}")
        if verbosity >= 2:
            raise
        sys.exit(1)


if __name__ == "__main__":
    main()
