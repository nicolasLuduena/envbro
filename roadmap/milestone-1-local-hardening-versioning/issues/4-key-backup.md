# Issue 4: Key Export & Paper Backup

## Context
If a user's hard drive crashes, their encrypted vault is useless without the Master Key. Since we are "Local-First" and "Trustless", there is no cloud server to reset their password. **If the key is lost, the data is gone properly.**

To mitigate this Single Point of Failure, we must allow users to export their key in a human-readable format for physical safekeeping.

## Requirements

### 1. Mnemonic Generation (BIP-39)
- When initializing a new vault (`envbro init`), generate a **12 or 24-word seed phrase**.
- Derive the Master Encryption Key from this seed.

### 2. Export Command
- Implement `envbro key export --reveal`.
- **UX**:
    - Display a big warning: "Authorized eyes only".
    - Show the Mnemonic phrase.
    - Show the raw Hex key (optional, for automation).
    - Suggest printing it and storing it in a physical safe.

### 3. Import / Recovery Command
- Implement `envbro recover` (or `init --recover`).
- Prompt the user to enter their seed phrase.
- Re-derive the keys and attempt to locate the Hypercore feed (if the path is known or if we implement a registry later).
    - *Note*: In M1 (Local Only), this just restores the ability to read the local `.envbro` folder. If the folder itself is deleted, the key alone won't get data back until M2 (P2P Sync).

### 4. Security Considerations
- The mnemonic should never be stored in plaintext on disk.
- It is only held in memory during the `export` or `init` phase.
