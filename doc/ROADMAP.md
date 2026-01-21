# AI-First Code Metadata Specification (AICMS)

## Vision

Create a **language-agnostic metadata standard** that enables AI coding assistants to understand, generate, verify, and maintain code more effectively. Unlike project-level context (CLAUDE.md, Spec Kit), this specification targets **function and module-level metadata** embedded directly in source code.

---

## The Bootstrapping Problem

**Key insight**: Claude doesn't know what `@ai:*` tags mean—they don't exist in its training data. The specification must be **self-documenting for AI consumption**.

This means the **primary deliverable** is not a parser tool, but a **skill file** that teaches Claude how to read, write, and validate AICMS annotations. Claude reads the spec → Claude understands the tags → Claude uses them correctly.

```
┌─────────────────────────────────────────────────────────────┐
│                   How Claude Learns AICMS                   │
├─────────────────────────────────────────────────────────────┤
│  1. Project includes .claude/skills/aicms/SKILL.md          │
│  2. CLAUDE.md imports: @.claude/skills/aicms/SKILL.md       │
│  3. Claude reads SKILL.md at session start                  │
│  4. Claude now understands @ai:* tags in all project files  │
│  5. Claude generates code WITH proper annotations           │
│  6. Claude validates existing code AGAINST annotations      │
└─────────────────────────────────────────────────────────────┘
```

The skill file IS the specification. No extraction tool needed for Claude to use it.

---

## Landscape Analysis

### What Exists Today

| Tool/Standard | Scope | Strengths | Gaps |
|---------------|-------|-----------|------|
| **GitHub Spec Kit** | Project-level specs | Excellent workflow structure, agent-agnostic | No code-level metadata, no formal contracts |
| **CLAUDE.md** | Project context | Persistent AI context, team-shareable | Project-wide only, not function-specific |
| **Dafny/Verus** | Full verification | Formal proofs, design-by-contract | Requires specialized language, steep learning curve |
| **JSDoc/Rustdoc/Docstrings** | Documentation | Widely adopted, IDE support | No AI-specific metadata, no intent/confidence |
| **OpenAPI/AsyncAPI** | API specs | Machine-readable, generates code | API boundaries only, not internal logic |
| **@ai-generated spec** | Provenance | Tracks AI authorship | Minimal—no intent, contracts, or verification |

### The Gap

**No standard exists for embedding AI-consumable metadata at the function level in existing languages.**

Current approaches force a choice:
1. Use specialized verification languages (Dafny, Verus) — high barrier
2. Use project-level specs (Spec Kit) — loses granularity
3. Use informal comments — not machine-parseable

---

## Core Specification

### Design Principles

1. **Language-Agnostic**: Works via structured comments in any language
2. **Incremental Adoption**: Start with intent, add contracts over time
3. **Machine-Parseable**: Strict format that tools can extract and validate
4. **Human-Readable**: Still useful documentation for developers
5. **Bidirectional**: AI can both read and write these annotations

### Metadata Categories

```
┌─────────────────────────────────────────────────────────────┐
│                    AICMS Metadata Layers                    │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: INTENT        │ What the code should do (natural  │
│                         │ language purpose)                  │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: CONTRACTS     │ Pre/post conditions, invariants   │
│                         │ (semi-formal, verifiable)          │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: EXAMPLES      │ Input/output pairs for validation │
│                         │ and test generation                │
├─────────────────────────────────────────────────────────────┤
│  Layer 4: EFFECTS       │ Side effects, idempotency,        │
│                         │ retry safety                       │
├─────────────────────────────────────────────────────────────┤
│  Layer 5: PROVENANCE    │ Who wrote it, when, confidence    │
│                         │ level, verification status         │
├─────────────────────────────────────────────────────────────┤
│  Layer 6: CONTEXT       │ Dependencies, related functions,  │
│                         │ architectural notes                │
└─────────────────────────────────────────────────────────────┘
```

### Syntax Specification (v0.1)

#### Comment Block Format

Use language-native block comments with `@ai:` prefix:

