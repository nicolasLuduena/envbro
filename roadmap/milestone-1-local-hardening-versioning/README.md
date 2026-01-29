# Milestone 1: Local Hardening & Versioning

## Goal
The objective of this milestone is to re-architect `envbro` as a **Local-First**, **Secure**, and **Versioned** configuration manager. Before we can share data efficiently, we must ensure it is stored securely on the local machine and that we can track changes over time.

We will move away from storing plain text files in `~/.envbro` and instead use **Iroh**, specifically **Iroh Docs**, wrapped in **AES-256-GCM** encryption.

## deliverables
- [ ] **Secure Storage**: All data at rest is encrypted.
- [ ] **Versioning**: Users can view history and checkout previous versions.
- [ ] **CLI Update**: Commands to interact with history (`log`, `checkout`).
- [ ] **Disaster Recovery**: Ability to export/import master keys (Mnemonic/Paper Key).

## Issues
| Issue | Description |
|Args|Args|
| [ISSUE-1](./issues/1-implement-encryption.md) | Implement AES-256-GCM Encryption for Local Storage |
| [ISSUE-2](./issues/2-integrate-iroh.md) | Integrate Iroh Docs for Local Storage |
| [ISSUE-3](./issues/3-version-commands.md) | Implement `envbro log` and `envbro checkout` |
| [ISSUE-4](./issues/4-key-backup.md) | Implement Key Export & Paper Backup |
