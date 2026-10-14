//! The hydrosphere (Phase 2): pipe-model water flow (Mei et al. 2007),
//! ocean boundary relaxation, land evaporation and infiltration,
//! mountain springs, and the emergent ocean/lake classification.
//! Rivers have momentum because flux is persistent state, not scratch.

use crate::pocketworld::tectonics::Tectonics;
use crate::pocketworld::world::World;

/// How fast an ocean cell relaxes back toward its fill depth: the ocean is a
/// boundary condition (connected to the unsimulated rest of the planet), not
/// a conservation tank. Rainfall never chases the world's water budget.
const OCEAN_RELAX: f32 = 0.15;

/// Flat evaporation rate (scaled by temperature in the climate layer).
const EVAP_RATE: f32 = 0.0002;
/// Extra sink for water sitting on land: soil infiltration. Flat wet ground
/// has no height gradient, so pipes alone cannot clear rain sheets.
const INFILTRATION: f32 = 0.0004;

// Pipe model constants (Mei et al. 2007).
/// Flux acceleration per unit of total-height difference.
const PIPE_ACCEL: f32 = 0.25;
/// Fraction of flux retained each step; damps oscillation.
const PIPE_DAMPING: f32 = 0.85;
/// Pipe sub-steps per tick, for stability.
const PIPE_SUBSTEPS: usize = 2;

// Aquifer (look 2b, 2026-10-14): the water table as vegetation
// moisture, independent of rain. Lowlands near sea level hold
// groundwater; rivers and lakes charge their banks.
/// Elevation band above sea level that holds a water table. Deepened
/// 2026-10-14 (living-lowlands round): 0.12 starved the forests — the
/// user wants woodland to own the wet lowlands.
const AQUIFER_DEPTH: f32 = 0.2;
/// Charge of cells carrying surface water (river/lake banks).
const AQUIFER_BANK: f32 = 0.8;
/// Relaxation toward the target per tick — water tables move slowly.
const AQUIFER_RELAX: f32 = 0.004;

/// Water depth above which a cell counts as water for classification.
pub(crate) const WET_MIN: f32 = 0.004;
/// Flux magnitude above which a wet cell counts as a river.
pub(crate) const RIVER_FLUX: f32 = 0.0012;
/// Smoothing factor for the render-side averages (flux and depth, per
/// tick): the raw fields oscillate with damped sloshing, which flaps
/// glyphs at every hard cutoff.
const RENDER_EMA: f32 = 0.3;
/// Recompute ocean connectivity every N ticks.
const CLASS_REFRESH: u32 = 90;
/// Cadence for relocating drowned headwaters: a spring rides its
/// plate, but the crust itself can subside below sea — a drowned
/// spring is relocated to the current best high ground within ~1.5 s.
const SPRING_REFRESH: u32 = 90;
/// Constant headwater emission per mountain spring, per tick.
const SPRING_FLOW: f32 = 0.0015;

/// `class` flag: cell belongs to the connected ocean body.
pub(crate) const OCEAN: u8 = 1;

