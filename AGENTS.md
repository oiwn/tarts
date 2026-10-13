# Project Overview

## Quick Reference

`tarts` (Terminal Arts) - A collection of terminal-based screensavers written in Rust.

### Main Commands
```bash
# Build
cargo build --release

# Run effects
tarts matrix      # Matrix digital rain
tarts life        # Conway's Game of Life  
tarts maze        # Maze generation
tarts boids       # Boids flocking simulation
tarts cube        # 3D cube rotation
tarts crab        # ASCII crab animation
tarts donut       # 3D donut rotation
tarts pipes       # Pipe maze animation
tarts plasma      # Plasma effect
tarts fire        # Fire simulation
tarts pocketworld # Living pocket world: rivers, rain, plates, seasons

# Development
cargo test        # Run tests
cargo bench       # Run benchmarks

# Headless render analysis (dev)
tarts --analyze pocketworld --view height --frames 3000

# Code Quality (run after changes)
cargo fmt --check  # Check formatting
cargo test --lib   # Run library tests
cargo clippy       # Run linter
```

### Agent Rules

- Edit files only with a proper patch tool (e.g. `apply_patch` or an equivalent structured edit). Never rewrite files via ad-hoc scripts (python/sed one-liners) — silent partial rewrites are how bugs get in.
- The pre-commit clippy gate is `-D warnings`: run `cargo clippy --all-targets -- -D warnings` before handing work back.

### Installation Methods
- **Homebrew**: `brew install oiwn/tap/tarts`
- **Cargo**: `cargo install tarts`
- **Manual**: Download from GitHub releases

## Project Status
- **Version**: 0.1.23
- **Effects**: 12 working screensavers
- **Platforms**: macOS (x86_64, arm64), Linux
- **Homebrew**: Tap available and working

## Current Focus
Preparing for public release and Reddit announcement. See `specs/current_task.md` for detailed release preparation plan.

## Architecture
See `specs/overview.md` for detailed project architecture and technical overview.

<!-- BEGIN specdev -->
## specdev

This project uses **specdev** (specification-driven development). Load the
specdev skill when starting a session, continuing from specs, or picking up a
task. Always read `specs/overview.md` and `specs/ctx.md` before coding.
<!-- END specdev -->

