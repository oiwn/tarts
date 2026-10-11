---
id: 0006-terrain-dynamic-world
status: in-progress
stage: implement
scope: [src/main.rs, src/check.rs, tests/degenerate_size.rs, README.md, src/lib.rs, AGENTS.md, src/common.rs, src/pocketworld/mod.rs, src/pocketworld/noise.rs, src/pocketworld/effect.rs, src/pocketworld/world.rs, src/pocketworld/elevation.rs, src/pocketworld/hydro.rs, src/pocketworld/climate.rs, src/pocketworld/erosion.rs, src/pocketworld/tectonics.rs, src/pocketworld/life.rs, src/pocketworld/sim.rs, src/pocketworld/render.rs, src/config.rs]
created: 2026-10-09
source: https://github.com/oiwn/tarts/issues/93
---
# Task: terrain dynamic world

## Plan

The single working task for the pocketworld rebuild (effect renamed terrain -> pocketworld 2026-10-11; the task title is pinned by specdev's log, which has no rename command; user decision: no per-phase subtasks). Each round lands as its own commit.

- [x] Phases 1-5b + reshape rounds — living world sketch: layered views on `l`/`k`, cloud fronts, rivers via springs, snow, seasons, vegetation + fire, plates. History in git.
- [x] Layer-file refactor + `k` key (2026-10-11) — one file per sim layer, runtime-composed via `PocketWorldOptions::disabled`; behavior-preserving, tests green.
- [x] Dynamics pass, tectonics half (2026-10-11) — focus mode (hydro/climate/erosion/life disabled via config), then: hot rates + round smoothstep kernel, warped-Voronoi margins, isostasy gated by a decaying activity memory, per-pair orogen segmentation (passive margins), fixed mantle plumes (`◉` on plates view) building island trails, crust advection (terrain rides the plates), volcanic lifecycle (islands age and resubmerge), crust-mass pin. **Approved: "Wow, now we're talking! looks good."**
- [ ] Erosion round — re-enable hydro + erosion; tune the counterweight against the approved morphology: rivers carve the new ranges and islands without erasing them. Instruments: Height + Hydro views.
- [ ] Climate round — clouds/rain/snow back on; orographic rain on the new ranges, seasons over advecting terrain.
- [ ] Life round — vegetation and fire.
- [ ] Biome palette round — the standing "looks like a shit" verdict on the biome view; owed once the world is fully alive again.
- [ ] Integration + perf — full stack, focus config removed, ~60 FPS bar, acceptance gates.

Quality process (user): step by step, one improvement per visual review, toward "a dynamic nice-looking pocket world".

## Architecture — state and tick

The render `Buffer` is output, rebuilt from fields each frame. State is split by ownership: `world.rs` holds the multi-writer commons (elevation, water, sea level, clock, noise, thresholds); each layer file owns its grids as a struct with an `on` flag; `sim.rs` composes them (canonical tick order, single RNG stream; `disabled` skips a layer's steps, freezing its state).

- `elevation` [0..1] — two-band fBm on a cylinder (seamless x-wrap), domain-warped; mountain mask added AFTER blending (blending compresses ridges below every threshold — the round-2 lesson); sea level = 28th-percentile fill; land stretched ×1.7; biome thresholds = land percentiles. Tectonics and erosion write later.
- `water`, `flux` — pipe model (Mei et al. 2007): flux ∝ total-height difference, damped, sub-stepped, never drains a cell dry. The ocean is a boundary condition, not a conservation tank. Infiltration clears standing sheets only — channelized flow is sealed. Springs guarantee headwaters by construction. Classification emergent: ocean = connected to the sea fill, lake = isolated standing water, river = flux above threshold — motion, not a body.
- `sediment` — carrying capacity ∝ flow; the gap erodes, the surplus deposits, drying cells drop their whole load. Talus sheds dry slopes past the angle of repose. Solid mass conserved.
- `humidity`, `snow`, `clouds` — clouds are 1-cell particles spawned as fronts over warm ocean, riding the banded winds; each rains its own cell (×2 against mountain walls — windward pile-up, leeward rain shadow); cold cells take snow instead of rain. Wind and temperature are derived, never stored: latitude bands + swirl, gradient + lapse + hemisphere-antisymmetric seasons.
- `vegetation`, `fire` — growth where humid and warm; lightning/dry-heat ignition, spread through fuel, ash fertilizes regrowth.
- tectonics — ~7 warped-Voronoi plates (wrap-x) with slow drift; membership refreshes lazily so margins meander. Boundary stress = relative velocity decomposed on the boundary normal: convergent uplift / divergent rift / transform quakes, each plate pair's margin scaled by a slow orogen factor in [0.15, 1] — passive margins exist. Deformation stamps a round smoothstep disk; land takes it in full, seafloor ×0.15. A decaying activity memory gates the isostatic sag — only recently deformed crust pays, so ancient interiors keep their generated character; the land-mass books balance per tick. Fixed mantle plumes punch magma through the crust gate (island trails, `◉` on plates view); a volcanic load memory decays and the crust subsides and relaxes toward sea, so only the young ends of trails stay emergent. Every refresh cadence the crust advects with its plate (semi-Lagrangian, bilinear) — continents travel and keep their outlines, ranges ride along after their front moves on; total crust mass is pinned to its world-gen value (the gather duplicates crust at divergent boundaries otherwise).

Tick order: drain → climate reset → clouds → springs → snowmelt → life → tectonics → pipes ×2 → sediment/erosion/talus → lazy classification. Seeded stochastic rolls (fronts, lightning, quakes) — replays are bit-exact.

Rendering is a pure function of the fields (`render.rs`, crossterm only there and in `effect.rs`): the `l`/`k` keys cycle the layer views; the HUD names the active layer on row 0.

## Palette

Narrow (width-1) Unicode only — double-width glyphs break the grid. Color carries elevation, glyph carries state; in the heightmap view brightness = height, strictly (8 hard steps, bathymetry below sea, NW hillshade, density glyphs ` ·░▒▓`). Water: deep `≈` → shallow `~`, lake `~` teal, rivers `˃ ˄ ˂ ˅` by flux. Cloud `▒`/`▓` dithered over terrain; snow-pack `*`/`❄`. Shore `.`, grass `"`, forest `♣`, swamp `Ψ`, desert `∴`, rock `▲`, scree `°`, ash `░`, fire `✱ ❋ ✹` animated, rain `|`/`⁄`.

## References

- Musgrave, Kolb, Mace — "The Synthesis and Rendering of Eroded Fractal Terrains" (SIGGRAPH 1989)
- Mei, Decaudin, Hu — "Fast Hydraulic Erosion Simulation and Visualization on GPU" (2007): the pipe model
- Perlin — "Improving Noise" (SIGGRAPH 2002)
- Dwarf Fortress world generation — Tarn Adams' GDC talks + DF wiki

## Acceptance

Global bar, applied per phase as it lands:

- [ ] ~60 FPS at 120x40 on a laptop; sim may run sub-rate
- [ ] Deterministic from `seed`; `reset()` re-rolls
- [ ] PTY smoke + degenerate-size safe (see 0004)
- [ ] `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`

Current state green (2026-10-11): 58 lib tests + 2 integration, clippy clean, release build, PTY quit, replay exact; tectonics equilibrium probed — land bounded, relief up, ~500 island cells cycling.

## Findings

Durable lessons (per-round history lives in git):

- **Terrain must ride the plates (advection) or the heightmap traces the plate network** — everything built along static boundaries reads as polygonal scars. The user's Earth intuition ("continents do not change that much, they're moving around with plates") was the round's unlock.
- **Semi-Lagrangian advection duplicates crust at divergent boundaries** — without pinning total crust mass to its world-gen value, land ran away to 99% of the map within minutes.
- **Free mass needs a free sink**: plume magma is unbooked until it surfaces; volcanic subsidence must be unbooked too — booking only one side grows land at the ocean's expense.
- **Fix causes, not aggregates**: a land-relief thermostat (rescaling all relief to pin land mass) converted coastline churn into height loss and drowned the world in low land; reverted. Per-cell island relax-toward-sea fixed the actual cause.
- **Focus mode is the tuning workflow**: `disabled` layers via config, one variable at a time, full stack only at round ends.
- The ocean is a boundary condition, not a conservation tank (two failed rain-budget controllers proved it); infiltration must not leak channelized flow (27× spring inflow); concentrated per-cell rain is what forms channels; masks are added after blending, never averaged in; verify renders by glyph census, never escape-stripping diffs; f32 water removals need clamps (one-ulp overflows); brightness-monotonic ramps only; the ocean invariant is aggregate.
- Session 2026-10-10/11: heightmap approved ("looks awesome"); biome view still owes its palette round ("looks like a shit"). Session 2026-10-11 close: tectonics round approved ("Wow, now we're talking! looks good"). Watch-item: land mean creeps ~+0.06 per 2400 ticks (decelerating) — re-examine in the erosion round.

## Context

User direction: "2d liquids, weather, rivers, geological changes — dynamic, something interesting." Vision + DF comparison: issue #93.

Decisions (user): one task per feature; cylinder topology, banded winds, no-flux poles, latitude+lapse temperature, hemisphere-antisymmetric seasons; ocean-fill initial state, emergent classification; clouds are 1-cell fronts, no field atmosphere; mountains block wind; cryosphere melts into the hydrosphere; color = elevation, glyph = state, heightmap brightness = height; layer cycling is a display control, no simulation controls; single seeded RNG stream, `reset()` re-rolls; narrow glyphs only; no accelerated-history boot; step-by-step visual reviews. 2026-10-11: effect renamed pocketworld; `disabled` is config-only groundwork (wasm/perf), not a user control; focus mode for tuning; tectonics 2× slower — motion carried by advection.

## Summary

## Log

- 2026-10-09 created: terrain dynamic world
- 2026-10-09 set source https://github.com/oiwn/tarts/issues/93
- 2026-10-10 scope add src/terrain/effect.rs — absorbing phase-1 scope from deleted 0010; terrain work happens here
- 2026-10-10 scope add src/terrain/noise.rs — absorbing phase-1 scope from deleted 0010; terrain work happens here
- 2026-10-10 scope add src/main.rs — absorbing phase-1 scope from deleted 0010; terrain work happens here
- 2026-10-10 scope add src/check.rs — absorbing phase-1 scope from deleted 0010; terrain work happens here
- 2026-10-10 scope add tests/degenerate_size.rs — absorbing phase-1 scope from deleted 0010; terrain work happens here
- 2026-10-10 scope add README.md — absorbing phase-1 scope from deleted 0010; terrain work happens here
- 2026-10-10 scope add src/lib.rs — absorbing phase-1 scope from deleted 0010; terrain work happens here
- 2026-10-10 advance draft → ready
- 2026-10-10 scope approved: src/terrain/effect.rs, src/terrain/noise.rs, src/main.rs, src/check.rs, tests/degenerate_size.rs, README.md, src/lib.rs
- 2026-10-10 advance ready → in-progress/implement
- 2026-10-10 scope add AGENTS.md — user-directed mid-task edit: agent rules (patch tools, clippy gate); not terrain work, rides with this task's commit
- 2026-10-10 scope add src/common.rs — TerminalEffect input hook for terrain layer cycling (reshape item 6)
- 2026-10-11 scope add src/terrain/world.rs — layer-file refactor: one file per simulation layer, runtime-composed (0006 next-session item 1)
- 2026-10-11 scope add src/terrain/elevation.rs — layer-file refactor: one file per simulation layer, runtime-composed (0006 next-session item 1)
- 2026-10-11 scope add src/terrain/hydro.rs — layer-file refactor: one file per simulation layer, runtime-composed (0006 next-session item 1)
- 2026-10-11 scope add src/terrain/climate.rs — layer-file refactor: one file per simulation layer, runtime-composed (0006 next-session item 1)
- 2026-10-11 scope add src/terrain/erosion.rs — layer-file refactor: one file per simulation layer, runtime-composed (0006 next-session item 1)
- 2026-10-11 scope add src/terrain/tectonics.rs — layer-file refactor: one file per simulation layer, runtime-composed (0006 next-session item 1)
- 2026-10-11 scope add src/terrain/life.rs — layer-file refactor: one file per simulation layer, runtime-composed (0006 next-session item 1)
- 2026-10-11 scope add src/terrain/sim.rs — layer-file refactor: one file per simulation layer, runtime-composed (0006 next-session item 1)
- 2026-10-11 scope add src/terrain/render.rs — layer-file refactor: one file per simulation layer, runtime-composed (0006 next-session item 1)
- 2026-10-11 scope add src/terrain/mod.rs — layer-file refactor: declare the new terrain layer modules
- 2026-10-11 scope add src/pocketworld/mod.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope add src/pocketworld/noise.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope add src/pocketworld/effect.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope add src/pocketworld/world.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope add src/pocketworld/elevation.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope add src/pocketworld/hydro.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope add src/pocketworld/climate.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope add src/pocketworld/erosion.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope add src/pocketworld/tectonics.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope add src/pocketworld/life.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope add src/pocketworld/sim.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope add src/pocketworld/render.rs — rename terrain -> pocketworld (user decision)
- 2026-10-11 scope rm src/terrain/effect.rs
- 2026-10-11 scope rm src/terrain/noise.rs
- 2026-10-11 scope rm src/terrain/world.rs
- 2026-10-11 scope rm src/terrain/elevation.rs
- 2026-10-11 scope rm src/terrain/hydro.rs
- 2026-10-11 scope rm src/terrain/climate.rs
- 2026-10-11 scope rm src/terrain/erosion.rs
- 2026-10-11 scope rm src/terrain/tectonics.rs
- 2026-10-11 scope rm src/terrain/life.rs
- 2026-10-11 scope rm src/terrain/sim.rs
- 2026-10-11 scope rm src/terrain/render.rs
- 2026-10-11 scope rm src/terrain/mod.rs
- 2026-10-11 scope add src/config.rs — rename terrain -> pocketworld: config section + getters
