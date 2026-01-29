# Issue 3: Implement `envbro clone` (Client)

## Context
A user wants to receive an environment from a teammate.

## Requirements
- **Command**: `envbro clone <connection-string>`.
- **Behavior**:
    1.  Parse Discovery Key.
    2.  Connect via Hyperswarm.
    3.  Replicate the Hypercore.
    4.  Prompt for Master Password (to decrypt locally).
    5.  Save/Register as a new local env.

## Implementation Steps
1.  Initialize a new empty Hypercore (or Sparse Hypercore to only fetch needed blocks).
2.  Join the swarm topic.
3.  On replication completion (or "update" event), try to decrypt using provided password.
4.  If successful, write to disk/register in `envbro` list.

## Acceptance Criteria
- [ ] User B can successfully receive the environment history from User A.
- [ ] User B can decrypt the content with the correct shared password.