// Drainage network (rivers round, 2026-10-14). Pipe-model water dies
// within ~8 cells of a spring (evaporation outpaces the headwater), so
// pipe rivers never reached the sea: 1.4-3.2% of land ever held water.
// Rivers are read off the terrain instead: steepest descent through a
// pit-filled surface, flow accumulated downstream.
/// Upstream contribution threshold (in cells of catchment) above which
/// a land cell is a river.
pub(crate) const RIVER_ACCUM: f32 = 26.0;
/// Extra catchment per unit of rain memory (humidity), so wet regions
/// grow denser networks than dry interiors. Kept modest: at 6.0 the
/// polar cell's heavy snowfall memory weighted every cell ~7× and short
/// slope columns all qualified (a fan of parallel arrows).
const RAIN_ACCUM: f32 = 1.5;
/// Catchment credited at each mountain spring — headwaters start
/// rivers even on small slopes.
const SPRING_ACCUM: f32 = 16.0;
/// Rise per cell imposed when filling pits, so every filled surface
/// still drains toward its outlet.
const FILL_EPS: f32 = 1e-5;
/// Amplitude and spatial frequency of the faint relief added to the
/// routing surface (valleys ~6 cells apart, depth ~ a gentle slope's
/// drop over two cells).
const ROUTE_RELIEF: f32 = 0.012;
const ROUTE_NOISE_SCALE: f64 = 0.18;
/// `receiver` sentinel: outlet (sea) or unreached.
const NO_RECEIVER: u32 = u32::MAX;
/// Fill depth above which a river-fed cell of the filled surface is a
/// lake rather than a channel.
const LAKE_DEPTH: f32 = 0.002;
/// `river` value for a lake cell (directions use 1..=4).
pub(crate) const LAKE: u8 = 5;
/// Catchment smoothing per drainage rebuild.
const ACCUM_EMA: f32 = 0.15;
/// A river cell stays a river down to this fraction of `RIVER_ACCUM`.
const RIVER_KEEP: f32 = 0.4;
/// A flanking same-direction river must carry this much more catchment
/// to suppress a parallel line.
const LATERAL_MARGIN: f32 = 1.3;
/// Flank strength needed to drop a line that was a river last rebuild.
const LATERAL_MARGIN_KEEP: f32 = 2.5;
/// A cell keeps its previous drainage route while that drop is at least
/// this fraction of the steepest available.
const ROUTE_STICKY: f32 = 0.5;

pub(crate) struct Hydro {
    /// Runtime enable flag (composed from `PocketWorldOptions`).
    pub(crate) on: bool,
    /// Persistent pipe outflux per cell: left, right, up, down.
    pub(crate) flux: Vec<[f32; 4]>,
    /// Slow average of `flux` for rendering and river classification
    /// only — the raw fluxes oscillate tick to tick (damped sloshing),
    /// which flips the flow-glyph argmax and shimmers the channels.
    /// The simulation itself (erosion, transport, infiltration) reads
    /// the raw flux.
    pub(crate) flux_ema: Vec<[f32; 4]>,
    /// Slow average of water depth for rendering only — the sea
    /// sloshes tick to tick (boundary-condition relax), which flaps
    /// the deep/shallow cutoffs and the depth-ramp bands.
    pub(crate) water_ema: Vec<f32>,
    /// Per-cell classification flags (OCEAN; lakes are unflagged water).
    pub(crate) class: Vec<u8>,
    /// Mountain spring headwaters — constant river sources (round 4).
    /// Float positions: they advect with their plate so headwaters
    /// keep their mountains as the crust drifts.
    pub(crate) springs: Vec<(f32, f32)>,
    /// Drainage receiver per cell: the neighbour it drains into on the
    /// pit-filled surface (`NO_RECEIVER` at sea).
    receiver: Vec<u32>,
    /// River cells of the drainage network: 0 none, else the flow
    /// direction + 1 (0 left, 1 right, 2 up, 3 down — flux order), or
    /// `LAKE` for a river-fed filled pit.
    pub(crate) river: Vec<u8>,
    /// Catchment per cell, smoothed across drainage rebuilds.
    accum_ema: Vec<f32>,
    /// Drainage route per land cell (direction + 1, 0 none), kept
    /// across rebuilds for sticky routing.
    route: Vec<u8>,
}

impl Hydro {
    pub(crate) fn new(cells: usize) -> Self {
        Self {
            on: true,
            flux: vec![[0.0; 4]; cells],
            flux_ema: vec![[0.0; 4]; cells],
            water_ema: vec![0.0; cells],
            class: vec![0; cells],
            springs: Vec::new(),
            receiver: vec![NO_RECEIVER; cells],
            river: vec![0; cells],
            accum_ema: Vec::new(),
            route: Vec::new(),
        }
    }

