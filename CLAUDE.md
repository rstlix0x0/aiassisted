# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`aiassisted` is a CLI tool that embeds a `.aiassisted/` directory into projects. This directory contains curated guidelines, instructions, prompts, and templates that AI assistants can reference for consistent, context-aware assistance.

**Note:** This project is being rewritten from POSIX shell to Rust. See `plans/README.md` for progress.

## Development Plans

See `plans/` directory:
- `plans/README.md` - Plan index and status
- `plans/overview.md` - Architecture and policies
- `plans/phase-*.md` - Individual phase plans

## Common Commands (Rust)

### Development

```bash
# Check code compiles
cargo check

# Run all tests
cargo test

# Run the CLI
cargo run -- --help
cargo run -- version
cargo run -- install

# Build release binary
cargo build --release

# Lint with clippy
cargo clippy

# Format code
cargo fmt
```

### Release (with cargo-dist)

```bash
# Tag a version and push to trigger release
git tag "v0.1.0"
git push --tags
```

## Architecture (Rust)

### Source Code Structure

```
src/
├── main.rs          # Entry point, composition root
├── cli.rs           # Clap CLI definitions
├── core/            # All abstractions (traits, types)
│   ├── types.rs     # Error, ToolType, Result, DTOs
│   ├── infra.rs     # FileSystem, HttpClient, Checksum, Logger
│   ├── content.rs   # ManifestStore, ContentDownloader
│   ├── templates.rs # TemplateEngine, TemplateResolver
│   ├── config.rs    # ConfigStore
│   └── selfupdate.rs# ReleaseProvider
├── infra/           # Shared infrastructure implementations
│   ├── fs.rs        # StdFileSystem
│   ├── http.rs      # ReqwestClient
│   ├── checksum.rs  # Sha2Checksum
│   └── logger.rs    # ColoredLogger
├── content/         # Content domain (install, update, check)
├── templates/       # Templates domain (setup-skills, setup-agents)
├── config/          # Config domain
└── selfupdate/      # Self-update domain
```

### Key Design Decisions

1. **Domain-based modular monolith** - Organized by business domains, not technical layers.

2. **Dependency inversion** - Domains depend on `core/` traits, receive implementations via DI.

3. **Flat domain structure** - No nested api/domain/infrastructure inside domains.

4. **cargo-dist for releases** - Automated cross-platform binary builds and GitHub Releases.

## Rust Development Policies

### 1. Zero Warning Policy

All code must compile with zero warnings. Run:
```bash
cargo check 2>&1 | grep -c warning  # Must be 0
```

### 2. Static Dispatch Over Dynamic Dispatch

Prefer generics over `dyn` traits:

```rust
// ❌ Avoid
fn process(handler: &dyn Handler) { }

// ✅ Prefer
fn process<H: Handler>(handler: &H) { }
```

### 3. Minimal Arc Usage

Only use `Arc` when concurrent shared ownership is required. For this CLI tool:
- Commands run sequentially
- No shared state between threads
- Use owned values or references instead

### 4. Comprehensive Testing Policy

All code must have comprehensive unit tests covering both positive and negative scenarios.

#### Testing Requirements

1. **Test Coverage**: Every module must include tests for:
   - Success paths (positive tests)
   - Error handling (negative tests)
   - Edge cases (empty data, large data, Unicode, etc.)
   - Boundary conditions

2. **Mock Dependencies**: Use `mockall` for trait-based mocking:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use mockall::{mock, predicate::*};

       mock! {
           pub FileSystem {}

           #[async_trait::async_trait]
           impl crate::core::infra::FileSystem for FileSystem {
               async fn read(&self, path: &Path) -> Result<String>;
               // ... other methods
           }
       }
   }
   ```

3. **Test Organization**:
   - Place tests in `#[cfg(test)] mod tests` at the end of each module
   - Use descriptive test names: `test_<function>_<scenario>`
   - Group related tests together

4. **Edge Cases to Cover**:
   - Empty inputs (empty strings, empty vectors, empty files)
   - Missing/nonexistent files or paths
   - Invalid data (malformed JSON, wrong checksums)
   - Network failures (timeouts, connection errors)
   - Large data handling (files > 1MB, long strings)
   - Unicode/UTF-8 content
   - Permission/IO errors
   - Concurrent operations (if applicable)

