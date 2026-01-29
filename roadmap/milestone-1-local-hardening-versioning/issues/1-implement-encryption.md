# Issue 1: Implement AES-256-GCM Encryption for Local Storage

## Context
Currently, `envbro` stores environment variables as plain text files in the user's home directory. This is a security vulnerability. We need to ensure that even if an attacker gains read access to the file system, they cannot read the secrets without the master key.

## Requirements
- **Algorithm**: Use `AES-256-GCM`. It provides both confidentiality and integrity checks.
- **Key Derivation**: Use `PBKDF2` or `Argon2` to derive a symmetric encryption key from a user-supplied Master Password.
- **Session Management**: 
    - On first run in a session, prompt user for Master Password.
    - Cache the derived key in memory (NOT on disk) for the duration of the process.

## Implementation Steps
1.  Add `createCipheriv` and `createDecipheriv` from the native `crypto` module.
2.  Create a `SecurityManager` class to handle key derivation and encryption/decryption routines.
3.  Update the `store` logic to encrypt payload before writing to disk.
4.  Update the `read` logic to decrypt payload after reading from disk.

## Acceptance Criteria
- [ ] Valid master password decrypts the store.
- [ ] Invalid master password throws an error.
- [ ] Inspection of storage files reveals high-entropy random bytes (ciphertext), not plain text.
