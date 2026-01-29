# Issue 1: Integrate Iroh Networking & Tickets

## Context
We need a way for two computers to connect to each other through the internet without a central server logic. Iroh uses **Derp Relays** and **Tickets** to achieve this securely.

## Requirements
- **Dependency**: Use `iroh` crate.
- **Functionality**:
    - **Join**: Accept a `Ticket` string.
    - **Connect**: Use `iroh::node::Node::import_doc(ticket)` to start syncing.
    - **Share**: Generate a ticket for a local document using `doc.share()`.

## Implementation Steps
1.  Create `NetworkManager` struct (wrapping Iroh Node).
2.  Implement `share(doc_id)` -> returns `Ticket`.
3.  Implement `join(ticket_str)` -> validates ticket, imports doc, and starts gossip loop.
4.  Ensure the "Gossip" loop is running in the background to receive live updates.

## Notes
- Ensure the locally configured Relay URL is used when generating tickets (see Issue 5).