5. **Test Examples**:

   **Infrastructure Tests** - Use real implementations with temporary files:
   ```rust
   #[tokio::test]
   async fn test_write_and_read() {
       let fs = StdFileSystem::new();
       let temp_dir = TempDir::new().unwrap();
       let path = temp_dir.path().join("test.txt");

       fs.write(&path, "content").await.unwrap();
       let result = fs.read(&path).await.unwrap();

       assert_eq!(result, "content");
   }

   #[tokio::test]
   async fn test_read_nonexistent_file() {
       let fs = StdFileSystem::new();
       let result = fs.read(Path::new("/nonexistent")).await;
       assert!(result.is_err());
   }
   ```

   **Domain Logic Tests** - Use mocks for dependencies:
   ```rust
   #[tokio::test]
   async fn test_download_file_checksum_mismatch() {
       let mut mock_http = MockHttpClient::new();
       let mut mock_checksum = MockChecksum::new();

       mock_http.expect_get()
           .returning(|_| Ok("content".to_string()));

       mock_checksum.expect_sha256()
           .returning(|_| "wrong_hash".to_string());

       let result = download_file(&mock_http, &mock_checksum, &entry).await;

       assert!(result.is_err());
       assert!(matches!(result, Err(Error::ChecksumMismatch { .. })));
   }
   ```

6. **Test Verification**:
   ```bash
   # All tests must pass
   cargo test

   # No warnings allowed
   cargo check 2>&1 | grep -c warning  # Must be 0
   ```

7. **When Adding New Code**:
   - Write tests alongside implementation
   - Test error paths as thoroughly as success paths
   - Consider what could go wrong in production
   - Verify tests fail appropriately before making them pass

### 5. Integration Testing

Integration tests verify that modules work correctly together using real implementations (no mocks).

#### Test Location and Structure

- Integration tests are in the `tests/` directory at project root
- Each test file is compiled as a separate crate with access to the library
- Tests use real implementations (StdFileSystem, ReqwestClient, Sha2Checksum)

#### HTTP Mocking

Use `wiremock` to create mock HTTP servers for integration tests:

```rust
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_download_workflow() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/file.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"content"))
        .mount(&mock_server)
        .await;

    let url = format!("{}/file.txt", mock_server.uri());
    // Test with real HTTP client against mock server
}
```

#### mockall vs wiremock: When to Use Each

This project uses two mocking libraries with different purposes:

**mockall - Trait/Interface Mocking (Unit Tests)**

- **Purpose**: Creates fake implementations of Rust traits
- **Level**: Code/interface level
- **I/O**: No real I/O - returns controlled fake data
- **Speed**: Very fast (no actual operations)
- **Use for**: Testing business logic in isolation
- **Location**: `#[cfg(test)]` modules in `src/`

Example:
```rust
// Unit test - mock the FileSystem trait
let mut mock_fs = MockFileSystem::new();
mock_fs.expect_read()
    .returning(|_| Ok("fake content".to_string()));

// Tests ONLY the logic, not the filesystem
let result = process_file(&mock_fs).await.unwrap();
```

**wiremock - HTTP Server Mocking (Integration Tests)**

- **Purpose**: Creates a real HTTP server for testing
- **Level**: Network/HTTP level
- **I/O**: Real HTTP requests to localhost
- **Speed**: Slower (real networking)
- **Use for**: Testing HTTP clients and workflows
- **Location**: `tests/` directory

Example:
```rust
// Integration test - real HTTP server
let mock_server = MockServer::start().await;
Mock::given(method("GET"))
    .respond_with(ResponseTemplate::new(200).set_body_bytes(b"data"))
    .mount(&mock_server)
    .await;

// Tests REAL HTTP client, serialization, networking
let http = ReqwestClient::new();
http.download(&url, &dest).await.unwrap();
```

**Key Differences**

