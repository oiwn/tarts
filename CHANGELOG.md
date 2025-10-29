# Changelog

All notable changes to this project will be documented in this file.

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

### Effects Available
- matrix, life, maze, boids, cube, crab, donut, pipes, plasma, fire, blank, terrain

## [0.1.23] - Previous

### Features
- Multiple terminal screensaver effects
- Homebrew tap integration
- Cross-platform support (macOS, Linux)