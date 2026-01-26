# Milestone 3: Privacy & Governance (Midnight)

## Goal
The ultimate goal is to remove the "Shared Master Password" anti-pattern. We want **Identity-Based Access Control** without a central IAM server.

We will use **Midnight**, a privacy-preserving blockchain, to act as the gatekeeper.
- **Model**: Data is stored on P2P (Milestone 2). The *Encryption Keys* are stored in a Midnight Smart Contract.
- **Privacy**: The contract uses Zero-Knowledge Proofs (ZKPs) to verify if a user (wallet) is allowed to access the key, without revealing the user's identity or the key to the public.

## deliverables
- [ ] **Smart Contract**: `EnvRegistry` contract deployed on Midnight Testnet.
- [ ] **Access Flow**: `envbro login` (Wallet Auth) -> `envbro pull` (Proof of Access).
- [ ] **Caching**: Keys are cached locally encrypted to allow offline/fast access after initial auth.

## Issues
| Issue | Description |
|Args|Args|
| [ISSUE-1](./issues/1-midnight-contract.md) | Design `EnvRegistry` Smart Contract |
| [ISSUE-2](./issues/2-zkp-access-flow.md) | Implement Inbox Scanner & Decryption |
| [ISSUE-3](./issues/3-local-caching.md) | Implement Local Encrypted Caching |
| [ISSUE-4](./issues/4-key-recover.md) | Social Recovery Protocol |
| [ISSUE-5](./issues/5-key-rotation.md) | Key Rotation Logic (Future) |
