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
