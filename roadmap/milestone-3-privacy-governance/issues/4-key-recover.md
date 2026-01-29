# Issue 4: Social Recovery Protocol

## Context
**Problem**: The "Single Point of Failure" in self-sovereign systems is the Loss of Private Key. If the Admin loses their Mnemonic (and didn't do the M1 Paper Backup), the entire team is locked out of administrative functions (or the data itself if no one else has a copy).

**Solution**: leverage the **Midnight** blockchain to implement **Social Recovery**.

## Concept
"Social Recovery" allows a user to regain access to their account if `k` of `n` trusted friends (guardians) vouch for them.

## Mechanism
1.  **Guardians Setup**:
    - The Admin selects 3 Guardians (e.g., The CTO, The DevOps Lead, and a Personal Backup Address).
    - These identities (Wallet Addresses) are registered in the Midnight Smart Contract linked to the Vault.

2.  **Recovery Trigger**:
    - The Admin loses their key. They generate a *new* address.
    - They initiate a "Recovery Request" on-chain: "I am the Admin, please switch master rights to my new address."

3.  **Vouching**:
    - The Guardians see the request.
    - They communicate offline ("Is this really you?").
    - If satisfied, they invoke `approveRecovery(request_id)` on the contract.

4.  **Restoration**:
    - Once the threshold (e.g., 2 of 3) is met, the Smart Contract updates the mapping.
    - The new address is now recognized as the Admin.
    - The Admin downloads the encrypted vault from the P2P swarm.
    - *Critical Note*: This recovers **Access Control Rights**, not necessarily the *Data Decryption Key* unless Sharding (Shamir's Secret Sharing) is also used to encrypt the master key on-chain.

## Deliverables
- [ ] **Smart Contract Logic**: `guardian_approve()` function.
- [ ] **Sharding (Optional)**: Encrypt the Master Verification Key into 3 shards, each encrypted with a Guardian's public key, stored in the contract metadata. This allows recovering the *actual data* too.
- [ ] **UI/CLI**: Flow for Guardians to sign the approval.
