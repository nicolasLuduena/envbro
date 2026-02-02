# Milestone 1: Local Hardening

## Goal
The objective of this milestone is to re-architect `envbro` as a **Local-First** and **Secure** configuration manager. Before we can share data efficiently, we must ensure it is stored securely on the local machine.

We will move away from storing plain text files in `~/.envbro` and instead use **Iroh** for robust data management and **AES-256-GCM** for encryption.

## Deliverables
- [ ] **Secure Storage**: All data at rest is encrypted.
- [ ] **Iroh Integration**: Use Iroh Blobs for storing environment data.

## Issues
| Issue | Description |
|Args|Args|
| [ISSUE-1](./issues/1-implement-encryption.md) | Implement AES-256-GCM Encryption for Local Storage |
| [ISSUE-2](./issues/2-integrate-iroh.md) | Integrate Iroh for Storage & Transfer |
| [ISSUE-3](./issues/3-atomic-manifest-updates.md) | Atomic Manifest Updates (Robustness & Deferred) |
| [ISSUE-4](./issues/4-key-backup.md) | Implement Key Export & Paper Backup (Deferred) |
