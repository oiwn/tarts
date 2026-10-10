# Project Architecture Overview

## Project Structure

`tarts` (Terminal Arts) is a collection of terminal-based screensavers written in Rust. The project follows a modular architecture with each effect implemented as a separate module.

## Core Architecture

### Main Components

- **main.rs**: Entry point that handles CLI argument parsing and effect selection
- **lib.rs**: Library module exports for all effects
- **common.rs**: Shared utilities and the main animation loop
- **buffer.rs**: Terminal buffer management for efficient rendering
- **config.rs**: Configuration management (currently minimal)
- **error.rs**: Error handling with custom error types

### Effect Modules

Each effect is implemented as a separate module with:
- `effect.rs`: Main effect implementation
- `mod.rs`: Module exports and trait implementations

`terrain` goes further (2026-10-11 refactor): one file per simulation layer — `world.rs` (multi-writer commons), `elevation.rs`, `hydro.rs`, `climate.rs`, `erosion.rs`, `tectonics.rs`, `life.rs`, `sim.rs` (composition root with runtime on/off flags via `TerrainOptions::disabled`), `render.rs` (all views), `effect.rs` (terminal adapter). Sim files are crossterm-free — groundwork for perf work and wasm.

Current effects:
- `matrix` - Matrix digital rain
- `life` - Conway's Game of Life
- `maze` - Maze generation
- `boids` - Boids flocking simulation
- `cube` - 3D cube rotation
- `crab` - ASCII art crab animation
- `donut` - 3D donut rotation
- `pipes` - Pipe maze animation
- `plasma` - Plasma effect
- `fire` - Fire simulation
- `terrain` - Terrain generation
- `butterfly` - Butterflies floating around
- `blank` - Blank screen (testing)

## Technical Architecture

### Dependencies

- **crossterm**: Terminal manipulation and input handling
- **rand**: Random number generation for effects
- **serde**: Configuration serialization
- **pico-args**: Lightweight CLI argument parsing

### Animation Loop

The `common::run_loop()` function provides the main animation engine:
- Manages frame timing and FPS calculation
- Handles terminal buffer updates
- Provides graceful cleanup on exit

### Terminal Management

- Uses alternate screen mode for full-screen effects
- Raw mode for direct terminal control
- Automatic cleanup with `TerminalGuard` RAII pattern

## Build Configuration

- Optimized for minimal binary size with panic="abort" and strip=true
- Link-time optimization enabled
- Profile optimized for size (opt-level = "s")

## Development Workflow

- **Testing**: `cargo test`
- **Benchmarking**: `cargo bench`
- **Formatting**: `rustfmt` with custom configuration
- **CI/CD**: GitHub Actions for testing, linting, and releases

## Gotchas

- **Deps policy**: deps tasks upgrade to latest major — run `cargo upgrade --incompatible`, not just compatible bumps. As of 2026-10-10 all 11 direct deps are at latest major; re-check when new majors land.
- **Release policy**: no `[package] version` bumps in PRs; releases happen by pushing a `v*` tag only. `publish.yml` and `release-on-version-bump.yml` historically fired on main pushes touching Cargo.toml — a contributor's bump auto-created (unpublished) v0.1.26. Fix pending in tag-based-releases task.
- **Degenerate terminal size panics**: crab (~effect.rs:347), matrix, life, boids panic with `cannot sample empty range` when the terminal reports 0 rows/cols (pty, CI, some panes). Butterfly survives via `.max(1.0)` range guards. Fix pending.
- **life `reset()` is `todo!()`** (effect.rs:158): any resize event panics `tarts life` (issue #80, Windows-reported but cross-platform). `pipes` also has three `unreachable!()` arms — unaudited.
- **`--check` mode needs a TTY**: enters the alternate screen, so piping fails with `Device not configured`; not usable in CI as-is. Its match also only knows matrix, life, maze, constellation, butterfly.
- **butterfly_count is `#[serde(skip)]`**: config files can't set it directly; only `butterfly_coeff` scales it.
- **PTY smoke test recipe**: `(sleep 2; printf 'q') | timeout 8 script -q /dev/null target/release/tarts <effect>` — exit 0 = clean quit. `script`'s pty reports size `0 0`, which doubles as a degenerate-size test.
- **Pre-commit clippy gate is `-D warnings`**: plain `cargo clippy` passing is not enough before committing — run `cargo clippy --all-targets -- -D warnings` (0009 was born from this trap).
