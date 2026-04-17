# Drift Exploit: Durable Nonce Detection PoC

This repository contains a Proof of Concept (PoC) demonstrating how the Drift Protocol security team could have detected compromised council members **before** executing the multisig migration on March 25, 2026.

## Context

During the Drift Protocol incident, exploiter utilized **Durable Nonce** accounts to execute governance transactions.

By simply querying the Solana blockchain for these nonce accounts prior to the migration, the protocol team could have identified the compromise and halted the procedure.

## How it Works

The script `src/lib.rs` performs the following steps:

1. Iterates through a list of Council Member public keys.
   
2. Queries the Solana **System Program** for accounts with:
   - Size: 80 bytes (standard Nonce Account size).
   - State: Initialized.
   - Authority: Matching the Council Member's public key.
  
3. Compares found accounts against a whitelist of known legitimate nonces.
   
4. Returns any "orphan" or suspicious nonce accounts.

## Project Structure

- `src/lib.rs`: Core logic for scanning durable nonces.
- `tests/test_durable_nonce_check.rs`: Integration test using local validator data.
- `deps/accounts/`: Pre-loaded account data simulating the state before the exploit.
- `setup.sh`: Script to fetch/save account data (for reproducibility).
- `Makefile`: Helper commands to run the local test validator.

## Prerequisites

- Rust toolchain (`cargo`)
- Solana CLI tools (`solana-test-validator`, `solana`)

## Usage

### 1. Start Local Validator with Exploit State

This loads the specific account `7s7s6saC5LHZoLyBXLM3pCjpWaA7meyQdP8NiH9ktAeC` (nonce_authority = `39JyWrdbVdRqjzw9yyEjxNtTbTKcTPLdtdCgbz7C7Aq8`) into the local validator ledger.

```bash
make start-local-validator
```

Keep this terminal open.

### 2. Run the Detection Test

In a new terminal, run the test to verify detection:

```bash
make test
```
or:

```bash
cargo test --test test_durable_nonce_check -- --nocapture
```

### Expected Output

The test will output the detected suspicious nonce account:

```bash
Scanning for durable nonces controlled by council members...
Found 1 suspicious nonce account(s):
  Nonce: 7s7s6saC5LHZoLyBXLM3pCjpWaA7meyQdP8NiH9ktAeC controlled by: 39JyWrdbVdRqjzw9yyEjxNtTbTKcTPLdtdCgbz7C7Aq8
```

## Conclusion

The existence of these accounts was public knowledge on-chain. Their presence was a clear "red flag" indicating that the private keys or signing authority of council members were potentially compromised or being prepared for misuse.

This PoC proves that basic due diligence could have prevented the subsequent exploit.