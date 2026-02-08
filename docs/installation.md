# Installation

There are several ways to install Graft depending on your environment.

## Using Cargo

If you have Rust installed, you can install Graft directly from the source:

```bash
cargo install --path .
```

## Using Nix

Graft provides a Flake for reproducible builds.

### Run directly
```bash
nix run github:Nymphium/graft -- --help
```

### Install to profile
```bash
nix profile install github:Nymphium/graft
```

## Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/Nymphium/graft.git
   cd graft
   ```

2. Build using Cargo:
   ```bash
   cargo build --release
   ```
   The binary will be located at `target/release/graft`.
