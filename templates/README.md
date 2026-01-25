# Agent Skills Templates

This directory contains templates for generating AI agent skills and agents for OpenCode and Claude Code.

## Structure

```
templates/
├── skills/
│   ├── opencode/          # OpenCode skill templates
│   │   ├── git-commit.SKILL.md.template
│   │   └── review-rust.SKILL.md.template
│   └── claude/            # Claude Code skill templates
│       ├── git-commit.SKILL.md.template
│       └── review-rust.SKILL.md.template
└── agents/
    ├── opencode/          # OpenCode agent templates
    │   ├── ai-knowledge-rust.AGENT.md.template
    │   └── ai-knowledge-architecture.AGENT.md.template
    └── claude/            # Claude Code agent templates (subagents)
        ├── ai-knowledge-rust.AGENT.md.template
        └── ai-knowledge-architecture.AGENT.md.template
```

## Template Variables

Templates use the following variables that are substituted during generation:

| Variable | Description | Example |
|----------|-------------|---------|
| `{{PROJECT_ROOT}}` | Git repository root path | `/Users/hiraq/Projects/myproject` |
| `{{RUST_GUIDELINES_LIST}}` | List of Rust guideline files | `- microsoft-rust-guidelines.md\n- rust-adt...` |
| `{{ARCH_GUIDELINES_LIST}}` | List of architecture guideline files | `- algebraic-data-types.md\n- builder-pattern...` |

## Skills

### OpenCode Skills

- **git-commit**: Commit changes following conventional commits
- **review-rust**: Review Rust code against project guidelines

### Claude Code Skills

- **git-commit**: Commit changes following conventional commits
- **review-rust**: Review Rust code against project guidelines

### Claude Code Agents (Subagents)

- **ai-knowledge-rust**: Rust guidelines expert subagent
- **ai-knowledge-architecture**: Architecture patterns expert subagent

## Agents (OpenCode & Claude Code)

### OpenCode Agents

- **ai-knowledge-rust**: Rust guidelines expert agent
- **ai-knowledge-architecture**: Architecture patterns expert agent

### Claude Code Subagents

- **ai-knowledge-rust**: Rust guidelines expert subagent
- **ai-knowledge-architecture**: Architecture patterns expert subagent

## Usage

These templates are used by the `aiassisted setup-skills` command to generate customized skills and agents in the project's `.opencode/` or `.claude/` directories.

## Design Principles

1. **References Only**: Skills reference `.aiassisted/` files, never copy content
2. **Dynamic Loading**: Agents load guidelines on-demand from source
3. **No Sync Needed**: Updates to `.aiassisted/` propagate automatically
4. **Markdown Only**: All configs use Markdown with YAML frontmatter
