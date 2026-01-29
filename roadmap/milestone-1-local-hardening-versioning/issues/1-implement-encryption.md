# Issue 1: Implement AES-256-GCM Encryption for Local Storage

## Context
Currently, `envbro` stores environment variables as plain text files in the user's home directory. This is a security vulnerability. We need to ensure that even if an attacker gains read access to the file system, they cannot read the secrets without the master key.

## Requirements
- **Algorithm**: Use `AES-256-GCM` (Authenticated Encryption).
- **Key Derivation**: Use `Argon2` to derive a symmetric encryption key from a user-supplied Passphrase.
- **Session Management**: 
    - On first run in a session, prompt user for Passphrase.
    - Keep the `SecretKey` in memory (pinned/zeroized if possible) for the session duration.

## Implementation Steps
1.  Add `aes-gcm` and `argon2` crates.
2.  Create a `security` module (`src/security.rs`) to handle key derivation and encryption/decryption routines.
3.  Implement `encrypt(data: &[u8], key: &Key) -> Result<Vec<u8>>`.
4.  Implement `decrypt(data: &[u8], key: &Key) -> Result<Vec<u8>>`.

## Acceptance Criteria
- [ ] Valid passphrase decrypts the store.
- [ ] Invalid passphrase returns an `AuthError`.
- [ ] Inspection of storage files reveals high-entropy random bytes (ciphertext).
