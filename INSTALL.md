# Installing FLUID RUST

FLUID RUST is a verified systems language combining ownership-based memory safety with algebraic effect handlers and formal proof verification.

## System Requirements

- **OS:** Linux (x86-64, ARM64), macOS (x86-64, ARM64), Windows (x86-64)
- **Rust:** 1.70 or later
- **RAM:** 4GB minimum (8GB recommended)
- **Disk:** 2GB for build artifacts

## Installation Methods

### Method 1: Cargo (Recommended)

```bash
cargo install fluid-rust-compiler fluid-rust-runtime fluid-rust-prover
```

### Method 2: From Source

```bash
git clone https://github.com/SNAPKITTYWEST/fluid-rust.git
cd fluid-rust
cargo build --release
cargo install --path compiler
cargo install --path prover
cargo install --path runtime
```

### Method 3: GitHub Releases

Download pre-built binaries from https://github.com/SNAPKITTYWEST/fluid-rust/releases/tag/v1.0.0

```bash
tar xzf fluid-rust-1.0.0-x86_64-linux.tar.gz
sudo mv fluid-rust/* /usr/local/bin/
```

### Method 4: Docker

```bash
docker pull snapkittywest/fluid-rust:1.0.0
docker run -it snapkittywest/fluid-rust:1.0.0 fluid-rust --version
```

## Quick Start

### Compile a Simple Program

```bash
cat > hello.rs << 'HELLO'
fn main() {
    println!("Hello, FLUID RUST!");
}
HELLO

fluid-rust-compiler hello.rs -o hello
./hello
```

### Verify Proof

```bash
fluid-rust-prover hello.rmir --verify
```

## Troubleshooting

**"command not found: fluid-rust-compiler"**
- Verify cargo bin directory is in PATH: `echo $PATH | grep ~/.cargo/bin`
- Add if missing: `export PATH="$HOME/.cargo/bin:$PATH"`

**Docker permission denied**
- Add user to docker group: `sudo usermod -aG docker $USER`

## Support

- GitHub Issues: https://github.com/SNAPKITTYWEST/fluid-rust/issues
- Documentation: https://github.com/SNAPKITTYWEST/fluid-rust#documentation
