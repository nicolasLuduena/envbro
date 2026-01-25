# Issue 2: Implement Inbox Scanner & Decryption

## Context
Instead of "requesting" access from a smart contract, the client must "scan" the blockchain for messages (Encrypted Keys) addressed to its Shielded Identity.

## Requirements
- **Inbox Scanning**: Retrieve all `EncryptedPacket` events associated with the user's ZK-ID (or scan all global packets if using a privacy-preserving "blind" inbox pattern).
- **Decryption Logic**:
    1. Retrieve `EphemeralKey` ($R$) and `Ciphertext` from the event.
    2. Compute Shared Secret $S = s \cdot R$.
    3. Attempt Decryption.
    4. If successful, we found a `VaultKey`!

## Implementation Steps
1.  **Event Listener**: Implement a listener/query for `InboxMessage` events on the Midnight contract.
2.  **Filter Logic**:
    -   *Optimization*: If tags/bloom filters are available, use them.
    -   *Fallback*: Trial-decryption (cpu intensive but private).
3.  **Key Recovery**: Valid packets yield a `VaultKey`. Pass this to the Hypercore layer to unlock the `.env` file.

## Acceptance Criteria
- [ ] User B can run `envbro pull` and automatically find the key Alice sent.
- [ ] Data is decrypted successfully using the recovered key.
