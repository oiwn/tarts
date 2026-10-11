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
- [x] pocketworld (0006, renamed from terrain 2026-10-11) — sketch + refactor + `k` done; tectonics round approved 2026-10-11 ("Wow, now we're talking! looks good"); task compressed; remaining rounds: erosion → climate → life → biome palette → integration — see 0006
- [x] wasm feasibility note into ideas.md
- [x] triage remaining open issues (below)
- [x] specdev flaws → upstream issues specdev#15, specdev#16 (task 0007)

## Specdev remarks (retro + upstream report)

Upstream (oiwn/specdev) — file or extend at session end:

- **filed, specdev#15** — `check` doesn't enforce the ctx.md header invariant; this repo silently ran with `# Current Task:`.
- **filed, specdev#16** — skill docs omit `task scope rm`, so agents can't discover cleanup and leave stray scope entries.
- **filed, specdev#17** — draft→ready advance cannot verify the human actually approved the draft; an agent can skip the review gate silently (happened twice here). Proposal: a human-run `specdev task approve <id>` command that `advance` requires out of draft; approval becomes visible in the log instead of trust-based.
- **filed, specdev#18** — quality limits (150 lines / 1000 words / 6 scope per task) assume small one-commit tasks. A legitimate single working-manual task (0006: five phase specs, architecture, palette, decisions) compresses to 139 lines but still ~1500 words; splitting it contradicts the user's explicit one-task-per-feature decision. Limits need per-project config headroom or a distinct "design/umbrella task" kind.
- **filed, specdev#19 (2026-10-11)** — agents create task files for trivial fixes (clippy one-liner, typo) because the skill routes "small one-commit work" into tasks and check's out-of-scope error pushes toward structure; proposal: never create tasks without explicit user instruction, `[task] explicit_only` config, out-of-scope as warning for small uncovered fixes. Local rule pending: AGENTS.md "no task creation unless instructed".

Agent process (harness, not specdev):

- Batch-implemented tasks on inferred approval instead of per-task draft review — twice; the standing rule now is explicit per-task approval.
- Edited files via ad-hoc python rewrites instead of the patch tool — caused three silent no-ops; fixed by AGENTS.md Agent Rules.

## Issue drafts

- **#80**: root cause `reset()` = `todo!()`, any resize panicked; fixed this session — close when the fix PR merges.
- **#88**: fixed since 0.1.25 (`initial_cells` builder default) — close now.

## Triage of remaining open issues

- **#88** — close now (draft above). **#80** — close when 0005 merges. **#94** — close when 0004 merges.
- **#55** (map generation) — superseded by the #93 terrain vision; close as dup with a pointer.
- **#87 + #82** — dupes of **#44** (distro packaging, help-wanted); keep #44 as umbrella, close the others as dupes.
- **#89** (gravity O(N)) — good perf idea, adjacent to boids/collision; keep open, roadmap candidate after terrain.
- **#83 / #71 / #65 / #64** — effect ideas (DNA helix, game theory, snake, fractals). Keep open; fractals is the cheapest win.

## Context

Audit of merged PRs #90/#91: clean, no malicious code (details: overview.md Gotchas). Terrain VALID_SAVERS fix rides along with the terrain redesign, no separate PR. v0.1.26 GitHub release was auto-created but unpublished; user is deleting it. User adds ZHIPU_API_KEY secret for opencode.yml.

## Next

0006 erosion round: set `disabled = ["climate", "life"]` in the focus config (hydro + erosion back on), tune the counterweight against the approved tectonics morphology; instruments Height + Hydro. Watch-item: the mild land-mean creep (see 0006 Findings). User's git queue: COMMIT — the worktree holds the layer refactor, the pocketworld rename, and the whole approved tectonics round (code + compressed specs) uncommitted. Session-end items still open: specdev retro conversation, ctx archive-or-keep.
