# GEMINI.md

## Project Overview

This project, "tarts" (Terminal Arts), is a collection of terminal-based
screensavers written in Rust. It aims to provide visually appealing effects
that can be run directly in the terminal. The project is built with a focus on
performance and memory safety, leveraging Rust's zero-cost abstractions.

The main technologies used are:
- **Rust**: The core programming language.
- **crossterm**: A terminal manipulation library for controlling the terminal's appearance and behavior.
- **rand**: A library for generating random numbers, used in various visual effects.
- **serde**: A framework for serializing and deserializing Rust data structures.

The project is structured as a binary crate with multiple modules, each
representing a different screensaver effect. The main `main.rs` file parses
command-line arguments to determine which effect to run.

## Building and Running

### Building the project

To build the project in release mode, run the following command:

```bash
cargo build --release
```

### Running the screensavers

To run a specific screensaver, use the `tarts` executable with the name of the
effect as an argument. For example:

```bash
# Run the Matrix digital rain effect
tarts matrix

# Run Conway's Game of Life
tarts life

# Run the maze generator
tarts maze
```

You can also run the project directly with `cargo run`:

```bash
cargo run --release -- matrix
```

### Running tests

To run the test suite, use the following command:

```bash
cargo test
```

### Running benchmarks

To run the benchmarks, use the following command:

```bash
cargo bench
```

## Development Conventions

- **Code Style**: The project uses `rustfmt` to maintain a consistent code style. The configuration can be found in the `rustfmt.toml` file.
- **Testing**: The project has a suite of unit and integration tests that can be run with `cargo test`.
- **Benchmarking**: The project uses the `criterion` library for benchmarking. Benchmarks can be run with `cargo bench`.
- **Contributions**: Contributions are welcome. The project encourages pull requests, bug reports, and feature suggestions.

## Session Notes (2025-10-05)

### Current Status: CI Mostly Fixed

- **Nix CI Issues Resolved**: All Nix-related CI failures (`Nix fmt`, `Nix Build`, `Nix Flake Check`) have been fixed
- **Remaining Issue**: Only Code Quality / Clippy job is currently failing
- **PR Status**: PR #67 (Cacafire Effect) is no longer blocked by CI issues

## Session Notes (2025-10-06)

### Homebrew Integration Progress - Phase 1 Complete

- **Release Workflow Updated**: Simplified to build only macOS binaries (x86_64, arm64) for initial Homebrew integration
- **Binary Naming**: Standardized to `tarts-{version}-{target}.tar.gz` format
- **Formula Template Created**: `homebrew-formula-template.rb` ready for tap repository
- **Next Steps**: Create tap repository and test installation

# Current Task: Homebrew Tap Integration

**Goal**: Make `tarts` installable via Homebrew by creating a tap repository and formula

**Requirements**:
- Set up GitHub Actions to build and package `tarts` executables for macOS and Linux on release
- Create Homebrew formula that downloads pre-built binaries
- Establish tap repository for formula distribution

# Plans

- [x] pin minimum rust version

## Homebrew Tap Integration Plan

**Phase 1: Research & Setup** ✅ COMPLETE
- [x] research Homebrew tap creation process and requirements
- [x] study existing Rust project Homebrew formulas for patterns
- [x] determine binary packaging format and naming conventions

**Phase 2: Release Automation** ✅ COMPLETE
- [x] create GitHub Actions workflow to build release binaries
- [x] package for macOS (x86_64, arm64) and Linux (x86_64)
- [x] generate checksums for binary verification

**Phase 3: Formula Creation**
- [ ] write Homebrew formula that downloads pre-built binaries
- [ ] add proper dependencies and installation instructions
- [ ] test formula locally before publishing

**Phase 4: Tap Repository**
- [ ] create dedicated tap repository (e.g., `homebrew-tarts`)
- [ ] set up automated formula updates on new releases
- [ ] document installation process

**Phase 5: Testing & Deployment**
- [ ] test installation from tap on different systems
- [ ] verify binary compatibility and functionality
- [ ] publish tap and update project documentation