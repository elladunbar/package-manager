# Homebrew Backend Implementation Plan

## Overview

Add a new homebrew package manager backend to the Rust CLI package manager tool. This backend will follow the same `Manager` trait pattern as the existing pacman and flatpak backends.

## Core Architecture

### Backend Pattern
The homebrew backend implements the `Manager` trait defined in `lib.rs`:
```rust
pub trait Manager: Send + Sync {
    fn name(&self) -> &'static str;
    fn remote_search(&self, query: &str) -> Result<Vec<Package>, Box<dyn Error>>;
}
```

### Package Classification
Homebrew treats different sources as separate repositories:
- **Formulae**: `homebrew/core` (traditional packages)
- **Casks**: `homebrew/cask` (macOS applications)
- **Custom Taps**: `anomalyco/tap`, `homebrew/cask-versions`, etc.

Each tap is treated as its own repository, consistent with pacman and flatpak patterns.

## Search Strategy

### Two-Step Search Process

1. **Search Step**: Execute `brew search "/query/"` using regex syntax
   - User provides: `firefox`
   - Command becomes: `brew search "/firefox/"`
   - Returns list of matching package names with tap prefixes

2. **Info Step**: For each result, execute `brew info --json=v2 <package>`
   - Examples: `brew info --json=v2 firefox`, `brew info --json=v2 anomalyco/tap/opencode`
   - Parses JSON v2 response for detailed package information

### Why Regex Format?
Homebrew's search command uses regex syntax when prefixed with slashes. This ensures proper matching without requiring users to understand regex syntax themselves.

## JSON v2 Structure

### Formulae (from homebrew/core or custom taps)
```json
{
  "name": "mpv",
  "full_name": "mpv",
  "tap": "homebrew/core",
  "desc": "Media player based on MPlayer and mplayer2",
  "versions": {
    "stable": "0.41.0",
    "head": "HEAD"
  },
  "installed": [
    {
      "version": "0.41.0_3",
      "installed_as_dependency": false,
      "installed_on_request": true
    }
  ]
}
```

**Key fields:**
- `name`: Package name (used in `Package.name`)
- `tap`: Repository identifier (used in `Package.repository`)
- `desc`: Description text (used in `Package.description`)
- `versions.stable`: Version string (used in `Package.version`)
- `installed`: Non-empty array indicates installed status

### Casks (from homebrew/cask or custom taps)
```json
{
  "token": "firefox",
  "full_token": "firefox",
  "tap": "homebrew/cask",
  "name": ["Mozilla Firefox"],
  "desc": "Web browser",
  "version": "148.0",
  "installed": "144.0"
}
```

**Key fields:**
- `token`: Package token (used in `Package.name`)
- `tap`: Repository identifier (used in `Package.repository`)
- `desc`: Description text (used in `Package.description`)
- `version`: Version string (used in `Package.version`)
- `installed`: String value indicates installed status

### Tap Packages
```json
{
  "name": "opencode",
  "full_name": "anomalyco/tap/opencode",
  "tap": "anomalyco/tap",
  "desc": "Open source package manager CLI",
  "versions": {
    "stable": "0.1.0"
  },
  "installed": [...]
}
```

**Key insight:** Custom taps are identified by the `tap` field not being `homebrew/core` or `homebrew/cask`.

## Implementation Details

### Files to Create/Modify

#### 1. `src/homebrew/mod.rs` (New File)
```rust
use crate::*;
use serde::Deserialize;
use std::{collections::HashMap, process::Command};

// Structs for JSON v2 parsing
#[derive(Deserialize)]
struct BrewInfoJson {
    formulae: Vec<Formula>,
    casks: Vec<Cask>,
}

#[derive(Deserialize)]
struct Formula {
    name: String,
    tap: String,
    desc: Option<String>,
    versions: Versions,
    installed: Option<Vec<InstalledInfo>>,
}

#[derive(Deserialize)]
struct Cask {
    token: String,
    tap: String,
    desc: Option<String>,
    version: String,
    installed: Option<String>,
}

// ... implementation of Manager trait
```

#### 2. `Cargo.toml` (Modified)
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[features]
default = ["pacman"]
pacman = []
flatpak = []
homebrew = []
```

#### 3. `src/lib.rs` (Modified)
```rust
#[cfg(feature = "homebrew")]
pub mod homebrew;

