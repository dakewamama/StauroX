# StauroX

A Solana verification service and client focused on tracking/verifying Wormhole bridge attestations.

**What's This?**

StauroX provides a Rust-based verification service and a small client that exercises the on-chain Anchor program. The service and client are intended for developers working with Wormhole bridge attestations and verification logs.

**Quick Start**

1. Build the service

```bash
# From the repository root
cargo build
```

2. Run the service (default: devnet)

```bash
# Run with informational logging
RUST_LOG=info cargo run

# Or explicitly run the binary (default-run is `staurox`)
cargo run --bin staurox
```

The service binary chooses `Devnet` by default. To run against mainnet, pass `mainnet` as the first command-line argument:

```bash
cargo run -- mainnet
```

3. Run the test client

The `staurox-client` is a small integration client that demonstrates initializing and attesting verification logs.

```bash
cd staurox-client
cargo run
```

The client expects a Solana keypair at `~/.config/solana/id.json` by default. If you need a local wallet for testing, create one with `solana-keygen new -o ~/.config/solana/id.json` or use your existing CLI config.

**Local development / validator**

- To run an on-machine validator for full local testing, start `solana-test-validator` in another terminal.
- The client in this repo targets Devnet by default. To point it at your local validator you can update the client code to use a custom RPC/Cluster or modify it to accept CLI args.

**What Just Happened (when running the client)**

- The client loads your wallet from `~/.config/solana/id.json`.
- It connects to a Solana cluster (Devnet by default).
- It initializes and queries the on-chain verification log PDA for the configured Wormhole bridge.
- It demonstrates attesting a verification entry and querying recent entries.

**Troubleshooting**

- **Missing keypair**: Ensure `~/.config/solana/id.json` exists.
- **Client can't connect**: Verify network connectivity to Devnet or run a local validator and point the client at it.
- **Program ID mismatch**: The client contains a `PROGRAM_ID` constant; ensure it matches the deployed `staurox_program` ID when testing locally.

**Next steps / Suggestions**

- Add a dedicated `examples/` folder to demonstrate integration flows and Web UI tests.
- Make `staurox-client` accept RPC/cluster via CLI or environment variables.
- Add documentation for PDA derivation and on-chain account layouts.

**License**

---

Contributions welcome. Open issues or PRs.
