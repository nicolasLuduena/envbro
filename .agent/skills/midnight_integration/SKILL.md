---
name: midnight_integration
description: Detailed knowledge and guidelines for Midnight Network integrations, covering ZK proofs, Compact contracts, Zswap, and architectural principles.
---

# Midnight Integration Skill

This skill provides the necessary context and knowledge for reasoning about Midnight Network integrations. Use this information when the user asks about Midnight, ZK proofs, Compact contracts, or related architectural decisions.

## 1. What Midnight Is

Midnight is a **data protection blockchain** that maintains two parallel states:

- **Public state** – on-chain, visible to everyone (transaction proofs, contract code, intentionally public data).
- **Private state** – off-chain, stored on users’ machines or controlled servers, never directly exposed to the network. [[What is Midnight](https://docs.midnight.network/learn/what-is-midnight)]

The bridge between these is **zero-knowledge (ZK) cryptography**, specifically zk‑SNARKs. Computation on private data happens off-chain; only a succinct proof plus chosen public outputs go on-chain. [[What is Midnight](https://docs.midnight.network/learn/what-is-midnight); [Zero-knowledge proofs](https://docs.midnight.network/learn/understanding-midnights-technology/zero-knowledge-proofs)]

Midnight is implemented as a proof‑of‑stake chain (a Polkadot‑SDK–style runtime with pallets like `pallet-midnight`), but most on-chain activity uses a **proof‑based transaction model** rather than ordinary signed extrinsics. [[Onchain logic](https://docs.midnight.network/operate/network-architecture/onchain-logic); [Transactions](https://docs.midnight.network/operate/network-architecture/transactions)]

---

## 2. Zero‑Knowledge Proofs on Midnight

### 2.1 ZK and zk‑SNARKs

A **zero‑knowledge proof** lets a prover convince a verifier a statement is true without revealing the underlying secret (the *witness*). Typical examples:

- “This customer is over 18” without revealing the exact birthdate.
- “This account has sufficient balance” without revealing the actual balance. [[Zero-knowledge proofs](https://docs.midnight.network/learn/understanding-midnights-technology/zero-knowledge-proofs)]

Midnight uses **zk‑SNARKs**:

- **Succinct**: proof size and verification cost are sublinear in computation size.
- **Non‑interactive**: the prover produces a single proof; the verifier checks it without back‑and‑forth. [[ZK Snarks](https://docs.midnight.network/learn/understanding-midnights-technology/zero-knowledge-proofs#zk-snarks)]

The process:

1. The contract logic is compiled into an arithmetic circuit (via the Compact compiler).
2. Off‑chain, a prover supplies private inputs (witness) and public inputs, runs the circuit, and produces a zk‑SNARK proof.
3. On‑chain, the verifier uses a **verifier key** plus the public inputs to check the proof in milliseconds, without seeing the private data. [[Smart contracts on Midnight](https://docs.midnight.network/develop/how-midnight-works/smart-contracts#transcripts-and-zk-snarks); [Semantics – contract state](https://docs.midnight.network/develop/how-midnight-works/semantics#contract-state)]

These proofs are used both for:

- **Zswap** (shielded UTXO‑style asset transfers and swaps). [[Zswap key features](https://docs.midnight.network/learn/understanding-midnights-technology/zswap#key-features-and-benefits-of-zswap)]
- **Smart contracts** (Compact circuits over public + private state). [[Smart contracts on Midnight](https://docs.midnight.network/develop/how-midnight-works/smart-contracts)]

---

## 3. Compact: Midnight’s Smart Contract Language

**Compact** is a TypeScript‑like DSL for privacy‑preserving smart contracts. [[What is Midnight](https://docs.midnight.network/learn/what-is-midnight); [Writing a contract](https://docs.midnight.network/develop/reference/compact/writing)]

Key points:

- Contracts expose **exported circuits** (entry points) that clients call.
- The compiler emits:
  - Zero‑knowledge circuits (ZKIR) + proving/verifier keys.
  - A TypeScript/JS implementation (`contract/index.cjs` + `index.d.cts`) used off‑chain. [[Compiler usage](https://docs.midnight.network/compact/reference/tools/compiler-usage); [TypeScript target](https://docs.midnight.network/develop/reference/compact/lang-ref#typescript-target); [JS implementation guide](https://docs.midnight.network/develop/guides/compact-javascript-runtime)]

Example structure (simplified lock contract):

```compact
pragma language_version 0.16;

import CompactStandardLibrary;

enum State {
  UNSET,
  SET
}

export ledger authority: Bytes<32>;
export ledger value: Uint<64>;
export ledger state: State;
export ledger round: Counter;

constructor(sk: Bytes<32>, v: Uint<64>) {
  authority = disclose(publicKey(round, sk));
  value = disclose(v);
  state = State.SET;
}

circuit publicKey(round: Field, sk: Bytes<32>): Bytes<32> {
  return persistentHash<Vector<3, Bytes<32>>>(
           [pad(32, "midnight:examples:lock:pk"), round as Bytes<32>, sk]);
}

export circuit get(): Uint<64> {
  assert(state == State.SET, "Attempted to get uninitialized value");
  return value;
}
```

This illustrates:

- `ledger` section defines **on‑chain public state**.
- `circuit` defines logic. `export circuit` makes an entry point. [[Writing a contract](https://docs.midnight.network/develop/reference/compact/writing)]

### 3.1 Public vs Private / Witness Data

Compact draws a strict line between:

- **Public / ledger data** – stored on chain, visible to all.
- **Witness / private data** – supplied via:
  - `witness` callbacks,
  - circuit arguments,
  - constructor arguments, and anything derived from them. [[Explicit disclosure](https://docs.midnight.network/develop/reference/compact/explicit_disclosure#introduction)]

By default, **witness data must not be disclosed**. If a contract wants to reveal something derived from private data (e.g., a risk score, compliance signal), the developer must mark this explicitly using **explicit disclosure** constructs before:

- Writing it to the ledger,
- Returning it from an exported circuit,
- Passing it to another contract. [[Explicit disclosure](https://docs.midnight.network/develop/reference/compact/explicit_disclosure#introduction)]

This “witness protection” model is crucial for any AI that reasons about **what information becomes public vs stays private**.

---

## 4. Public and Private State Model (Kachina)

Midnight’s contract model is based on **Kachina**, which formalizes smart contracts as **reactive state machines** with two states per contract: [[Kachina](https://docs.midnight.network/learn/understanding-midnights-technology/kachina)]

- **Public state** – global, replicated on chain (Impact/ledger state).
- **Private state** – local per user, stored off‑chain.

Important properties:

- A single user action (a **transition function** / circuit call) can conceptually update **both** public and private state.
- On‑chain, only the public update is committed (after proof verification).
- Off‑chain, each client maintains their own private state and updates it using **private transcripts** or private event handlers when they observe relevant on‑chain transactions. [[Private state management](https://github.com/midnightntwrk/midnight-architecture/blob/main/components/lares/private-state-management/README.md)]

The Kachina pattern:

1. User calls a transition function off‑chain.
2. Runtime:
   - Executes logic against current public+private state.
   - Records **public** and **private** transcripts of oracle calls (reads/writes).
   - Extracts a **witness** for the ZK circuit.
   - Rolls back speculative state changes; only transcripts + witness remain. [[Flow – generating a transaction](https://github.com/midnightntwrk/midnight-architecture/blob/main/user-flows/dapp-user/dApp%20User%20Generating%20a%20Transaction.md)]

3. Proof server generates a ZK proof from the circuit + witness.
4. Wallet wraps this in a transaction and submits it.
5. On‑chain verification:
   - Verifies the proof.
   - Replays/applies the **public transcript** to the contract state.
6. Off‑chain, when the dApp sees the confirmed transaction, it:
   - Re-applies the saved **private transcript** to its local private state. [[Transcript application](https://github.com/midnightntwrk/midnight-architecture/blob/main/user-flows/dapp-user/dApp%20User%20Generating%20a%20Transaction.md)]

For an AI reasoner, the key takeaway: **contract semantics are split into a verifiable public transcript and a private transcript**, and correctness is enforced via ZK + transcript replay. This shapes how you model side effects and data flows.

---

## 5. Transaction and Execution Semantics

### 5.1 Ledger Structure

Midnight’s ledger state holds: [[Transaction semantics](https://docs.midnight.network/develop/how-midnight-works/semantics)]

- **Zswap state**:
  - Merkle tree of coin commitments.
  - Nullifier set.
  - Set of valid past Merkle roots.
- **Contract map** from addresses to:
  - Impact state value (contract public state).
  - Map of entry points → operations (each holds a verifier key).

### 5.2 Transactions: High-Level Structure

Each transaction generally includes:

- A mandatory **guaranteed Zswap offer**.
- An optional **fallible Zswap offer**.
- An optional **contract call section**, with:
  - Sequence of `ContractCall` and/or contract deploy items.
  - Binding commitment + randomness for integrity. [[Building blocks](https://docs.midnight.network/develop/how-midnight-works/building-blocks)]

Each `ContractCall` has: [[ContractCall proposal](https://github.com/midnightntwrk/midnight-architecture/blob/main/proposals/0005-transaction-structure-v1.md)]

- Contract address.
- Entry point identifier.
- Public oracle transcript.
- Communication commitment.
- ZK proof bound to that transcript and contract operation.

### 5.3 Execution Phases and Fallibility

Transactions execute in three stages: [[Transaction semantics](https://docs.midnight.network/develop/how-midnight-works/semantics)]

1. **Well‑formedness check** (no state):
   - Structural correctness.
   - zk‑proofs in Zswap offers verify.
   - Schnorr proof for contract section.
   - Guaranteed and fallible offers are value‑balanced (with adjustments for fees and mints).
   - Inputs/outputs/contract calls are claimed exactly once according to their phase.

2. **Guaranteed phase** (against current state):
   - Apply Zswap guaranteed offer: add commitments, check nullifiers unused, check Merkle roots valid.
   - Verify contract proofs:
     - Look up contract operations and verifier keys.
     - Run Impact program with transcript in “verification mode”, check effects equal declared effects.
     - Commit resulting contract state if it is “strong”.
   - **Fees are charged here**.

3. **Fallible phase**:
   - Similar logic for any fallible Zswap/contract sections.
   - If the fallible phase fails, **effects of the guaranteed phase remain** (partial success), but the fallible phase effects are discarded. [[Transaction semantics – fallibility](https://docs.midnight.network/develop/how-midnight-works/semantics#transaction-fallibility); [Phase execution](https://docs.midnight.network/develop/how-midnight-works/semantics#phase-execution)]

For AI reasoning, this means:

- Effects may be **partially applied** if only the fallible phase fails.
- Fee payment is independent of fallible success.

---

## 6. Zswap: Private Assets and Commitments

Midnight’s asset layer is built on **Zswap**, a non‑interactive multi‑asset swap and shielded transfer protocol based on zk‑SNARKs. [[Zswap key features](https://docs.midnight.network/learn/understanding-midnights-technology/zswap#key-features-and-benefits-of-zswap)]

Goals:

1. Preserve confidentiality:
   - Sender, receiver, amounts, and asset types are hidden from observers.
2. Non‑interactive protocol (no post‑broadcast interaction).
3. Support private **atomic swaps** across multiple assets. [[Wallet spec – intro](https://github.com/midnightntwrk/midnight-architecture/blob/main/components/WalletEngine/Specification.md)]

Core mechanics: [[Wallet spec – coin nullifiers](https://github.com/midnightntwrk/midnight-architecture/blob/main/components/WalletEngine/Specification.md)]

- **Coin commitment**:
  - Function of (value, asset type, nonce, receiver address).
  - Output is public; inputs are secret.
  - Commitments are stored in a Merkle tree; spender must prove inclusion using a Merkle proof.
- **Nullifier**:
  - Hash of (coin, sender secret key).
  - Ledger stores a set of nullifiers; if a nullifier is reused, it indicates a double‑spend attempt and transaction is rejected.

The zero‑knowledge proof ensures:

- Inputs are owned by the sender.
- Inputs and outputs are consistent (no creation of value), up to a public **imbalance map** for each asset.
- Commitments correspond to valid coins.
- Nullifiers are correctly derived.

An AI reasoning about balances, privacy, and double-spend checks must model **commitment inclusion** and **nullifier uniqueness**, not explicit balances.

---

## 7. Proof Server and Proving Model

Because proving zk‑SNARKs is computationally heavy, Midnight uses a separate **proof server** process:

- Runs natively (e.g., in Docker) to handle:
  - **Contract proofs** (Compact circuits).
  - **Zswap proofs** (wallet spends/transfers). [[Proof server tutorial](https://docs.midnight.network/develop/tutorial/using/proof-server); [Wallet dev guide – proving a transaction](https://docs.midnight.network/develop/guides/wallet-dev-guide#proving-a-transaction)]

Recommended use:

- Run a **local** proof server or a remote one you control, over encrypted channels, because private data (ownership details, contract witness data) are sent to it. [[Proof server tutorial](https://docs.midnight.network/develop/tutorial/using/proof-server)]

Integration options:

- Wallets/DApps call `proveTransaction` or a similar API using “proving recipes” that bundle together the pieces that need proving. [[Wallet dev guide – proving a transaction](https://docs.midnight.network/develop/guides/wallet-dev-guide#proving-a-transaction)]
- JS integration via `@midnight-ntwrk/midnight-js-http-client-proof-provider` which creates a `ProofProvider` from a given proof server URL. [[httpClientProofProvider](https://docs.midnight.network/develop/reference/midnight-api/midnight-js/@midnight-ntwrk/midnight-js-http-client-proof-provider/functions/httpClientProofProvider)]

For AI modeling, treat proof server calls as:

- Local or controlled remote RPC.
- Deterministic: given circuit, state snapshot, and witness, they return a proof or fail.

---

## 8. Off-Chain Contract Execution via JS Runtime

When you compile a Compact contract:

- The compiler emits `contract/index.cjs` and `index.d.cts` that encode the **semantics in TypeScript/JS** plus type information. [[Compiler usage](https://docs.midnight.network/compact/reference/tools/compiler-usage); [TypeScript target](https://docs.midnight.network/develop/reference/compact/lang-ref#typescript-target); [JS implementation guide](https://docs.midnight.network/develop/guides/compact-javascript-runtime)]

The generated module exports:

- Types for user-defined Compact types.
- `Witnesses<T>` and `Circuits<T>` interfaces.
- A `Contract<T, W>` class with:
   - `circuits` and `impureCircuits`.
   - `initialState(privateState: T)` → `[T, ContractState]`.
- A `ledger(state)` function to inspect current ledger fields. [[TypeScript target](https://docs.midnight.network/develop/reference/compact/lang-ref#typescript-target)]

This lets DApps:

- Run contract logic **off-chain** in JS to:
  - Build transcripts.
  - Estimate effects and gas (where relevant).
  - Generate witnesses for proof creation.

An AI reasoning about **integration workflows** should be aware that **off-chain simulation** uses exactly this JS implementation, not an ad‑hoc re‑write.

---

## 9. RPC and Node Integration

Midnight nodes expose a JSON‑RPC interface (HTTP/WS), including: [[RPC networking](https://docs.midnight.network/operate/network-architecture/rpc-networking)]

- `midnight_jsonContractState` / `midnight_contractState` – fetch contract state (JSON or raw).
- `midnight_zswapChainState` – get current Zswap chain state (for wallets).
- `midnight_unclaimedAmount` – query unclaimed amounts for an address.
- `midnight_ledgerVersion` – snapshot of full ledger at a block.
- `midnight_apiVersions` – supported API versions.

Plus standard Polkadot‑SDK methods (`chain_getBlock`, `state_getStorage`, `rpc_methods`, etc.).

This is the **primary read interface** for dApps and tools; writes are performed via proof‑based transactions broadcast by the wallet/node.

---

## 10. Design Themes Relevant for AI Reasoning

When designing an LLM/AI agent around Midnight integrations, keep these invariant principles in its prompt/context:

1. **Separation of concerns**:
   - Contracts don’t see raw private data; they see commitments and ZK‑verified facts.
   - DApps manage private state off-chain and synchronize via transcripts + events.

2. **Selective disclosure by default**:
   - Private/witness data is never disclosed unless explicitly declared in Compact.
   - Any suggestion to “just store X on-chain” must consider explicit disclosure and privacy requirements. [[Explicit disclosure](https://docs.midnight.network/develop/reference/compact/explicit_disclosure#introduction)]

3. **Proof‑based authorization**:
   - Most Midnight transactions are unsigned; authorization is via zk‑proofs, not simple signatures. [[Transactions](https://docs.midnight.network/operate/network-architecture/transactions)]

4. **UTXO‑style privacy with Zswap**:
   - Model balances via commitments + nullifiers, not visible balances.
   - Track state by scanning Merkle trees and shielded outputs (wallet responsibility). [[Wallet spec – coin nullifiers](https://github.com/midnightntwrk/midnight-architecture/blob/main/components/WalletEngine/Specification.md)]

5. **Two‑phase execution and partial success**:
   - Distinguish between guaranteed vs fallible sections and understand partial success semantics. [[Transaction semantics](https://docs.midnight.network/develop/how-midnight-works/semantics)]

6. **Compact as the source of truth**:
   - All contract logic must be expressed in Compact; JS/TS is generated and should not be manually modified.
   - Any AI‑assisted refactor must respect this compile pipeline. [[JS implementation guide](https://docs.midnight.network/develop/guides/compact-javascript-runtime); [Compiler usage](https://docs.midnight.network/compact/reference/tools/compiler-usage)]
