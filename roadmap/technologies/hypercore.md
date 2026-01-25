# Technical Deep Dive: Hypercore

## What is Hypercore?
Hypercore is a secure, distributed append-only log structure. It is the building block of the Hypercore Protocol (formerly Dat).

Think of it as a **Blockchain without Global Consensus**. Use it when you want a verifiable history that can be shared via P2P, but you don't need the entire world to agree on the state—only the interested peers.

## Why Hypercore for EnvBro?
1.  **Append-Only**: Perfect for version history. Every `envbro set` is just an `append()` operation.
2.  **Verifiable**: It uses a Merkle Tree. When you sync, you cryptographically verify that the data you received matches the public key (Discovery Key).
3.  **Encrypted Transport**: Connections between peers are encrypted by default (NOISE protocol).
4.  **Sparse Replication**: You don't need to download the *entire* history if you only want the latest version (though for env vars, history is small enough to keep).

## Architectural Decisions

### 1. One Hypercore per "Environment"
**Decision**: Each `Project` + `Environment` pair (e.g., `my-app/production`) will have its own dedicated Hypercore.

**Reasoning**:
-   **Granular Access Control**: If we put all environments in one core, sharing that core shares EVERYTHING. By splitting them, we can share *only* the "production" keys with the DevOps team and *only* the "staging" keys with the contractors.
-   **Performance**: Smaller, focused logs are faster to sync.

**Complexity**:
-   **Managing Many Keys**: The client needs to track a mapping: `Alias -> Discovery Key`.
-   **Storage**: On disk, this creates a folder structure like `~/.envbro/storage/<discovery-key>/...`.

### 2. Multi-Writer Complexity
**The Challenge**: Hypercore is *Single-Writer* by default. Only the creator (who has the Private Key) can append.
-   *Scenario*: If Developer A creates the env, Developer B can read it but cannot add new variables.

**The Solution (Autobase)**:
-   For true multi-user editing, we need **Autobase** (built on Hypercore). It allows multiple peers to have their *own* input cores, and sets up a linearized "view" (output) that merges them.
-   **MVP Decision**: For Milestone 1 & 2, we will stick to **Single-Writer** (Owner-mode) or **Shared Private Key** (Trusted Team Mode).
    -   *Trusted Team Mode*: We share the Write Key (Private Key) among the team. This is simple but means if one person is compromised, the write-access is compromised.
    -   *Future*: Migrate to Autobase for per-user write capability.

## Data Structure
Inside the Hypercore block, we store:
```json
{
  "seq": 12,
  "timestamp": 1700000000,
  "author": "Alice",
  "data": "ENCRYPTED_BLOB_BASE64",
  "nonce": "IV_FOR_AES_GCM"
}
```
*Note: The `data` field is the AES-256-GCM encrypted JSON of the environment variables.*
