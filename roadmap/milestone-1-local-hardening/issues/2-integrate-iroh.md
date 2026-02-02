# Issue 2: Integrate Iroh for Storage & Transfer

## Context
To support efficient P2P data transfer, we are integrating **Iroh**. We will use `iroh-blobs` for simple, efficient data storage and transfer.

## Requirements
- **Dependency**: Add `iroh` crate.
- **Node**: Initialize an Iroh Node (persistent on disk).
- **Structure**: Each "environment" (project/env combination) is stored as a blob.
    - **Note**: The blob content is the *Encrypted* ciphertext of the environment data (e.g., JSON or .env format).
- **Data Format**: 
    - Content: `AES(env_vars_content)`
    - Reference: We need to store the `BlobTicket` or `Hash` somewhere locally to know which blob corresponds to which environment.

## Implementation Steps
1.  Initialize an Iroh Node in `~/.envbro/iroh-data`.
2.  When `envbro set` or `envbro save` is called:
    - Encrypt the full environment data.
    - Add the encrypted data as a blob to the Iroh node (`node.blobs().add_bytes()`).
    - Store the resulting `Hash` in a local `manifest.json` or `sqlite` db mapping `Project+Env -> BlobHash`.
3.  Ensure the node is running and ready for future connections (essential for Milestone 2).

## Acceptance Criteria
- [ ] Data can be stored as a blob in the Iroh node.
- [ ] We can retrieve the blob using its hash.
- [ ] Data persists in `~/.envbro`.
