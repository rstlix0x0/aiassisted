# Phase 7: Update Tests

**Status**: Pending

## Objective

Remove old template tests and add comprehensive tests for the new skills module.

## Tests to Remove

### Integration Tests

Delete or update `tests/templates_integration.rs` (if exists).

### Unit Tests in Deleted Files

Tests in these deleted files are automatically removed:
- `src/templates/engine.rs` - 14 unit tests
- `src/templates/resolver.rs` - 7+ unit tests
- `src/templates/discovery.rs` - Unit tests (but keep logic in new location)
- `src/templates/generator.rs` - SkillGenerator, AgentGenerator tests
- `src/templates/commands.rs` - Command tests

## New Tests to Add

### 1. `src/skills/discovery.rs` - ToolDetector Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::MockFileSystem;

    #[test]
    fn test_detect_opencode_by_config() {
        let mut fs = MockFileSystem::new();
        fs.set_exists(".opencode.json", true);

        let detector = ToolDetector::new(&fs, Path::new("."));
        assert_eq!(detector.detect(), ToolType::OpenCode);
    }

    #[test]
    fn test_detect_claude_by_directory() {
        let mut fs = MockFileSystem::new();
        fs.set_exists(".claude", true);
        fs.set_is_dir(".claude", true);

        let detector = ToolDetector::new(&fs, Path::new("."));
        assert_eq!(detector.detect(), ToolType::Claude);
    }

    #[test]
    fn test_detect_claude_by_md_file() {
        let mut fs = MockFileSystem::new();
        fs.set_exists("CLAUDE.md", true);

        let detector = ToolDetector::new(&fs, Path::new("."));
        assert_eq!(detector.detect(), ToolType::Claude);
    }

    #[test]
    fn test_detect_defaults_to_claude() {
        let fs = MockFileSystem::new();
        let detector = ToolDetector::new(&fs, Path::new("."));
        assert_eq!(detector.detect(), ToolType::Claude);
    }

    #[test]
    fn test_skills_dir_claude() {
        let fs = MockFileSystem::new();
        let detector = ToolDetector::new(&fs, Path::new("/project"));

        assert_eq!(
            detector.skills_dir(ToolType::Claude),
            PathBuf::from("/project/.claude/skills")
        );
    }

    #[test]
    fn test_skills_dir_opencode() {
        let fs = MockFileSystem::new();
        let detector = ToolDetector::new(&fs, Path::new("/project"));

        assert_eq!(
            detector.skills_dir(ToolType::OpenCode),
            PathBuf::from("/project/.opencode/skills")
        );
    }

    #[test]
    fn test_skills_source_dir() {
        let fs = MockFileSystem::new();
        let detector = ToolDetector::new(&fs, Path::new("/project"));

        assert_eq!(
            detector.skills_source_dir(),
            PathBuf::from("/project/.aiassisted/skills")
        );
    }
}
```

### 2. `src/skills/copier.rs` - SkillCopier Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::MockFileSystem;

    #[test]
    fn test_discover_skills_finds_valid_skills() {
        let mut fs = MockFileSystem::new();
        fs.set_exists("/source", true);
        fs.set_is_dir("/source", true);
        fs.set_list_dir("/source", vec![
            PathBuf::from("/source/git-commit"),
            PathBuf::from("/source/review-rust"),
        ]);
        fs.set_is_dir("/source/git-commit", true);
        fs.set_is_dir("/source/review-rust", true);
        fs.set_exists("/source/git-commit/SKILL.md", true);
        fs.set_exists("/source/review-rust/SKILL.md", true);

        let copier = SkillCopier::new(&fs);
        let skills = copier.discover_skills(Path::new("/source")).unwrap();

        assert_eq!(skills.len(), 2);
        assert_eq!(skills[0].name, "git-commit");
        assert_eq!(skills[1].name, "review-rust");
    }

    #[test]
    fn test_discover_skills_ignores_non_skill_dirs() {
        let mut fs = MockFileSystem::new();
        fs.set_exists("/source", true);
        fs.set_is_dir("/source", true);
        fs.set_list_dir("/source", vec![
            PathBuf::from("/source/not-a-skill"),
        ]);
        fs.set_is_dir("/source/not-a-skill", true);
        // No SKILL.md file

        let copier = SkillCopier::new(&fs);
        let skills = copier.discover_skills(Path::new("/source")).unwrap();

        assert_eq!(skills.len(), 0);
    }

    #[test]
    fn test_discover_skills_error_when_source_missing() {
        let fs = MockFileSystem::new();
        let copier = SkillCopier::new(&fs);

        let result = copier.discover_skills(Path::new("/nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_skill_creates_directory() {
        let mut fs = MockFileSystem::new();
        fs.set_list_dir("/source/my-skill", vec![
            PathBuf::from("/source/my-skill/SKILL.md"),
        ]);
        fs.set_is_file("/source/my-skill/SKILL.md", true);

        let copier = SkillCopier::new(&fs);
        let skill = SkillInfo {
            name: "my-skill".to_string(),
            source_path: PathBuf::from("/source/my-skill"),
        };

        let result = copier.copy_skill(&skill, Path::new("/target"), false);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Was copied
    }

    #[test]
    fn test_copy_skill_skips_existing_without_force() {
        let mut fs = MockFileSystem::new();
        fs.set_exists("/target/my-skill", true);

        let copier = SkillCopier::new(&fs);
        let skill = SkillInfo {
            name: "my-skill".to_string(),
            source_path: PathBuf::from("/source/my-skill"),
        };

        let result = copier.copy_skill(&skill, Path::new("/target"), false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Was skipped
    }

    #[test]
    fn test_copy_skill_overwrites_with_force() {
        let mut fs = MockFileSystem::new();
        fs.set_exists("/target/my-skill", true);
        fs.set_list_dir("/source/my-skill", vec![
            PathBuf::from("/source/my-skill/SKILL.md"),
        ]);
        fs.set_is_file("/source/my-skill/SKILL.md", true);

        let copier = SkillCopier::new(&fs);
        let skill = SkillInfo {
            name: "my-skill".to_string(),
            source_path: PathBuf::from("/source/my-skill"),
        };

        let result = copier.copy_skill(&skill, Path::new("/target"), true);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Was copied (forced)
    }

    #[test]
    fn test_copy_skill_handles_nested_directories() {
        let mut fs = MockFileSystem::new();
        fs.set_list_dir("/source/git-commit", vec![
            PathBuf::from("/source/git-commit/SKILL.md"),
            PathBuf::from("/source/git-commit/references"),
        ]);
        fs.set_is_file("/source/git-commit/SKILL.md", true);
        fs.set_is_dir("/source/git-commit/references", true);
        fs.set_list_dir("/source/git-commit/references", vec![
            PathBuf::from("/source/git-commit/references/guide.md"),
        ]);
        fs.set_is_file("/source/git-commit/references/guide.md", true);

        let copier = SkillCopier::new(&fs);
        let skill = SkillInfo {
            name: "git-commit".to_string(),
            source_path: PathBuf::from("/source/git-commit"),
        };

        let result = copier.copy_skill(&skill, Path::new("/target"), false);
        assert!(result.is_ok());
    }
}
```

