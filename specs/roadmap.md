# Roadmap

Committed future direction, in rough priority order. Promote items here from `ideas.md` when decided; move items into `ctx.md` when they become the active task.

## Next

- **Issue cleanup** (triage 2026-10-10) — close #88 (fixed since 0.1.25), #80 and #94 (fixed, ship in 0.1.27), #55 (dup of #93 terrain), #87 + #82 (dups of #44 packaging umbrella). Keep #44, #89 (gravity O(N), after terrain), #83/#71/#65/#64 (effect ideas; fractals cheapest).
- **Pocketworld sim params from config** (user, 2026-10-14) — ~95 tuning `const`s across hydro/climate/tectonics/render/life/erosion/sim move into per-layer `#[serde(default)]` structs under `[pocketworld.<layer>]`, defaults = today's values, no behavior change. Needs a task.

- **Pocketworld clouds linger** (user, 2026-10-14, after 0006 closed) — clouds sometimes sit in one place for a long period; investigate the cloud fleet / wind coupling. Use the analyzer's frame-to-frame dump diff (see 0006 Findings) to measure it.

## Later
