---
id: 0010-effect-analyzer
status: done
scope: [src/analyze.rs, src/common.rs, src/pocketworld/effect.rs, src/main.rs, src/lib.rs, README.md, AGENTS.md]
created: 2026-10-13
source: user decision 2026-10-13: numeric-probe tuning missed render flaws (the plateau was found only by measurement); build the analysis instrument first
---
# Task: effect analyzer

## Plan

Headless instrument: run ONE effect, ONE view, N frames; report the final-frame glyph census and per-sample change rate. Deterministic seeds (pocketworld today, every effect via 0009) make any other view a re-run away — no batch mode. Report stays minimal: census + change rate, plus `--dump` for the whole final frame; no latitude/color stats, no batch flags.

- [x] `TerminalEffect`: default `views()` returning `["default"]`, no-op `set_view()` — layer-less effects ignore the concept.
- [x] Pocketworld override: view name → renderer Layer (the programmatic face of `j`/`k`).
- [x] `src/analyze.rs`: construct → `set_view` → N frames → glyph census (count/share/distinct) + change rate between evenly spaced samples; non-seeded effects print a one-line statistical note until 0009.
- [x] CLI: `tarts --analyze <effect> [--frames N] [--seed S] [--view NAME]`.
- [x] README + AGENTS: one line each.
- [x] First runs: `--analyze pocketworld --view height --frames 3000` (plateau), then per-flaw single-view runs as required — written diagnosis before 0006 resumes.
- [x] `--dump`: print the whole final frame (glyphs only, one row per line) between the header and the census; stats stay.
- [x] `--size WxH`: override the analysis canvas (default 100x40, bounds 1..=1000 per dimension); malformed specs error loudly. Options now grouped in `AnalysisOptions`.

## Acceptance

- [x] `cargo fmt --check`
- [x] `cargo test --lib`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `tarts --analyze pocketworld --view height --frames 300` runs headless, exit 0, produces a census report
- [x] `tarts --analyze life --frames 300` runs (generic single-view path)
- [x] `tarts --analyze pocketworld --view height --frames 30 --dump` prints the full 100x40 frame, stats still present
- [x] `tarts --analyze pocketworld --view biome --frames 20 --size 60x20` reports `canvas: 60x20` and `CENSUS total=1200`; `--size 160` errors with exit 1

## Summary

`--analyze`: headless one-effect/one-view render report (glyph census + change rate) for every effect; pocketworld's 7 views exposed via new `views()`/`set_view()` trait defaults. Fix rounds added `--dump` (whole final frame as glyphs) and `--size WxH` (canvas override, default 100x40), both from user remarks.

## Review

- Scope held: the 7 scoped files, nothing else; the trait change is default methods only, no existing effect touched.
- Report kept minimal per remarks (census + change rate); non-seeded effects carry the 0009 note.
- The analyzer runs default options and ignores the user config on purpose — remember this when comparing analyzer numbers against focus-mode runs.
- All acceptance commands run green; the first diagnosis battery is recorded in 0006 (2026-10-13 baseline).

## Log

- 2026-10-13 created: effect analyzer
- 2026-10-13 set source user decision 2026-10-13: numeric-probe tuning missed render flaws (the plateau was found only by measurement); build the analysis instrument first
- 2026-10-13 scope add src/analyze.rs — headless analysis facility: views()/set_view() trait defaults + pocketworld override, analyzer module, CLI wiring, docs
- 2026-10-13 scope add src/common.rs — headless analysis facility: views()/set_view() trait defaults + pocketworld override, analyzer module, CLI wiring, docs
- 2026-10-13 scope add src/pocketworld/effect.rs — headless analysis facility: views()/set_view() trait defaults + pocketworld override, analyzer module, CLI wiring, docs
- 2026-10-13 scope add src/main.rs — headless analysis facility: views()/set_view() trait defaults + pocketworld override, analyzer module, CLI wiring, docs
- 2026-10-13 scope add src/lib.rs — headless analysis facility: views()/set_view() trait defaults + pocketworld override, analyzer module, CLI wiring, docs
- 2026-10-13 scope add README.md — headless analysis facility: views()/set_view() trait defaults + pocketworld override, analyzer module, CLI wiring, docs
- 2026-10-13 scope add AGENTS.md — headless analysis facility: views()/set_view() trait defaults + pocketworld override, analyzer module, CLI wiring, docs
- 2026-10-13 set source user decision 2026-10-13: numeric-probe tuning missed render flaws (the plateau was found only by measurement); build the analysis instrument first
- 2026-10-13 advance draft → ready
- 2026-10-13 scope approved: src/analyze.rs, src/common.rs, src/pocketworld/effect.rs, src/main.rs, src/lib.rs, README.md, AGENTS.md
- 2026-10-13 advance ready → in-progress/implement
- 2026-10-13 baseline: src/pocketworld/climate.rs@c46bd594078e, src/pocketworld/elevation.rs@88d9479339ee, src/pocketworld/erosion.rs@24469ef24705, src/pocketworld/hydro.rs@ca803eb75f0b, src/pocketworld/render.rs@671435f3af83, src/pocketworld/sim.rs@305a4a2707ee, src/pocketworld/tectonics.rs@b0f0e411dd40, src/pocketworld/world.rs@12e70f8b1009
- 2026-10-13 advance in-progress/implement → in-progress/verify
- 2026-10-13 advance in-progress/verify → in-progress/review
- 2026-10-13 advance in-progress/review → approval
- 2026-10-13 advance approval → in-progress/fix (revision)
- 2026-10-13 baseline: src/pocketworld/climate.rs@c46bd594078e, src/pocketworld/elevation.rs@88d9479339ee, src/pocketworld/erosion.rs@24469ef24705, src/pocketworld/hydro.rs@ca803eb75f0b, src/pocketworld/render.rs@671435f3af83, src/pocketworld/sim.rs@305a4a2707ee, src/pocketworld/tectonics.rs@b0f0e411dd40, src/pocketworld/world.rs@12e70f8b1009
- 2026-10-13 advance in-progress/fix → in-progress/verify
- 2026-10-13 advance in-progress/verify → in-progress/review
- 2026-10-13 advance in-progress/review → approval
- 2026-10-13 advance approval → in-progress/fix (revision)
- 2026-10-13 baseline: src/pocketworld/climate.rs@c46bd594078e, src/pocketworld/elevation.rs@88d9479339ee, src/pocketworld/erosion.rs@24469ef24705, src/pocketworld/hydro.rs@ca803eb75f0b, src/pocketworld/render.rs@671435f3af83, src/pocketworld/sim.rs@305a4a2707ee, src/pocketworld/tectonics.rs@b0f0e411dd40, src/pocketworld/world.rs@12e70f8b1009
- 2026-10-13 advance in-progress/fix → in-progress/verify
- 2026-10-13 advance in-progress/verify → in-progress/review
- 2026-10-13 advance in-progress/review → approval
- 2026-10-13 approved: Looks good! i think we done with this task compress it and mark as done
- 2026-10-13 advance approval → done
