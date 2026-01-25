## Context
We need a decentralized way to securely share encryption keys with authorized users.
**Challenge**: How do I share a secret with "Bob" if Bob isn't online, without trusting a central server?
**Solution**: The "Inbox" Pattern (Signal-on-Chain). We use Midnight to store *Encrypted Keys* (Inboxes) and verify *Identity*.

## Architecture Flow
The blockchain acts as a **Shielded Key Exchange**. We use ZK Circuits to prove identity without revealing the underlying wallet address.

**The Secret**: A symmetric key (`VaultKey`) that decrypts the Hypercore data.
**The Goal**: Securely send `VaultKey` from Alice to Bob using a **Privacy-Preserving Identifier** (Shielded Address / ZK-Commitment).

## client vs. Chain Responsibilities
**Crucial Distinction**: The expensive cryptographic math (ECC, Encryption, Decryption) happens on the **CLIENT** (Alice/Bob's machine). The Chain is just a verified storage layer.

| Component | Responsibility (Where it runs) | Logic |
| :--- | :--- | :--- |
| **Bob's Client** | **Key Generation** (Off-chain) | Generates Secret ($s$) and Public ($P$). |
| **Midnight Contract** | **Identity Registry** (On-chain ZK) | **Verifies $P = s \cdot G$**. Use ZK to prove Bob *knows* $s$ without revealing it. |
| **Alice's Client** | **Encryption** (Off-chain) | Fetches $P$, generates ephemeral ($r$), calculates $S = r \cdot P$. Encrypts data. |
| **Midnight Contract** | **Inbox Storage** (On-chain) | Stores the Encrypted Blob and Ephemeral Key ($R$). No calculation needed. |
| **Bob's Client** | **Decryption** (Off-chain) | Fetches Blob + $R$. Calculates $S = s \cdot R$. Decrypts. |

### The Value of the Smart Contract (ZK PoK)
You asked: *"If the contract doesn't verify anything, isn't it just a dictionary?"*
**Answer**: Yes! That's why the **Registration Step** MUST be verified in the Circuit.

**The Circuit Logic (`ProveIdentity`)**:
-   **Private Input**: $s$ (Bob's Secret)
-   **Public Input**: $P$ (Bob's Public Key)
-   **Verification**:
    1.  Assert $P == s \cdot G$ (ECC Scalar Multiplication).
    2.  This proves "I own the private key for $P$" (Proof of Knowledge).
    3.  *Bonus*: You can bind this to a DID or NFT ("I own $P$ AND I own NFT #123").

**Feasibility**: Midnight uses **Pallas/Vesta** curves (Halo2). ECC Multiplication is a **native, efficient operation** in these proof systems. It is NOT "expensive" in the validation sense—it's exactly what ZK circuits are designed to do efficiently.

### How Decryption Works (The "Stealth Address" Pattern)
Bob's "ZK-ID" effectively acts as a **Public Key**.
1.  **Bob's Keys**: Bob has a Secret Spending Key ($s$) and a Public Viewing Key ($P = s \cdot G$).
2.  **Encryption (Alice)**:
    -   Alice generates a random ephemeral key ($r$).
    -   Alice computes a **Shared Secret** using Diffie-Hellman: $S = r \cdot P$ (which is equal to $r \cdot s \cdot G$).
    -   Alice effectively creates a "One-Time Address" for this message.
    -   Alice encrypts the `VaultKey` using $S$.
3.  **Decryption (Bob)**:
    -   Bob sees Alice's ephemeral public key ($R = r \cdot G$).
    -   Bob computes the SAME shared secret: $S' = s \cdot R$ (which is $s \cdot r \cdot G$).
    -   Since $S == S'$, Bob can decrypt the message.
    -   *Crucially*: Only Bob (who knows $s$) can compute $S$. To everyone else, including the chain, it looks like random noise.

> [!NOTE]
> **Future Consideration (Quantum Safety)**: Standard ECDH is vulnerable to future quantum computers. In the long term, we may consider switching to **ML-KEM (Kyber)**, though this introduces larger key sizes (~1KB vs 32 bytes) which complicates on-chain storage.

### Future Research: Proxy Re-Encryption (PRE)
What if we use a shielded token to control access to the vault key?
Anyone with it could use its secure type to decrypt the vault key. Basically the token would be the vault key.
> [!WARNING]
> **Out of Scope**: This feature is NOT part of the current Milestone 3. It is listed here solely for future scalability research.

**The Problem**: Currently, Alice must manually encrypt the `VaultKey` for *every* new user ($O(N)$ work).
**The Solution**: **Proxy Re-Encryption (PRE)** (e.g., NuCypher / Threshold).

1.  **Alice** encrypts the `VaultKey` *once* for the "Contract/Network".
2.  **Alice** creates a "Policy": "Anyone with NFT #123 can access".
3.  **Bob** (who has NFT #123) requests access.
4.  **The Network** (authorized via ZK) transforms Alice's ciphertext into a format Bob can decrypt using his own Private Key.
5.  **Result**: Alice is offline; Bob gets access; The Network never sees the plaintext key.


## Revised Requirements
-   **ZK-Identity**: Use a ZK-Circuit to generate a stable *Public Identifier* that does **not** link back to the user's wallet address or transaction history.
-   **Stealth Inbox**: Messages sent to this identifier should not look linkable on-chain (observers shouldn't know Alice is talking to Bob).





## Notes
- Start simple: Policy = "List of allowed public keys".
