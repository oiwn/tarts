---
id: 0002-ci-workflows
status: done
scope: [.github/workflows/opencode.yml, .github/workflows/deps-audit.yml]
created: 2026-10-09
---
# Task: ci workflows

## Plan

- [x] Add `.github/workflows/deps-audit.yml` copied from oiwn/exodata (cargo-audit on schedule/PR/push touching manifests)
- [x] Add `.github/workflows/opencode.yml` adapted from oiwn/exodata with an actor gate: the `/oc` inference job must be restricted to `github.actor == 'oiwn'` (public repo — anyone else commenting `/oc` would burn the ZHIPU key)
- [x] Keep the `todo-to-issues` job as-is (already gated to same-repo `todos/*` branches); model stays `zai-coding-plan/glm-5.3-flash`
- [x] Validate YAML parses; dry-check trigger conditions

## Acceptance

- [x] `cargo fmt --check`
- [x] `cargo test --lib` — 37 passed
- [x] `cargo clippy` — same 1 pre-existing warning, unchanged
- [x] YAML parse — pyyaml unavailable; equivalent via `ruby -ryaml` safe_load on both files: parsed (command in Review)

## Findings

- Trigger semantics kept stricter than a plain `contains` match: bare `/oc` must start the comment (exodata's startsWith), space-prefixed `/oc` matches anywhere. Prevents accidental fires on substrings like paths.

## Manual checks

- [x] After merge, `/oc` comment from owner starts the job; a non-owner comment does not — waived by user: "done 0002" (will verify naturally post-merge)
- [x] ZHIPU_API_KEY added to repo secrets — waived by user: "done 0002" (secret to be added before merge)

## Context

User adds `ZHIPU_API_KEY` to repo secrets (manual, before merge).

## Summary

Two new workflows from exodata: deps-audit.yml (weekly + manifest-change cargo-audit) and opencode.yml (/oc inference + todo-to-issues), with the /oc job additionally gated to github.actor == 'oiwn'.

## Review

Only additions under .github/workflows/, matching scope. /oc job: exodata's conditions verbatim plus `github.actor == 'oiwn'` (comment author == actor for comment events; owner-only inference). todo-to-issues untouched from source (same-repo todos/* branch gate). deps-audit.yml verbatim. YAML validated with ruby safe_load (python3 lacks yaml module on this machine). Code untouched — fmt/test/clippy re-run to confirm.

## Log

- 2026-10-09 created: ci workflows
- 2026-10-09 scope add .github/workflows/opencode.yml — the two new workflow files this task adds
- 2026-10-09 scope add .github/workflows/deps-audit.yml — the two new workflow files this task adds
- 2026-10-09 advance draft → ready
- 2026-10-09 scope approved: .github/workflows/opencode.yml, .github/workflows/deps-audit.yml
- 2026-10-09 advance ready → in-progress/implement
- 2026-10-09 baseline: Cargo.lock@533c10b2dc53, specdev.toml@2033f8a8ce31
- 2026-10-09 advance in-progress/implement → in-progress/verify
- 2026-10-09 advance in-progress/verify → in-progress/review
- 2026-10-09 advance in-progress/review → approval
- 2026-10-10 approved: done 0002 — waived the two post-merge manual checks
- 2026-10-10 advance approval → done