**Rust:**
```rust
/// @ai:intent Calculate compound interest for a loan
/// @ai:pre principal > 0, rate >= 0, years >= 0
/// @ai:post result >= principal
/// @ai:example (1000.0, 0.05, 10) -> 1628.89
/// @ai:effects pure
/// @ai:confidence 0.95
/// @ai:author claude-4, verified:human:alice@example.com
fn calculate_compound_interest(principal: f64, rate: f64, years: u32) -> f64 {
    principal * (1.0 + rate).powi(years as i32)
}
```

**Python:**
```python
# @ai:intent Calculate compound interest for a loan
# @ai:pre principal > 0 and rate >= 0 and years >= 0
# @ai:post result >= principal
# @ai:example (1000.0, 0.05, 10) -> 1628.89
# @ai:effects pure
# @ai:confidence 0.95
def calculate_compound_interest(principal: float, rate: float, years: int) -> float:
    return principal * (1 + rate) ** years
```

**TypeScript:**
```typescript
/**
 * @ai:intent Calculate compound interest for a loan
 * @ai:pre principal > 0 && rate >= 0 && years >= 0
 * @ai:post result >= principal
 * @ai:example (1000, 0.05, 10) -> 1628.89
 * @ai:effects pure
 * @ai:confidence 0.95
 */
function calculateCompoundInterest(principal: number, rate: number, years: number): number {
    return principal * Math.pow(1 + rate, years);
}
```

#### Full Tag Reference

| Tag | Required | Description | Format |
|-----|----------|-------------|--------|
| `@ai:intent` | Yes | Natural language purpose | Free text |
| `@ai:pre` | No | Preconditions | Comma-separated expressions |
| `@ai:post` | No | Postconditions (`result` = return value) | Comma-separated expressions |
| `@ai:invariant` | No | Always-true conditions | Expression |
| `@ai:example` | No | Test cases | `(args) -> expected` |
| `@ai:effects` | No | Side effects | `pure`, `io`, `db:read`, `db:write`, `network`, `fs` |
| `@ai:idempotent` | No | Safe to call multiple times with same result | `true`/`false` |
| `@ai:confidence` | No | AI's confidence in implementation | `0.0-1.0` |
| `@ai:needs_review` | No | Flags for human review | Free text reason |
| `@ai:author` | No | Provenance tracking | `model-name` or `human:email` |
| `@ai:verified` | No | Verification status | `human:email:date`, `test:suite`, `formal:tool` |
| `@ai:assumes` | No | Unstated assumptions | Free text |
| `@ai:context` | No | Required context to understand | File paths, module names |
| `@ai:related` | No | Related functions | Function names |
| `@ai:deprecated` | No | Deprecation notice | Replacement function |
| `@ai:complexity` | No | Time/space complexity | `O(n)`, `O(n log n)`, etc. |
| `@ai:edge_cases` | No | Known edge cases | `condition -> behavior` |

---

## Roadmap

### Phase 1: The Skill File (Week 1-2) ⭐ PRIMARY DELIVERABLE ✅ COMPLETE

The skill file teaches Claude what AICMS is. Without this, nothing else works.

#### 1.1 Create AICMS Skill File

**File: `.claude/skills/aicms/SKILL.md`**

