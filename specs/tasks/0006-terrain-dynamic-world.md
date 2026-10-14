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
- [x] Water-representation polish (2026-10-12) — ocean/lake decided by geometry (below sea = sea), water depth EMA, channel-gated flow arrows. Strobe eliminated (0 ocean-depth flips per refresh window).
- [x] Climate/wind round (2026-10-12) — **wind is now a simulated momentum field**: self-advection, thermal band forcing, Coriolis (∝ sin lat), orographic/land drag, turbulence. General circulation emerges (locked by test). Approved: "wind now looks more realistic."
- [x] Sky & ice + brightness/variety rounds B/C/C2 (2026-10-12) — cohesive cloud coverage (soft stamps + EMA), color-milk translucency (checkerboard dither dead), cloud vocabulary; cryosphere bands with temp hue; brightness pass; per-object glyph/color variety via stable position hash. Approved: "WOW, this is AWESOME."
- [x] Keys + legend (2026-10-12) — `j` prev / `k` next / `l` toggles a full legend screen (own page, not an overlay).
- [x] Relief round (2026-10-12/13) — sea-from-base, mask strengthened, erosion slowed; bugs fixed along the way: sediment-deficit mass-pin payout, erosion conveyor filling the ocean (burial sink), **eustasy**. The stretch-saturation plateau flaw was resolved by the height round below.
- [x] Height-distribution round (2026-10-13) — **root-curve soft saturation** replaces the capped ×2.4 stretch (normalize by the pre-map max — the real plateau culprit was the pre-compression `min(1.0)` clamp parking the mask's overflow on one atom); **rock strength** scales hydraulic + talus erosion ×[0.15..1] by elevation so ranges resist; height-view ramp re-anchored to the rock level (85th land percentile, refreshed with eustasy). Analyzer-verified: four land bands at boot (~17% each), all persisting at 3000 frames (`▒` was extinct, now 9%), seed 7 same shape; high country now grows (boot 478 → 1500-tick 960 cells above boot rock level; locked by `mountains_survive_long_runs`).
- [x] Mountain-band + forests round (2026-10-13) — **thresholds frame bug**: rock/scree levels were stored absolute but compared against relative relief — `rel > rock_level` was unreachable, rock/scree never rendered on any seed; now stored relative. Frost bar `temp < -0.05 → -0.5` and snow-pack tier `0.14 → 0.22` (the ranges were blank white); rock/scree get grey faces (green ramp made them vanish). Trees: clouds rain their visible 3×3 (was 1 cell — 94% of land never wet), humidity memory 0.0007 → 0.00025 (climate, not weather), forest gate retuned to the footprint. Analyzer: `▲∆` 0% → 9.8% of canvas, scree 5%, forests 2.4% → 5.8% ≈ 12% of land @1500 (but 3.2% @3000 — the fire cycle eats them; the life round owns that balance).
- [x] View glyph-band fix (2026-10-13) — temp + humidity truncation bug (`t as usize` → `t * 3.0`): temp view went from 95.8% one band to 45/29/26% across three latitude bands. Analyzer-verified.
- [x] Palette & light round, look 1 (2026-10-14) — rain animation removed entirely (user: "clouds looks nice, we don't need rain" — rain still falls in the sim, renders as nothing); brightness pass (rock 150→236 grey, scree/grass/canopy lifted, orphaned `ramp_height` deleted — every biome owns its color now); cryosphere ladder rebalanced (`*` 0.16 / `❄` 0.26 / `▓` 0.36, frost bar −0.15 — all three tiers render, caps back on peaks); desert sand-yellow, savanna `;`/`:` its own biome, swamp its own murky green.
- [x] Mountain flicker + caps, look 2a (2026-10-14) — **sticky land tiers**: the renderer keeps per-cell tier memory (lowland/scree/rock) with entry/leave margins (30% of the inter-threshold gaps), so ±0.002/tick erosion jitter can no longer flip cells across `rock_level`/`scree_level` (the flicker; `classify` went `&mut`, locked by `tier_hysteresis_resists_jitter`). Frost bar raised to temp < 0.05 on tier 2 — mid-latitude ranges now wear caps. Side effects, measured: forests 5.8% → 10% of canvas (~20% of land — the sticky scree band stopped over-claiming), biome CHANGE 65.6% → 61.9%.
- [x] Aquifers + forest belts, look 2b (2026-10-14) — **aquifer field** (world commons; water table by elevation `AQUIFER_DEPTH 0.12` + river/lake banks, slow relax, initialized at gen); vegetation moisture = max(rain humidity, aquifer·0.6) with neighbor establishment (+50% per seeded neighbor) — trees follow groundwater, not clouds (locked: `aquifer_grows_forests_without_rain` grows lowland forest with the climate layer dead). Fire re-balanced for the wider fuel base (LIGHTNING 0.0004→0.00004, SPREAD 0.35→0.15, BURN 0.04→0.06): ash 9.6%→2-4% (patchy scars, quick recovery). Forests 10.6%→**15% of canvas @1500, 10.7% @3000 (~30%/21% of land)** — no long-run collapse. Accepted: "ok now looks good".
- [x] Living lowlands round (2026-10-14) — **aquifer view**: 8th layer (`·` dry / `░` damp / `▒` saturated, blue-green ramp, HUD hint, `views()` grew to 8, cycle tests updated). **Living grass**: color follows biomass × seasonal temperature — meadows green in and dry out over hundreds of ticks, glyphs stay position-hashed (no flicker). **Fire equilibrium took three structural fixes**: wet-fuel spread resistance, dry-scaled vegetation decay (semi-dry cells equilibrate below fuel — no tinder belt), wet-canopy ignition resistance + sub-critical spread in moist fuel (0.06 base × clamp(1−moist·1.2)). Fire is a rare patchy disturbance.
- [x] Forest expansion + continentality (2026-10-14, "not enough forests") — three levers: water table deeper (`AQUIFER_DEPTH` 0.16→0.2) and more concave (`aquifer^0.55`), woodland renders younger (veg 0.35); **moisture unified** between life and render (the desert/savanna gates read raw humidity and painted arid over groundwater jungle); **continentality**: BFS distance-to-sea (`world.dist_sea`, refreshed with the ocean classification) fades the water table exponentially (÷5) while rivers keep charging their banks — jungle coasts, savanna rings, dry interiors, green river corridors. Result: forests **~28% of canvas (~55-60% of land) sustained @3000** on both seeds, ash ≤1%; savanna/desert appear where continents are wide (seed 7: 2.8%+; narrow-maritime seed 42 stays green — geography, not a bug).
- [ ] Default seed 7 + glyph diversity (2026-10-14, user) — default world seed 42 → 7 (the wide-continent ring structure becomes the boot world; analyzer default follows; recorded baselines stay reproducible via `--seed 42`); biome glyph vocabulary expands — forest `♣♠§`, grass + `` ` `` `¸`, desert `∴∵∷`, rock `▲∆∧`, scree `°˚·`, swamp `Ψψξ`, ocean three depth bands `~≋≈` — all width-1, position-hashed; legend + palette section updated; gate: distinct-glyph census @3000 seed 7 ~41 → 46+.
- [ ] Mountain round (user: mountains first) — measure rock + height-band shares by latitude first; then ridged/spine detail in the mountain mask/uplift so orogens read as ranges, not domes; height ramp re-anchored to ~98th land percentile (`rock_level` at band 6); caps only on the upper part of tier 2 (height gate + temp); investigate sudden mountains (2a objection 3). Gates @1500/3000, seeds 7 + 42: rock visible at |lat| < 0.6, height `▓` ≤ ~10% of land, caps ≤ ~⅓ of the rock tier.
- [ ] Rivers round — why the flux/depth gates hide channels; target visible connected spring-to-sea rivers, arrows ≥ ~2% of land.
- [ ] Biome variety round — grass/savanna/desert survive in interiors @3000 (grass goes extinct on long runs); then the biome palette verification look.
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

Current state green (2026-10-14): 74 lib tests + 2 integration, clippy clean, release build, PTY quit, replay exact; forests ~55-60% of land sustained, fire at rare-disturbance equilibrium, continentality shaping biomes. Status: in-progress (implement).

## Findings

Durable lessons (per-round history lives in git):

- **Analyzer before tuning** (user, 2026-10-13): probe-number tuning missed the land plateau until a human looked — the agent cannot see renders; the analysis facility (0010) is the feedback loop.
- **Sea level must come from the base field, before mountains** — a percentile of the masked field rises with the mountains and eats the relief; then the headroom cap flattens the rest into a plateau.
- **Mass pins must be step-local, never against a gen-time total** — erosion parks mass in the sediment field; paying that "deficit" every advection lifted the whole seafloor (+0.42; ocean collapsed to 45 cells).
- **Free mass needs a free sink, and the sea is the rock cycle's sink** — the uplift→erosion→coastal-deposition conveyor filled the ocean until below-sea sediment became burial (returned to the mantle).
- **Eustasy** — sea relaxing toward its target fraction of the live crust is the ocean's homeostat against whatever the crust does.
- **Terrain must ride the plates** (advection) or the heightmap traces the plate network; semi-Lagrangian self-advection of wind gives meanders/eddies cheaply; polar precipitation can't wait for the cloud fleet (probed: zero clouds past |lat| 0.5) — the polar cell delivers its own snowfall; sublimation bounds the caps; naive hashes show lattice patterns (use a mixing hash).
- Focus mode (`disabled` via config) is the tuning workflow; the ocean is a boundary condition; infiltration must not leak channelized flow; concentrated per-cell rain forms channels; masks add after blending; verify by glyph census; f32 removals clamp; brightness-monotonic ramps only.
- **Threshold frames must match** (2026-10-13): levels stored absolute but read as relief made the entire rock/scree cascade dead code — silent, because the fallbacks rendered plausibly. Same family as the desert-green debt (comparisons vs paint). When a branch "has never fired", grep both frames before tuning anything.
- **User objections to 2a (2026-10-14, address when the full picture is in)**: (1) white mountain caps too wide — snow should sit only on the highest ranges, not most of the rock tier; (2) the height view is too white up top — "high" must be rarer (ramp currently crowns the 85th-percentile band); (3) mountains sometimes appear very suddenly (orogen stamps + sticky-tier entry margin have no gradual transition — needs investigation).
- **Plateau post-mortem (2026-10-13 height round)**: the atom's root was the pre-compression `min(1.0)` clamp — the mountain mask pushes a large land mass past 1.0 and the clamp parks it all on one elevation, whatever curve maps it afterwards (tanh saturation merely re-creates the atom in f32: the top 15% rounds to one value, putting the 85th percentile at the max). Fix: mask uncapped, normalize land rel by its true max, root-map (`^0.7`) into the headroom. Second half: erosion was flat-rate — streams+talus ground ranges to the plain; rock strength (erosion ×[0.15..1] by elevation, keyed to the rock level) flips the balance from decay to growth (boot 478 → 960 high cells at 1500 ticks). Height churn stays high (94% of cells cross band edges per sample — bands are ~0.04 wide) — if it reads as shimmer, the fix is render-side band hysteresis, not slower sim.
- **2026-10-14 diagnosis (seed 7, 100x40, after forest expansion):** (1) no mid-latitude mountains — rock `▲∆` 2.2% @300 → 0% @1500/3000; tier 2 survives only at the poles under snow; mid-latitude high ground is broad smooth domes just under `rock_level` (85th land pct, owned by the polar ranges), painted forest. (2) Height view `▓` = 17% of canvas (~33% of land): the ramp tops out at `rock_level`, so `t ≥ 0.75·rock_level` is already `▓` — objection 2's cause. (3) Caps cover all of tier 2 when `temp < 0.05` — width follows temperature, not height (objection 1). (4) Rivers 0.1-0.5% at every frame count (gates: `flux_ema > RIVER_FLUX && d < 0.08`). (5) `♠♣Ψψ` ≈ 85% of land @3000; grass 11% of canvas @300 → 0% @3000; desert 0.2%. (6) Analyzer blind spots: HUD row 0 adds ~25 junk census glyphs (0012), CHANGE compares far-apart samples (drift + shimmer mixed, no flicker metric), shares diluted by ~48% ocean — 0013 adds the skill + metrics.
- Known render debts: desert/swamp painted with the green elevation ramp; rivers thin; temp reads static.
- **Measured baseline (2026-10-13, analyzer seed 42, 100×40, default options — focus config NOT applied):** height@3000: land renders only `·` 30% + `░` 23%, zero `▒`/`▓`/`█` cells; worse, `▒` is 9% at frame 300 and decays to 0 by 3000 — erosion eats the mid band live. temp@900: 95.8% of cells in ONE ramp band — the ramp/normalization shows no spatial structure. biome@1500 (life on): grass family 37.7% of canvas, forests (`♠`+`♣`) 4.8% ≈ 12% of land, rock 1.8%. hydro@900: flow arrows ~3.1%. Churn paradox: 60–95% of cells change per sample yet the screen reads static — in-place glyph jitter with no feature motion. Fix order: height distribution (gen-time stretch + run-time band loss) → temp ramp → desert color → life round.
- **Causes pinned 2026-10-13 (code + data):** (1) temp/humidity one-band collapse is a truncation bug — `match t as usize` with `t ∈ [0,1]` puts everything below 1.0 in band 0 and makes band 2 unreachable; height view's `(t * 8.0) as usize` is the correct pattern (`* 3.0` missing). (2) Height ramp spans `rel / 0.75` but populated land relief ≈ [0, 0.25] — the ramp range is ~3× the field's; fix the distribution first, then rescale the ramp by land percentiles. (3) Wind view reads the instantaneous momentum field — no render EMA (the strobe lesson), so 93% of cells flip per sample; 97.9% of arrows render zonal, hiding the meridional component. (4) Seed 7 reproduces the zero-upper-bands height collapse — structural, not seed luck. Determinism verified: the baseline reproduces byte-identically per seed. (5) Post-fix: humidity's bands are reachable but the field is genuinely sparse — 94% of cells carry no recent-rain memory at 900 frames; vegetation keyed to humidity will stay rare until this broadens. Life-round material.

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
- 2026-10-13 advance blocked → in-progress/implement
