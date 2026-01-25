# Milestone 2: P2P Sync & Collaboration

## Goal
The goal of this milestone is to enable **Peer-to-Peer** sharing of environment vaults. We will remove the need for emailing `.env` files or using Slack/Teams/Drive to share secrets.

We will use **Hyperswarm** to discover peers via a Distributed Hash Table (DHT) and replicate the Hypercore logs securely.

## deliverables
- [ ] **Discovery**: Peers can find each other using a shared key.
- [ ] **Syncing**: Data is replicated automatically when peers connect.
- [ ] **Access Control (Basic)**: Only those with the "Discovery Key" (Capability) can find and sync the data.

## Issues
| Issue | Description |
|Args|Args|
| [ISSUE-1](./issues/1-integrate-hyperswarm.md) | Integrate Hyperswarm for P2P Discovery |
| [ISSUE-2](./issues/2-share-command.md) | Implement `envbro share` (Host) |
| [ISSUE-3](./issues/3-clone-command.md) | Implement `envbro clone` (Client) |
