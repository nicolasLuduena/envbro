# Issue 5: Key Rotation & Disaster Recovery (Deferred)

## Context
**Note**: This feature is scheduled for implementation after the core MVP is complete.

**The Emergency**: A developer's laptop is stolen, or the vault passphrase/key is accidentally pasted into a public chat.
**The Consequence**: The attacker can read every blob encrypted with that key (because the old key works on old data).

You cannot "un-leak" a key. Once it's out, the **History is Compromised**.
However, we must protect the **Future**.

## Protocol 1: The "Vault Rotation" (Protecting the Future)
Use this when you want to revoke access for a specific user or group going forward.

1.  **Generate a New Key**: Derive `VaultKey_v2` from a new passphrase.
2.  **Re-Encrypt Head**: Take the *current* `.env` state, encrypt it with `VaultKey_v2`, and store it as new blobs.
3.  **Update Manifest**: Point the manifest at the new blobs and mark the change (e.g., `KeyRotation: true`).
4.  **Re-Share**: Share the rotated vault (`envbro share`) *only* with the remaining trusted team members.
    *   *Effect*: The compromised user (with `VaultKey_v1`) can still read old blobs, but will fail to decrypt any new ones.

## Protocol 2: The "Scorched Earth" (Mitigating the Past)
Use this when the actual secrets (API Keys) are compromised.

1.  **Rotate Provider Credentials**: Go to AWS/Stripe/etc. and revoke the leaked API keys. Issue new ones.
    *   *Note*: No software tool can do this for you automagically; this is a manual DevOps task.
2.  **Fork the Vault**:
    *   Create a *brand new* vault.
    *   Add the *new* credentials.
    *   Encrypt with a *new* `VaultKey`.
3.  **Abandon the Old Vault**: Tell the team to drop the old vault and clone the new one.
    *   *Effect*: The old vault contains useless (revoked) keys. The new vault is clean and secure.

## Implementation Requirements
- [ ] Add `envbro rotate-key` command (Protocol 1).
- [ ] Add manifest metadata to indicate "Key Changed".
