# AICMS Project Context

@.claude/skills/aicms/SKILL.md

## Project Overview

AICMS (AI-First Code Metadata Specification) is a language-agnostic standard for embedding AI-consumable metadata at the function level in source code. Distributed as a Claude Code plugin.

## Project Structure

```
aicms/
├── .claude-plugin/           # Plugin configuration
│   ├── plugin.json           # Plugin manifest
│   └── marketplace.json      # Marketplace config
├── skills/aicms/SKILL.md     # Core skill (teaches Claude AICMS)
├── commands/                 # Slash commands (/aicms:*)
│   ├── implement.md
│   ├── infer.md
│   ├── tests.md
│   ├── contracts.md
│   └── diff.md
├── hooks/hooks.json          # Pre-commit hooks
├── parser/                   # Rust CLI (lint, extract, diff)
├── examples/                 # Annotated code examples
├── benchmark/                # Effectiveness benchmarks
├── integrations/github-action/  # CI/CD integration
└── templates/                # Legacy manual setup templates
```

## Key Files

- `skills/aicms/SKILL.md` - Core specification teaching Claude AICMS
- `commands/*.md` - Slash commands for implement/infer/tests/contracts/diff
- `parser/` - CLI tool (`aicms lint`, `aicms extract`, `aicms diff`)
- `.claude-plugin/plugin.json` - Plugin manifest for distribution

## Working in This Codebase

1. Read and respect existing `@ai:*` annotations
2. Add annotations to new functions
3. Validate implementations against specs
4. Keep plugin.json updated when adding features
