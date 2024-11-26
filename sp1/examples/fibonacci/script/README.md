# kalypso-executable-base

The Repo contains template/base code to create `prover-executable` and `input-verification-executable` required by Kalypso

## Prover Executable
Edit code `prover/src/main.rs`
Build
```rust
cargo build --release --bin prover
```

Copy and Rename `prover` to `prover-executable` (As expected by kalypso's enclave image)
```
cp ./target/x86_64-unknown-linux-musl/release/prover prover-executable
```

## Input Verification Executable
Edit code `ivs/src/main.rs`
Build
```rust
cargo build --release --bin prover
```

Copy and Rename `ivs` to `input-verification-executable` (As expected by kalypso's enclave image)
```
cp ./target/x86_64-unknown-linux-musl/release/ivs input-verification-executable
```
