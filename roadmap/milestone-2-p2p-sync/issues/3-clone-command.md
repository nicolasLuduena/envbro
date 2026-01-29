# Issue 3: Implement `envbro clone` (Client)

## Context
Clients need to join an existing environment using the ticket provided by the host.

## Requirements
- **Input**: `ticket-string`.
- **Functionality**:
    - Validate ticket format.
    - Tell Iroh Node to "Import" this document.
    - Start syncing.

## Implementation Steps
1.  Parse arguments: `envbro clone <ticket>`.
2.  `node.import_doc(ticket)`.
3.  Wait for initial sync (until content is available).
4.  Save the Doc ID to the local registry so we know we have this environment.

## Acceptance Criteria
- [ ] `envbro clone <ticket>` successfully replicates the data.
- [ ] The environment variables appear in `envbro ls`.