```markdown
# AICMS: AI-First Code Metadata Specification

You are now equipped to read and write AICMS annotations—structured
metadata in code comments that help you understand intent, verify
correctness, and generate better code.

## Reading AICMS Annotations

When you encounter `@ai:` prefixed comments in code, interpret them as:

### @ai:intent (REQUIRED)
Natural language description of what the function SHOULD do.
Use this to verify the implementation matches the intent.

### @ai:pre (Preconditions)
Conditions that MUST be true before the function executes.
- Validate inputs against these when reviewing code
- Generate test cases that violate preconditions (expect errors)
- Format: comma-separated boolean expressions

### @ai:post (Postconditions)  
Conditions that MUST be true after the function returns.
- `result` refers to the return value
- `old(x)` refers to the value of x before execution
- Use these to verify implementation correctness

### @ai:example
Concrete input/output pairs. Format: `(args) -> expected`
- Use these as test cases
- If implementation doesn't match examples, flag the discrepancy

### @ai:effects
Side effects the function may have:
- `pure` - No side effects (same input = same output)
- `io` - General I/O
- `db:read` / `db:write` - Database operations
- `network` - Network calls
- `fs:read` / `fs:write` - File system operations
- `env` - Reads environment variables

### @ai:confidence
A 0.0-1.0 score indicating trust in the implementation.
- < 0.7: Flag for human review
- 0.7-0.9: Standard implementation
- > 0.9: Well-tested, high confidence

### @ai:needs_review
Explicit flag that human review is required. Reason provided.

### @ai:author
Who wrote this: `claude-4`, `human:email@example.com`, etc.

### @ai:verified  
Verification status: `human:email:date`, `tests:passing`, `formal:dafny`

### @ai:assumes
Implicit assumptions not captured in preconditions.

### @ai:complexity
Time/space complexity: `O(n)`, `O(n log n)`, `O(1) space`

### @ai:edge_cases
Known edge cases: `condition -> behavior`

## Writing AICMS Annotations

When generating or modifying code, ADD appropriate annotations:

1. ALWAYS add `@ai:intent` - describe what the function does
2. Add `@ai:pre` for any input validation requirements
3. Add `@ai:post` for any guarantees about return values
4. Add `@ai:example` with at least one concrete test case
5. Add `@ai:effects` if function has side effects (omit for pure functions)
6. Add `@ai:confidence` reflecting your certainty (be honest!)
7. Add `@ai:needs_review` for complex logic or security-sensitive code

## Validation Behavior

When asked to review or modify annotated code:

1. Check implementation against `@ai:intent` - do they match?
2. Verify `@ai:pre` conditions are enforced (or function fails gracefully)
3. Verify `@ai:post` conditions hold for all code paths
4. Run `@ai:example` cases mentally - do they pass?
5. Verify declared `@ai:effects` matches actual side effects
6. If anything mismatches, REPORT THE DISCREPANCY

## Language-Specific Formats

Use the native comment syntax for each language:

**Rust:** `/// @ai:intent ...`
**Python:** `# @ai:intent ...`
**TypeScript/JavaScript:** `/** @ai:intent ... */` or `// @ai:intent`
**Go:** `// @ai:intent ...`
**Java:** `/** @ai:intent ... */`
**C/C++:** `// @ai:intent ...`
```

#### 1.2 Integration with CLAUDE.md

**File: `CLAUDE.md`**

```markdown
# Project Context

@.claude/skills/aicms/SKILL.md

