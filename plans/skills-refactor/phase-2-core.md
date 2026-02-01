# Phase 2: Simplify Core Traits

**Status**: Pending

## Objective

Remove template-related traits from `src/core/` and add minimal traits for the new skills module.

## Changes

### 1. Remove `src/core/templates.rs`

Delete the entire file which contains:
- `TemplateEngine` trait
- `TemplateResolver` trait

### 2. Update `src/core/mod.rs`

Remove the templates module export:
```rust
// DELETE THIS LINE
pub mod templates;

// DELETE THESE RE-EXPORTS
pub use templates::TemplateEngine;
pub use templates::TemplateResolver;
```

### 3. Create `src/core/skills.rs` (Optional)

If we need trait abstractions for testing, create a minimal traits file:

```rust
//! Skills domain abstractions

use crate::core::Result;
use std::path::Path;

/// Represents a skill that can be installed
#[derive(Debug, Clone)]
pub struct SkillInfo {
    /// Skill name (directory name)
    pub name: String,
    /// Path to the skill directory
    pub path: std::path::PathBuf,
    /// Whether the skill has supporting files (references/, etc.)
    pub has_references: bool,
}

/// Trait for discovering available skills
pub trait SkillsDiscovery: Send + Sync {
    /// List all available skills from the source directory
    fn discover(&self, source_dir: &Path) -> Result<Vec<SkillInfo>>;
}

/// Trait for copying skills to target directory
pub trait SkillsCopier: Send + Sync {
    /// Copy a skill to the target directory
    /// Returns true if copied, false if skipped (already exists and not forced)
    fn copy_skill(
        &self,
        skill: &SkillInfo,
        target_dir: &Path,
        force: bool,
    ) -> Result<bool>;
}
```

### 4. Update Core Module Exports

If we add skills.rs, update `src/core/mod.rs`:
```rust
pub mod skills;

pub use skills::{SkillInfo, SkillsDiscovery, SkillsCopier};
```

## Alternative: No Core Traits

Since the skills module is simple, we might not need trait abstractions. The implementation can use `FileSystem` trait directly from `core/infra.rs`.

**Decision**: Start without traits in `core/skills.rs`. Add them later if needed for testing.

## Verification

```bash
cargo check
```

Should compile with no errors after removing template traits (will have unused import warnings until we complete the cleanup).

## Dependencies

- Phase 1 (CLI changes are independent but should be done first)

## Next Phase

[Phase 3: Create New Skills Module](phase-3-skills-module.md)
