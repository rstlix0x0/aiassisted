/**
 * File manifest handling with SHA256 checksums
 */

import { readFile } from 'fs/promises';
import { createHash } from 'crypto';

export interface ManifestOptions {
  verbosity?: number;
}

export class Manifest {
  public files: Map<string, string> = new Map(); // filepath -> sha256 hash
  private verbosity: number;

  constructor(options: ManifestOptions = {}) {
    this.verbosity = options.verbosity ?? 1;
  }

  private logDebug(message: string): void {
    if (this.verbosity >= 2) {
      console.log(`\x1b[34m[DEBUG]\x1b[0m ${message}`);
    }
  }

  static async calculateSha256(filePath: string): Promise<string> {
    const content = await readFile(filePath);
    return createHash('sha256').update(content).digest('hex');
  }

  async verifyChecksum(filePath: string, expectedHash: string): Promise<boolean> {
    try {
      const actualHash = await Manifest.calculateSha256(filePath);
      
      if (actualHash === expectedHash) {
        return true;
      } else {
        this.logDebug(`Checksum mismatch for ${filePath}`);
        this.logDebug(`  Expected: ${expectedHash}`);
        this.logDebug(`  Actual:   ${actualHash}`);
        return false;
      }
    } catch {
      return false;
    }
  }

  async loadFromFile(manifestPath: string): Promise<boolean> {
    try {
      const content = await readFile(manifestPath, 'utf-8');
      return this.loadFromContent(content);
    } catch (error) {
      console.error(`\x1b[31m[ERROR]\x1b[0m Failed to load manifest: ${error}`);
      return false;
    }
  }

  loadFromContent(content: string): boolean {
    try {
      for (const line of content.split('\n')) {
        const trimmed = line.trim();
        
        // Skip empty lines and comments
        if (!trimmed || trimmed.startsWith('#')) {
          continue;
        }
        
        // Parse filepath:hash format
        const colonIndex = trimmed.indexOf(':');
        if (colonIndex > 0) {
          const filepath = trimmed.substring(0, colonIndex);
          const hash = trimmed.substring(colonIndex + 1);
          this.files.set(filepath, hash);
        }
      }
      
      return true;
    } catch (error) {
      console.error(`\x1b[31m[ERROR]\x1b[0m Failed to parse manifest: ${error}`);
      return false;
    }
  }

  compareWith(other: Manifest): { changed: string[]; unchanged: string[] } {
    const changed: string[] = [];
    const unchanged: string[] = [];
    
    for (const [filepath, remoteHash] of other.files.entries()) {
      const localHash = this.files.get(filepath);
      
      if (!localHash || localHash !== remoteHash) {
        changed.push(filepath);
      } else {
        unchanged.push(filepath);
      }
    }
    
    return { changed, unchanged };
  }
}
