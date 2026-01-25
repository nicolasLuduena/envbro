# Issue 3: Implement `envbro log` and `envbro checkout`

## Context
With Hypercore, we have a history of all changes. We need to expose this to the user so they can audit changes or revert to a previous working state.

## Requirements
- **`envbro log <project> <env>`**: Lists the history of changes.
    - Shows: Sequence Number (Version ID), Timestamp (added as metadata), and Author (if available, "Local" for now).
- **`envbro checkout <project> <env> <seq>`**: Restores the environment file to the state at sequence `<seq>`.
    - *Note*: This should likely "apply" that old version as a *new* append to the log (like `git revert`), effectively making the "current" state equal to the old state, rather than moving the HEAD pointer back permanently.

## Implementation Steps
1.  Create `log` command in `commander`.
2.  Read the Hypercore stream from 0 to `length`.
3.  Create `checkout` command.
4.  Fetch block at `seq`, decrypt it, and write it to the project's target `.env` file.
5.  *Optional*: Prompt to append this restored version as the new "latest".

## Acceptance Criteria
- [ ] `envbro log` prints a list of versions.
- [ ] `envbro checkout 0` effectively restores the very first version of the file.
