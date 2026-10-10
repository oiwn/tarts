---
id: 0006-terrain-dynamic-world
status: in-progress
stage: implement
scope: [src/terrain/effect.rs, src/terrain/noise.rs, src/main.rs, src/check.rs, tests/degenerate_size.rs, README.md, src/lib.rs, AGENTS.md, src/common.rs, src/terrain/world.rs, src/terrain/elevation.rs, src/terrain/hydro.rs, src/terrain/climate.rs, src/terrain/erosion.rs, src/terrain/tectonics.rs, src/terrain/life.rs, src/terrain/sim.rs, src/terrain/render.rs, src/terrain/mod.rs]
created: 2026-10-09
source: https://github.com/oiwn/tarts/issues/93
---
# Task: terrain dynamic world

## Plan

The single working task for the terrain rebuild (user decision: no per-phase subtasks). Each phase lands as its own commit/PR.

- [x] Phases 1-5 + reshape rounds — **the basic visual sketch works**: a living world with layers (biome / height / hydro / temperature / wind / humidity on `l`), particle cloud fronts, rivers via mountain springs, snow-pack, seasons, vegetation + fire, tectonics, wind-blocking mountains, HUD. Four visual review rounds folded into this state; history in git.
- [x] Round 5a — heightmap readability: brightness-monotonic hypsometric ramp ("bright = higher", 8 hard steps, bathymetry below sea), hillshade from the elevation gradient (NW light) multiplied in, density glyphs ` ·░▒▓`
- [x] Round 5b — lithospheric plates: ~7 Voronoi plates (wrap-x) with slow drift velocities; membership refreshed lazily so boundaries migrate; boundary stress = relative plate velocity decomposed on the boundary normal — convergent uplift grows ranges ON plate edges (replaces random orogenies), divergent opens rift lakes, transform fires quakes; interiors stable (cratons). Plates layer is #1 after biome in the cycle: dim plate hues, boundaries typed red `≡` / cyan `¦` / yellow `≈`, drift arrows at seeds

Quality process (user, 2026-10-10): improve step by step, one item per look, toward "a dynamic nice-looking pocket world".

Next session (user, 2026-10-11):

- [x] Refactor — one file per simulation layer (elevation, hydro, climate/clouds, erosion, tectonics, life, render), dynamically composed at runtime so layers can be enabled/disabled individually — also groundwork for perf work and the parked wasm idea (specs/ideas.md). Done 2026-10-11: `world.rs` holds the multi-writer commons; layers own their grids as structs (`on` flags); `sim.rs` is the composition root (canonical tick order, single RNG stream, `TerrainOptions::disabled`); `render.rs` holds all views; sim files are crossterm-free. Behavior-preserving: all 56 lib tests green (2 new: runtime disable/re-enable, all-views render), census unchanged, PTY smoke clean.
- [x] `k` key — cycle to the previous layer (backward); `l` stays forward. Done with the refactor: `Layer::prev()`, HUD reads `[l/k]`, test cycles both directions.
- [ ] Dynamics pass — user: "I don't see terrain changing that much as I was expecting": plate uplift vs erosion throughput needs a tuning/mechanism look; verify on the plates + height layers where change should be visible

## Architecture — state and tick

