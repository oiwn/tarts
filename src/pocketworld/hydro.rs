//! The hydrosphere (Phase 2): pipe-model water flow (Mei et al. 2007),
//! ocean boundary relaxation, land evaporation and infiltration,
//! mountain springs, and the emergent ocean/lake classification.
//! Rivers have momentum because flux is persistent state, not scratch.

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

/// Water depth above which a cell counts as water for classification.
pub(crate) const WET_MIN: f32 = 0.004;
/// Flux magnitude above which a wet cell counts as a river.
pub(crate) const RIVER_FLUX: f32 = 0.0012;
/// Recompute ocean connectivity every N ticks.
const CLASS_REFRESH: u32 = 90;
/// Constant headwater emission per mountain spring, per tick.
const SPRING_FLOW: f32 = 0.0015;

/// `class` flag: cell belongs to the connected ocean body.
pub(crate) const OCEAN: u8 = 1;

pub(crate) struct Hydro {
    /// Runtime enable flag (composed from `TerrainOptions`).
    pub(crate) on: bool,
    /// Persistent pipe outflux per cell: left, right, up, down.
    pub(crate) flux: Vec<[f32; 4]>,
    /// Per-cell classification flags (OCEAN; lakes are unflagged water).
    pub(crate) class: Vec<u8>,
    /// Mountain spring headwaters — constant river sources (round 4).
    pub(crate) springs: Vec<usize>,
}

impl Hydro {
    pub(crate) fn new(cells: usize) -> Self {
        Self {
            on: true,
            flux: vec![[0.0; 4]; cells],
            class: vec![0; cells],
            springs: Vec::new(),
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
        let mut springs = Vec::new();
        for cand in candidates {
            if springs.len() >= 8 {
                break;
            }
            let (cx, cy) = (cand % width, cand / width);
            let far_enough = springs.iter().all(|s| {
                let (sx, sy) = (s % width, s / width);
                let dx = (sx as isize - cx as isize).abs();
                let dy = (sy as isize - cy as isize).abs();
                dx + dy >= min_dist
            });
            if far_enough {
                springs.push(cand);
            }
        }
        self.springs = springs;
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
        for s in &self.springs {
            world.water[*s] += SPRING_FLOW;
        }
    }

    /// Pipe flow, sub-stepped for stability.
    pub(crate) fn pipes(&mut self, world: &mut World) {
        for _ in 0..PIPE_SUBSTEPS {
            self.pipes_once(world);
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
    pub(crate) fn classify_if_due(&mut self, world: &World) {
        if world.tick % CLASS_REFRESH == 0 {
            self.refresh_classification(world);
        }
    }

    /// Ocean = the largest 4-connected water body (wrap on x). Smaller
    /// flooded depressions stay unflagged and render as lakes.
    pub(crate) fn refresh_classification(&mut self, world: &World) {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pocketworld::sim::test_sim;

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
