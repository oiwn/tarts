---
id: 0014-prek-hook-order
status: draft
scope: [prek.toml]
created: 2026-10-14
source: user request 2026-10-14: prek runs light hooks first (formatting, typos), heavy last (tests)
---
# Task: prek hook order

## Plan

`prek.toml` has `fail_fast = true` but runs fmt → clippy → test → typos → gitleaks. So a typo is only reported after a full clippy + test build. Cheap checks should fail first.

- [ ] Reorder the hooks to fmt → typos → gitleaks → clippy → test. No compile → no compile → git scan → compile → compile + run. Hook definitions stay the same; only the order changes.

## Acceptance

- [ ] `prek run --all-files` passes, and its output lists the hooks in the new order
- [ ] `cargo fmt --check`

## Log

- 2026-10-14 created: prek hook order
- 2026-10-14 set source user request 2026-10-14: prek runs light hooks first (formatting, typos), heavy last (tests)
- 2026-10-14 scope add prek.toml — reorder hooks light → heavy
