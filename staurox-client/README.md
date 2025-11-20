# staurox-client

A small integration client that demonstrates the common verification flows used by `staurox_program`.

Quick Start

1. Ensure you have a Solana keypair.

```bash
solana-keygen new -o ~/.config/solana/id.json
```

2. (Optional) Run a local validator for offline testing

```bash
# in a separate terminal
solana-test-validator
```

3. Run the client

```bash
cd staurox-client
cargo run
```

What the client does

- Loads the wallet at `~/.config/solana/id.json`.
- Connects to `Devnet` by default and looks up the program ID configured in the code.
- Initializes the verification log PDA for the configured Wormhole bridge if missing.
- Sends a sample attestation and then queries recent verifications.

Notes

- To point the client at a different cluster (e.g., a local validator), edit the `Cluster::Devnet` value in `src/main.rs` or change the client to accept a CLI argument.
- If you deploy `staurox_program` locally, update the `PROGRAM_ID` constant to match your deployed ID.

Troubleshooting

- **Keypair not found**: the client expects `~/.config/solana/id.json`.
- **RPC/connection issues**: verify RPC URL and network connectivity.