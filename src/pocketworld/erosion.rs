//! Erosion (Phase 3): hydraulic erosion through the pipe model, and
//! dry-slope talus. Solid mass is conserved — material only moves
//! between `elevation` and this layer's `sediment` (test-verified).

use crate::pocketworld::hydro::{Hydro, WET_MIN};
use crate::pocketworld::world::World;

// Deliberately untuned — visual pass comes after the whole system runs;
// adjust with the map on screen, not blind.
/// Suspended-load carrying capacity per unit of outflux. Lowered
/// 2026-10-12 (relief round): with halved uplift, the old rate ground
/// ranges flat over a long session.
const EROSION_CAPACITY: f32 = 0.28;
/// Fraction of the capacity gap eroded/deposited per tick.
const ERODE_K: f32 = 0.3;
const DEPOSIT_K: f32 = 0.5;
/// How much of a cell's sediment rides the outflow each tick.
const SEDIMENT_TRANSPORT: f32 = 0.5;
/// Talus (dry slope) erosion: height difference past the angle of
/// repose. Rate lowered with the relief round — slow the grinding.
const TALUS_SLOPE: f32 = 0.06;
const TALUS_RATE: f32 = 0.12;

pub(crate) struct Erosion {
    /// Runtime enable flag (composed from `TerrainOptions`).
    pub(crate) on: bool,
    /// Suspended sediment per cell.
    pub(crate) sediment: Vec<f32>,
}

impl Erosion {
    pub(crate) fn new(cells: usize) -> Self {
        Self {
            on: true,
            sediment: vec![0.0; cells],
        }
    }

    /// One tick: sediment rides the pipe currents, flowing water erodes
    /// and deposits, then dry slopes shed into scree.
    pub(crate) fn step(&mut self, world: &mut World, hydro: &Hydro) {
        self.transport(world, hydro);
        self.erode(world, hydro);
        self.talus(world);
    }

    /// Sediment rides the pipe outflow, proportionally to each direction's
    /// share of the total.
    fn transport(&mut self, world: &mut World, hydro: &Hydro) {
        let width = world.w;
        let height = world.h;
        for y in 0..height {
            for x in 0..width {
                let i = y * width + x;
                let out: f32 = hydro.flux[i].iter().filter(|f| **f > 0.0).sum();
                if out <= 0.0 || self.sediment[i] <= 0.0 {
                    continue;
                }
                let frac =
                    SEDIMENT_TRANSPORT.min(out / (world.water[i] + out).max(1e-6));
                let s_move = self.sediment[i] * frac;
                self.sediment[i] -= s_move;
                for (dir, (dx, dy)) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)]
                    .into_iter()
                    .enumerate()
                {
                    let f = hydro.flux[i][dir];
                    if f <= 0.0 {
                        continue;
                    }
                    let ny = y as isize + dy;
                    if ny < 0 || ny >= height as isize {
                        continue;
                    }
                    let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                    self.sediment[ny as usize * width + nx] += s_move * (f / out);
                }
            }
        }
    }

    /// Hydraulic erosion: carrying capacity grows with flow; the gap between
    /// capacity and held sediment erodes bedrock, the surplus deposits.
    /// Drying cells drop their whole load (floodplains, deltas). The
    /// deep sea is a sediment SINK — river loads arriving below sea
    /// level are buried and return to the mantle (the rock cycle's
    /// closing half; without it, deposition accreted the ocean floor
    /// into land: the ocean halved within 900 ticks).
    fn erode(&mut self, world: &mut World, hydro: &Hydro) {
        for i in 0..world.elevation.len() {
            if world.elevation[i] < world.sea_level {
                self.sediment[i] = 0.0; // buried at sea
                continue;
            }
            if world.water[i] > WET_MIN {
                let flow: f32 = hydro.flux[i].iter().filter(|f| **f > 0.0).sum();
                let capacity = EROSION_CAPACITY * flow;
                let s = self.sediment[i];
                if capacity > s {
                    let erode = ((capacity - s) * ERODE_K).min(0.002);
                    world.elevation[i] -= erode;
                    self.sediment[i] += erode;
                } else {
                    let dep = (s - capacity) * DEPOSIT_K;
                    world.elevation[i] += dep;
                    self.sediment[i] -= dep;
                }
            } else if self.sediment[i] > 0.0 {
                world.elevation[i] += self.sediment[i];
                self.sediment[i] = 0.0;
            }
        }
    }

    /// Talus erosion: dry slopes past the angle of repose shed material
    /// downhill into scree.
    fn talus(&mut self, world: &mut World) {
        let width = world.w;
        let height = world.h;
        for y in 0..height {
            for x in 0..width {
                let i = y * width + x;
                if world.water[i] > WET_MIN {
                    continue;
                }
                for (dx, dy) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                    let ny = y as isize + dy;
                    if ny < 0 || ny >= height as isize {
                        continue;
                    }
                    let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                    let j = ny as usize * width + nx;
                    let diff = world.elevation[i] - world.elevation[j];
                    if diff > TALUS_SLOPE {
                        let move_m = (diff - TALUS_SLOPE) * TALUS_RATE * 0.25;
                        world.elevation[i] -= move_m;
                        world.elevation[j] += move_m;
                    }
                }
            }
        }
    }
}
