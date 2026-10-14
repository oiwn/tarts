---
id: 0004-fix-degenerate-size-panic
status: done
scope: [src/main.rs, src/common.rs, src/crab/effect.rs, tests/degenerate_size.rs, src/buffer.rs, src/rain/rain_drop.rs, src/rain/draw.rs, src/life/effect.rs, src/boids/effect.rs]
created: 2026-10-09
source: https://github.com/oiwn/tarts/issues/94
---
# Task: fix degenerate size panic

## Plan

- [x] Clamp terminal size to at least (1, 1) at the entry points: main.rs where `(width, height)` is first read, and the resize handler in `common.rs` `process_input`
- [x] Add range guards in crab's `Crab::new` sampling (butterfly-style `.max(...)`) as belt-and-braces for library users constructing effects directly
- [x] Regression test: construct each affected effect at (0, 0) / (1, 1) without panic

## Acceptance

- [x] `cargo fmt --check`
- [x] `cargo test --lib` — 37 passed; full `cargo test` adds the 2 new integration tests, all green
- [x] `cargo clippy` — `--all-targets` clean except the pre-existing common.rs collapsible_match
- [x] PTY smoke (0x0 pty) for crab, matrix, life, boids — all exit 0, zero panics (crab needed a second rain_drop fix first; matrix initially still panicked at rain_drop.rs:155)

## Findings

- The bug class was wider than the issue says: besides empty `random_range`s (crab, life new/update, boids, rain_drop new/reset), there was a **hang** — crab's spread-crabs-apart retry loop has no bail-out, so tiny screens spin forever in `Crab::new`; now bounded at 100 attempts per crab.
- `Buffer::new` debug-asserted on 0x0 and `Buffer::set` indexed OOB; Buffer now allows empty and clips out-of-bounds writes (render-buffer semantics).
- boids divides by screen width/height (modulo) in get_diff — guarded.
- Scope grew from 3 files to 9 (buffer, rain_drop, draw, life, boids, integration test): the test-first loop exposed each site; every addition was scoped before touching.

## Summary

Effects no longer panic or hang on degenerate terminal sizes: entry-point clamps (main.rs, common.rs resize), bounded crab spawn retries, per-effect range guards, clipping Buffer writes, plus regression tests at 0x0/1x1 (issue #94).

## Review

Diff vs plan: plan named 3 fix sites; findings above document why scope grew (test-driven discovery, all scoped pre-edit). Entry-point clamps protect the binary; per-effect guards protect library consumers; Buffer clipping is the backstop. Behavior at normal sizes unchanged (full suite + PTY smokes pass). No new clippy warnings from this task's code.

## Log

- 2026-10-09 created: fix degenerate size panic
- 2026-10-09 set source https://github.com/oiwn/tarts/issues/94
- 2026-10-09 scope add src/main.rs — size clamps at entry points, crab constructor guards, lib test module
- 2026-10-09 scope add src/common.rs — size clamps at entry points, crab constructor guards, lib test module
- 2026-10-09 scope add src/crab/effect.rs — size clamps at entry points, crab constructor guards, lib test module
- 2026-10-09 scope add src/lib.rs — size clamps at entry points, crab constructor guards, lib test module
- 2026-10-09 advance draft → ready
- 2026-10-09 scope approved: src/main.rs, src/common.rs, src/crab/effect.rs, src/lib.rs
- 2026-10-09 advance ready → in-progress/implement
- 2026-10-09 baseline: .github/workflows/deps-audit.yml@1b7e595c1d64, .github/workflows/opencode.yml@cf8fd4d5b54d, .github/workflows/publish.yml@422abf96dd0e, .github/workflows/release-on-version-bump.yml@deleted, Cargo.lock@533c10b2dc53, specdev.toml@2033f8a8ce31
- 2026-10-09 scope add tests/degenerate_size.rs — integration test constructing each affected effect at degenerate sizes
- 2026-10-09 scope add src/buffer.rs — Buffer::new debug_asserts on 0x0 and set() panics OOB (class-level fix); rain drop samples empty ranges at small heights
- 2026-10-09 scope add src/rain/rain_drop.rs — Buffer::new debug_asserts on 0x0 and set() panics OOB (class-level fix); rain drop samples empty ranges at small heights
- 2026-10-09 scope add src/rain/draw.rs — gradient row indexed OOB at tiny sizes; life samples empty position ranges at 0x0
- 2026-10-09 scope add src/life/effect.rs — gradient row indexed OOB at tiny sizes; life samples empty position ranges at 0x0
- 2026-10-09 scope add src/boids/effect.rs — boids samples empty position ranges at 0x0
- 2026-10-09 advance in-progress/implement → in-progress/verify
- 2026-10-09 advance in-progress/verify → in-progress/review
- 2026-10-09 advance in-progress/review → approval
- 2026-10-09 scope rm src/lib.rs
- 2026-10-10 approved: hmm nice, done 0004
- 2026-10-10 advance approval → done
