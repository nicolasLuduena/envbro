# Issue: Atomic Manifest Updates

## Context
Currently, the `IrohStore` implementation in `src/store/iroh.rs` saves the `manifest.json` file by directly overwriting it.

```rust
async fn save_manifest(&self) -> Result<()> {
    let manifest = self.manifest.lock().await;
    let content = serde_json::to_string_pretty(&*manifest)?;
    tokio::fs::write(&self.manifest_path, content).await?;
    Ok(())
}
```

## Problem
In a concurrent environment, or if the process crashes during the write operation (power loss, OOM, etc.), the `manifest.json` file could become corrupted or partially written. This would result in data loss or an invalid state where the CLI cannot read the existing blob mappings.

## Solution
Implement an atomic write strategy:
1.  Serialize the manifest content.
2.  Write the content to a temporary file (e.g., `manifest.json.tmp`) in the same directory.
3.  Sync the file to disk.
4.  Rename the temporary file to `manifest.json` (replacing the existing one atomically).

This ensures that `manifest.json` always points to a valid file.

## Requirements
- Use `tempfile` or manual temp file creation in the same file system.
- Ensure cross-platform compatibility (rename atomicity varies but is generally supported on POSIX systems).
- Update `save_manifest` method in `IrohStore`.

## Acceptance Criteria
- [ ] `save_manifest` writes to a temp file first.
- [ ] `save_manifest` renames the temp file to the target path.
- [ ] Tests verify that the manifest is not corrupted if an error occurs during write (mocking might be hard, but logic review is key).
