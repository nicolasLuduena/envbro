---
description: Performs an elite-level audit of Rust code, focusing on performance, memory safety, and idiomatic patterns.
---

# 🦀 Rust Production-Ready Audit

1. **Static Analysis & Tooling**:
   - Run `cargo check`.
   - Run `cargo clippy --all-targets --all-features -- -D warnings`. 
   - *Note: If clippy fails, the review ends. Fix warnings first.*

2. **The "Well-Done" Rubric**:
   - **Ownership & Borrowing**: Identify unnecessary `.clone()` calls or over-use of `Arc<Mutex<T>>`. Propose stack-based or scoped alternatives where possible.
   - **Error Handling**: Audit for `.unwrap()`, `.expect()`, or `panic!`. Verify that errors are handled using the `Result` pattern and idiomatic crates like `anyhow` (for apps) or `thiserror` (for libs).
   - **Async Integrity**: Scan for blocking I/O inside async functions. Verify efficient use of `tokio::select!` and proper task spawning.
   - **Type System**: Check for "Primitive Obsession." Suggest Newtype patterns or Enums to leverage Rust's compile-time safety.

3. **Performance Audit**:
   - Look for unoptimized iterators (e.g., redundant allocations).
   - Check if large structs are passed by value instead of by reference.

4. **Workflow Output**:
   - Generate a **Walkthrough Artifact** (`REVIEW.md`) summarizing:
     - **Criticals**: Logic bugs or safety violations.
     - **Refactors**: Idiomatic improvements.
     - **Confidence Score**: A 1-10 rating of how "production-ready" the code is.