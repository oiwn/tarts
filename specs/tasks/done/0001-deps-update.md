---
id: 0001-deps-update
status: done
scope: [Cargo.toml, Cargo.lock]
created: 2026-10-09
---
# Task: deps update

## Plan

- [x] `cargo upgrade` (11 known upgradable manifests; review the diff — no `[package] version` change allowed)
- [x] `cargo update` to refresh the lockfile and pull crossbeam-epoch to >=0.9.20 (RUSTSEC-2026-0204)
- [x] Read changelels of anything semver-incompatible or risky (rand line, crossterm); roll back a bump that breaks behavior
- [x] Re-run the full acceptance set

## Acceptance

- [x] `cargo fmt --check`
- [x] `cargo test --lib` — 37 passed
- [x] `cargo clippy` — 1 pre-existing warning (collapsible_match, PR #90 code) routed to cleanup.md; not a lockfile regression
- [x] `cargo audit` — 0 vulnerabilities; anyhow (unsound) gone from tree, chacha20 past yanked 0.10.0; 153 deps (was 172)
- [x] `cargo build --release`
- [x] PTY smoke: `(sleep 2; printf 'q') | timeout 8 script -q /dev/null target/release/tarts butterfly` exits 0

## Findings

- All 11 direct deps already at latest compatible and latest major; `cargo upgrade` was a no-op on manifests — the update is entirely in Cargo.lock (crossbeam-epoch 0.9.18→0.9.21, chacha20 0.10.0→0.10.2, anyhow removed, wit-bindgen stack removed, zerocopy refresh, +zmij via serde_json←criterion dev-dep only).

## Context

Do NOT touch `version` in `[package]` — pushes to main touching Cargo.toml can trigger publish/release workflows (v0.1.26 lesson; disarmed separately in 0003).

## Summary

Lockfile refresh: crossbeam-epoch RUSTSEC-2026-0204 fixed (0.9.21), chacha20 past yanked version, anyhow dropped; 19 fewer dep entries; all checks and smoke pass.

## Review

Diff is Cargo.lock-only (Cargo.toml untouched — no version bump). No new runtime deps; the one addition (zmij) is dev-only via serde_json←criterion, verified with cargo tree -i. All Acceptance commands run and ticked with results inline. Matches Plan and scope.

## Log

- 2026-10-09 created: deps update
- 2026-10-09 scope add Cargo.toml — dependency manifests and lockfile are the task's entire surface
- 2026-10-09 scope add Cargo.lock — dependency manifests and lockfile are the task's entire surface
- 2026-10-09 advance draft → ready
- 2026-10-09 scope approved: Cargo.toml, Cargo.lock
- 2026-10-09 advance ready → in-progress/implement
- 2026-10-09 baseline: specdev.toml@2033f8a8ce31
- 2026-10-09 advance in-progress/implement → in-progress/verify
- 2026-10-09 advance in-progress/verify → in-progress/review
- 2026-10-09 advance in-progress/review → approval
- 2026-10-10 approved: ok let's done 0001-deps-update
- 2026-10-10 advance approval → done
