# Issue 1: Integrate Iroh Networking & Tickets

## Context
We need a way for two computers to connect to each other through the internet without a central server logic. Iroh uses **Derp Relays** and **Tickets** to achieve this securely.

## Requirements
- **Dependency**: Use `iroh` crate.
- **Functionality**:
    - **Join**: Accept a `Ticket` string.
    - **Connect**: Use `iroh::node::Node` to connect to the peer.
    - **Share**: Generate a `BlobTicket` for an encrypted environment blob.
    - **Download**: Download the blob content using the ticket.

## Implementation Steps
1.  Create `NetworkManager` struct (wrapping Iroh Node).
2.  Implement `share(hash)` -> returns `BlobTicket`.
3.  Implement `get(ticket_str)` -> reads ticket, connects to peer, downloads content to local `blobs`.
4.  Once downloaded, decrypt using the shared project key.

## Notes
- Ensure the locally configured Relay URL is used when generating tickets (see Issue 5).