// ... in init_backends()
let managers: Vec<&'static dyn Manager> = vec![
    #[cfg(feature = "pacman")]
    &pacman::Pacman,
    #[cfg(feature = "flatpak")]
    &flatpak::Flatpak,
    #[cfg(feature = "homebrew")]
    &homebrew::Homebrew,
];
```

### Serde Dependencies
- **serde**: Version 1.x with `derive` feature for trait macros
- **serde_json**: Version 1.x (latest compatible)

### UnmergedPackage Structure
```rust
struct UnmergedPackage {
    repository: String,  // The tap value
    name: String,
    version: String,
    description: Option<String>,
    installed: Option<bool>,
}
```

### Search Algorithm

```rust
fn remote_search(&self, query: &str) -> Result<Vec<Package>, Box<dyn Error>> {
    // Step 1: Search with regex format
    let search_output = Command::new("brew")
        .arg("search")
        .arg(&format!("/{}/", query))
        .output()?;
    
    let results = parse_search_output(String::from_utf8(search_output.stdout)?);
    
    // Step 2: Get detailed info for each result
    let mut unmerged_packages = Vec::new();
    for package_name in results {
        let info_output = Command::new("brew")
            .arg("info")
            .arg("--json=v2")
            .arg(&package_name)
            .output()?;
        
        let json: BrewInfoJson = serde_json::from_str(
            &String::from_utf8(info_output.stdout)?
        )?;
        
        // Parse formulae
        for formula in json.formulae {
            unmerged_packages.push(UnmergedPackage {
                repository: formula.tap,
                name: formula.name,
                version: get_version_from_versions(&formula.versions),
                description: formula.desc,
                installed: Some(formula.installed.is_some_and(|i| !i.is_empty())),
            });
        }
        
        // Parse casks
        for cask in json.casks {
            unmerged_packages.push(UnmergedPackage {
                repository: cask.tap,
                name: cask.token,
                version: cask.version,
                description: cask.desc,
                installed: Some(cask.installed.is_some()),
            });
        }
    }
    
    Ok(merge_packages(unmerged_packages))
}
```

### Merge Strategy
- Group packages by `name` field
- If the same package appears in multiple taps (rare but possible), merge into single entry
- Collect all repository/tap names into the `repository` vector
- Use first formula's/cask's version and description
- Mark as installed if any variant is installed

### Version Extraction

**Formulae:**
```rust
fn get_version_from_versions(versions: &Versions) -> String {
    versions.stable
        .or_else(|| versions.head.clone())
        .unwrap_or_else(|| "unknown".to_string())
}
```

**Casks:**
```rust
// Direct from cask.version field
let version = cask.version;
```

### Installed Status Detection

**Formulae:**
```rust
let installed = formula.installed
    .map(|arr| !arr.is_empty())
    .or(None);
```

**Casks:**
```rust
let installed = cask.installed
    .map(|_| true)
    .or(None);
```

## Testing Strategy

### Test Packages Available
- **Firefox**: Installed cask (`brew cask info firefox`)
- **mpv**: Installed formula (`brew info mpv`)
- **opencode**: Installed via custom tap (`anomalyco/tap/opencode`)

### Test Commands
```bash
# Search all packages
brew search "firefox"
brew search "/firefox/"

# Get JSON info for specific package
brew info --json=v2 firefox
brew info --json=v2 mpv
brew info --json=v2 anomalyco/tap/opencode

# Verify installed status
brew list --cellar  # Shows installed formulae
brew list --casks   # Shows installed casks
```

### Test Cases
1. **Core formula search**: Search for `mpv`, verify `tap: "homebrew/core"`
2. **Cask search**: Search for `firefox`, verify `tap: "homebrew/cask"`
3. **Tap package search**: Search for `opencode`, verify `tap: "anomalyco/tap"`
4. **Installed detection**: Verify installed status is correctly set for test packages
5. **Multi-tap scenario**: If same package exists in multiple taps, verify merge behavior

## Edge Cases and Error Handling

### No Results
- Empty search results return empty `Vec<Package>`
- Handle gracefully without errors

### Invalid JSON
- Wrap serde deserialization in error handling
- Return descriptive error if JSON parsing fails

### Network/Command Errors
- Propagate Command errors from brew execution
- Handle non-zero exit codes gracefully

### Missing Fields
- Use `Option<T>` fields with `unwrap_or_default()` for missing optional data
- Provide sensible defaults (e.g., "unknown" for version)

## Performance Considerations

### Optimizations
1. **Batch info requests**: Could potentially parallelize multiple `brew info` calls
2. **Caching**: Could cache search results to avoid repeated `brew search` calls
3. **Lazy JSON parsing**: Parse JSON only for packages that match filters

### Current Implementation
- Sequential processing (one `brew info` per search result)
- No caching
- Simple and maintainable

## Feature Flag Usage

### Build Time
```bash
# Build with homebrew support
cargo build --features homebrew

# Build without homebrew (default)
cargo build
```

### Runtime
```bash
# List available backends
cargo run -- --list-backends

# Search using homebrew (only works if compiled with --features homebrew)
cargo run --features homebrew -- firefox
```

## File Structure

```
src/
├── lib.rs              # Updated: Add homebrew module and feature flag
├── main.rs             # No changes needed (will work with new backend)
├── pacman/
│   └── mod.rs          # Existing implementation
├── flatpak/
│   └── mod.rs          # Existing implementation
└── homebrew/
    └── mod.rs          # NEW: Homebrew implementation
```

## Dependencies Summary

| Dependency | Version | Features | Purpose |
|------------|---------|----------|---------|
| serde | 1.x | derive | Struct deserialization from JSON |
| serde_json | 1.x | default | JSON parsing |

Both dependencies are minimal and widely used Rust crates with no known security issues.

## Migration Notes

### No Breaking Changes
- Existing pacman and flatpak backends remain unchanged
- Feature flags preserve backward compatibility
- Default build only includes pacman (as before)

### Optional Backend
- Homebrew is opt-in via feature flag
- Users who don't need homebrew can build without it to reduce binary size
- Runtime backend discovery (`init_backends()`) automatically handles disabled features

---

**Author**: AI Assistant (based on user requirements)  
**Date**: 2026-03-02  
**Status**: Ready for implementation