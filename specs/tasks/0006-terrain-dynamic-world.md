---
id: 0006-terrain-dynamic-world
status: draft
scope: []
created: 2026-10-09
source: https://github.com/oiwn/tarts/issues/93
---
# Task: terrain dynamic world

## Plan

Design task — stays in draft until the user reviews the vision (issue #93) and picks scope. Likely split into phase tasks:

- [ ] Phase 1: fBm + domain-warp base map, animated, wire `terrain` into `VALID_SAVERS` (fixes the unreachable effect)
- [ ] Phase 2: water — pipe-model flow on a global water budget (sea level, rainfall rate); lakes, rivers
^^^ how? depends on amount of water? i would like kinda full mini world simulation
&&& Pipe model (Mei et al. 2007): each cell has 4 virtual pipes to its
neighbors; flux ∝ difference in total height (elevation + water depth), outflow clamped so a cell never goes negative. One rule gives rivers finding valleys, depressions filling into lakes, floods overtopping ridges — no pathfinding. "Amount of water" is a first-class knob: a global budget (sea-level fill + rainfall) decides ocean world vs lake districts vs arid washes, and water cycles through the world (rain in, evaporation out) instead of sitting as a static layer. See Open questions for where initial water comes from.
^^^ let's move it into separate section with proper description
- [ ] Phase 3: hydraulic erosion + sediment transport, talus (slope) erosion
^^^ how?
&&& Erosion rides the same fluxes: when flow through a cell is strong, its
carrying capacity (≈ k·|flow|) exceeds held sediment → erode elevation into suspended load; where flow slows (flats, lakes, ocean) capacity drops → deposit. Evaporating water drops its load on the spot → floodplains and deltas emerge on their own. Talus erosion is the cheap companion: slopes past an angle of repose shed material. This is the phase where the map starts looking carved rather than generated.
^^^ move into separate section
- [ ] Phase 4: weather — wind + moisture advection, rain fronts, evaporation; biome ramp reads (h, water, moisture, vegetation)
^^^ how?
&&& Moisture field advected by a wind field (prevailing direction + noise
swirl); evaporation off water bodies proportional to area/temperature; rain out where moisture saturates — fronts visibly drift across the map. Biome glyph/color reads the fields directly, so climate shifts biomes: deserts creep, swamps form, shores green. Rain cells are drawable glyphs themselves.
^^^ move to the separate section
- [ ] Phase 5: vegetation + fire; flooding events; tectonic events (uplift/subsidence along a drifting ridge, earthquakes); overlays, options, benchmarks
^^^ forest fire maybe also? flooding.
&&& Both are cheap in this model. Fire: vegetation grows where moist;
ignition from lightning or prolonged hot/dry; spreads to adjacent vegetation, consumes it to ash, ash briefly fertilizes regrowth — self-balancing and very watchable. Flooding: mostly free — heavy rain bursts or a lake overtopping a ridge floods downstream through the pipes; scripted flash-flood/sea-level-rise events add drama.
^^^ move into the separate section

^^^ are you aware how map generated in games like Dwarf Fortress? it run full history simulation also. I would like to have something like that, but without civilizations for now...
&&& Yes — DF stamps statistical fields (elevation noise, rainfall, drainage,
temperature), traces rivers downhill by flow accumulation, fills lakes, then runs centuries of event history on top; the geology itself is painted, not run. Our approach is weaker on story, stronger on physics: one shared grid of fields (elevation, water, sediment, moisture, vegetation, fire), evolved forward by local rules every tick — the history IS the simulation, visible as it happens. No civilizations. No accelerated-history boot either (user decision, 2026-10-10): the world runs at watchable speed from t=0; interesting starting conditions are handled by initial-state generation instead (see Open questions).

^^^ add "Reference" section

## Acceptance

To be defined per phase task when split out. Global bar from issue #93:

- [ ] ~60 FPS at 120x40 on a laptop; sim may run sub-rate
- [ ] Deterministic from `seed`; `reset()` re-rolls
- [ ] PTY smoke + degenerate-size safe (see 0004)

## Context

User direction: "2d liquids (Navier-Stokes?), weather, rivers and other geological changes — dynamic, something interesting." Each phase is its own PR. The full concept, rendering, and constraints live in issue #93.

Core model (agreed in discussion 2026-10-10): one shared grid, one cell per terminal character, fields per cell — elevation `h`, water depth `d`, suspended sediment `s`, moisture `m`, vegetation `v`, fire/ash state. Every phase is stencil updates on those fields; no particles, no meshes. Perf: ~4,800 cells at 120x40 with small stencils is well under 1 ms/tick in Rust; water may sub-step 2-3x for stability.

## Open questions

User-raised 2026-10-10, to settle before or during Phase 1/2 planning:

- **World topology**: torus (wrap both axes — no edges, cleanest for advection/wind), cylinder (wrap x only, poles at top/bottom), or bounded edges (rivers can run off-map, weather enters from a side). Affects wind, moisture advection, river outlets, and rendering of edges.
- **Initial state / where water comes from**: ocean fill to a sea level? scattered lakes in depressions? springs at high elevation? rain-only start on a dry world? Options likely: `water_budget`, `sea_level`, `rain_rate`.
- **Terrain types palette**: the biome set and glyphs/colors — which types (deep/shallow water, shore, grass, forest, swamp, desert, rock, scree, snow, ash, fire...) and how many the biome ramp should carry.
- **Time scale**: geology-in-minutes knobs (`erosion_rate`, `time_scale`) and how they interact with the 60 FPS budget (sim may run sub-rate).

## Summary

## Log

- 2026-10-09 created: terrain dynamic world
- 2026-10-09 set source https://github.com/oiwn/tarts/issues/93
