/**
 * Installation and update logic
 */

import { join, dirname } from 'path';
import { mkdtemp, rm, readFile, writeFile, cp, mkdir } from 'fs/promises';
import { tmpdir } from 'os';
import { existsSync } from 'fs';
import { spawn } from 'child_process';
import { Downloader } from './downloader';
import { Manifest } from './manifest';

export interface InstallerOptions {
  githubRepo: string;
  verbosity?: number;
}

export class Installer {
  private githubRepo: string;
  private githubRawUrl: string;
  private verbosity: number;
  private downloader: Downloader;

  constructor(options: InstallerOptions) {
    this.githubRepo = options.githubRepo;
    this.githubRawUrl = `https://raw.githubusercontent.com/${options.githubRepo}/main`;
    this.verbosity = options.verbosity ?? 1;
    this.downloader = new Downloader({ verbosity: this.verbosity });
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

  private logDebug(message: string): void {
    if (this.verbosity >= 2) {
      console.log(`\x1b[34m[DEBUG]\x1b[0m ${message}`);
    }
  }

  static async parseVersionFile(versionFilePath: string, key: string): Promise<string | null> {
    if (!existsSync(versionFilePath)) {
      return null;
    }

    try {
      const file = Bun.file(versionFilePath);
      const textContent = await file.text();
      const lines = textContent.split('\n');
      
      for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed.startsWith(`${key}=`)) {
          return trimmed.split('=', 2)[1];
        }
      }
      
      return null;
    } catch {
      return null;
    }
  }

  async getRemoteVersion(): Promise<string | null> {
    const url = `${this.githubRawUrl}/.aiassisted/.version`;
    const content = await this.downloader.downloadText(url);

    if (!content) {
      return null;
    }

    // Parse COMMIT_HASH from content
    for (const line of content.split('\n')) {
      const trimmed = line.trim();
      if (trimmed.startsWith('COMMIT_HASH=')) {
        return trimmed.split('=', 2)[1];
      }
    }

    return null;
  }

  async downloadAiassisted(tempDir: string): Promise<boolean> {
    const targetDir = join(tempDir, '.aiassisted');
    await mkdir(targetDir, { recursive: true });

    this.logInfo('Downloading .aiassisted directory...');

    // Download .version file first
    const versionUrl = `${this.githubRawUrl}/.aiassisted/.version`;
    if (!(await this.downloader.downloadFile(versionUrl, join(targetDir, '.version')))) {
      return false;
    }

    // Download FILES.txt manifest
    const manifestUrl = `${this.githubRawUrl}/.aiassisted/FILES.txt`;
    const manifestFile = join(tempDir, 'FILES.txt');
    if (!(await this.downloader.downloadFile(manifestUrl, manifestFile))) {
      this.logError('Failed to download FILES.txt manifest');
      return false;
    }

    // Load manifest
    const manifest = new Manifest({ verbosity: this.verbosity });
    if (!(await manifest.loadFromFile(manifestFile))) {
      return false;
    }

    // Download each file with checksum verification
    for (const [filepath, expectedHash] of manifest.files.entries()) {
      const fileUrl = `${this.githubRawUrl}/.aiassisted/${filepath}`;
      const filePath = join(targetDir, filepath);

      this.logDebug(`Downloading ${filepath}...`);
      if (!(await this.downloader.downloadFile(fileUrl, filePath))) {
        this.logError(`Failed to download ${filepath}`);
        return false;
      }

      // Verify checksum
      if (!(await manifest.verifyChecksum(filePath, expectedHash))) {
        this.logError(`Checksum verification failed for ${filepath}`);
        return false;
      }

      this.logDebug(`Verified checksum for ${filepath}`);
    }

    // Copy FILES.txt to target
    await cp(manifestFile, join(targetDir, 'FILES.txt'));

    this.logSuccess(`Downloaded .aiassisted directory to ${tempDir}`);
    return true;
  }

  async downloadChangedFiles(targetPath: string, remoteManifest: Manifest): Promise<string | null> {
    const localManifestFile = join(targetPath, '.aiassisted', 'FILES.txt');
    const localManifest = new Manifest({ verbosity: this.verbosity });

    if (existsSync(localManifestFile)) {
      await localManifest.loadFromFile(localManifestFile);
    }

    // Compare manifests
    const { changed, unchanged } = localManifest.compareWith(remoteManifest);

    this.logInfo(`Changed: ${changed.length} file(s), Unchanged: ${unchanged.length} file(s)`);

    // Create temp directory
    const tempDir = await mkdtemp(join(tmpdir(), 'aiassisted-'));
    const targetDir = join(tempDir, '.aiassisted');
    await mkdir(targetDir, { recursive: true });

    // Download only changed files
    for (const filepath of changed) {
      const expectedHash = remoteManifest.files.get(filepath)!;
      const fileUrl = `${this.githubRawUrl}/.aiassisted/${filepath}`;
      const filePath = join(targetDir, filepath);

      this.logDebug(`Downloading changed file: ${filepath}...`);
      if (!(await this.downloader.downloadFile(fileUrl, filePath))) {
        this.logError(`Failed to download ${filepath}`);
        await rm(tempDir, { recursive: true, force: true });
        return null;
      }

      // Verify checksum
      if (!(await remoteManifest.verifyChecksum(filePath, expectedHash))) {
        this.logError(`Checksum verification failed for ${filepath}`);
        await rm(tempDir, { recursive: true, force: true });
        return null;
      }
    }

    return tempDir;
  }

  async install(targetPath: string): Promise<boolean> {
    // Create temp directory
    const tempDir = await mkdtemp(join(tmpdir(), 'aiassisted-'));

    try {
      // Download .aiassisted
      if (!(await this.downloadAiassisted(tempDir))) {
        return false;
      }

      // Copy to target
      const targetDir = join(targetPath, '.aiassisted');
      if (existsSync(targetDir)) {
        await rm(targetDir, { recursive: true, force: true });
      }

      await cp(join(tempDir, '.aiassisted'), targetDir, { recursive: true });

      this.logSuccess(`Installed .aiassisted to ${targetPath}`);
      return true;
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  }

  async update(targetPath: string, force: boolean = false): Promise<boolean> {
    // Download remote manifest
    const manifestUrl = `${this.githubRawUrl}/.aiassisted/FILES.txt`;
    const manifestContent = await this.downloader.downloadText(manifestUrl);

    if (!manifestContent) {
      this.logError('Failed to download remote manifest');
      return false;
    }

    const remoteManifest = new Manifest({ verbosity: this.verbosity });
    if (!remoteManifest.loadFromContent(manifestContent)) {
      return false;
    }

    // Download only changed files
    const tempDir = await this.downloadChangedFiles(targetPath, remoteManifest);
    if (!tempDir) {
      return false;
    }

    try {
      // Download .version file
      const versionUrl = `${this.githubRawUrl}/.aiassisted/.version`;
      if (!(await this.downloader.downloadFile(versionUrl, join(tempDir, '.aiassisted', '.version')))) {
        this.logError('Failed to download .version file');
        return false;
      }

      // Show diff unless --force
      if (!force) {
        const tempAiassisted = join(tempDir, '.aiassisted');
        const hasChanges = existsSync(tempAiassisted);

        if (hasChanges) {
          console.log('\n\x1b[1m\x1b[33mChanges to be applied:\x1b[0m\n');

          // Generate diff
          try {
            const diff = await this.generateDiff(
              join(targetPath, '.aiassisted'),
              tempAiassisted
            );
            
            if (diff) {
              const lines = diff.split('\n');
              for (const line of lines.slice(0, 100)) {
                console.log(line);
              }

              if (lines.length > 100) {
                console.log(`\n\x1b[34m... (${lines.length - 100} more lines)\x1b[0m\n`);
              } else {
                console.log();
              }
            }
          } catch {
            this.logWarn('Could not generate diff');
          }
        } else {
          console.log('\n\x1b[1m\x1b[34mNo file content changes (version metadata update only)\x1b[0m\n');
        }

        // Prompt for confirmation
        const answer = prompt('Apply these changes? [y/N]: ');
        if (!answer || !['y', 'yes'].includes(answer.toLowerCase())) {
          this.logInfo('Update cancelled');
          return false;
        }
      }

      // Apply selective updates
      if (!(await this.applySelectiveUpdate(tempDir, targetPath, remoteManifest))) {
        return false;
      }

      return true;
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  }

  private async generateDiff(oldPath: string, newPath: string): Promise<string | null> {
    return new Promise((resolve) => {
      const diff = spawn('diff', ['-ru', oldPath, newPath]);
      let output = '';

      diff.stdout.on('data', (data) => {
        output += data.toString();
      });

      diff.on('close', () => {
        resolve(output || null);
      });

      diff.on('error', () => {
        resolve(null);
      });
    });
  }

  private async applySelectiveUpdate(
    sourceDir: string,
    targetPath: string,
    remoteManifest: Manifest
  ): Promise<boolean> {
    this.logInfo('Applying selective updates...');

    const targetAiassisted = join(targetPath, '.aiassisted');
    const sourceAiassisted = join(sourceDir, '.aiassisted');

    let updated = 0;
    let skipped = 0;

    // Update .version file
    const versionSource = join(sourceAiassisted, '.version');
    const versionTarget = join(targetAiassisted, '.version');
    if (existsSync(versionSource)) {
      await cp(versionSource, versionTarget);
    }

    // Update FILES.txt manifest
    const manifestTarget = join(targetAiassisted, 'FILES.txt');
    let manifestContent = '';
    for (const [filepath, hash] of remoteManifest.files.entries()) {
      manifestContent += `${filepath}:${hash}\n`;
    }
    await writeFile(manifestTarget, manifestContent);

    // Copy only changed files
    for (const filepath of remoteManifest.files.keys()) {
      const sourceFile = join(sourceAiassisted, filepath);
      const targetFile = join(targetAiassisted, filepath);

      // If file exists in source (was downloaded), copy it
      if (existsSync(sourceFile)) {
        await mkdir(dirname(targetFile), { recursive: true });
        await cp(sourceFile, targetFile);
        this.logDebug(`Updated: ${filepath}`);
        updated++;
      } else {
        skipped++;
      }
    }

    this.logSuccess(`Updated ${updated} file(s), skipped ${skipped} unchanged file(s)`);
    return true;
  }
}
