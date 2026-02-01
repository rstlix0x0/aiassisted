# Phase 6: Update Content Files

**Status**: Pending

## Objective

Remove old template files and update the manifest to reflect the new structure.

## Files to Delete

### Templates Directory

Delete entire `.aiassisted/templates/` directory:
```
.aiassisted/templates/
├── README.md
├── agents/
│   ├── claude/
│   │   ├── ai-knowledge-architecture.md.template
│   │   ├── ai-knowledge-rust.md.template
│   │   └── git-commit.md.template
│   └── opencode/
│       ├── ai-knowledge-architecture.md.template
│       ├── ai-knowledge-rust.md.template
│       └── git-commit.md.template
└── skills/
    ├── claude/
    │   ├── git-commit.SKILL.md.template
    │   └── review-rust.SKILL.md.template
    └── opencode/
        ├── git-commit.SKILL.md.template
        └── review-rust.SKILL.md.template
```

### Command

```bash
rm -rf .aiassisted/templates/
```

## Files to Keep

### Skills Directory

Keep `.aiassisted/skills/` as-is:
```
.aiassisted/skills/
├── doc-code/
│   └── SKILL.md
├── doc-project/
│   └── SKILL.md
├── git-commit/
│   ├── SKILL.md
│   └── references/
│       └── conventional-commits.md
├── policy-rust/
│   └── SKILL.md
├── review-codes/
│   └── SKILL.md
└── review-rust/
    └── SKILL.md
```

## Update Manifest

### Regenerate manifest.json

After removing templates directory:

```bash
# Update the manifest to reflect removed files
make update-version
```

Or manually run the manifest generation script.

### Verify Manifest Changes

The manifest should:
1. **Remove** entries for all `.aiassisted/templates/**` files
2. **Keep** entries for all `.aiassisted/skills/**` files
3. **Keep** entries for all `.aiassisted/guidelines/**` files
4. **Keep** entries for all `.aiassisted/instructions/**` files

### Expected Manifest Diff

Files removed from manifest:
- `templates/README.md`
- `templates/agents/claude/ai-knowledge-architecture.md.template`
- `templates/agents/claude/ai-knowledge-rust.md.template`
- `templates/agents/claude/git-commit.md.template`
- `templates/agents/opencode/ai-knowledge-architecture.md.template`
- `templates/agents/opencode/ai-knowledge-rust.md.template`
- `templates/agents/opencode/git-commit.md.template`
- `templates/skills/claude/git-commit.SKILL.md.template`
- `templates/skills/claude/review-rust.SKILL.md.template`
- `templates/skills/opencode/git-commit.SKILL.md.template`
- `templates/skills/opencode/review-rust.SKILL.md.template`

Files that should remain in manifest:
- All `skills/**/*.md` files
- All `guidelines/**/*.md` files
- All `instructions/**/*.md` files
- All `prompts/**/*.md` files

## Verification

```bash
# Verify templates directory is gone
ls .aiassisted/templates  # Should fail (not found)

# Verify skills directory exists
ls .aiassisted/skills/    # Should list skill directories

# Verify manifest is valid JSON
cat .aiassisted/manifest.json | jq .

# Test install command still works
cargo run -- install --path=/tmp/test-project
ls /tmp/test-project/.aiassisted/skills/  # Should have skills
```

## Dependencies

- Phase 5 (source code cleanup should be done first)

## Next Phase

[Phase 7: Update Tests](phase-7-tests.md)
