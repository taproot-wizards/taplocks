# Taplocks-ZKP - Zero-Knowledge Proofs for Tapleaf Verification

This repository contains a proof-of-concept zero-knowledge proof (ZKP) component of the Taplocks protocol, implemented using RISC Zero's zkVM. It provides the cryptographic proofs needed to verify tapleaf constructions without revealing sensitive information.

## Overview

In the broader Taplocks protocol, this ZKP component serves a critical role in the trust model:

1. **Oracle Verification**: When an oracle creates a tapleaf header with a secret value, they generate a ZKP to prove that:
   - The header was constructed correctly
   - The padding is valid and contains no malicious code
   - The SHA-256 midstate was computed properly

2. **Multi-Oracle Support**: For multi-oracle constructions, each oracle in the chain generates a ZKP proving their contribution to the final tapleaf hash.

3. **Trust Minimization**: The ZKPs allow users to verify that oracles are following the protocol correctly without needing to trust them completely.

## Quick Start

First, make sure [rustup] is installed. The [`rust-toolchain.toml`][rust-toolchain] file will be used by `cargo` to automatically install the correct version.

To build and run the ZKP generation and verification:

```bash
cargo run
```

### Development Mode

For faster iteration during development, you can run the project in development mode with execution statistics:

```bash
RUST_LOG="[executor]=info" RISC0_DEV_MODE=1 cargo run
```

## Project Structure

The project follows the standard RISC Zero zkVM application structure:

```text
taplock_zkp
├── Cargo.toml
├── host
│   ├── Cargo.toml
│   └── src
│       └── main.rs                    <-- Host code for ZKP verification
└── methods
    ├── Cargo.toml
    ├── build.rs
    ├── guest
    │   ├── Cargo.toml
    │   └── src
    │       └── method_name.rs         <-- Guest code for ZKP generation
    └── src
        └── lib.rs
```

## Technical Details

The ZKP implementation focuses on proving the following properties:

1. **Header Construction**: The proof demonstrates that a tapleaf header was constructed according to the protocol:
   - Correct secret value length and format
   - Valid padding construction
   - Proper SHA-256 midstate computation

2. **Security Properties**: The ZKP ensures that:
   - The header contains no executable code
   - The padding is properly constructed
   - The midstate computation is correct

3. **Multi-Oracle Chain** (not yet implemented): For multi-oracle constructions, each proof in the chain verifies:
   - The previous oracle's contribution
   - The current oracle's addition
   - The correct chaining of SHA-256 states

For a complete explanation of the Taplocks protocol and how this ZKP component fits in, see the [main README](../README.md).