The render `Buffer` is output, not state: rebuilt from fields each frame and diffed. State is split by ownership (2026-10-11 refactor): `world.rs` holds the multi-writer commons (elevation, water, sea level, clock, noise, thresholds); each layer file owns its private grids as a struct with an `on` flag; `sim.rs` composes them at runtime (`TerrainOptions::disabled` skips a layer's steps, freezing its state). Flat W×H grids (~4,800 cells at 120x40):

- `elevation` [0..1] — fBm + mountain mask at gen (land-stretched ×1.7); erosion and tectonics write later
- `water` — rain/springs in, evaporation/infiltration out (infiltration only on standing water), pipe fluxes move it
- `flux` — 4 pipe outfluxes per cell, persistent and damped; rivers have momentum because flux is state, not scratch
- `sediment` — erosion picks it up, pipes carry it, slow/drying water drops it
- `humidity` — decaying rainfall memory; biomes read it. Clouds move water, not fields
- `snow` — cryosphere: accumulates where cold + wet, melts when warm
- `clouds` — 1-cell particles spawned as fronts; the wind made visible
- `vegetation`, `fire` — growth, lightning/spread/ash-regrowth
- `springs` — mountain headwaters, constant river sources
- `plate` (round 5b, replaces the orogeny event list) — plate membership per cell; seeds carry drift velocities
- `class` flags — emergent ocean/lake labels, flood-filled lazily

Derived, never stored: wind = f(row, elevation, t) — latitude bands + swirl, slowed by mountain walls; temperature = f(y, elevation, t) — gradient + lapse + hemisphere-antisymmetric seasonal sine.

Tick order: ocean relax + land evaporation → cloud fronts (spawn, drift, rain, snowfall) → springs → snow melt → vegetation/fire → tectonics → pipes ×2 → sediment transport → erosion → talus → lazy classification. Pure arithmetic plus seeded stochastic rolls (spawns, lightning, quakes) — replays are bit-exact.

Rendering is a pure function of the fields (`render.rs`, crossterm only there and in `effect.rs`): the `l`/`k` keys cycle the layer views forward/backward through a `TerminalEffect::handle_key` hook (default no-op for other effects); HUD line names the active layer on row 0.

## Phase 1 — base map

Two-band fBm (continent `scale` + detail `detail_scale`, contrast-gained) on a cylinder via 3D Perlin — seamless x-wrap, test-verified. Domain warp bends ridgelines. Mountain mask added AFTER blending (blending compresses ridges below every threshold — the round-2 lesson): ridged chains where a mountain-field says so, tall enough to clear the rock/snow bands. Land heights stretched ×1.7 after sea selection. Sea level = height percentile (28% — land-dominant world). Biome thresholds = percentiles of the land distribution, so bands exist on every seed. `PerlinNoise` is seed-shuffled per instance (the original ignored its seed).

## Phase 2 — water

Pipe model (Mei et al. 2007): flux ∝ total-height difference, outflow clamped so a cell never drains dry, damped, sub-stepped. Rivers find valleys, depressions fill into lakes, floods overtop ridges. **The ocean is a boundary condition, not a conservation tank** — it relaxes toward fill depth; rain never chases the water budget. Infiltration clears standing sheets only — channelized flow is sealed (leaking it drained rivers 27× faster than springs could feed). Mountain springs guarantee headwaters by construction. Classification is emergent: ocean = connected to the sea fill; lake = isolated standing water; river = flux above threshold — motion, not a body.

## Phase 3 — erosion

Hydraulic: carrying capacity ∝ flow; the capacity gap erodes elevation into sediment, the surplus deposits; drying cells drop their whole load (floodplains, deltas). Sediment rides the outflow share-weighted. Talus: dry slopes past the angle of repose shed into scree. Solid mass conserved, test-verified.

## Phase 4 — weather: clouds

Clouds are 1-cell particles spawned as **fronts** (clusters of ~11) over warm ocean, riding the banded winds with turbulence — trains and pile-ups emerge from the shared flow; the sky is the wind made visible. Each particle rains its own cell gently (0.06/tick, ×2 against mountain walls), so fronts cross land raining instead of dying at the coast; cold cells take **snow** instead of rain (accumulates, melts in warm spells — poles and winters hold water, spring releases it). Dithered parity transparency (`▒`/`▓` on the checkerboard, terrain shining through) makes fronts shimmer. Rainfall writes the decaying humidity memory that biomes read. Mountains block wind: the field reads elevation; windward pile-up and rain-out, leeward rain-shadow deserts. Temperature: latitude gradient + lapse + seasons — derived, never stored.

## Phase 5 — tectonics, life

**Round 5b: lithospheric plates replace the random orogeny bumps.** ~7 plates seeded at world gen (Voronoi membership, wrap-x, seeded drift ~0.01 cells/tick). Membership refreshes lazily so boundaries migrate over minutes. At each boundary cell, the plates' relative velocity decomposes on the boundary normal: convergent → slow uplift along the edge (mountain chains grow exactly on plate fronts); divergent → subsidence (rift valleys that flood into lakes); transform → earthquake jolts. Interiors are stable cratons — deformation only at edges. Erosion stays the counterweight: ranges rise on convergent fronts, wear down as boundaries migrate away.

**Life**: vegetation grows where humid and warm; fire ignites from lightning under rain or prolonged hot dry spells, spreads through fueled neighbors, consumes to ash, ash fertilizes regrowth — self-balancing. Flooding is free through the pipes.

Stochastic events (front spawns, lightning, quakes) roll from the single seeded RNG stream: same seed replays the same history; `reset()` re-rolls a new world.

## Parameters

No simulation controls; the screensaver is autonomous. `TerrainOptions` (derive_builder + serde, config-overridable): `seed`, `scale`, `detail_scale`, `octaves`, `detail_octaves`, `persistence`, `warp_strength`, `warp_scale`. Layer cycling is a display control, not a simulation knob. Per-seed jitter for run-to-run variety.

## Terrain palette

Narrow (width-1) Unicode only — double-width glyphs break the 1-char-per-cell grid. Division of labor: **color carries elevation, glyph carries state**; in the heightmap layer **brightness = height, strictly**.

Layers (cycled with `l`): biome (default) / heightmap / hydro (depth + flux vectors) / temperature / wind / humidity — the tuning instruments.

**Heightmap layer spec (round 5a)**: brightness-monotonic hypsometric ramp, 8 hard steps; below sea in bathymetry blues (dark deep → lighter shelf), land dark-green → green → yellow-brown → grey → near-white peaks. Hillshade (fake NW sun from the elevation gradient) multiplies in — ridges catch light, valleys shadow. Density glyphs ` ·░▒▓` for monochrome terminals.

- water — deep `≈` dark blue → shallow `~` lighter; lake `~` teal; river `˃ ˄ ˂ ˅` by flux, cyan
- cloud — `▒`/`▓` by water, dithered over terrain; snow-pack `*`/`❄`
- shore `.`; grass `"`; forest `♣`; swamp `Ψ`; desert `∴`; rock `▲`; scree `°`; ash `░`; fire `✱ ❋ ✹` animated; rain `|`/`⁄`

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

Current state green (2026-10-10): 53 lib tests + 2 integration, clippy clean, release build, PTY quit, replay bit-exact, census with 1000+ river cells.

## Findings

Durable lessons (per-round history lives in git):

- **Session 2026-10-10/11 close (user)**: heightmap layer approved ("looks awesome, really sick"); plates layer + mechanics in; plates sped to 4× on request; overall verdict on the biome view remains "looks like a shit" — its palette/structure round is still owed. Standing complaint recorded: terrain changes less than expected (see Next session, dynamics pass).
- **The ocean is a boundary condition, not a conservation tank** — two failed rain-budget controllers proved it; the sea relaxes toward fill.
- **Infiltration must not leak channelized flow** — standing sheets soak, rushing rivers are sealed. Leaking them made rivers arithmetically impossible (27× spring inflow). This was the river breakthrough.
- **Concentrated per-cell rain** (1-cloud-1-cell, fronts) is what lets channels accumulate; disc-spread rain never could.
- **Blended mountain noise compresses below every threshold** — masks are added after blending, never averaged in.
- **Verify renders by glyph census**, never by escape-stripping diff streams — flicker over-represents and lies.
- **f32 identity failures**: `evap + (w − evap)` can exceed `w` by one ulp; every water removal clamps. Caught twice.
- Brightness-monotonic ramps only: pastel palettes with similar luminance read as mush (round-5 review).
- Ocean invariant is aggregate: the body keeps >85% of fill; individual straits legitimately run through-flow currents.

## Context

User direction: "2d liquids, weather, rivers, geological changes — dynamic, something interesting." Vision + DF comparison: issue #93.

Decisions (user, 2026-10-10):

- one task per feature: 0006 is the terrain working task; no per-phase subtasks
- cylinder topology; banded winds; no-flux poles; temperature by latitude + lapse; hemisphere-antisymmetric seasons
- ocean-fill initial state (~28% sea, land-dominant); emergent classification; ocean as boundary condition
- clouds are 1-cell particle fronts; no field atmosphere, no ellipse stamps
- mountains block wind; ranges are post-blend masks; tectonics (plates, round 5b) is the counterweight to erosion
- cryosphere: snow accumulates and melts into the hydrosphere
- color = elevation, glyph = state; brightness = height in the heightmap layer; layer cycling is a display control, no simulation controls
- stochastic events from the single seeded RNG stream; `reset()` re-rolls
- narrow-width Unicode glyphs only
- no accelerated-history boot — world runs at watchable speed from t=0
- quality process: step by step, one improvement per visual review

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
