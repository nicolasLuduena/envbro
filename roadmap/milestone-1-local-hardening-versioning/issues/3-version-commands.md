# Issue 3: Implement `envbro log` and `envbro checkout`

## Context
With Iroh Docs, we have a history of all changes (if we model it that way, or we can just access the current state). For this milestone, we will implement a simple log of changes.

## Requirements
- **`envbro log`**: Lists the history of changes for the current environment.
    - Format: `<timestamp> - <author-id> - <action>`
- **`envbro checkout`**: Reverts the environment to a specific state.

## Implementation Steps
1.  Create `log` command in `commander`.
5.  *Optional*: Prompt to append this restored version as the new "latest".

## Acceptance Criteria
- [ ] `envbro log` prints a list of versions.
- [ ] `envbro checkout 0` effectively restores the very first version of the file.
