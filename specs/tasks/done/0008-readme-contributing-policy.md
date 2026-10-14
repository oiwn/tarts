---
id: 0008-readme-contributing-policy
status: done
scope: [README.md]
created: 2026-10-10
source: user request during 0004 review
---
# Task: readme contributing policy

## Plan

- [x] Rewrite the Contributing section: bug fixes and improvements welcome; new effects not accepted without prior discussion (open an issue first)

## Acceptance

- [x] README renders the policy in the Contributing section (visual check of the diff)
- [x] `cargo fmt --check`
- [x] `cargo test --lib` — 38 passed
- [x] `cargo clippy` — unchanged

## Summary

README Contributing section now states: bug fixes and improvements welcome; new effects require opening an issue for discussion first.

## Review

One paragraph replaced in README.md's Contributing section; no other files touched beyond this task file. Matches the user's request verbatim in intent.

## Log

- 2026-10-10 created: readme contributing policy
- 2026-10-10 set source user request during 0004 review
- 2026-10-10 scope add README.md — README contributing section is the entire change
- 2026-10-10 advance draft → ready
- 2026-10-10 scope approved: README.md
- 2026-10-10 advance ready → in-progress/implement
- 2026-10-10 baseline: .github/workflows/deps-audit.yml@1b7e595c1d64, .github/workflows/opencode.yml@cf8fd4d5b54d, .github/workflows/publish.yml@422abf96dd0e, .github/workflows/release-on-version-bump.yml@deleted, Cargo.lock@533c10b2dc53, specdev.toml@2033f8a8ce31, src/boids/effect.rs@c1b85ad7b8e1, src/buffer.rs@e48472b285ae, src/common.rs@6104efd17683, src/crab/effect.rs@6ae1b669ce9b, src/life/effect.rs@9620bb6fda0f, src/main.rs@9fc2cfaa6950, src/rain/draw.rs@4ce142650de2, src/rain/rain_drop.rs@4be1a8540842, tests/degenerate_size.rs@e3476d7843e3
- 2026-10-10 advance in-progress/implement → in-progress/verify
- 2026-10-10 advance in-progress/verify → in-progress/review
- 2026-10-10 advance in-progress/review → approval
- 2026-10-10 approved: done 0008
- 2026-10-10 advance approval → done
