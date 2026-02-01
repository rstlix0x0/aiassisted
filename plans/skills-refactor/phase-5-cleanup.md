# Phase 5: Remove Old Templates Code

**Status**: Pending

## Objective

Remove the old templates module and related code that is no longer needed.

## Files to Delete

### Source Files

Delete entire `src/templates/` directory:
```
src/templates/
├── mod.rs
├── commands.rs      # All template commands
├── engine.rs        # SimpleTemplateEngine
├── generator.rs     # SkillGenerator, AgentGenerator
├── resolver.rs      # CascadingResolver
└── discovery.rs     # ToolDetector (moved to skills)
```

### Core Files

Delete `src/core/templates.rs`:
```
src/core/templates.rs  # TemplateEngine, TemplateResolver traits
```

## Files to Modify

### `src/core/mod.rs`

Remove templates module:
```rust
// DELETE THESE LINES
pub mod templates;
pub use templates::TemplateEngine;
pub use templates::TemplateResolver;
```

### `src/lib.rs` (if exists)

Remove any template exports.

## Cleanup Steps

### Step 1: Delete Templates Directory

```bash
rm -rf src/templates/
```

### Step 2: Delete Core Templates

```bash
rm src/core/templates.rs
```

### Step 3: Update Core Module

Edit `src/core/mod.rs` to remove templates references.

### Step 4: Remove Unused Imports

Search for and remove any remaining imports:
```rust
// Search for these patterns and remove
use crate::templates::*;
use crate::core::templates::*;
use crate::core::TemplateEngine;
use crate::core::TemplateResolver;
```

### Step 5: Remove from main.rs

Ensure `mod templates;` is removed from `src/main.rs`.

## Verification

```bash
# Should compile with no errors
cargo check

# Should have no warnings
cargo check 2>&1 | grep -c warning  # Should be 0

# All tests should pass
cargo test
```

## Files That Should Remain

After cleanup, the skills-related code should only be in:
```
src/skills/
├── mod.rs
├── commands.rs
├── copier.rs
└── discovery.rs
```

## Dependencies

- Phase 3 (skills module must be working first)
- Phase 4 (main.rs must use new skills module)

## Next Phase

[Phase 6: Update Content Files](phase-6-content.md)
