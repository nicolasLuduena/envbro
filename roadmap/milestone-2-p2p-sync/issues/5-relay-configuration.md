# Issue 5: Relay Infrastructure & Configuration

## Context
By default, Iroh uses public relays maintained by n0. To be good citizens of the ecosystem (and avoid potential rate limits), **EnvBro should host its own community relay**.

Additionally, Enterprise users often want to host their *own* private relay infrastructure to ensure no encrypted traffic passes through third-party servers.

## Requirements
- **Infrastructure**: Deploy a cheap, high-bandwidth VPS (e.g. $5/mo) running `iroh-relay`.
    - Domain: `relay.envbro.com` (example).
- **Configuration**:
    - Update `Config` struct to include `Option<String> relay_url`.
    - **Default**: Point to `relay.envbro.com` instead of n0 default (or use n0 as fallback).
- **Commands**: 
    - `envbro config set relay <url>` (for enterprises).
    - `envbro config get relay`.

## Implementation Steps
1.  **Ops**: Terraform/Ansible script to provision an `iroh-relay` server on DigitalOcean/Hetzner.
2.  **CLI**: Update `envbro` to prefer this relay when creating tickets.
3.  **CLI**: Add `config set relay` command for users who want to override it.

## Acceptance Criteria
- [ ] An official `relay.envbro.com` is online.
- [ ] `envbro share` generates tickets pointing to this relay by default.
- [ ] Users can override this with `envbro config set relay`.
