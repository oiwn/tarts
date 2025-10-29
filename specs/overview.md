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