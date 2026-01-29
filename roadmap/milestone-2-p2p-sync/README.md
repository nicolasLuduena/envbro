# Milestone 2: P2P Sync & Collaboration

## Goal
The goal of this milestone is to enable **Peer-to-Peer** sharing of environment vaults. We will remove the need for emailing `.env` files or using Slack/Teams/Drive to share secrets.

We will use **Iroh** to discover peers via **Derp Relays** and tickets to sync the environment documents securely.

## deliverables
- [ ] **Discovery**: Peers can find each other using a **Ticket** (containing Public Key + Relay URL).
- [ ] **Syncing**: Data is synced automatically using Iroh's *live sync* protocol.
- [ ] **Reliability**: Use Relay servers to guarantee connection even across firewalls.
- [ ] **CI/CD Reliability**: High-availability read-only access for build pipelines (HTTP Gateway).

## Issues
| Issue | Description |
|Args|Args|
| [ISSUE-1](./issues/1-integrate-iroh-net.md) | Integrate Iroh Networking & Tickets |
| [ISSUE-2](./issues/2-share-command.md) | Implement `envbro share` (Generate Ticket) |
| [ISSUE-3](./issues/3-clone-command.md) | Implement `envbro clone` (Consume Ticket) |
| [ISSUE-4](./issues/4-ci-cd-gateway.md) | Implement HTTP Gateway for CI/CD |
| [ISSUE-5](./issues/5-relay-configuration.md) | Implement Custom Relay Configuration |
