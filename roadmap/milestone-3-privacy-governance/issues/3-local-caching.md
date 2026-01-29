# Issue 3: Implement Local Encrypted Caching

## Context
Blockchain interactions are slow. We don't want to hit the Midnight network every time we run `envbro get`. We need to cache the retrieved keys locally, but securely.

## Requirements
- **Storage**: `~/.envbro/cache/keys`.
- **Encryption**: The cached keys MUST be encrypted with the user's Local Master Password (from Milestone 1).
- **Flow**:
    1.  `envbro get`: Check cache.
    2.  If in cache -> Decrypt with Master Password -> Use.
    3.  If NOT in cache -> Trigger **Inbox Scan** (Issue 2) -> Store in Cache -> Use.

## Implementation Steps
1.  Use `security` module from Milestone 1.
2.  Implement `KeyCache` struct.
3.  Add logic to invalidate cache (e.g., if key rotation happens on-chain).

## Acceptance Criteria
- [ ] Subsequent accesses to the environment do NOT trigger network requests.
- [ ] Clearing the cache forces a new blockchain interaction.
- [ ] Cached file is encrypted on disk.
