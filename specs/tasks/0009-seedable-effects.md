---
id: 0009-seedable-effects
status: draft
scope: [src/rain/digital_rain.rs, src/life/effect.rs, src/maze/effect.rs, src/boids/effect.rs, src/cube/effect.rs, src/crab/effect.rs, src/donut/effect.rs, src/pipes/effect.rs, src/plasma/effect.rs, src/fire/effect.rs, src/constellation/effect.rs, src/butterfly/effect.rs, src/config.rs]
created: 2026-10-12
source: analyzer follow-up (session 2026-10-12): every effect except pocketworld rolls from ThreadRng — --analyze reports for them are statistical, not replayable
---
# Task: seedable effects

## Plan

Every effect except pocketworld rolls from `ThreadRng` — its `--analyze` reports are statistical, not replayable. Make them all deterministic the pocketworld way.

- [ ] Add `seed: u32` to every effect's Options (serde default + builder default); pocketworld already has one — skip it.
- [ ] Replace `ThreadRng`/`rand::rng()` in effect structs with `StdRng::seed_from_u64(options.seed)` stored in the struct (the pocketworld `world_rng` pattern).
- [ ] `reset()` re-rolls the seed from fresh entropy (keep the pocketworld pattern: options.seed = thread roll, re-seed).
- [ ] Config: each effect section gains `seed` (fields already `#[serde(default)]`).
- [ ] Analyzer (`--analyze`): pass `--seed` through to every effect constructor; drop the non-determinism caveat once wired.
- [ ] Verify: same `tarts --analyze <effect> --seed 7` twice → byte-identical reports (run for at least life, boids, butterfly — the ThreadRng users).

## Acceptance

- [ ] `cargo fmt --check`
- [ ] `cargo test --lib`
- [ ] `cargo clippy --all-targets -- -D warnings`

## Log

- 2026-10-12 created: seedable effects
- 2026-10-12 set source analyzer follow-up (session 2026-10-12): every effect except pocketworld rolls from ThreadRng — --analyze reports for them are statistical, not replayable
- 2026-10-12 scope add src/rain/digital_rain.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/life/effect.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/maze/effect.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/boids/effect.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/cube/effect.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/crab/effect.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/donut/effect.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/pipes/effect.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/plasma/effect.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/fire/effect.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/constellation/effect.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/butterfly/effect.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
- 2026-10-12 scope add src/config.rs — add seed to every effect's options; replace ThreadRng with seeded StdRng; config sections gain seed fields