When working in this codebase:
- Read and respect existing @ai:* annotations
- Add annotations to new functions you create
- Validate implementations against their specs
- Flag discrepancies between intent and implementation
```

#### 1.3 Validation

- [x] Test with 10+ annotated functions across languages (12 functions in examples/)
- [ ] Verify Claude correctly interprets annotations (manual testing required)
- [ ] Verify Claude generates annotations on new code (manual testing required)
- [ ] Verify Claude catches intentional spec violations (manual testing required)

---

### Phase 2: Test Generation (Months 1-2) ✅ COMPLETE

Now that Claude understands the spec, we can build tooling.

#### 2.1 Claude Code Commands ✅

Created slash commands that leverage Claude's understanding:

**Files created:**
- `templates/commands/aicms-tests.md` - Generate tests from annotations
- `templates/commands/aicms-infer.md` - Infer specs from existing code
- `templates/commands/aicms-implement.md` - Implement from spec annotations

Usage: `/project:aicms-tests src/finance.rs`

#### 2.2 Parser Library (For CI/CD) ✅

The parser is needed for **non-AI tooling**, not for Claude:

- [x] Build parser in Rust (`parser/` directory)
- [x] Extract annotations to JSON
- [x] Enable: linting, coverage reports, documentation generation

**CLI Commands:**
- `aicms lint <path>` - Lint files for AICMS compliance
- `aicms extract <file>` - Extract annotations to JSON
- `aicms parse <file>` - Show detected functions and annotations

**Supported Languages:** Rust, Python, TypeScript, JavaScript, Go, Java, C, C++

```bash
# Example: CI/CD validation
$ aicms lint src/
ERROR: src/auth.rs:42 - Function `validate_token` missing @ai:intent
WARN:  src/db.rs:87 - Function has @ai:effects db:write but no @ai:pre
OK:    147 functions checked, 2 issues found
```

---

### Phase 3: Validation & Hooks (Month 2-3) ✅ COMPLETE

#### 3.1 Claude Code Hooks for Enforcement ✅

**Created:** `templates/hooks.json` with pre-commit hook examples:

```json
{
  "hooks": [
    {
      "name": "aicms-validate",
      "trigger": "pre_commit",
      "command": "aicms lint --require-intent {{staged_files}}",
      "description": "Ensure all functions have @ai:intent"
    }
  ]
}
```

Users can copy this template and customize for their needs.

#### 3.2 Runtime Contract Validation (Optional) ✅

**Created:** `templates/commands/aicms-contracts.md` slash command for generating runtime assertion wrappers.

Supports Rust, Python, and TypeScript with language-appropriate debug guards:

```rust
// Generated wrapper for debug builds
#[cfg(debug_assertions)]
pub fn transfer_funds_checked(from: &mut Account, to: &mut Account, amount: u64) -> Result<(), Error> {
    // @ai:pre from.balance >= amount
    assert!(from.balance >= amount, "AICMS pre-condition failed: from.balance >= amount");
    // @ai:pre amount > 0
    assert!(amount > 0, "AICMS pre-condition failed: amount > 0");

    let old_total = from.balance + to.balance;
    let result = transfer_funds(from, to, amount);

    // @ai:post from.balance + to.balance == old(from.balance + to.balance)
    assert!(from.balance + to.balance == old_total, "AICMS post-condition failed: conservation of funds");

    result
}
```

Usage: `/aicms-contracts src/banking.rs`

---

### Phase 4: Spec-First Workflows (Months 3-4) ✅ COMPLETE

#### 4.1 Spec-First Development Mode ✅

**Implemented:** `templates/commands/aicms-implement.md`

Claude Code command: `/project:aicms-implement`

**Workflow:**
```
Human writes spec only:
    /// @ai:intent Sort a list using merge sort
    /// @ai:pre list.len() > 0
    /// @ai:post result.is_sorted()
    /// @ai:post result.len() == list.len()
    /// @ai:complexity O(n log n)
    /// @ai:effects pure
    fn merge_sort<T: Ord>(list: Vec<T>) -> Vec<T> {
        todo!()
    }

Claude runs /project:aicms-implement:
    1. Reads spec annotations
    2. Generates implementation
    3. Generates tests from @ai:example and @ai:pre/@ai:post
    4. Runs tests
    5. Reports: "Implementation complete, 5/5 tests pass, confidence: 0.92"
```

#### 4.2 Automatic Spec Inference ✅

**Implemented:** `templates/commands/aicms-infer.md`

Claude Code command: `/project:aicms-infer`

For existing unannoted code, Claude can infer specs:

```
Human: /project:aicms-infer src/legacy/payment.rs

Claude analyzes code and proposes:
    /// @ai:intent Process a payment transaction
    /// @ai:pre amount > 0
    /// @ai:pre account.is_active()
    /// @ai:post account.balance == old(account.balance) - amount
    /// @ai:effects db:write, network
    /// @ai:confidence 0.75
    /// @ai:needs_review Inferred from implementation, verify business logic
```

#### 4.3 Semantic Diff ✅

**Implemented:**
- `templates/commands/aicms-diff.md` - Slash command for Claude
- `parser/src/diff.rs` - CLI tool for CI/CD integration

Detect breaking contract changes in PRs:

```bash
# CLI usage
aicms diff old_version.rs new_version.rs
aicms diff old.rs new.rs --fail-on-breaking  # Exit code 1 if breaking

# Slash command
/aicms-diff src/payment.rs
```

**Change Classification:**
- 🔴 **BREAKING**: Pre-conditions strengthened, post-conditions weakened, effects expanded
- 🟡 **NOTABLE**: Intent changed, confidence decreased, deprecation added
- 🟢 **NON-BREAKING**: Pre-conditions weakened, post-conditions strengthened, effects reduced

```diff
# Semantic diff output
- @ai:pre amount > 0
+ @ai:pre amount >= 0
⚠️ Contract RELAXED: Now accepts zero amounts (previously rejected)

