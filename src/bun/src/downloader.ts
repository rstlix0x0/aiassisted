/**
 * Download utilities for fetching files from GitHub
 */

import { writeFile, mkdir } from 'fs/promises';
import { dirname } from 'path';

export interface DownloaderOptions {
  verbosity?: number;
}

export class Downloader {
  private verbosity: number;

  constructor(options: DownloaderOptions = {}) {
    this.verbosity = options.verbosity ?? 1;
  }

  private logDebug(message: string): void {
    if (this.verbosity >= 2) {
      console.log(`\x1b[34m[DEBUG]\x1b[0m ${message}`);
    }
  }

  async downloadFile(url: string, outputPath: string): Promise<boolean> {
    this.logDebug(`Downloading ${url} to ${outputPath}`);

    try {
      const response = await fetch(url);
      
      if (!response.ok) {
        console.error(`\x1b[31m[ERROR]\x1b[0m Failed to download ${url}: HTTP ${response.status}`);
        return false;
      }

      const content = await response.arrayBuffer();
      
      // Ensure parent directory exists
      await mkdir(dirname(outputPath), { recursive: true });
      
      // Write file
      await writeFile(outputPath, new Uint8Array(content));
      
      return true;
    } catch (error) {
      console.error(`\x1b[31m[ERROR]\x1b[0m Failed to download ${url}: ${error}`);
      return false;
    }
  }

  async downloadText(url: string): Promise<string | null> {
    this.logDebug(`Downloading ${url}`);

    try {
      const response = await fetch(url);
      
      if (!response.ok) {
        console.error(`\x1b[31m[ERROR]\x1b[0m Failed to download ${url}: HTTP ${response.status}`);
        return null;
      }

      return await response.text();
    } catch (error) {
      console.error(`\x1b[31m[ERROR]\x1b[0m Failed to download ${url}: ${error}`);
      return null;
    }
  }
}
