# Issue 4: Key Export & Paper Backup

## Context
If a user's hard drive crashes, their encrypted vault is useless without the Master Key. Since we are "Local-First" and "Trustless", there is no cloud server to reset their password. **If the key is lost, the data is gone properly.**

To mitigate this Single Point of Failure, we must allow users to export their key in a human-readable format for physical safekeeping.

## Requirements
- **Command**: `envbro key export`.
- **Output**: A 12/24-word Mnemonic phrase (BIP-39) OR the raw Secret Key (base32).

## Implementation Steps
1.  Access the root Iroh SecretKey.
2.  Display it to the user with a warning.
3.  Implement `envbro key import <phrase/key>` to restore identity.

## Acceptance Criteria
- [ ] `envbro key export` displays the secret.
- [ ] `envbro key import` restores the identity on a fresh machine.

### 4. Security Considerations
- The mnemonic should never be stored in plaintext on disk.
- It is only held in memory during the `export` or `init` phase.
