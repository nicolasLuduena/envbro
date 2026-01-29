# Issue 4: Key Rotation & Disaster Recovery (Future Feature)
> [!WARNING]
> **Out of Scope**: This feature is NOT part of the current Milestone 3. It is documented for future implementation.


## Context
**The Emergency**: A developer's laptop is stolen, or the `VaultKey` is accidentally pasted into a public chat.
**The Consequence**: The attacker can read the *entire* history of that environment (because the old key works on old data).

You cannot "un-leak" a key. Once it's out, the **History is Compromised**.
However, we must protect the **Future**.

## Protocol 1: The "Vault Rotation" (Protecting the Future)
Use this when you want to revoke access for a specific user or group going forward.

1.  **Generate a New Key**: Admin generates `VaultKey_v2`.
2.  **Re-Encrypt Head**: Admin takes the *current* `.env` state, encrypts it with `VaultKey_v2`.
3.  **Append Checkpoint**: Push this new blob to the Hypercore.
    *   *Metadata*: Mark this block as `KeyRotation: true`.
4.  **Distribute v2**: Use the Inbox Scanner (Issue 2) to send `VaultKey_v2` *only* to the remaining secure team members.
    *   *Effect*: The compromised user (with `VaultKey_v1`) can still read old blocks, but will fail to decrypt any new blocks.

## Protocol 2: The "Scorched Earth" (Mitigating the Past)
Use this when the actual secrets (API Keys) are compromised.

1.  **Rotate Provider Credentials**: Go to AWS/Stripe/etc. and revoke the leaked API keys. Issue new ones.
    *   *Note*: No software tool can do this for you automagically; this is a manual DevOps task.
2.  **Fork the Vault**:
    *   Create a *brand new* Hypercore feed.
    *   Add the *new* credentials.
    *   Encrypt with a *new* `VaultKey`.
3.  **Abandon the Old Core**: Tell the team to update their config to point to the new Hypercore Discovery Key.
    *   *Effect*: The old Hypercore contains useless (revoked) keys. The new Hypercore is clean and secure.

## Implementation Requirements
- [ ] Add `envbro rotate-key` command (Protocol 1).
- [ ] Add metadata support in Hypercore blocks to indicate "Key Changed".
