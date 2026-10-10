---
id: 0009-clippy-collapsible-match
status: done
scope: [src/common.rs]
created: 2026-10-10
source: pre-commit hook failure during user commit, 2026-10-10
---
# Task: clippy collapsible match

## Plan

- [x] Collapse the nested key-event match in `process_input` (src/common.rs:37) into the outer match pattern, per the clippy hint; semantics unchanged (q/Esc with any modifiers, Ctrl+C)

## Acceptance

- [x] `cargo fmt --check`
- [x] `cargo test --lib` — 38 passed
- [x] `cargo clippy --all-targets -- -D warnings` — clean (pre-commit parity)
- [x] PTY smoke: butterfly still quits on `q` (exit 0) — input path touched

## Summary

Collapsed the nested key-event match in process_input into the outer pattern (clippy collapsible_match); unblocks the pre-commit clippy -D warnings gate.

## Review

Single match expression rewritten in src/common.rs per clippy's own suggestion; arm set identical (q, Esc, Ctrl+C), resize handling untouched. Verified with -D warnings build, tests, and a live quit.

## Log

- 2026-10-10 created: clippy collapsible match
- 2026-10-10 set source pre-commit hook failure during user commit, 2026-10-10
- 2026-10-10 scope add src/common.rs — the flagged match lives here
- 2026-10-10 advance draft → ready
- 2026-10-10 scope approved: src/common.rs
- 2026-10-10 advance ready → in-progress/implement
- 2026-10-10 baseline: .github/workflows/deps-audit.yml@1b7e595c1d64, .github/workflows/opencode.yml@cf8fd4d5b54d, .github/workflows/publish.yml@422abf96dd0e, .github/workflows/release-on-version-bump.yml@deleted, Cargo.lock@533c10b2dc53, README.md@c631d9ee57e8, specdev.toml@2033f8a8ce31, src/boids/effect.rs@c1b85ad7b8e1, src/buffer.rs@e48472b285ae, src/crab/effect.rs@6ae1b669ce9b, src/life/effect.rs@9620bb6fda0f, src/main.rs@9fc2cfaa6950, src/rain/draw.rs@4ce142650de2, src/rain/rain_drop.rs@4be1a8540842, tests/degenerate_size.rs@e3476d7843e3
- 2026-10-10 advance in-progress/implement → in-progress/verify
- 2026-10-10 advance in-progress/verify → in-progress/review
- 2026-10-10 advance in-progress/review → approval
- 2026-10-10 approved: ok committed done 0009
- 2026-10-10 advance approval → done
