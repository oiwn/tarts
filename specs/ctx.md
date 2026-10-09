# Current Task: revival session — deps, CI, release pipeline, bug fixes

State: in progress

## Plan

The session book. One task = one PR, merged one by one. No version bumps until a deliberate v2; releases move to tag-push only.

- [x] deps-update — DONE (crossbeam-epoch vuln fix; locked at latest majors)
- [x] ci-workflows — DONE (opencode.yml gated to oiwn + deps-audit.yml)
- [x] tag-based-releases — DONE (publish.yml tag-gated with version guard; release-on-version-bump.yml deleted)
- [x] fix-degenerate-size-panic — DONE (0x0 panics + crab hang; 8 src files + tests)
- [x] fix-life-reset — DONE (issue #80 resize panic fixed; src/life/effect.rs)
- [x] readme-contributing-policy (0008) — DONE (new effects need an issue first)
- [x] terrain dynamic-world — vision filed as issue #93; task 0006 in draft awaiting user ^^^ review
- [x] wasm feasibility note into ideas.md
- [x] triage remaining open issues (below)
- [x] specdev flaws → upstream issues specdev#15, specdev#16 (task 0007)

## Issue drafts

- **#80** (life Windows panic): root cause was `reset()` = `todo!()` — any resize panicked the effect, all platforms. Fixed in the fix-life-reset PR this session; closing, will ship with the next tag.
- **#88** (--check life panic): already fixed — `initial_cells` has had a builder default since 0.1.25. Close.

## Triage of remaining open issues

- **#88** — close now (draft above). **#80** — close when 0005 merges. **#94** — close when 0004 merges.
- **#55** (map generation) — superseded by the #93 terrain vision; close as dup with a pointer.
- **#87 + #82** — dupes of **#44** (distro packaging, help-wanted); keep #44 as umbrella, close the others as dupes.
- **#89** (gravity O(N)) — good perf idea, adjacent to boids/collision; keep open, roadmap candidate after terrain.
- **#83 / #71 / #65 / #64** — effect ideas (DNA helix, game theory, snake, fractals). Keep open; fractals is the cheapest win.

## Context

Audit of merged PRs #90/#91: clean, no malicious code (details: overview.md Gotchas). Terrain VALID_SAVERS fix rides along with the terrain redesign, no separate PR. v0.1.26 GitHub release was auto-created but unpublished; user is deleting it. User adds ZHIPU_API_KEY secret for opencode.yml.

## Next

User: review tasks at approval, commit/PR per task group, add ZHIPU_API_KEY, post #88/#80 drafts, close dupes per triage, review 0006 terrain draft with ^^^.