### 3. `src/skills/commands.rs` - Command Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::{MockFileSystem, MockLogger};

    #[tokio::test]
    async fn test_setup_skills_dry_run() {
        let mut fs = MockFileSystem::new();
        // Setup source skills
        fs.set_exists(".aiassisted/skills", true);
        fs.set_is_dir(".aiassisted/skills", true);
        fs.set_list_dir(".aiassisted/skills", vec![
            PathBuf::from(".aiassisted/skills/git-commit"),
        ]);
        fs.set_is_dir(".aiassisted/skills/git-commit", true);
        fs.set_exists(".aiassisted/skills/git-commit/SKILL.md", true);

        let logger = MockLogger::new();
        let cmd = SetupSkillsCommand {
            tool: ToolType::Claude,
            dry_run: true,
            force: false,
        };

        let result = cmd.execute(&fs, &logger, Path::new(".")).await;
        assert!(result.is_ok());

        // Verify no files were actually copied
        assert!(!fs.was_copy_called());
    }

    #[tokio::test]
    async fn test_setup_skills_no_skills_found() {
        let mut fs = MockFileSystem::new();
        fs.set_exists(".aiassisted/skills", true);
        fs.set_is_dir(".aiassisted/skills", true);
        fs.set_list_dir(".aiassisted/skills", vec![]);

        let logger = MockLogger::new();
        let cmd = SetupSkillsCommand {
            tool: ToolType::Claude,
            dry_run: false,
            force: false,
        };

        let result = cmd.execute(&fs, &logger, Path::new(".")).await;
        assert!(result.is_ok());
        // Should log warning about no skills
    }

    #[tokio::test]
    async fn test_skills_list_shows_available() {
        let mut fs = MockFileSystem::new();
        fs.set_exists(".aiassisted/skills", true);
        fs.set_is_dir(".aiassisted/skills", true);
        fs.set_list_dir(".aiassisted/skills", vec![
            PathBuf::from(".aiassisted/skills/git-commit"),
            PathBuf::from(".aiassisted/skills/review-rust"),
        ]);
        fs.set_is_dir(".aiassisted/skills/git-commit", true);
        fs.set_is_dir(".aiassisted/skills/review-rust", true);
        fs.set_exists(".aiassisted/skills/git-commit/SKILL.md", true);
        fs.set_exists(".aiassisted/skills/review-rust/SKILL.md", true);

        let logger = MockLogger::new();
        let cmd = SkillsListCommand {
            tool: ToolType::Claude,
        };

        let result = cmd.execute(&fs, &logger, Path::new(".")).await;
        assert!(result.is_ok());
    }
}
```

### 4. Integration Tests

Create `tests/skills_integration.rs`:

```rust
//! Integration tests for skills module

use aiassisted::skills::{SetupSkillsCommand, SkillsListCommand};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_full_skills_workflow() {
    // Create temp directory with .aiassisted/skills structure
    let temp = TempDir::new().unwrap();
    let project = temp.path();

    // Create source skills
    let skills_dir = project.join(".aiassisted/skills/test-skill");
    std::fs::create_dir_all(&skills_dir).unwrap();
    std::fs::write(
        skills_dir.join("SKILL.md"),
        "---\nname: test-skill\ndescription: Test\n---\n# Test",
    ).unwrap();

    // Run setup-skills
    // ... test implementation
}
```

## Test Count Targets

| Module | Tests | Coverage |
|--------|-------|----------|
| `skills/discovery.rs` | 7+ | Tool detection, paths |
| `skills/copier.rs` | 6+ | Discovery, copying, force |
| `skills/commands.rs` | 3+ | Command execution |
| Integration | 2+ | Full workflows |
| **Total** | ~18+ | Core functionality |

## Verification

```bash
# Run all tests
cargo test

# Run only skills tests
cargo test skills

# Check coverage (if using tarpaulin)
cargo tarpaulin --out Html
```

## Dependencies

- Phase 3 (skills module must be implemented)
- Phase 5 (old templates tests must be removed)

## Next Phase

[Phase 8: Update Documentation](phase-8-docs.md)
