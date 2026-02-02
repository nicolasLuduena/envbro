# Issue 4: Implement Key Export & Paper Backup (Deferred)

## Context
**Note**: This feature is scheduled for implementation after the core MVP is complete.

Users need a way to back up their master encryption keys in case their local device is lost. Since we are sovereign/local-first, there is no "Forgot Password" link.

## Requirements
- **Export**: Command to export the master key (or seed phrase) to a user-readable format (e.g., BIP-39 Mnemonic).
- **Import**: Command to restore the environment from the backup.

## Implementation Steps
1.  Implement `envbro key export` -> prompts for separate password, outputs Mnemonic.
2.  Implement `envbro key restore` -> accepts Mnemonic, re-derives keys.
3.  Ensure this process is secure and warns the user about handling plain-text keys.

## Acceptance Criteria
- [ ] `envbro key export` prints a valid mnemonic.
- [ ] `envbro key restore` successfully recreates the `~/.envbro` key files.
