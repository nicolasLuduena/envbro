# Issue 2: Implement `envbro share` (Host)

## Context
Sharing an environment should be as easy as sharing a link. With Iroh, we generate a **Ticket** that contains everything the peer needs to connect (Public Key + Relay URL).

## Requirements
- **Output**: Print a friendly string: `envbro clone ticket-abc...`.
- **Functionality**:
    - Load the local Iroh Doc for the environment.
    - Call `doc.share(ShareMode::Read)` (or Write).
    - Display the resulting Ticket.

## Implementation Steps
1.  Load `IrohNode` and `Doc`.
2.  Generate ticket `auth_ticket`.
3.  Print to console.

## Acceptance Criteria
- [ ] `envbro share` prints a valid Iroh ticket.
- [ ] The ticket works for a peer to join.
 until `Ctrl+C`.