- @ai:post result.is_some()
+ @ai:post result.is_ok()
⚠️ Contract CHANGED: Return type semantics differ
```

---

### Phase 5: Ecosystem Integration (Months 5-6)

#### 5.1 IDE Extensions
- [ ] VSCode extension: syntax highlighting for `@ai:` tags
- [ ] Inline display of contract status
- [ ] Quick actions: "Generate tests", "Verify contracts"

#### 5.2 CI/CD Integration ✅

**Implemented:** `integrations/github-action/`

GitHub Action for AICMS validation:

```yaml
# .github/workflows/aicms.yml
- name: AICMS Validation
  uses: aicms/check@v1
  with:
    path: 'src/'
    require-intent: true
    require-module-intent: false
    warn-low-confidence: true
    confidence-threshold: 0.7
    check-breaking-changes: true
    base-branch: main
```

**Features:**
- Lint annotations for compliance
- Detect breaking contract changes in PRs
- Configurable requirements and thresholds
- PR comments for failures

See `integrations/github-action/examples/` for workflow templates.

#### 5.3 Documentation Generation
- [ ] Generate API docs from specs
- [ ] Include examples as runnable snippets
- [ ] Contract visualization

#### 5.4 MCP Server
- [ ] Expose AICMS as Model Context Protocol server
- [ ] Any AI tool can query function specs
- [ ] Standard interface for spec-aware AI coding

---

### Phase 6: Benchmark System (Month 5) ✅ COMPLETE

#### 6.1 Automated Benchmark System ✅

**Implemented:** `benchmark/` directory

A comprehensive benchmark system to measure AICMS effectiveness by running Claude on coding tasks with and without AICMS context.

**Features:**
- Comparative benchmarking: Baseline vs AICMS-aware prompts
- Multi-language support: Rust, Python, TypeScript
- Multiple task categories: Implementation, bugfix, refactor, inference
- Automated evaluation: Compilation, tests, examples, linting
- Comprehensive reporting: JSON, Markdown, PNG charts
- **Compilation verification**: Both implementations must compile before comparison
- **Fair comparison**: Comparisons ignore `@ai:*` annotations to evaluate only the code
- **Isolated generation**: Uses `--setting-sources project,local` to exclude user-level settings from influencing results

**CLI Commands:**
```bash
aicms-bench run                     # Run all benchmarks
aicms-bench run --dry-run           # Test without API calls
aicms-bench run --categories impl   # Filter by category
aicms-bench run --compare           # Run with Claude-based comparison scoring
aicms-bench compare -r results/...  # Compare existing results
aicms-bench list                    # List available tasks
aicms-bench validate                # Validate corpus
aicms-bench report --results ...    # Generate reports
```

**Metrics:**
| Metric | Description |
|--------|-------------|
| Compilation rate | Percentage of code that compiles |
| Test pass rate | Percentage of tests passed |
| Example satisfaction | Percentage of @ai:example cases satisfied |
| Lint compliance | Percentage of valid AICMS annotations |
| Annotation quality | Quality score for inferred annotations |

**Test Corpus:** 38 tasks across categories and languages

**Output Directory Structure:**
```
results/<timestamp>/
├── baseline/
│   ├── code/{task-id}/     # Generated code for baseline mode
│   └── report/{task-id}/   # Logs and interaction records
├── aicms/
│   ├── code/{task-id}/     # Generated code for AICMS mode
│   └── report/{task-id}/   # Logs and interaction records
├── results.json            # Complete benchmark data
├── results.md              # Human-readable summary
├── comparison.png          # Overall comparison chart
└── comparison_results.json # Detailed comparison results
```

---

### Phase 7: Advanced Features (Months 7-9)

#### 7.1 Formal Verification Bridge
- [ ] Transpile `@ai:pre`/`@ai:post` to Dafny/Verus
- [ ] For critical code, enable formal proofs
- [ ] Progressive formalization path

#### 7.2 Multi-Language Consistency
- [ ] Verify specs match across language boundaries
- [ ] API client specs must match server specs
- [ ] Cross-language test generation

#### 7.3 Provenance Chain
- [ ] Full audit trail: who wrote what, when
- [ ] Cryptographic signing of verified code
- [ ] Integration with software supply chain security

#### 7.4 AI Training Feedback Loop
- [ ] Collect anonymized spec/implementation pairs
- [ ] Improve AI models on verified correct code
- [ ] Community benchmark for AI code generation

---

## Implementation Priority Matrix

| Feature | Impact | Effort | Priority | Status |
|---------|--------|--------|----------|--------|
| **SKILL.md specification** | **Critical** | **Low** | **P0** | ✅ Done |
| CLAUDE.md integration | High | Low | P0 | ✅ Done |
| Claude Code slash commands | High | Low | P0 | ✅ Done |
| Parser library (for CI/CD) | Medium | Medium | P1 | ✅ Done |
| Test generation commands | High | Medium | P1 | ✅ Done |
| Spec inference command | Medium | Medium | P1 | ✅ Done |
| CI/CD GitHub Action | Medium | Low | P2 | ✅ Done |
| **Benchmark system** | **High** | **Medium** | **P2** | ✅ Done |
| IDE extension | Medium | Medium | P2 | Pending |
| MCP server | Low | Medium | P3 | Pending |
| Formal verification bridge | Low | High | P3 | Pending |

**Key insight**: The SKILL.md file that teaches Claude the spec is the foundation. Everything else builds on Claude understanding the annotations.

---

## Getting Started (For Contributors)

### Immediate Next Steps

1. **Create the SKILL.md file** - This is the spec that teaches Claude
2. **Test with real code** - Annotate 10 functions, verify Claude understands
3. **Create slash commands** - `/project:aicms-tests`, `/project:aicms-infer`
4. **Build parser** - Only needed for CI/CD, not for Claude usage
5. **Gather feedback** - Share with AI coding community

### Repository Structure (Proposed)

```
aicms/
├── skill/
│   └── SKILL.md               # THE PRIMARY DELIVERABLE - teaches Claude
├── templates/
│   ├── CLAUDE.md.template     # Example CLAUDE.md with AICMS import
│   └── commands/              # Slash command templates
│       ├── aicms-tests.md
│       ├── aicms-infer.md
│       └── aicms-implement.md
├── parser/                    # For CI/CD tooling (not required for Claude)
│   ├── Cargo.toml
│   └── src/
├── examples/
│   ├── rust/
│   ├── python/
│   └── typescript/
├── integrations/
│   ├── github-action/
│   └── vscode/
└── docs/
    ├── specification.md       # Formal spec for humans
    └── guides/