    /// Rebuild the drainage network from the terrain: priority-flood
    /// from the sea (wrap on x, poles bound) assigns each land cell a
    /// receiver on the pit-filled surface — depressions fill and spill
    /// instead of trapping flow — then catchment accumulates downstream
    /// in reverse flood order. Cells above `RIVER_ACCUM` are rivers:
    /// connected from headwaters to the coast by construction. Runs on
    /// the crust-advection cadence so rivers ride the terrain.
    /// `gather` is the last crust advection's source map (empty if the
    /// crust didn't move): the network's memory (smoothed catchment,
    /// river cells) rides the crust through it, so hysteresis follows
    /// a drifting river instead of lagging at its old cells.
    pub(crate) fn drainage(
        &mut self,
        world: &World,
        humidity: &[f32],
        gather: &[u32],
    ) {
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;

        let (w, h) = (world.w, world.h);
        let n = w * h;
        if gather.len() == n && self.accum_ema.len() == n {
            self.accum_ema =
                gather.iter().map(|&j| self.accum_ema[j as usize]).collect();
            self.river = gather.iter().map(|&j| self.river[j as usize]).collect();
            self.route = gather.iter().map(|&j| self.route[j as usize]).collect();
        }
        if self.route.len() != n {
            self.route = vec![0; n];
        }
        let land = |i: usize| world.elevation[i] >= world.sea_level;
        // The routing surface: terrain plus faint smooth relief. On a
        // planar tilted flank every cell's only downhill neighbour is
        // straight ahead — flow ran in side-by-side columns (a 10-wide
        // band of parallel arrows); the noise carves shallow valleys so
        // flow converges into branching streams. Cylinder-sampled.
        let radius = w as f64 / std::f64::consts::TAU;
        let route_e: Vec<f32> = (0..n)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let (s, c) = (x as f64 / radius).sin_cos();
                let v = world.noise.octave_noise_3d(
                    radius * c + 71.3,
                    y as f64,
                    radius * s + 18.9,
                    2,
                    0.5,
                    ROUTE_NOISE_SCALE,
                ) as f32;
                world.elevation[i].max(0.0) + ROUTE_RELIEF * (v + 1.0)
            })
            .collect();
        // Heap key: f32 bits order like the floats for non-negative
        // values (the routing surface is ≥ 0), index breaks ties —
        // deterministic replays.
        let mut filled = vec![f32::INFINITY; n];
        let mut heap = BinaryHeap::new();
        for (i, f) in filled.iter_mut().enumerate() {
            if !land(i) {
                *f = route_e[i];
                heap.push(Reverse((f.to_bits(), i as u32)));
            }
        }
        self.receiver.fill(NO_RECEIVER);
        let mut order: Vec<usize> = Vec::with_capacity(n);
        while let Some(Reverse((_, i))) = heap.pop() {
            let i = i as usize;
            order.push(i);
            let (x, y) = (i % w, i / w);
            for (dx, dy) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                let ny = y as isize + dy;
                if ny < 0 || ny >= h as isize {
                    continue;
                }
                let nx = (x as isize + dx).rem_euclid(w as isize) as usize;
                let j = ny as usize * w + nx;
                if filled[j].is_finite() {
                    continue; // already reached (or sea)
                }
                filled[j] = route_e[j].max(filled[i] + FILL_EPS);
                heap.push(Reverse((filled[j].to_bits(), j as u32)));
            }
        }

        // Receivers: steepest descent on the filled surface, sticky.
        // The flood parent alone flipped with discovery order on gentle
        // slopes — a 0.001 erosion change swapped routes and strobed
        // the network. A cell keeps its previous route (carried by the
        // crust) while that neighbour is still downhill and at least
        // `ROUTE_STICKY` as steep as the best. Filled heights strictly
        // fall along every route (the flood parent always qualifies),
        // so no cycles, and reverse flood order stays topological.
        for i in 0..n {
            if !land(i) {
                self.route[i] = 0;
                continue;
            }
            let (x, y) = (i % w, i / w);
            let mut drops = [None::<(usize, f32)>; 4];
            for (dir, (dx, dy)) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)]
                .into_iter()
                .enumerate()
            {
                let ny = y as isize + dy;
                if ny < 0 || ny >= h as isize {
                    continue;
                }
                let nx = (x as isize + dx).rem_euclid(w as isize) as usize;
                let j = ny as usize * w + nx;
                if filled[j] < filled[i] {
                    drops[dir] = Some((j, filled[i] - filled[j]));
                }
            }
            let best = drops
                .iter()
                .enumerate()
                .filter_map(|(d, o)| o.map(|(j, drop)| (d, j, drop)))
                .max_by(|a, b| a.2.total_cmp(&b.2));
            let Some((mut dir, mut j, best_drop)) = best else {
                continue;
            };
            let prev = self.route[i];
            if prev != 0
                && let Some((pj, pdrop)) = drops[prev as usize - 1]
                && pdrop >= best_drop * ROUTE_STICKY
            {
                dir = prev as usize - 1;
                j = pj;
            }
            self.receiver[i] = j as u32;
            self.route[i] = dir as u8 + 1;
        }

        let mut accum: Vec<f32> = (0..n)
            .map(|i| {
                if land(i) {
                    1.0 + RAIN_ACCUM * humidity.get(i).copied().unwrap_or(0.0)
                } else {
                    0.0
                }
            })
            .collect();
        for &(sx, sy) in &self.springs {
            let x = (sx.rem_euclid(w as f32)) as usize;
            let y = (sy.clamp(0.0, (h - 1) as f32)) as usize;
            accum[y * w + x] += SPRING_ACCUM;
        }
        for &i in order.iter().rev() {
            let r = self.receiver[i];
            if r != NO_RECEIVER {
                accum[r as usize] += accum[i];
            }
        }
        // Stability: on gentle slopes small erosion changes reorder the
        // flood and swap routes; raw rebuilds flipped 71% of river
        // cells each time (probed 2026-10-14). The catchment is
        // smoothed across rebuilds, and a river takes a margin to enter
        // and to leave.
        if self.accum_ema.len() != n {
            self.accum_ema = accum.clone();
        }
        for (e, a) in self.accum_ema.iter_mut().zip(accum.iter()) {
            *e += (a - *e) * ACCUM_EMA;
        }
        let accum = &self.accum_ema;

        let was = std::mem::replace(&mut self.river, vec![0; n]);
        for i in 0..n {
            let r = self.receiver[i];
            let bar = if was[i] != 0 {
                RIVER_ACCUM * RIVER_KEEP
            } else {
                RIVER_ACCUM
            };
            if r == NO_RECEIVER || !land(i) || accum[i] <= bar {
                continue;
            }
            // A filled pit fed by a river is a lake: the flood leveled
            // it, and flow paths across a flat run side by side — they
            // drew as blocks of parallel arrows.
            if filled[i] - route_e[i] > LAKE_DEPTH {
                self.river[i] = LAKE;
                continue;
            }
            self.river[i] = self.route[i];
        }

        // Uniform slopes route as side-by-side flow lines (4-neighbour
        // descent), and each collects enough catchment to qualify —
        // they drew as bands of parallel arrows. Keep the strongest
        // line: a river cell flanked (across its flow) by a stronger
        // river flowing the same way is dropped. Along a band the
        // ranking is stable, so the surviving stem stays continuous.
        let mut keep = self.river.clone();
        for i in 0..n {
            let r = self.river[i];
            if r == 0 || r == LAKE {
                continue;
            }
            let (x, y) = (i % w, i / w);
            let lateral: [(isize, isize); 2] = if r <= 2 {
                [(0, -1), (0, 1)] // flows sideways: flanks are up/down
            } else {
                [(-1, 0), (1, 0)]
            };
            for (dx, dy) in lateral {
                let ny = y as isize + dy;
                if ny < 0 || ny >= h as isize {
                    continue;
                }
                let nx = (x as isize + dx).rem_euclid(w as isize) as usize;
                let j = ny as usize * w + nx;
                // Hysteretic too: an established line takes a much
                // stronger flank to drop than a new one takes to block.
                let margin = if was[i] != 0 {
                    LATERAL_MARGIN_KEEP
                } else {
                    LATERAL_MARGIN
                };
                if self.river[j] == r && accum[j] > accum[i] * margin {
                    keep[i] = 0;
                }
            }
        }
        self.river = keep;
    }

    /// Seed the water table at generation: lowlands near sea level
    /// hold groundwater from day one (no slow boot-up charge).
    pub(crate) fn init_aquifers(&self, world: &mut World) {
        for i in 0..world.aquifer.len() {
            world.aquifer[i] = aquifer_target(world, i, self.river[i] != 0);
        }
    }

    /// One tick: relax the water table toward its target — the table
    /// follows the terrain as it erodes and drifts, slowly.
    pub(crate) fn charge_aquifers(&self, world: &mut World) {
        for i in 0..world.aquifer.len() {
            let target = aquifer_target(world, i, self.river[i] != 0);
            world.aquifer[i] += (target - world.aquifer[i]) * AQUIFER_RELAX;
        }
    }

    /// Place mountain springs at generation: the highest cells, spaced
    /// apart, each feeding a constant headwater — rivers are guaranteed
    /// by construction and carved by erosion, joined by cloud rain and
    /// polar melt.
    pub(crate) fn place_springs(&mut self, world: &World) {
        let width = world.w;
        let mut candidates: Vec<usize> = (0..world.elevation.len())
            .filter(|i| world.elevation[*i] > world.sea_level)
            .collect();
        candidates
            .sort_by(|a, b| world.elevation[*b].total_cmp(&world.elevation[*a]));
        let min_dist = 8;
        let mut springs: Vec<(f32, f32)> = Vec::new();
        for cand in candidates {
            if springs.len() >= 12 {
                break;
            }
            let (cx, cy) = (cand % width, cand / width);
            let far_enough = springs.iter().all(|s| {
                let dx = (s.0 as isize - cx as isize).abs();
                let dy = (s.1 as isize - cy as isize).abs();
                dx + dy >= min_dist
            });
            if far_enough {
                springs.push((cx as f32, cy as f32));
            }
        }
        self.springs = springs;
    }

    /// Springs ride their plate: headwater positions advect with the
    /// crust on the same cadence (and by the same displacement) as the
    /// terrain gather, so rivers keep their mountains instead of being
    /// stranded by drifting topography.
    pub(crate) fn ride_plates(&mut self, world: &World, tectonics: &Tectonics) {
        for s in &mut self.springs {
            let (x, y) = *s;
            let xi = x.rem_euclid(world.w as f32) as usize;
            let yi = (y.clamp(0.0, (world.h - 1) as f32)) as usize;
            // The crust moves in whole cells (lossless advection); the
            // spring takes exactly its plate's last shift.
            let p = tectonics.plates[tectonics.plate[yi * world.w + xi] as usize];
            *s = (
                (x + p.shift.0 as f32).rem_euclid(world.w as f32),
                (y + p.shift.1 as f32).clamp(0.0, (world.h - 1) as f32),
            );
        }
    }

    /// The ocean relaxes toward fill (boundary condition); land water
    /// evaporates and infiltrates away — evaporation no longer feeds a
    /// field: clouds are spawned by the ocean directly (climate layer).
    pub(crate) fn drain(&self, world: &mut World) {
        for i in 0..world.water.len() {
            let fill = (world.sea_level - world.elevation[i]).max(0.0);
            if fill > 0.0 {
                world.water[i] += (fill - world.water[i]) * OCEAN_RELAX;
            } else if world.water[i] > 0.0 {
                let evap = EVAP_RATE.min(world.water[i]);
                // Infiltration clears standing sheets, not channelized flow:
                // a rushing river is sealed by its own water and sediment;
                // leaking it makes every channel drain faster than springs
                // can feed (measured: 27x).
                let flux_mag: f32 = self.flux[i].iter().filter(|f| **f > 0.0).sum();
                let infil = if flux_mag > RIVER_FLUX {
                    0.0
                } else {
                    INFILTRATION.min(world.water[i] - evap).max(0.0)
                };
                // evap + (w - evap) can exceed w by one ulp in f32.
                world.water[i] = (world.water[i] - (evap + infil)).max(0.0);
            }
        }
    }

    /// Mountain springs: guaranteed headwaters — rivers by construction,
    /// carved by erosion, fed forever.
    pub(crate) fn emit_springs(&self, world: &mut World) {
        for &(sx, sy) in &self.springs {
            let x = (sx.rem_euclid(world.w as f32)) as usize;
            let y = (sy.clamp(0.0, (world.h - 1) as f32)) as usize;
            world.water[y * world.w + x] += SPRING_FLOW;
        }
    }

    /// Relocate drowned headwaters: a spring rides its plate, but the
    /// crust under it can subside below sea (rifts, isostasy). A
    /// drowned spring moves to the highest cell that keeps its distance
    /// from the other springs — deterministic, no RNG.
    pub(crate) fn maintain_springs(&mut self, world: &World) {
        if world.tick % SPRING_REFRESH != 0 {
            return;
        }
        let min_elev = world.sea_level + 0.05;
        for si in 0..self.springs.len() {
            let (sx, sy) = self.springs[si];
            let x = (sx.rem_euclid(world.w as f32)) as usize;
            let y = (sy.clamp(0.0, (world.h - 1) as f32)) as usize;
            if world.elevation[y * world.w + x] > min_elev {
                continue;
            }
            let mut best: Option<usize> = None;
            let mut best_e = min_elev;
            for i in 0..world.elevation.len() {
                let e = world.elevation[i];
                if e <= best_e {
                    continue;
                }
                let (cx, cy) = (i % world.w, i / world.w);
                let spaced =
                    self.springs.iter().enumerate().all(|(j, &(ox, oy))| {
                        j == si || {
                            let dx = (ox as isize - cx as isize).abs();
                            let dy = (oy as isize - cy as isize).abs();
                            dx + dy >= 8
                        }
                    });
                if spaced {
                    best = Some(i);
                    best_e = e;
                }
            }
            if let Some(i) = best {
                self.springs[si] = ((i % world.w) as f32, (i / world.w) as f32);
            }
        }
    }

    /// Pipe flow, sub-stepped for stability.
    pub(crate) fn pipes(&mut self, world: &mut World) {
        for _ in 0..PIPE_SUBSTEPS {
            self.pipes_once(world);
        }
        // Refresh the render-side averages: classification and flow
        // glyphs read a smooth signal, the simulation reads the raw.
        for (ema, f) in self.flux_ema.iter_mut().zip(self.flux.iter()) {
            for d in 0..4 {
                ema[d] += (f[d] - ema[d]) * RENDER_EMA;
            }
        }
        for (we, d) in self.water_ema.iter_mut().zip(world.water.iter()) {
            *we += (*d - *we) * RENDER_EMA;
        }
    }

    /// One pipe-model sub-step (Mei et al. 2007): accelerate fluxes by total
    /// height differences, damp them, then apply — never draining a cell dry.
    fn pipes_once(&mut self, world: &mut World) {
        let width = world.w;
        let height = world.h;

        // Update fluxes from height differences.
        for y in 0..height {
            for x in 0..width {
                let i = y * width + x;
                let total = world.elevation[i] + world.water[i];
                for (dir, (dx, dy)) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)]
                    .into_iter()
                    .enumerate()
                {
                    let ny = y as isize + dy;
                    if ny < 0 || ny >= height as isize {
                        self.flux[i][dir] = 0.0; // no-flux pole boundary
                        continue;
                    }
                    let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                    let j = ny as usize * width + nx;
                    let diff = total - (world.elevation[j] + world.water[j]);
                    let f = &mut self.flux[i][dir];
                    *f = *f * PIPE_DAMPING + PIPE_ACCEL * diff;
                }
            }
        }

        // Apply positive outflow, clamped to available water.
        for y in 0..height {
            for x in 0..width {
                let i = y * width + x;
                let outflow: f32 = self.flux[i].iter().filter(|f| **f > 0.0).sum();
                if outflow <= 0.0 {
                    continue;
                }
                let scale = if outflow > world.water[i] {
                    world.water[i] / outflow
                } else {
                    1.0
                };
                for (dir, (dx, dy)) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)]
                    .into_iter()
                    .enumerate()
                {
                    let f = self.flux[i][dir];
                    if f <= 0.0 {
                        continue;
                    }
                    let dv = f * scale;
                    let ny = y as isize + dy;
                    if ny < 0 || ny >= height as isize {
                        continue;
                    }
                    let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                    let j = ny as usize * width + nx;
                    world.water[i] = (world.water[i] - dv).max(0.0);
                    world.water[j] += dv;
                }
            }
        }
    }

    /// Lazy classification refresh, every `CLASS_REFRESH` ticks.
    pub(crate) fn classify_if_due(&mut self, world: &mut World) {
        if world.tick % CLASS_REFRESH == 0 {
            self.refresh_classification(world);
        }
    }

    /// Ocean = the largest 4-connected water body (wrap on x). Smaller
    /// flooded depressions stay unflagged and render as lakes.
    pub(crate) fn refresh_classification(&mut self, world: &mut World) {
        let width = world.w;
        let height = world.h;
        self.class.iter_mut().for_each(|c| *c &= !OCEAN);

        let mut visited = vec![false; self.class.len()];
        let mut best: Option<(usize, Vec<usize>)> = None;

        for start in 0..self.class.len() {
            if visited[start] || world.water[start] <= WET_MIN {
                continue;
            }
            // Flood fill this water body.
            let mut component = Vec::new();
            let mut stack = vec![start];
            visited[start] = true;
            while let Some(i) = stack.pop() {
                component.push(i);
                let (x, y) = (i % width, i / width);
                for (dx, dy) in [(-1isize, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if ny < 0 || ny >= height as isize {
                        continue; // poles bound the cylinder
                    }
                    let nx = nx.rem_euclid(width as isize) as usize;
                    let j = ny as usize * width + nx;
                    if !visited[j] && world.water[j] > WET_MIN {
                        visited[j] = true;
                        stack.push(j);
                    }
                }
            }
            match &best {
                Some((size, _)) if *size >= component.len() => {}
                _ => best = Some((component.len(), component)),
            }
        }

        if let Some((_, component)) = best {
            for i in component {
                self.class[i] |= OCEAN;
            }
        }

        // Continentality: BFS distance from the sea over land (wrap
        // on x, poles bound). The aquifer fades with it — wet coasts,
        // dry interiors — so the hot belt gets its jungle → savanna →
        // desert rings instead of one uniform biome (2026-10-14).
        // Rivers and lakes keep charging their banks un-faded: green
        // corridors through the arid interior.
        world.dist_sea.fill(u16::MAX);
        let mut queue = std::collections::VecDeque::new();
        for i in 0..self.class.len() {
            if self.class[i] & OCEAN != 0 {
                world.dist_sea[i] = 0;
                queue.push_back(i);
            }
        }
        while let Some(i) = queue.pop_front() {
            let (x, y) = (i % width, i / width);
            let d = world.dist_sea[i];
            for (dx, dy) in [(-1isize, 0), (1, 0), (0, -1), (0, 1)] {
                let ny = y as isize + dy;
                if ny < 0 || ny >= height as isize {
                    continue;
                }
                let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                let j = ny as usize * width + nx;
                if world.dist_sea[j] > d + 1 {
                    world.dist_sea[j] = d + 1;
                    queue.push_back(j);
                }
            }
        }
    }
}

