# Issue 2: Integrate Hypercore for Local Append-Only Logs

## Context
To support versioning and future P2P syncing, we are migrating the storage backend to **Hypercore**. Hypercore is a secure, distributed append-only log.

## Requirements
- **Dependency**: Install `hypercore`.
- **Structure**: Each "environment" (project/env combination) should be its own Hypercore feed (or a sub-namespace within one if using Hyperbee later, but let's start with one core per env for simplicity or one core for the whole app).
    - *Decision*: Use one Hypercore per Project-Environment pair to allow granular sharing later.
- **Data Format**: The "Block" stored in Hypercore should be the *Encrypted* JSON blob of the environment variables.

## Implementation Steps
1.  Initialize a Hypercore feed in `~/.envbro/storage/<id>`.
2.  When `envbro register` or `actions.set` is called, verify if the content has changed.
3.  `append()` the new state to the Hypercore log.
4.  Ensure the "latest" version is always readily available (Hypercore does this by default via `length - 1`).

## Acceptance Criteria
- [ ] A new `envbro set` operation increments the Hypercore length by 1.
- [ ] Data is stored in the binary Hypercore format in `~/.envbro`.
