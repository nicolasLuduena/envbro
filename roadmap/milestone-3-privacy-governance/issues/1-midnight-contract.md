## Context
We need a decentralized way to securely share encryption keys with authorized users.
**Challenge**: How do I share a 32-byte secret with "Bob" privately, cheaply, and with auditability?
**Solution**: **Zswap Secret Field Abuse**. We use Midnight's native shielded value transfer to carry the secret payload.

## Architecture Flow

The blockchain acts as a **Token-Gated Key Distribution System**.

1.  **The Secret**: `ProjectKey` (32 bytes).
2.  **The Vehicle**: A Zswap Shielded Output.
3.  **The Payload**: The `nonce` field of the `CoinInfo` struct (32 bytes).

### The "EnvAccess" Protocol

Unlike a standard transfer, here the **value is 0** but the **metadata is valuable**.

| Component | Responsibility | Logic |
| :--- | :--- | :--- |
| **Alice's Client** | **Key Packaging** | Encrypts `ProjectKey` into the `nonce` field of a shielded output destined for Bob (or a Token Contract). |
| **Midnight Network** | **Transport** | Validators verify ZK proof and store ciphertext. Validators DO NOT see the key. |
| **Bob's Client** | **Key Extraction** | Scans chain, identifies output, decrypts `nonce` to recover `ProjectKey`. |

## Smart Contract Role: "The Gatekeeper"

While Alice *can* send the key directly to Bob (P2P Handshake), we want **Governed Access** (IAM).
**The Policy Contract**:
-   **State**: Maintains a list of "Allow Policies" (e.g., "Must confirm email via Oracle", "Must pay 10 NIGHT").
-   **Action**:
    -   Bob interacts with Contract to prove eligibility (ZK Proof).
    -   Contract (or Alice's listening bot) mints/sends the "Key Token" to Bob.

> [!IMPORTANT]
> **External Authorization**: We can introduce a centralized "Approval Bot" that provides a final go-ahead based on external system triggers (e.g., enterprise IAM or off-chain compliance checks) before the contract finalizes the key distribution.

> [!NOTE]
> Since the Contract itself cannot hold the *plaintext* key to mint new outputs with it (privacy leak), the actual **Broadcasting of the Key** still effectively comes from Alice (or a Keeper bot) that listens to the contract's "Approval" event.
>
> **Flow**:
> 1. Bob -> Contract: "I am eligible."
> 2. Contract -> Event: "Bob Approved."
> 3. Alice's Node (listening) -> Bob: Sends Shielded Output with Key.

## Revised Requirements
-   **Key Size**: Must fit in 32 bytes (AES-256). verified.
-   **Encryption**: Uses Midnight's native ElGamal-on-JubJub.
-   **Audit**: Ledger records *that* a transfer happened, but not *what* was transferred.
