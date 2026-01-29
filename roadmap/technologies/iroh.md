# Technical Deep Dive: Iroh

## What is Iroh?
Iroh is a next-generation peer-to-peer (P2P) sync protocol written in Rust. It focuses on making direct device-to-device connections "just work" (via hole-punching and relays) and providing efficient primitives for syncing data.

## Why Iroh for EnvBro?
1.  **Reliability (Relays)**: Unlike traditional DHTs which can be flaky on restrictive networks, Iroh uses a network of **DERP Relays**. If a direct P2P connection fails, the encrypted stream is seamlessly routed through a high-speed relay, ensuring 100% connectivity.
2.  **Rust Native**: Built in and for Rust, offering type safety, performance, and a robust ecosystem (unlike the fragmented Node.js <-> Rust ports of other protocols).
3.  **Efficient Sync**: Uses **Iroh Docs** (based on CRDTs) to sync state efficiently. We don't send the whole `.env` file every time; we only send the changes.

## Architectural Components

### 1. Iroh Docs (The "Live" Environment)
Instead of an "Append-only Log", we treat each environment (e.g., `production`) as an **Iroh Document**.
*   **Structure**: A key-value store where keys are the environment variable names (e.g., `DATABASE_URL`) and values are the encrypted secrets.
*   **Sync**: When a developer changes a variable, they are effectively "editing the document". These edits are automatically synced to other peers subscribed to that document.
*   **Conflict Resolution**: Uses LWW (Last-Write-Wins) by default, which is perfect for environment variables (the latest value is usually the correct one).

### 2. The Ticket (Identity & Discovery)
Sharing an environment is done via a **Ticket**.
```
ticket-blob-...
```
This ticket contains:
*   **Capability**: The permission to Read (or Write) to the document.
*   **Node ID**: The public key of the sharer.
*   **Relay URL**: The address of the relay server to help find the sharer.

### 3. DERP Relays (Connectivity)
*   **Public Relays**: By default, EnvBro uses Iroh's global public relays for free dev usage.
*   **Private Relays**: Enterprise users can host their own `iroh-relay` instance (e.g., `relay.corp.com`) for total privacy and control. The relay acts only as a blind packet forwarder; it cannot decrypt the traffic.

## Data Structure
Inside an Iroh Doc entry:

```rust
struct EnvEntry {
    key: String,       // "DATABASE_URL"
    value: Vec<u8>,    // AES-256-GCM Encrypted ciphertext
    updated_at: u64,   // Timestamp
    author: PublicKey, // Who made the change
}
```
