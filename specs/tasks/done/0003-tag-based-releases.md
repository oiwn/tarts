---
id: 0003-tag-based-releases
status: done
scope: [.github/workflows/publish.yml, .github/workflows/release-on-version-bump.yml, .github/workflows/release.yml, specs/release.md]
created: 2026-10-09
---
# Task: tag based releases

## Plan

- [x] Read `release.yml`, `publish.yml`, `release-on-version-bump.yml` fully; map which workflow does what today
- [x] Retrigger `publish.yml` (crates.io) and `release-on-version-bump.yml` (GH release) on `push: tags: v*` only — never on main/Cargo.toml paths
- [x] Keep `release.yml` (assets) as the tag-driven builder; make the chain deterministic: tag → GH release → assets → crates.io publish, in that order
- [x] Remove dead path filters/branches leftovers; update specs/release.md to the new flow

## Acceptance

- [x] `cargo fmt --check`
- [x] `cargo test --lib` — 37 passed
- [x] `cargo clippy` — same 1 pre-existing warning
- [x] YAML parse of all workflows via `ruby -ryaml` (pyyaml unavailable): 9/9 parsed
- [x] Trigger audit: only deps-audit.yml keeps a main-push + Cargo.toml path trigger (audit-only, no publish); release-on-version-bump.yml deleted

## Findings

- `release.yml` uploads via `softprops/action-gh-release@v1`, which creates the release if missing — a bare `v*` tag push needs nothing else. `release-on-version-bump.yml` was therefore pure redundancy with the failure mode that shipped v0.1.26; deleted rather than retriggered.
- New `publish.yml` is one job: checkout tag → verify `GITHUB_REF_NAME#v` equals Cargo.toml version (guards against tagging the wrong commit) → dry-run → publish.

## Summary

Releases are tag-driven only: deleted release-on-version-bump.yml, reworked publish.yml onto v* tags with a tag↔manifest version guard, documented the push-a-tag flow in specs/release.md.

## Review

Diff: publish.yml rewritten (trigger + guard, publish steps preserved incl. CRATES_IO_TOKEN), release-on-version-bump.yml deleted via git rm, release.yml untouched (already correct), specs/release.md Step 2 rewritten to tag-only with manual steps removed. All workflows parse; no remaining main-push publish path. Matches Plan and scope.

## Log

- 2026-10-09 created: tag based releases
- 2026-10-09 scope add .github/workflows/publish.yml — release pipeline rework touches these workflows and the release doc
- 2026-10-09 scope add .github/workflows/release-on-version-bump.yml — release pipeline rework touches these workflows and the release doc
- 2026-10-09 scope add .github/workflows/release.yml — release pipeline rework touches these workflows and the release doc
- 2026-10-09 scope add specs/release.md — release pipeline rework touches these workflows and the release doc
- 2026-10-09 advance draft → ready
- 2026-10-09 scope approved: .github/workflows/publish.yml, .github/workflows/release-on-version-bump.yml, .github/workflows/release.yml, specs/release.md
- 2026-10-09 advance ready → in-progress/implement
- 2026-10-09 baseline: .github/workflows/deps-audit.yml@1b7e595c1d64, .github/workflows/opencode.yml@cf8fd4d5b54d, Cargo.lock@533c10b2dc53, specdev.toml@2033f8a8ce31
- 2026-10-09 advance in-progress/implement → in-progress/verify
- 2026-10-09 advance in-progress/verify → in-progress/review
- 2026-10-09 advance in-progress/review → approval
- 2026-10-10 approved: let's done 003
- 2026-10-10 advance approval → done
