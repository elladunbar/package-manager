# package-manager

A unified CLI front end for multiple package managers, currently supporting Arch Linux's pacman with extensibility for additional backends like Flatpak.

## Features

- Unified search interface across multiple package managers
- Colorized output with package metadata (version, repositories, description)
- Installation status detection
- Easy extensibility via the `Manager` trait

## Installation

```bash
cargo install --path .
```

Or build locally:

```bash
cargo build --release
```

## Usage

```
Usage: package-manager [OPTIONS] <search-query>

A unified CLI for searching packages across multiple package managers.

Arguments:
  <search-query>    The package name or search term

Options:
  --backend=<NAME>  Specify which backend(s) to use (can be repeated)
                    Available: pacman, flatpak
  --help            Show this help message

Examples:
  package-manager firefox
  package-manager --backend=pacman firefox
  package-manager --backend=pacman --backend=flatpak firefox
```

## Supported Backends

- **pacman** (Arch Linux) - enabled by default
- **flatpak** - available via feature flag `--features flatpak`

## Architecture

### Package Structure

```
src/
├── main.rs          # Entry point - CLI argument handling, search invocation
├── lib.rs           # Core abstractions - Package struct, Manager trait
└── pacman/
    └── mod.rs       # Pacman implementation
```

### Core Components

- **`Package` struct**: Contains repository, name, version, description, and installation status
- **`Manager` trait**: Defines the interface for package manager backends (`name()` and `remote_search()`)
- **Registry pattern**: Backends are registered at startup and resolved by name

### Adding New Backends

1. Create a new module (e.g., `flatpak/mod.rs`)
2. Implement the `Manager` trait:

```rust
pub struct Flatpak;

impl Manager for Flatpak {
    fn name(&self) -> &'static str {
        "flatpak"
    }

    fn remote_search(&self, query: &str) -> Result<Vec<Package>, Box<dyn Error>> {
        // Implementation
    }
}
```

3. Add to `lib.rs`:
   - Add `#[cfg(feature = "flatpak")] pub mod flatpak;`
   - Add `&flatpak::Flatpak` to the registry in `init_backends()`
4. Add feature to `Cargo.toml`

## Development

```bash
# Run tests
cargo test

# Check clippy
cargo clippy

# Format code
cargo fmt

# Run with debug output
cargo run -- firefox
```

## Example Output

```
[ Pacman ]
  firefox 120.0 [installed] extra / core
    High-performance, low-resource web browser
```