| Aspect | mockall | wiremock |
|--------|---------|----------|
| Test Type | Unit tests | Integration tests |
| What's mocked | Rust traits | HTTP endpoints |
| Real I/O? | No (fake returns) | Yes (real HTTP to localhost) |
| Speed | Very fast | Slower |
| Tests | Business logic | HTTP layer + serialization |

**When to Use**

- Use **mockall** to test logic without dependencies (unit tests)
- Use **wiremock** to test HTTP interactions without external APIs (integration tests)
- Use **both** for comprehensive coverage

#### What to Test

1. **Module Integration**: Verify modules work together correctly
   - Manifest loading + GitHub downloading + sync operations
   - File system + checksum verification workflow
   - HTTP download + file save + checksum verification

2. **Full Workflows**: Test complete user scenarios
   - Install: Download manifest → download files → verify checksums
   - Update: Compare manifests → download changed files → verify
   - Check: Load local/remote manifests → calculate diff

3. **Real Data Handling**:
   - Large files (10KB+)
   - Unicode content
   - Nested directory structures
   - Network errors with real HTTP clients

#### Example Integration Test

```rust
#[tokio::test]
async fn test_full_download_workflow() {
    let mock_server = MockServer::start().await;
    let fs = StdFileSystem::new();
    let http = ReqwestClient::new();
    let checksum = Sha2Checksum::new();

    // Setup mock endpoints
    Mock::given(method("GET"))
        .and(path("/file1.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"content"))
        .mount(&mock_server)
        .await;

    // Execute workflow with real implementations
    let url = format!("{}/file1.txt", mock_server.uri());
    http.download(&url, &dest).await.unwrap();

    // Verify with real filesystem and checksum
    assert!(fs.exists(&dest));
    let hash = checksum.sha256_file(&dest).unwrap();
    assert_eq!(hash, expected_hash);
}
```

#### Running Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run only integration tests
cargo test --test content_integration

# Run specific integration test
cargo test --test content_integration test_full_download_workflow
```

## Content Organization

The `.aiassisted/` directory contains:

- `guidelines/` - Architecture patterns, documentation standards, language-specific guides
- `instructions/` - AI agent behavior rules and constraints
- `prompts/` - Reusable prompt templates (e.g., commit messages)
- `templates/` - Skill and agent templates for OpenCode and Claude Code
- `config/` - Configuration documentation

## Workflow for Updating Guidelines

1. Edit files in `.aiassisted/`
2. Run `make update-version` to regenerate manifest
3. Run `make test` to verify
4. Commit changes

## Git Commit Policy

This project follows [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) specification for all commit messages.

### Commit Message Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Allowed Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **build**: Changes that affect the build system or external dependencies
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

### Commit Rules

1. **Subject Line**:
   - Use the imperative mood ("add" not "added", "change" not "changed")
   - No period at the end
   - Keep it short (preferably under 50 chars, max 72)

2. **Body** (Optional):
   - Use the imperative mood
   - Wrap lines at 72 characters
   - Explain *what* and *why* vs. *how*

3. **Footer** (Optional):
   - Reference issues (e.g., `Closes #123`)
   - Mention breaking changes

4. **Breaking Changes**:
   - Append a `!` after the type/scope, e.g., `feat!: ...` or `feat(api)!: ...`
   - OR include a footer with `BREAKING CHANGE: <description>`

### Commit Examples

**Feature:**
```
feat(templates): add recursive directory copying for init command
```

**Bug Fix:**
```
fix(config): prevent panic when home directory is not found
```

**Breaking Change:**
```
feat(api)!: remove deprecated v1 endpoints

BREAKING CHANGE: The /v1/* endpoints have been removed. Use /v2/* instead.
```

**Documentation:**
```
docs: update CLAUDE.md with git commit policy
```

**Tests:**
```
test(templates): add 81 comprehensive unit tests for templates domain
```

**Multiple Changes:**
```
feat(templates): complete placeholder implementations

- Implement recursive copy for templates init
- Add smart sync with modification time comparison
- Implement SHA256-based diffing

Closes #8
```

### When Committing

Always include the co-authored-by tag when commits are made with AI assistance:

```
feat(domain): implement new feature

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

For detailed instructions, see `.aiassisted/instructions/conventional-commits.instructions.md`.
