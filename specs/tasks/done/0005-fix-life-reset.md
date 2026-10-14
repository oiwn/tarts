---
id: 0005-fix-life-reset
status: done
scope: [src/life/effect.rs]
created: 2026-10-09
source: https://github.com/oiwn/tarts/issues/80
---
# Task: fix life reset

## Plan

- [x] Implement `ConwayLife::reset()` (currently `todo!()` at src/life/effect.rs:158): rebuild the cell field with the same initialization logic as `new()` (random cells from `initial_cells`), clear generation state, keep options and screen size
- [x] Unit test: `reset()` twice in a row leaves a populated, non-panicking simulation at small sizes
- [x] Draft the #80 reply (in ctx.md) pointing at the fix

## Acceptance

- [x] `cargo fmt --check`
- [x] `cargo test --lib` — 38 passed (1 new reset test)
- [x] `cargo clippy` — only the pre-existing common.rs warning
- [x] PTY resize smoke — agent: scripted SIGWINCH via `script` is unreliable; the exact resize path (`update_size` → `reset` → `update` → `get_diff`, including degenerate sizes) is exercised by `reset_rebuilds_a_populated_simulation`; plain life PTY run exits 0

## Findings

- `reset()` is one line: `*self = Self::new(self.options.clone(), self.screen_size)` — same pattern as butterfly/crab. Nothing else on the resize path needs changes.

## Summary

Implemented life's todo!() reset() (issue #80): resize no longer panics the effect; rebuild-from-scratch with a regression test covering the resize path.

## Review

Single-line fix plus a focused test module; matches plan and scope exactly. options is Clone, screen_size preserved, generation reset to 0, cells re-seeded. The ctx.md #80 draft already points at the incoming fix PR.

## Log

- 2026-10-09 created: fix life reset
- 2026-10-09 set source https://github.com/oiwn/tarts/issues/80
- 2026-10-09 scope add src/life/effect.rs — reset() implementation lives here
- 2026-10-09 advance draft → ready
- 2026-10-09 scope approved: src/life/effect.rs
- 2026-10-09 advance ready → in-progress/implement
- 2026-10-09 baseline: .github/workflows/deps-audit.yml@1b7e595c1d64, .github/workflows/opencode.yml@cf8fd4d5b54d, .github/workflows/publish.yml@422abf96dd0e, .github/workflows/release-on-version-bump.yml@deleted, Cargo.lock@533c10b2dc53, specdev.toml@2033f8a8ce31, src/boids/effect.rs@c1b85ad7b8e1, src/buffer.rs@e48472b285ae, src/common.rs@6104efd17683, src/crab/effect.rs@6ae1b669ce9b, src/main.rs@9fc2cfaa6950, src/rain/draw.rs@4ce142650de2, src/rain/rain_drop.rs@4be1a8540842, tests/degenerate_size.rs@e3476d7843e3
- 2026-10-09 advance in-progress/implement → in-progress/verify
- 2026-10-09 advance in-progress/verify → in-progress/review
- 2026-10-09 advance in-progress/review → approval
- 2026-10-10 approved: done 0005
- 2026-10-10 advance approval → done
