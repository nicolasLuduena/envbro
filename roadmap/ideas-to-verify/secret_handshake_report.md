# Midnight Secret Handshake Report

## 1. Executive Summary

This report outlines a method for two parties (Alice and Bob) to securely share a 32-byte secret (such as a symmetric encryption key or a shared secret for further communication) using the Midnight Network's existing shielded transaction infrastructure. 

By leveraging the architecture of **Zswap Shielded Outputs**, specifically the unconstrained 32-byte `nonce` field in the `CoinInfo` structure, we can create an encrypted communication channel. This method requires no changes to the Midnight protocol or consensus rules; it is a compliant usage of the existing shielded value transfer primitives.

## 2. Technical Mechanism

The core mechanism relies on "piggybacking" on the Zswap shielded output structure. A standard shielded output contains an encrypted payload intended to help the recipient identify their funds.

### 2.1 The Data Payload (`CoinInfo`)
Every shielded output encrypts a `CoinInfo` struct for the recipient. The structure is defined in `coin-structure/src/coin.rs`:

```rust
pub struct Info {
    pub nonce: Nonce,             // [u8; 32] - Arbitrary 32 bytes
    pub type_: ShieldedTokenType, // Token Type Identifier
    pub value: u128,              // Amount
}
```

*   **`value`**: Must be set to `0` to avoid spending actual funds.
*   **`type_`**: Can be any valid token type (e.g., a "Dust" type or a common asset), technically irrelevant for a zero-value transfer but required for the proof.
*   **`nonce`**: **The Payload**. 
    *   **Standard Behavior**: In typical wallet operations, this is a randomly generated 32-byte value used to ensure the nullifier (unique ID) of the coin is unique and unpredictable.
    *   **Handshake Behavior (Option A - Alice Chooses)**: Alice manually constructs the `CoinInfo` struct, setting the `nonce` field to a specific 32-byte data payload (e.g., a pre-agreed identifier or a specific key).
    *   **Handshake Behavior (Option B - Random Secret)**: Alice allows the wallet to generate a perfectly random `nonce`. She records this value before sending. Bob receives it. Now both parties possess a high-entropy 32-byte shared secret that was securely transferred.
    *   In both cases, this field is encrypted in the `CoinCiphertext`. The network validators verify the ZK proof but do not see the nonce. Only Alice (sender) and Bob (recipient) ever know this 32-byte value.

### 2.2 Encryption Standard
Midnight uses **ElGamal-like encryption** (over the JubJub curve) for shielded outputs.
*   **Sender** uses the Recipient's **Encryption Public Key (EPK)**.
*   **Recipient** uses their **Encryption Secret Key (`esk`)**.
*   The ciphertext is posted publicly on-chain but is mathematically indistinguishable from random noise to anyone without the `esk`.

## 3. Implementation Protocol

### Phase 1: Setup & Discovery
Before the handshake, Alice (Sender) needs to know where to send the secret.
1.  **Bob** provides his **Shielded Address** to Alice.
    *   This address encodes Bob's **Coin Public Key (CPK)** and **Encryption Public Key (EPK)**.
    *   This exchange can happen via any public channel (e.g., email, DM, public registry), as the address itself is not sensitive.

### Phase 2: Transmission (Sender - Alice)
Alice wants to send a 32-byte Secret Key (`SK`) to Bob.

1.  **Construct CoinInfo**:
    *   `value`: `0`
    *   `nonce`: `SK` (The 32-byte secret)
    *   `type`: `NIGHT` (or any valid unshielded/shielded type hash)
2.  **Encrypt**:
    *   Alice's wallet generates an ephemeral key pair.
    *   Computes shared secret: `Diffie_Hellman(Alice_Ephemeral_Priv, Bob_EPK)`.
    *   Encrypts the `CoinInfo` (containing `SK`) to produce the `CoinCiphertext`.
3.  **Prove & Submit**:
    *   Alice constructs a standard Midnight transaction containing this output.
    *   **Note**: To prevent metadata leakage (an output with 0 inputs looks suspicious), Alice should preferably bundle this "Message Output" with a standard "Value Output" (e.g., sending 1 NIGHT to herself or Bob), or mix it in a split transaction.
    *   The transaction is submitted to the ledger.

### Phase 3: Reception (Recipient - Bob)
Bob monitors the Midnight ledger.

1.  **Scan**:
    *   Bob's wallet downloads every new Zswap output.
    *   For each output, it attempts to decrypt the ciphertext using Bob's `esk`.
2.  **Decrypt**:
    *   For Alice's transaction, the decryption succeeds.
    *   The wallet recovers the plaintext `CoinInfo`.
3.  **Extract**:
    *   Bob reads the `nonce` field.
    *   Bob interprets these 32 bytes as the shared secret `SK`.

## 4. Constraints & Limitations

