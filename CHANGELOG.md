# Changelog

All notable changes to this project will be documented in this file.

## [0.1.27] - 2026-10-14

Release summary. Per-task entries below hold the details.

### Added
- **Pocketworld effect** (`tarts pocketworld`, replaces `terrain`): drifting tectonic plates and mountain ranges, rivers and lakes, weather and seasons, groundwater-fed biomes, fire; 8 layer views (`j`/`k`) and a legend page (`l`).
- `--analyze` dev mode: headless render report per effect/view, with `--view`, `--frames`, `--size WxH`, `--dump`.

### Fixed
- `tarts life` no longer panics on resize (#80).
- No panics or hangs on 0x0/1x1 terminals (#94).
- Security: crossbeam-epoch RUSTSEC-2026-0204 via deps update.

### Changed
- Releases are tag-push only (`v*`); CI adds weekly deps audit.
- README: new effects need an issue first.

## 2026-10-14 — revival session (ctx archive)

- Session book closed: deps, CI, tag releases, size/reset fixes, contributing policy, analyzer, pocketworld — tasks 0001–0006, 0008, 0010 (entries below).
- Upstream specdev issues filed: #15 (ctx header not checked), #16 (`task scope rm` undocumented), #17 (no human approve step out of draft), #18 (quality limits vs umbrella tasks), #19 (agents create tasks for trivial fixes; local rule now in AGENTS.md).
- Agent process lessons now live in AGENTS.md Agent Rules (patch tool only; explicit per-task draft approval).

## 2026-10-14 — terrain dynamic world

- Pocketworld: a living pocket world — drifting plates with lossless crust advection and real mountain ranges, a drainage network of rivers and lakes, weather, cryosphere, groundwater-fed forests, taiga, grassland and savanna/desert rings, fire; 8 layer views, legend page, seed 7 boot world, analyzer-verified.
- Task `0006-terrain-dynamic-world`; source: https://github.com/oiwn/tarts/issues/93

## 2026-10-13 — effect analyzer

- `--analyze`: headless one-effect/one-view render report (glyph census + change rate) for every effect; pocketworld's 7 views exposed via new `views()`/`set_view()` trait defaults. Fix rounds added `--dump` (whole final frame as glyphs) and `--size WxH` (canvas override, default 100x40), both from user remarks.
- Task `0010-effect-analyzer`; source: user decision 2026-10-13: numeric-probe tuning missed render flaws (the plateau was found only by measurement); build the analysis instrument first

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
