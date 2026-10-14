# Project Architecture Overview

## Project Structure

`tarts` (Terminal Arts) is a collection of terminal-based screensavers written in Rust. The project follows a modular architecture with each effect implemented as a separate module.

## Core Architecture

### Main Components

- **main.rs**: Entry point that handles CLI argument parsing and effect selection
- **lib.rs**: Library module exports for all effects
- **common.rs**: Shared utilities and the main animation loop
- **buffer.rs**: Terminal buffer management for efficient rendering
- **config.rs**: Loads `~/.config/tarts.toml` (`%APPDATA%\tarts.toml` on Windows); one `#[serde(default)]` section per effect, so partial files work
- **analyze.rs**: Headless analyzer (`--analyze <effect> [--view V] [--frames N] [--size WxH] [--dump]`): glyph census + change rate for one effect/view; ignores the user config for reproducibility. Effects expose views via `views()`/`set_view()` trait defaults
- **check.rs**: `--check` mode (see Gotchas)
- **error.rs**: Error handling with custom error types

### Effect Modules

Each effect is implemented as a separate module with:
- `effect.rs`: Main effect implementation
- `mod.rs`: Module exports and trait implementations

The matrix effect lives in `rain/` (`rain::digital_rain`), not `matrix/`.

`pocketworld` goes further (2026-10-11 refactor, renamed from `terrain`): one file per simulation layer — `world.rs` (multi-writer commons), `elevation.rs`, `hydro.rs`, `climate.rs`, `erosion.rs`, `tectonics.rs`, `life.rs`, `sim.rs` (composition root with runtime on/off flags via `PocketWorldOptions::disabled`), `render.rs` (all views), `effect.rs` (terminal adapter). Sim files are crossterm-free — groundwork for perf work and wasm. Only generation params (`PocketWorldOptions`: seed, noise scales, warp, disabled layers) come from config; ~95 sim tuning values are `const`s in the layer files.

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
- `constellation` - Drifting, twinkling stars joined by dotted lines
- `pocketworld` - Living pocket world (renamed from `terrain`, 2026-10-11): rivers, weather, seasons, plates, life
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

Dev commands and agent rules: `AGENTS.md`.

- **Pre-commit**: `prek.toml` (fmt, clippy `-D warnings`, tests, typos, gitleaks)
- **CI/CD**: GitHub Actions — tests/lint, weekly `deps-audit.yml`, `opencode.yml`, tag-driven `release.yml` + `publish.yml`

## Gotchas

- **Deps policy**: deps tasks upgrade to latest major — run `cargo upgrade --incompatible`, not just compatible bumps. As of 2026-10-10 all 11 direct deps are at latest major; re-check when new majors land.
- **Release policy**: no `[package] version` bumps in PRs; releases happen by pushing a `v*` tag only (`specs/release.md`). `publish.yml` guards tag ↔ `Cargo.toml` version.
- **Degenerate terminal sizes**: effects must survive 0x0/1x1 (pty, CI, some panes) — guard every `gen_range` on screen dims; regression tests exist per effect.
- **`pipes` has three `unreachable!()` arms** — unaudited.
- **`--check` mode needs a TTY**: enters the alternate screen, so piping fails with `Device not configured`; not usable in CI as-is. Its match also only knows matrix, life, maze, constellation, butterfly.
- **butterfly_count is `#[serde(skip)]`**: config files can't set it directly; only `butterfly_coeff` scales it.