### 4.1 Data Size (32 Bytes)
The hard limit is **32 bytes** per output. This is sufficient for:
*   A 256-bit AES key.
*   A ChaCha20 key.
*   A 32-byte random seed.
*   A SHA-256 hash of an off-chain payload (e.g., IPFS hash).

It is **not** sufficient for text messages, documents, or large metadata. To send larger data, you would need to chain multiple outputs (Splitting 64 bytes into two outputs), which increases proof generation time and transaction fees linearly.

### 4.2 Cost
Sending this secret requires generating a Zero-Knowledge Proof (ZKP) and paying the standard transaction fee for a Midnight transaction. It is not "free" messaging; it is priced similarly to a standard financial transfer.

### 4.3 Nullifier Management
If the "Message Coin" is ever spent, its nullifier is revealed.
*   **The Shared Secret acts as the Salt**: The nullifier is derived from `Hash(nonce, sk, ...)`.
*   If Bob spends this coin, he reveals a nullifier derived from the secret. This generally does not reveal the secret itself (due to the hashing), but it links the "Message Coin" to the "Spending Transaction" on the nullifier set.
*   **Best Practice**: **Never spend the Message Coin.** Treat it as a "Burned" coin. Since it has 0 value, there is no economic loss in discarding it.

## 5. Security Analysis

| Threat Vector | Risk Level | Mitigation |
| :--- | :--- | :--- |
| **Interception** | None | The payload is encrypted with the same security guarantees as Midnight's financial privacy. Breaking this encryption would equivalent to breaking Zswap. |
| **Traffic Analysis** | Low | An observer sees a transaction with an output. They cannot distinguish a "Message Output" from a "Payment Output" unless Alice sends a transaction with *only* one 0-value output and no inputs (which is invalid anyway as fees must be paid). |
| **Mallory (Man-in-the-Middle)** | Low | Depends on Phase 1 (Address Discovery). If Alice gets Mallory's address thinking it's Bob's, she sends the secret to Mallory. |
| **Ledger Bloat** | Low | Since there is a TX fee, spamming secrets is economically decentivized. |

## 6. Diagram of Flow

```mermaid
sequenceDiagram
    participant Alice
    participant Ledger
    participant Bob

    Note over Alice, Bob: Phase 1: Discovery
    Bob->>Alice: Sends Shielded Address (CPK + EPK)

    Note over Alice, Bob: Phase 2: Transmission
    Alice->>Alice: Generate Secret (32 bytes)
    Alice->>Alice: Create CoinInfo (Value=0, Nonce=Secret)
    Alice->>Alice: Encrypt CoinInfo using Bob's EPK
    Alice->>Alice: Generate ZK Proof
    Alice->>Ledger: Submit Transaction (ciphertext)

    Note over Alice, Bob: Phase 3: Reception
    Ledger-->>Bob: Broadcast new Block (ciphertext)
    Bob->>Bob: Attempt Decrypt with esk
    Bob->>Bob: Success! Recover CoinInfo
    Bob->>Bob: Extract Secret from Nonce

## 7. Extension: Secure Group Secret Distribution

This protocol scales effectively for distributing the **same secret** to **multiple recipients** (e.g., Alice sends a shared access key to Bob, Charlie, and Dave).

### 7.1 Mechanism
Alice executes the "Transmission" phase multiple times, once for each recipient:
1.  **To Bob**: Encrypts `Token(Nonce=S)` using `EPK_Bob`.
2.  **To Charlie**: Encrypts `Token(Nonce=S)` using `EPK_Charlie`.
3.  **To Dave**: Encrypts `Token(Nonce=S)` using `EPK_Dave`.

### 7.2 Security Properties
*   **Network Privacy**: Each transaction uses a fresh ephemeral key (`r`) for encryption. The resulting ciphertexts (`Ciphertext_Bob`, `Ciphertext_Charlie`, etc.) are completely uncorrelated bytes. An external observer cannot link these transactions or determine they carry the same payload.
*   **Recipient Isolation**: Bob **cannot** determine that Charlie also received the secret. Bob can only decrypt his own output. He cannot decrypt Charlie's output to compare the payload. The distribution list is known **only** to Alice.
*   **Spend Independence**: Even if Bob and Charlie decide to "spend" these coins later (creating nullifiers), their nullifiers will be distinct because the nullifier derives from the *owner's key* as well as the nonce.
    *   `Nullifier_Bob = Hash(SK_Bob, Nonce_S, ...)`
    *   `Nullifier_Charlie = Hash(SK_Charlie, Nonce_S, ...)`
    *   The network sees two unrelated nullifiers, maintaining privacy even if the "message coins" are consumed.

This extension effectively turns Midnight into a **private multicast channel** where a single source can securely provision a group with shared credentials without leaking the group's membership or size.
```
