# Agent Guide

Agent and dev rules only. User-facing CLI and install info live in `README.md`; architecture and gotchas live in `specs/overview.md`.

## Dev Commands

```bash
cargo build --release
cargo test --lib

# Quality gate — run before handing work back (pre-commit uses -D warnings)
cargo fmt --check
cargo clippy --all-targets -- -D warnings
prek run --all-files   # fmt, clippy, tests, typos, gitleaks

# Headless render analysis — debug an effect without a TTY
cargo run --release -- --analyze pocketworld --view height --frames 3000
cargo run --release -- --analyze pocketworld --view biome --size 160x50 --dump

# PTY smoke test — exit 0 = clean quit; script's pty reports 0x0 (degenerate-size test)
(sleep 2; printf 'q') | timeout 8 script -q /dev/null target/release/tarts <effect>
```

## Agent Rules

- Edit files only with a proper patch tool (e.g. `apply_patch` or an equivalent structured edit). Never rewrite files via ad-hoc scripts (python/sed one-liners) — silent partial rewrites are how bugs get in.
- Plain `cargo clippy` passing is not enough: the pre-commit gate is `cargo clippy --all-targets -- -D warnings`.
- No `[package] version` bumps in PRs; releases are tag-push only (`specs/release.md`).
- New effects need a GitHub issue first (README Contributing policy).
- Don't create task files unless the user asks for one.
- Each draft task needs the user's explicit approval before `advance`; never batch-implement on inferred approval.

<!-- BEGIN specdev -->
## specdev

This project uses **specdev** (specification-driven development). Load the
specdev skill when starting a session, continuing from specs, or picking up a
task. Always read `specs/overview.md` and `specs/ctx.md` before coding.
<!-- END specdev -->