/// The water-table target for one cell: high near sea level, full
/// under surface water (river/lake banks), zero up the mountains.
/// The elevation term fades with distance from the sea — interiors
/// run dry (continentality) while river corridors stay charged.
fn aquifer_target(world: &World, i: usize, river: bool) -> f32 {
    let rel = world.elevation[i] - world.sea_level;
    let fade = (-(world.dist_sea[i].min(80) as f32) / 5.0).exp();
    let table = ((AQUIFER_DEPTH - rel) / AQUIFER_DEPTH).clamp(0.0, 1.0) * fade;
    let bank = if rel >= 0.0 && (river || world.water[i] > WET_MIN) {
        AQUIFER_BANK
    } else {
        0.0
    };
    table.max(bank)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pocketworld::sim::test_sim;

    #[test]
    fn rivers_reach_the_sea_and_hold_still() {
        // Rivers-round lock (2026-10-14): pipe water died within ~8
        // cells of a spring, so rivers covered 1.4-3.2% of land at best
        // and never reached the coast. The drainage network must cover
        // a real share of land, drain every river cell to the sea, and
        // not strobe between rebuilds.
        for seed in [7u32, 42] {
            let mut t = test_sim(seed, 100, 40);
            for _ in 0..1500 {
                t.tick();
            }
            let n = t.world.elevation.len();
            let land = (0..n)
                .filter(|&i| t.world.elevation[i] >= t.world.sea_level)
                .count();
            let rivers = t.hydro.river.iter().filter(|r| **r != 0).count();
            // Every river cell's receiver chain ends at the sea — checked
            // on a fresh rebuild (between rebuilds, coastal deposition
            // can lift an outlet above sea).
            let humidity = t.climate.humidity.clone();
            t.hydro.drainage(&t.world, &humidity, &[]);
            for i in (0..n).filter(|&i| t.hydro.river[i] != 0) {
                let (mut c, mut steps) = (i, 0);
                while t.hydro.receiver[c] != NO_RECEIVER {
                    c = t.hydro.receiver[c] as usize;
                    steps += 1;
                    assert!(steps <= n, "drainage cycle from cell {i}");
                }
                assert!(
                    t.world.elevation[c] < t.world.sea_level,
                    "river at {i} drains to a land dead end"
                );
            }
            // Over 20 rebuilds, against the previous networks carried by
            // the crust shift (rivers riding a plate are motion): churn
            // is cells flipped per rebuild; strobe is a flip that undoes
            // the one before it (A→B→A). Deforming margins make rivers
            // wander (~14% churn, probed 2026-10-14) — that's allowed;
            // flicker is not (~3% of flips revert).
            let (mut flips, mut reverts, mut rebuilds) = (0usize, 0usize, 0usize);
            let mut prev = t.hydro.river.clone();
            let mut prev2 = prev.clone();
            for _ in 0..300 {
                let rebuild = t.world.tick
                    % crate::pocketworld::tectonics::PLATE_REFRESH
                    == 0;
                t.tick();
                if !rebuild {
                    continue;
                }
                if t.tectonics.gather.len() == n {
                    let g = &t.tectonics.gather;
                    prev = g.iter().map(|&j| prev[j as usize]).collect();
                    prev2 = g.iter().map(|&j| prev2[j as usize]).collect();
                }
                for i in 0..n {
                    let now = t.hydro.river[i] != 0;
                    if now != (prev[i] != 0) {
                        flips += 1;
                        if now == (prev2[i] != 0) {
                            reverts += 1;
                        }
                    }
                }
                rebuilds += 1;
                prev2 = std::mem::replace(&mut prev, t.hydro.river.clone());
            }
            let churn =
                flips as f32 / rebuilds.max(1) as f32 / rivers.max(1) as f32;
            let strobe = reverts as f32 / flips.max(1) as f32;
            eprintln!(
                "RIVERS seed {seed}: land {land} rivers {rivers} ({:.1}%) churn/rebuild {:.1}% strobe {:.1}% of flips",
                100.0 * rivers as f32 / land as f32,
                100.0 * churn,
                100.0 * strobe
            );
            assert!(rivers * 50 > land, "rivers too sparse: {rivers}/{land}");
            assert!(churn < 0.3, "river network churns: {churn}");
            assert!(strobe < 0.1, "river network strobes: {strobe}");
        }
    }

    #[test]
    fn ocean_classified_and_filled() {
        let t = test_sim(5, 80, 24);
        let ocean_cells = t.hydro.class.iter().filter(|c| **c & OCEAN != 0).count();
        assert!(ocean_cells > 50, "ocean too small: {ocean_cells} cells");
        for (i, c) in t.hydro.class.iter().enumerate() {
            if c & OCEAN != 0 {
                assert!(t.world.water[i] > WET_MIN, "ocean cell without water");
            }
        }
    }
}
