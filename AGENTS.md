# envbro — agent guide

Single Rust CLI crate. Manages encrypted `.env` files locally and shares them via Iroh P2P.

## Commands

```
cargo build                    # debug build
cargo run -- <subcommand>      # run CLI
cargo test                     # all tests incl. integration
cargo test --test cli          # binary integration tests only (fast)
cargo test --test p2p_sync     # P2P blob transfer test
cargo test security            # unit tests in src/security.rs
cargo clippy                   # no CI — run manually
```

Single test binary or `cargo test` works. No special order required.

## Architecture

| What | Where |
|---|---|
| Binary entrypoint | `src/main.rs` (tokio async, clap derive) |
| Lib entrypoint | `src/lib.rs` — re-exports all modules |
| CLI handlers | `src/commands.rs` |
| Encryption | `src/security.rs` — Argon2 → AES-256-GCM |
| Storage | `src/store/iroh.rs` — Iroh blob store + `manifest.json` |
| P2P networking | `src/network/iroh.rs` — Iroh provider/downloader |

## Quirks

- **Passphrase**: read from `ENVBRO_PASSPHRASE` env var, or prompted interactively (dialoguer). `list` and `share` skip it. `register --ticket` skips it (blob is already encrypted).
- **Storage**: `~/.envbro/` by default. Override via `ENVBRO_ROOT` env var.
- **Security format**: `[16B salt][12B nonce][ciphertext+tag]` — all inline, no envelope.
- **P2P tests**: fully offline (temp dirs, no external services). `p2p_sync.rs` sleeps 1s for local discovery.
- **Store atomicity**: `manifest.json` written via tmp → backup → rename dance.
- **No CI, no toolchain file, no formatter config**. Edition 2021, `clippy` expected but not enforced.
