# Changelog

All notable changes to this project will be documented in this file.

## [0.1.25] - 2026-05-11

### Added
- **Constellation effect** — drifting stars and dotted connections (thanks [@AnarchistHoneybun](https://github.com/AnarchistHoneybun))
- Config system: loads from `~/.config/tarts.toml` (macOS/Linux) or `%APPDATA%/tarts.toml` (Windows)
- `--print-config` flag: prints default config as TOML to stdout (pipe to file)
- Screen-adaptive coefficients: effects scale naturally to terminal size (`drops_coeff`, `speed_coeff`, `cells_coeff`, `boid_coeff`, `crab_coeff`, `k1_coeff`)
- `BoidCharset` enum: Braille (default), Arrow, Simple, Dot — configurable boid character sets
- `src/lib.rs` module-level docstring with effect description table
- `prek` pre-commit checks

### Changed
- Builder defaults aligned with original `DefaultOptions` values: cube size, fire colors, donut projection, pipes density, plasma speed, terrain scale, crab count
- `Config::load()` returns defaults in memory when config file missing (no auto-write)
- Config status printed after effect exits (loaded path or "using defaults")
- Updated dependencies: rand 0.10, toml 1.x, criterion 0.8, tempfile 3.27

### Removed
- `--generate-config` flag (replaced by `--print-config`)

## [0.1.24] - 2025-01-27

### Changed
- **Removed pico-args dependency** - Replaced with manual argument parsing for better control and maintainability
- **Improved CLI experience** - Added comprehensive help system with detailed descriptions for all effects
- **Enhanced argument parsing** - Added support for `--version`, `--help`, and improved `--check` functionality

### Added
- New help system with detailed effect descriptions and usage examples
- Version information via `--version` or `-v` flag
- Better error handling for invalid arguments
- Comprehensive help text for all 12 available effects

### Benefits
- **Zero external dependencies** for argument parsing
- **Smaller binary size**
- **Faster compilation**
- **Full control** over CLI behavior
- **Better user experience** with proper help and error messages

## [0.1.23] - Previous

### Features
- Multiple terminal screensaver effects
- Homebrew tap integration
- Cross-platform support (macOS, Linux)
