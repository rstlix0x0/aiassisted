#!/usr/bin/env bun
/**
 * Main entry point for aiassisted Bun CLI
 */

import { CLI } from './cli';

async function main() {
  const args = process.argv.slice(2);
  
  // Parse arguments
  let command = 'help';
  let path = '.';
  let force = false;
  let verbosity = 1;
  
  for (const arg of args) {
    if (arg.startsWith('--path=')) {
      path = arg.substring(7);
    } else if (arg === '--force') {
      force = true;
    } else if (arg === '--verbose') {
      verbosity = 2;
    } else if (arg === '--quiet') {
      verbosity = 0;
    } else if (arg.startsWith('--runtime=')) {
      // Ignore runtime flag (handled by orchestrator)
    } else if (!arg.startsWith('--')) {
      command = arg;
    }
  }
  
  // Create CLI instance
  const cli = new CLI(verbosity);
  
  try {
    let exitCode = 0;
    
    switch (command) {
      case 'install':
        exitCode = await cli.cmdInstall(path);
        break;
      case 'update':
        exitCode = await cli.cmdUpdate(path, force);
        break;
      case 'check':
        exitCode = await cli.cmdCheck(path);
        break;
      case 'version':
        exitCode = cli.cmdVersion();
        break;
      case 'help':
        exitCode = cli.cmdHelp();
        break;
      case 'self-update':
        console.error('\x1b[31m[ERROR]\x1b[0m self-update is handled by the shell orchestrator');
        exitCode = 1;
        break;
      default:
        console.error(`\x1b[31m[ERROR]\x1b[0m Unknown command: ${command}`);
        console.log("Run 'aiassisted help' for usage information");
        exitCode = 1;
    }
    
    process.exit(exitCode);
  } catch (error) {
    if (error instanceof Error && error.message === 'SIGINT') {
      console.log('\n\x1b[33mCancelled by user\x1b[0m');
      process.exit(130);
    }
    
    console.error(`\x1b[31m[ERROR]\x1b[0m Unexpected error: ${error}`);
    if (verbosity >= 2) {
      throw error;
    }
    process.exit(1);
  }
}

main();
