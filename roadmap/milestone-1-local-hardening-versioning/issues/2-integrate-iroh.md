# Issue 2: Integrate Iroh Docs for Local Storage

## Context
To support versioning and future P2P syncing, we are migrating the storage backend to **Iroh**. Iroh provides "Documents" (key-value stores based on CRDTs) that can be synced efficiently.

## Requirements
- **Dependency**: Add `iroh` crate.
- **Node**: Initialize an Iroh Node (persistent on disk).
- **Structure**: Each "environment" (project/env combination) should be its own **Iroh Doc**.
    - *Decision*: Map `Project + Env` -> `DocId`.
- **Data Format**: The Value stored in the Iroh Doc should be the *Encrypted* ciphertext of the value.
    - Key: `VAR_NAME`
    - Value: `AES(value)`

## Implementation Steps
1.  Initialize an Iroh Node in `~/.envbro/iroh-data`.
2.  When `envbro set <key> <value>` is called:
    - Load (or create) the Doc for this environment.
    - Encrypt the value.
    - `doc.set_bytes(key, encrypted_value)`.
3.  Ensure the author ID is tracked (Iroh does this automatically).

## Acceptance Criteria
- [ ] A new `envbro set` operation creates/updates an entry in the Iroh Doc.
- [ ] Data is persisted in `~/.envbro` (via Iroh's internal storage).
