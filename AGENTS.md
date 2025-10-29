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
tarts terrain     # Terrain generation

# Development
cargo test        # Run tests
cargo bench       # Run benchmarks

# Code Quality (run after changes)
cargo fmt --check  # Check formatting
cargo test --lib   # Run library tests
cargo clippy       # Run linter
```

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