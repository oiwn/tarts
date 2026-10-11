# Changelog

All notable changes to this project will be documented in this file.

## 2026-10-10 — fix life reset

- Implemented life's todo!() reset() (issue #80): resize no longer panics the effect; rebuild-from-scratch with a regression test covering the resize path.
- Task `0005-fix-life-reset`; source: https://github.com/oiwn/tarts/issues/80

## 2026-10-10 — readme contributing policy

- README Contributing section now states: bug fixes and improvements welcome; new effects require opening an issue for discussion first.
- Task `0008-readme-contributing-policy`; source: user request during 0004 review

## 2026-10-10 — fix degenerate size panic

- Effects no longer panic or hang on degenerate terminal sizes: entry-point clamps (main.rs, common.rs resize), bounded crab spawn retries, per-effect range guards, clipping Buffer writes, plus regression tests at 0x0/1x1 (issue #94).
- Task `0004-fix-degenerate-size-panic`; source: https://github.com/oiwn/tarts/issues/94

## 2026-10-10 — tag based releases

- Releases are tag-driven only: deleted release-on-version-bump.yml, reworked publish.yml onto v* tags with a tag↔manifest version guard, documented the push-a-tag flow in specs/release.md.
- Task `0003-tag-based-releases`

## 2026-10-10 — ci workflows

- Two new workflows from exodata: deps-audit.yml (weekly + manifest-change cargo-audit) and opencode.yml (/oc inference + todo-to-issues), with the /oc job additionally gated to github.actor == 'oiwn'.
- Task `0002-ci-workflows`

## 2026-10-10 — deps update

- Lockfile refresh: crossbeam-epoch RUSTSEC-2026-0204 fixed (0.9.21), chacha20 past yanked version, anyhow dropped; 19 fewer dep entries; all checks and smoke pass.
- Task `0001-deps-update`

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
