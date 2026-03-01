# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust CLI tool providing a unified front end for multiple package managers. The project is version 0.1.0, uses Rust 2024 edition, and has zero external dependencies.

**Supported backends:**
- **pacman** (Arch Linux) - currently implemented and functional
- Additional package manager backends can be added via the `Manager` trait

## Key Commands

**Build:**
```bash
cargo build
```

**Run:**
```bash
cargo run -- <search-query>
```
Example: `cargo run -- firefox`

**Run tests:**
```bash
cargo test
```

**Check clippy:**
```bash
cargo clippy
```

**Format:**
```bash
cargo fmt
```

## Code Architecture

### Structure

```
src/
├── main.rs          # Entry point - CLI argument handling, search invocation, output
├── lib.rs           # Core abstractions - Package struct, Manager trait, Display impl
└── pacman/
    └── mod.rs       # Pacman implementation of Manager trait
```

### Core Components

**`Package` struct (lib.rs:5-11)**
- Fields: `repository`, `name`, `version`, `description`, `installed`
- Implements `Display` with colorized output using ANSI escape codes

**`Manager` trait (lib.rs:39-41)**
- Defines the interface: `fn remote_search(query: &str) -> Result<Vec<Package>, Box<dyn Error>>`
- Enables multiple package manager implementations (Strategy pattern)

**`Pacman` implementation (pacman/mod.rs)**
- Executes `pacman -Ss <query>` via `std::process::Command`
- Parses two-line output format (package info + description)
- Merges multi-repository packages using `merge_packages()` function
- Handles version string cleanup (strips trailing `.1`)

### Key Patterns

1. **Strategy Pattern**: `Manager` trait allows extensible package manager implementations
2. **Parser/Aggregator**: Pacman output is parsed into `UnmergedPackage`, then merged by name
3. **Zero Dependencies**: Pure standard library usage

## Development Notes

- The codebase is minimal (~60 lines total, no tests currently)
- No external crates are used - only `std` library
- Output is colorized with ANSI codes: bold (name), green (version), cyan (repositories), magenta (end)
- Multi-repository packages are merged into single entries with repository list
