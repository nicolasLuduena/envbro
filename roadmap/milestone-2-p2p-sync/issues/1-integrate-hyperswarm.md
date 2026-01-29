# Issue 1: Integrate Hyperswarm for P2P Discovery

## Context
We need a way for two computers to connect to each other through the internet without a central server. Hyperswarm provides this via a DHT.

## Requirements
- **Dependency**: Install `hyperswarm`.
- **Functionality**:
    - Abstraction layer to "Join" a topic (Discovery Key).
    - Handle connection events (`connection`) to pipe the Hypercore replication stream.

## Implementation Steps
1.  Create `NetworkManager` class.
2.  Initialize `Hyperswarm` instance.
3.  Implement `join(topic)` method.
4.  Setup replication stream logic: `core.replicate(socket)`.

## Notes
- Ensure firewall/NAT traversal is handled (Hyperswarm does most of this, but testing is required).
