## Context
We have established that we can share a 32-byte secret by embedding it in the `nonce` field of a Midnight Shielded Output.
This issue tracks the technical implementation of the **Sender** and **Receiver** logic for this "Secret Handshake".

## Technical Implementation

### 1. The Data Structure (`CoinInfo`)
We need to bypass the standard wallet's random nonce generation and inject our own.

```rust
// Pseudo-code for custom Coin injection
struct CustomCoinInfo {
    nonce: [u8; 32], // <-- HERE IS THE SECRET
    asset_type: AssetType, // DUST or special marker
    value: 0,
}
```

### 2. Sender Logic (`envbro share`)
1.  **Input**: Recipient Address (Bob), ProjectKey.
2.  **Validation**: Ensure ProjectKey is exactly 32 bytes.
3.  **Construction**:
    -   Create `CoinInfo { nonce: ProjectKey, value: 0, ... }`.
    -   Encrypt using Bob's Encryption Public Key (EPK).
    -   Generate ZK Proof (standard output proof).
4.  **Broadcast**: Submit transaction.

### 3. Receiver Logic (`envbro pull`)
1.  **Scanning**:
    -   Sync with Midnight Node.
    -   Trial-decrypt every new output using Wallet's `IncomingViewingKey`.
2.  **Extraction**:
    -   If decryption succeeds -> We have `CoinInfo`.
    -   Extract `CoinInfo.nonce`.
3.  **Verification**:
    -   Try to decrypt the local Iroh `iroh.json` (or minimal verified blob) with this key.
    -   If successful, save `ProjectKey` to local secure storage.

## Challenges & Mitigations
-   **Nullifiers**: If Bob "spends" this coin, he generates a nullifier derived from the nonce.
    -   *Risk*: If he spends it, he can't "receive" the key again in the same UTXO (double spend protection).
    -   *Mitigation*: Treat these as "Burnt" coins. Never spend them.
-   **Discovery**: How does Bob know *which* transaction contains the key vs regular money?
    -   *Solution*: Use a specific `AssetType` (e.g., `EnvBroKeyToken`) effectively tagging the output.

## Checklist
- [ ] Prototype `CoinInfo` injection in Midnight JS/Rust SDK.
- [ ] Verify 32-byte constraint strictly.
- [ ] Implement command `envbro share <user_address>`.
