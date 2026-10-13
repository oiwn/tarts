---
id: 0006-terrain-dynamic-world
status: blocked
scope: [src/main.rs, src/check.rs, tests/degenerate_size.rs, README.md, src/lib.rs, AGENTS.md, src/common.rs, src/pocketworld/mod.rs, src/pocketworld/noise.rs, src/pocketworld/effect.rs, src/pocketworld/world.rs, src/pocketworld/elevation.rs, src/pocketworld/hydro.rs, src/pocketworld/climate.rs, src/pocketworld/erosion.rs, src/pocketworld/tectonics.rs, src/pocketworld/life.rs, src/pocketworld/sim.rs, src/pocketworld/render.rs, src/config.rs]
created: 2026-10-09
source: https://github.com/oiwn/tarts/issues/93
blocked_reason: visual-tuning rounds blocked on the analysis facility (user, 2026-10-13): agent cannot see renders; probe-number tuning missed the land plateau until measured. Analyzer (0010) lands first; then the stretch-saturation fix and remaining rounds resume.
---
# Task: terrain dynamic world

## Plan

The single working task for the pocketworld rebuild (effect renamed terrain -> pocketworld 2026-10-11; the task title is pinned by specdev's log, which has no rename command; user decision: no per-phase subtasks). Each round lands as its own commit.

- [x] Phases 1-5b + reshape rounds — living world sketch: layered views on `l`/`k`, cloud fronts, rivers via springs, snow, seasons, vegetation + fire, plates. History in git.
- [x] Layer-file refactor + `k` key (2026-10-11) — one file per sim layer, runtime-composed via `PocketWorldOptions::disabled`; behavior-preserving, tests green.
- [x] Dynamics pass, tectonics half (2026-10-11) — focus mode (hydro/climate/erosion/life disabled via config), then: hot rates + round smoothstep kernel, warped-Voronoi margins, isostasy gated by a decaying activity memory, per-pair orogen segmentation (passive margins), fixed mantle plumes (`◉` on plates view) building island trails, crust advection (terrain rides the plates), volcanic lifecycle (islands age and resubmerge), crust-mass pin. **Approved: "Wow, now we're talking! looks good."**
- [x] Water-representation polish (2026-10-12) — ocean/lake decided by geometry (below sea = sea), water depth EMA, channel-gated flow arrows. Strobe eliminated (0 ocean-depth flips per refresh window).
- [x] Climate/wind round (2026-10-12) — **wind is now a simulated momentum field**: self-advection, thermal band forcing, Coriolis (∝ sin lat), orographic/land drag, turbulence. General circulation emerges (locked by test). Approved: "wind now looks more realistic."
- [x] Sky & ice + brightness/variety rounds B/C/C2 (2026-10-12) — cohesive cloud coverage (soft stamps + EMA), color-milk translucency (checkerboard dither dead), cloud vocabulary; cryosphere bands with temp hue; brightness pass; per-object glyph/color variety via stable position hash. Approved: "WOW, this is AWESOME."
- [x] Keys + legend (2026-10-12) — `j` prev / `k` next / `l` toggles a full legend screen (own page, not an overlay).
- [~] Relief round (2026-10-12/13) — partial, **see block**: sea-from-base (percentile of the masked field rose with the mountains and ate relief), mountain mask strengthened, erosion slowed. Follow-on bugs found and fixed: sediment-deficit mass-pin payout (seafloor +0.42), erosion conveyor filling the ocean (deep sea = burial sink), **eustasy** (sea relaxes to target fraction of live crust). Remaining flaw: **stretch saturation plateau** — land mean 0.93, std 0.11: the ×2.4 stretch pins land near the headroom cap; heightmap shows 1–2 bands. Fix is designed; verification blocked on the analyzer (0010).
- [ ] Life round — vegetation + fire on the finished palette (forests are still humidity-shadows; the round makes them organisms).
- [ ] Biome palette verification + remaining flaws (thin rivers 1.9%, static temp, green deserts — desert/swamp paint with the green elevation ramp — rare forests).
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

Current state green (2026-10-13): 61 lib tests + 2 integration, clippy clean, release build, PTY quit, replay exact. Status: **blocked** on 0010 (analyzer) — the stretch-saturation plateau fix and remaining visual rounds resume with measured data.

## Findings

Durable lessons (per-round history lives in git):

- **Analyzer before tuning** (user, 2026-10-13): probe-number tuning missed the land plateau until a human looked — the agent cannot see renders; the analysis facility (0010) is the feedback loop.
- **Sea level must come from the base field, before mountains** — a percentile of the masked field rises with the mountains and eats the relief; then the headroom cap flattens the rest into a plateau.
- **Mass pins must be step-local, never against a gen-time total** — erosion parks mass in the sediment field; paying that "deficit" every advection lifted the whole seafloor (+0.42; ocean collapsed to 45 cells).
- **Free mass needs a free sink, and the sea is the rock cycle's sink** — the uplift→erosion→coastal-deposition conveyor filled the ocean until below-sea sediment became burial (returned to the mantle).
- **Eustasy** — sea relaxing toward its target fraction of the live crust is the ocean's homeostat against whatever the crust does.
- **Terrain must ride the plates** (advection) or the heightmap traces the plate network; semi-Lagrangian self-advection of wind gives meanders/eddies cheaply; polar precipitation can't wait for the cloud fleet (probed: zero clouds past |lat| 0.5) — the polar cell delivers its own snowfall; sublimation bounds the caps; naive hashes show lattice patterns (use a mixing hash).
- Focus mode (`disabled` via config) is the tuning workflow; the ocean is a boundary condition; infiltration must not leak channelized flow; concentrated per-cell rain forms channels; masks add after blending; verify by glyph census; f32 removals clamp; brightness-monotonic ramps only.
- Known render debts: desert/swamp painted with the green elevation ramp; rivers thin; temp reads static.
- **Measured baseline (2026-10-13, analyzer seed 42, 100×40, default options — focus config NOT applied):** height@3000: land renders only `·` 30% + `░` 23%, zero `▒`/`▓`/`█` cells; worse, `▒` is 9% at frame 300 and decays to 0 by 3000 — erosion eats the mid band live. temp@900: 95.8% of cells in ONE ramp band — the ramp/normalization shows no spatial structure. biome@1500 (life on): grass family 37.7% of canvas, forests (`♠`+`♣`) 4.8% ≈ 12% of land, rock 1.8%. hydro@900: flow arrows ~3.1%. Churn paradox: 60–95% of cells change per sample yet the screen reads static — in-place glyph jitter with no feature motion. Fix order: height distribution (gen-time stretch + run-time band loss) → temp ramp → desert color → life round.

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
- 2026-10-13 blocked: visual-tuning rounds blocked on the analysis facility (user, 2026-10-13): agent cannot see renders; probe-number tuning missed the land plateau until measured. Analyzer (0010) lands first; then the stretch-saturation fix and remaining rounds resume.
