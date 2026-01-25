# Issue 2: Implement `envbro share` (Host)

## Context
A user wants to share their local environment with a teammate.

## Requirements
- **Command**: `envbro share <project> <env>`.
- **Output**: Display the **Discovery Key**.
    - *Security Note (MVP)*: For this milestone, we will use a **Pre-Shared Key (PSK)** model. The user must communicate a password to the recipient via a secure channel (Signal, etc.). This password derives the decryption key.
    - *Endgame (Milestone 3)*: We will move to **Asymmetric Key Exchange** (see [Milestone 3 Diagram](../milestone-3-privacy-governance/issues/1-midnight-contract.md)). Alice will encrypt the vault key using Bob's on-chain Public Key and deposit it in his "Inbox", removing the need for manual password sharing.


- **Behavior**: Keep the process running (daemon mode) to listen for connections and serve data.

## Implementation Steps
1.  Derive the Discovery Key from the Hypercore public key.
2.  Call `NetworkManager.join(discoveryKey)`.
3.  Keep process alive.
4.  Log when a peer connects.

## Acceptance Criteria
- [ ] Command outputs a connection string.
- [ ] Process stays alive until `Ctrl+C`.