```

### Quick Start for Your Project

1. Copy `skills/aicms/SKILL.md` to `.claude/skills/aicms/SKILL.md`
2. Add to your `CLAUDE.md`:
   ```markdown
   @.claude/skills/aicms/SKILL.md
   ```
3. Start annotating functions with `@ai:intent`, `@ai:pre`, etc.
4. Claude now understands and uses the annotations

---

## Success Metrics

1. **Adoption**: 1000+ repos using AICMS within 12 months
2. **AI Accuracy**: Measurable improvement in AI code generation correctness when specs present
3. **Test Coverage**: 50%+ of annotated functions have auto-generated tests
4. **Community**: Active contributors across 5+ programming languages

---

## Open Questions

1. **Spec language**: Should contracts use a universal DSL or language-native expressions?
2. **Versioning**: How to handle spec version evolution?
3. **Granularity**: Should we support block-level specs (not just function-level)?
4. **Privacy**: How to handle sensitive business logic in specs?
5. **AI model compatibility**: How to make SKILL.md work across different AI assistants (Copilot, Cursor, etc.)?
6. **Spec conflicts**: What happens when implementation and spec diverge? Who wins?
7. **Inheritance**: Should specs be inherited by overriding methods?

---

## References

- [GitHub Spec Kit](https://github.com/github/spec-kit) - Project-level spec-driven development
- [Dafny](https://dafny.org/) - Verification-aware programming language
- [Verus](https://github.com/verus-lang/verus) - Rust formal verification
- [AutoVerus](https://arxiv.org/html/2409.13082) - AI-assisted Verus proof generation
- [Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)
- [CLAUDE.md Guide](https://claude.com/blog/using-claude-md-files)

---

*This roadmap is a living document. Last updated: 2026-01-20*
