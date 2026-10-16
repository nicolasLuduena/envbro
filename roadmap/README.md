# EnvBro 2.0 Roadmap: Decentralized & Private Configuration Management

## Vision
In the modern distributed web, developers shouldn't have to choose between convenience and security. **EnvBro 2.0** envisions a world where environment variables and secrets are managed without trusting a central authority. By combining an **encrypted local vault** with **Peer-to-Peer (P2P)** sharing, we aim to give developers complete sovereignty over their configuration data.

## Core Objectives

### 1. Zero-Trust Sovereignty
Your secrets should never leave your control in plaintext. We aim to remove reliance on centralized SaaS providers for secret management. "Don't trust, verify."

### 2. Privacy-First Collaboration
Sharing secrets shouldn't mean broadcasting them. Secrets are encrypted before they leave your machine and travel directly between peers, so only the people you share a ticket with can ever get the data.

### 3. Simplicity (KISS)
We prioritize a simple, robust "Keep It Simple Stupid" approach. Our goal is to provide a seamless P2P file sharing experience for environment variables without unnecessary complexity.

## Why EnvBro? (vs. The World)

| Feature | **EnvBro** | **Doppler / AWS Secrets** | **HashiCorp Vault** | **Dotenv (.env files)** |
| :--- | :--- | :--- | :--- | :--- |
| **Trust Model** | **Trustless** (You hold the keys) | **Trusted Third Party** (They hold the keys) | **Trusted Server** (You manage the server) | **Insecure** (Plaintext) |
| **Storage** | **Iroh Blobs + Local Encrypted** | Centralized Cloud DB | Centralized Private DB | Local Disk / Git (Bad) |
| **Setup** | Zero-Config (`cargo install envbro`) | Account Creation & Billing | Complex Infrastructure | Zero-Config |
| **Offline** | **First-Class Citizen** | Requires Connectivity (mostly) | Requires Connectivity | First-Class Citizen |
| **Identity** | Decentralized (Keypair) | IAM / SSO | IAM | None |

**EnvBro fills the gap**: It offers the *security* of Vault and the *cloud-sync* of Doppler, but with the *simplicity* of local `.env` files and the *sovereignty* of P2P.

## Sustainability & Monetization
How do we keep the lights on while remaining open-source and sovereign?
1.  **"EnvBro Cloud" Backup**:
    *   An optional encrypted backup service for users who don't want to rely solely on P2P for data availability (e.g., if all peers are offline).

## Milestones Overview

### [Milestone 1: Local Hardening](./milestone-1-local-hardening/README.md)
**Goal**: Transform the local client into a secure, encrypted vault using **Iroh**.
- *Focus*: Encryption at rest, Iroh Blobs, CLI UX.

### [Milestone 2: P2P Sync & Collaboration](./milestone-2-p2p-sync/README.md)
**Goal**: Enable serverless sharing of environment vaults between trusted peers.
- *Focus*: Iroh Networking, Real-time sync, Ticket-based sharing, Relay Configuration.
