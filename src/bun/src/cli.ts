/**
 * CLI command routing and interface
 */

import { resolve } from 'path';
import { existsSync } from 'fs';
import { Installer } from './installer';

const VERSION = '2.0.0';
const GITHUB_REPO = 'rstlix0x0/aiassisted';

export class CLI {
  private verbosity: number;
  private installer: Installer;

  constructor(verbosity: number = 1) {
    this.verbosity = verbosity;
    this.installer = new Installer({ githubRepo: GITHUB_REPO, verbosity });
  }

  private logError(message: string): void {
    console.error(`\x1b[31m[ERROR]\x1b[0m ${message}`);
  }

  private logSuccess(message: string): void {
    if (this.verbosity >= 1) {
      console.log(`\x1b[32m[SUCCESS]\x1b[0m ${message}`);
    }
  }

  private logInfo(message: string): void {
    if (this.verbosity >= 1) {
      console.log(`\x1b[34m[INFO]\x1b[0m ${message}`);
    }
  }

  private logWarn(message: string): void {
    if (this.verbosity >= 1) {
      console.log(`\x1b[33m[WARN]\x1b[0m ${message}`);
    }
  }

  async cmdInstall(path: string = '.'): Promise<number> {
    const targetPath = resolve(path);
    this.logInfo(`Installing .aiassisted to ${targetPath}`);

    // Check if already exists
    const aiassistedDir = `${targetPath}/.aiassisted`;
    if (existsSync(aiassistedDir)) {
      const versionFile = `${aiassistedDir}/.version`;
      if (existsSync(versionFile)) {
        const localHash = await Installer.parseVersionFile(versionFile, 'COMMIT_HASH');
        const remoteHash = await this.installer.getRemoteVersion();

        if (localHash === remoteHash) {
          this.logSuccess(`.aiassisted is already up-to-date (version: ${localHash})`);
          return 0;
        } else {
          this.logWarn('.aiassisted already exists but is outdated');
          this.logInfo(`Current version: ${localHash}`);
          this.logInfo(`Latest version:  ${remoteHash}`);
          this.logInfo("Run 'aiassisted update' to update to the latest version");
          return 0;
        }
      } else {
        this.logWarn('.aiassisted exists but no version information found');
        this.logInfo("Run 'aiassisted update --force' to overwrite with the latest version");
        return 0;
      }
    }

    // Download and install
    if (!(await this.installer.install(targetPath))) {
      this.logError('Failed to install .aiassisted');
      return 1;
    }

    // Show success message
    const versionFile = `${aiassistedDir}/.version`;
    const installedHash = await Installer.parseVersionFile(versionFile, 'COMMIT_HASH');
    this.logSuccess(`Successfully installed .aiassisted (version: ${installedHash})`);

    // Show tips
    console.log('\n\x1b[1m\x1b[32mQuick Tips:\x1b[0m');
    console.log('  • Your .aiassisted directory is now ready to use');
    console.log('  • Update to latest version: \x1b[1maiassisted update\x1b[0m');
    console.log('  • Check for updates: \x1b[1maiassisted check\x1b[0m');
    console.log('  • View all commands: \x1b[1maiassisted help\x1b[0m\n');

    return 0;
  }

  async cmdUpdate(path: string = '.', force: boolean = false): Promise<number> {
    const targetPath = resolve(path);
    this.logInfo(`Checking for updates in ${targetPath}`);

    // Check if .aiassisted exists
    const aiassistedDir = `${targetPath}/.aiassisted`;
    if (!existsSync(aiassistedDir)) {
      this.logError(`.aiassisted not found in ${targetPath}`);
      this.logInfo("Run 'aiassisted install' first");
      return 1;
    }

    // Get local version
    const versionFile = `${aiassistedDir}/.version`;
    let localHash = 'unknown';
    if (!existsSync(versionFile)) {
      this.logWarn('No version information found locally');
    } else {
      const hash = await Installer.parseVersionFile(versionFile, 'COMMIT_HASH');
      localHash = hash || 'unknown';
    }

    // Get remote version
    const remoteHash = await this.installer.getRemoteVersion();

    this.logInfo(`Local version:  ${localHash}`);
    this.logInfo(`Remote version: ${remoteHash}`);

    // Compare versions
    if (localHash !== 'unknown' && localHash === remoteHash) {
      this.logSuccess(`.aiassisted is already up-to-date (version: ${localHash})`);
      return 0;
    }

    this.logInfo('Update available!');
    this.logInfo(`Current version: ${localHash}`);
    this.logInfo(`Latest version:  ${remoteHash}`);

    // Update
    if (!(await this.installer.update(targetPath, force))) {
      this.logError('Failed to update .aiassisted');
      return 1;
    }

    this.logSuccess(`Successfully updated to version ${remoteHash}`);
    return 0;
  }

  async cmdCheck(path: string = '.'): Promise<number> {
    const targetPath = resolve(path);
    this.logInfo(`Checking version in ${targetPath}`);

    // Check if .aiassisted exists
    const aiassistedDir = `${targetPath}/.aiassisted`;
    if (!existsSync(aiassistedDir)) {
      this.logError(`.aiassisted not found in ${targetPath}`);
      this.logInfo("Run 'aiassisted install' to install");
      return 1;
    }

    // Get local version
    const versionFile = `${aiassistedDir}/.version`;
    let localHash = 'unknown';
    if (!existsSync(versionFile)) {
      this.logWarn('No version information found locally');
    } else {
      const hash = await Installer.parseVersionFile(versionFile, 'COMMIT_HASH');
      localHash = hash || 'unknown';
    }

    // Get remote version
    const remoteHash = await this.installer.getRemoteVersion();

    console.log(`\n\x1b[1mCurrent version:\x1b[0m ${localHash}`);
    console.log(`\x1b[1mLatest version:\x1b[0m  ${remoteHash}\n`);

    if (localHash !== 'unknown' && localHash === remoteHash) {
      this.logSuccess('You are up-to-date!');
    } else {
      this.logWarn('An update is available');
      this.logInfo("Run 'aiassisted update' to update to the latest version");
    }

    return 0;
  }

  cmdVersion(): number {
    console.log(`aiassisted version ${VERSION} (bun runtime)`);
    return 0;
  }

  cmdHelp(): number {
    const helpText = `aiassisted - AI-Assisted Engineering Guidelines Installer

Usage:
  aiassisted <command> [options]

Commands:
  install [--path=DIR]              Install .aiassisted to directory (default: current)
  update [--force] [--path=DIR]     Update existing .aiassisted installation
  check [--path=DIR]                Check if updates are available
  version                           Show CLI version
  self-update                       Update the aiassisted CLI itself
  help                              Show this help message

Options:
  --path=DIR                        Target directory (default: current directory)
  --force                           Skip confirmation prompts during update
  --verbose                         Show detailed output
  --quiet                           Show only errors
  --runtime=<shell|python|bun>      Select runtime backend

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

  # Update CLI tool itself
  aiassisted self-update

For more information, visit:
  https://github.com/rstlix0x0/aiassisted
`;
    console.log(helpText);
    return 0;
  }
}
