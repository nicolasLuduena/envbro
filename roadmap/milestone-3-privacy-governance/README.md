# Milestone 3: Privacy & Governance (Midnight)

## Goal
The ultimate goal is to remove the "Shared Master Password" anti-pattern. We want **Identity-Based Access Control** without a central IAM server.

We will use **Midnight**, a privacy-preserving blockchain, to act as the gatekeeper.
- **Model**: Data is stored on P2P (Milestone 2). The *Access Token* (Shielded UTXO) carries the decryption key.
- **Privacy**: We leverage Zswap's shielded infrastructure to securely transfer the 32-byte `ProjectKey` via the unconstrained `nonce` field of a shielded output.

## deliverables
- [ ] **Smart Contract**: `EnvAccess` contract (Policy Manager) deployed on Midnight Testnet.
- [ ] **Key Distribution**: `envbro share` (Mints Shielded UTXO with Key).
- [ ] **Access Flow**: `envbro pull` (Scans for Shielded UTXO -> Extracts Key -> Decrypts Iroh).

## Issues
| Issue | Description |
|Args|Args|
| [ISSUE-1](./issues/1-midnight-contract.md) | Design `EnvAccess` Policy Contract |
| [ISSUE-2](./issues/2-zkp-access-flow.md) | Implement Shielded Key Extraction (Secret Field Abuse) |
| [ISSUE-3](./issues/3-local-caching.md) | Implement Local Encrypted Caching |
| [ISSUE-4](./issues/4-key-recover.md) | Social Recovery Protocol |
