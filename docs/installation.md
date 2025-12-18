# Installation

## Prerequisites

- **Rust** 1.70+ (for building from source)
- **Git** (required at runtime for cloning web UIs)

## Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/tbl.git
cd tbl

# Build release binary
cargo build --release

# Binary located at
./target/release/tbl
```

## Static Binary (Linux)

Build a fully static binary using MUSL:

```bash
# Install MUSL target
rustup target add x86_64-unknown-linux-musl

# Build static binary
make static

# Binary located at
./dist/tbl
```

## Platform Support

| Platform              | Status  | Notes                   |
| --------------------- | ------- | ----------------------- |
| Linux x86_64          | ✅ Full | Static binary available |
| macOS (Intel)         | ✅ Full | Dynamic linking         |
| macOS (Apple Silicon) | ✅ Full | Dynamic linking         |
| Windows               | ✅ Full | Dynamic linking         |

## Verifying Installation

```bash
# Check version
tbl --version

# Check help
tbl --help
```

## Git Requirement

tbl requires `git` to be available on PATH for cloning web UIs. If git is missing, tbl will display OS-specific installation instructions:

**macOS:**

```bash
xcode-select --install
# or
brew install git
```

**Windows:**

```
Download from https://git-scm.com/download/win
# or
winget install --id Git.Git -e
```

**Linux:**

```bash
# Debian/Ubuntu
sudo apt-get install git

# Fedora
sudo dnf install git

# Arch Linux
sudo pacman -S git
```
