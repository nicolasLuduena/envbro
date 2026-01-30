---
description: Performs a context-aware PR review by comparing current changes against a base branch (defaulting to main/master).
---

# 🚀 Rust PR-Centric Review Workflow

### 1. Identify Target Branch
- Check the user input for a target branch name. 
- If no branch is provided, check if `main` exists in the local git repository; if not, use `master`. 
- Set this as `TARGET_BRANCH`.

### 2. Contextual Diff Analysis
- **Generate Diff**: Run `git diff TARGET_BRANCH...HEAD` to get the changes.
- **Deep Context**: For every file changed in the diff, read the entire file (not just the snippets) to understand how the new code interacts with existing modules, traits, and error handling.
- **Architectural Check**: Identify if new dependencies were added to `Cargo.toml` and evaluate their impact on the build graph.

### 3. Verification & Safety
- **Tooling**: Run `cargo clippy --all-targets --all-features -- -D warnings`.
- **Logic**: Run `cargo test`.
- **Safety**: Audit the diff for `.unwrap()` or potential race conditions in `async` blocks. Ensure no breaking changes to public APIs unless intentional.

### 4. Review Artifact Generation
Produce a **Walkthrough Artifact** (`PR_AUDIT.md`) including:
- **Summary**: A high-level overview of what this PR accomplishes.
- **Contextual Findings**: Feedback on how the changes impact the rest of the codebase (e.g., "The change in `lib.rs` breaks the trait implementation in `handler.rs`").
- **Required Fixes**: Blocking issues found by clippy or tests.
- **Rustacean Tips**: Suggestions for more idiomatic expressions.