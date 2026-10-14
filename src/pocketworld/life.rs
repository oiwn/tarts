//! Life (Phase 5): vegetation and fire. Growth follows humidity and
//! warmth; fire ignites from lightning under rain or prolonged hot dry
//! spells, spreads through fueled neighbors, consumes them to ash, and
//! ash fertilizes regrowth — self-balancing, seeded-deterministic.
//! Flooding reaches here for free through the hydro pipes.

use crate::pocketworld::climate::{Climate, temp_at};
use crate::pocketworld::hydro::WET_MIN;
use crate::pocketworld::world::World;
use rand::RngExt;
use rand::rngs::StdRng;

/// Vegetation growth per tick where moist and warm, and its decay
/// elsewhere. Raised 2026-10-14 (living lowlands): forests own the
/// wet belt — target ≥25% of land at 3000 frames.
const VEG_GROW: f32 = 0.0035;
const VEG_DECAY: f32 = 0.0004;
/// Lightning ignition chance per raining cell with fuel; spread chance to a
/// fueled neighbor per tick; burn-down rate. Lowered 2026-10-14 (look
/// 2b): aquifer moisture tripled the fuel base, and the old rates
/// burned whole continents to ash (9.6% of the canvas) — fire is a
/// patchy disturbance, not a plague.
const LIGHTNING: f64 = 0.00004;
const FIRE_SPREAD: f64 = 0.06;
const FIRE_BURN: f32 = 0.06;

/// `fire` cell states: burning, and ash left behind.
pub(crate) const BURNING: u8 = 1;
pub(crate) const ASH: u8 = 2;

/// Cell moisture for life: rain memory or the water table, whichever
/// is wetter. The aquifer term is concave — groundwater sustains
/// forest well beyond the immediate coast (linear scaling kept
/// woodland a coastal sliver, 2026-10-14). Shared with the biome
/// render: the desert/savanna gates must read the same moisture the
/// vegetation grows on, or the tropics paint arid over jungle.
pub(crate) fn moisture(climate: &Climate, world: &World, i: usize) -> f32 {
    climate.humidity[i].max(world.aquifer[i].powf(0.55))
}

/// Wet fuel doesn't burn: fire spread resistance from moisture, 1
/// when bone dry → 0.2 over saturated ground. Keeps the fuel/fire
/// coupling one-sided — growing more forest on wet ground must not
/// grow the burn (2026-10-14, the aquifer round's lesson: every fuel
/// increase lit the map).
fn fire_wetness(world: &World, climate: &Climate, i: usize) -> f64 {
    (1.0 - moisture(climate, world, i) * 1.2).clamp(0.04, 1.0) as f64
}

pub(crate) struct Life {
    /// Runtime enable flag (composed from `TerrainOptions`).
    pub(crate) on: bool,
    /// Vegetation biomass [0..1].
    pub(crate) vegetation: Vec<f32>,
    /// Fire state per cell: 0 none, BURNING, ASH.
    pub(crate) fire: Vec<u8>,
}

impl Life {
    pub(crate) fn new(cells: usize) -> Self {
        Self {
            on: true,
            vegetation: vec![0.0; cells],
            fire: vec![0; cells],
        }
    }

    pub(crate) fn step(
        &mut self,
        world: &World,
        climate: &Climate,
        rng: &mut StdRng,
    ) {
        let width = world.w;
        let height = world.h;
        let mut ignitions: Vec<usize> = Vec::new();

        for i in 0..self.vegetation.len() {
            let y = i / width;
            let temp = temp_at(world, y, i, world.time);
            let is_land = world.elevation[i] > world.sea_level;
            let dry_land = is_land && world.water[i] <= WET_MIN;
            if !dry_land {
                self.vegetation[i] =
                    (self.vegetation[i] - VEG_DECAY * 5.0).max(0.0);
                continue;
            }

            if self.fire[i] == BURNING {
                // Burn down and spread through fueled neighbors.
                self.vegetation[i] = (self.vegetation[i] - FIRE_BURN).max(0.0);
                let x = i % width;
                for (dx, dy) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                    let ny = y as isize + dy;
                    if ny < 0 || ny >= height as isize {
                        continue;
                    }
                    let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                    let j = ny as usize * width + nx;
                    if self.fire[j] == 0
                        && self.vegetation[j] > 0.3
                        && rng.random::<f64>()
                            < FIRE_SPREAD * fire_wetness(world, climate, j)
                    {
                        ignitions.push(j);
                    }
                }
                if self.vegetation[i] <= 0.05 {
                    self.fire[i] = ASH;
                }
                continue;
            }

            // Growth: moisture (rain memory OR the water table — trees
            // follow groundwater, not clouds, look 2b 2026-10-14) and
            // warmth; ash briefly fertilizes; established neighbors
            // seed outward, so forests read as belts and clumps.
            let moist = moisture(climate, world, i);
            // Dry-scaled decay: semi-dry cells must equilibrate BELOW
            // the fuel threshold, or the whole grass belt slowly
            // saturates into tinder and one ignition torches a region
            // (the 2026-10-14 ash storms). Wet cells decay at the
            // base rate — forests keep growing to full canopy.
            let moist_f = (moist * 8.0).clamp(0.0, 1.0);
            let decay = VEG_DECAY * (1.0 + 12.0 * (1.0 - moist_f));
            let mut neighbors = 0.0f32;
            {
                let x = i % width;
                for (dx, dy) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
                    let ny = y as isize + dy;
                    if ny < 0 || ny >= height as isize {
                        continue;
                    }
                    let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                    let j = ny as usize * width + nx;
                    if self.vegetation[j] > 0.3 {
                        neighbors += 1.0;
                    }
                }
            }
            let grow = VEG_GROW
                * (moist * 8.0).clamp(0.0, 1.0)
                * (temp * 2.0).clamp(0.0, 1.0)
                * (1.0 + 0.5 * neighbors / 4.0)
                * if self.fire[i] == ASH { 2.0 } else { 1.0 };
            self.vegetation[i] =
                (self.vegetation[i] + grow - decay).clamp(0.0, 1.0);
            if self.fire[i] == ASH && self.vegetation[i] > 0.2 {
                self.fire[i] = 0; // recovered
            }

            // Ignition: lightning under rain (wet canopies rarely
            // catch — the same wetness resistance as spread), or
            // prolonged hot and dry.
            let fuel = self.vegetation[i] > 0.5;
            let lightning = fuel
                && climate.rain[i] > 1e-4
                && rng.random::<f64>()
                    < LIGHTNING * fire_wetness(world, climate, i);
            let heat = fuel
                && temp > 0.9
                && climate.humidity[i] < 0.01
                && rng.random::<f64>() < LIGHTNING * 0.5;
            if lightning || heat {
                ignitions.push(i);
            }
        }

        for i in ignitions {
            self.fire[i] = BURNING;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pocketworld::effect::PocketWorldOptionsBuilder;
    use crate::pocketworld::sim::{Sim, SimLayer};

    #[test]
    fn aquifer_grows_forests_without_rain() {
        // Look 2b regression: with the climate layer off (no rain, no
        // humidity), lowland vegetation must still grow on the water
        // table — trees follow groundwater, not clouds.
        let options = PocketWorldOptionsBuilder::default()
            .seed(9u32)
            .disabled(vec![SimLayer::Climate])
            .build()
            .unwrap();
        let mut t = Sim::new(&options, (100, 40));
        for _ in 0..600 {
            t.tick();
        }
        let sea = t.world.sea_level;
        let lowland_green = (0..t.world.elevation.len())
            .filter(|&i| {
                let rel = t.world.elevation[i] - sea;
                rel > 0.0 && rel < 0.06 && t.life.vegetation[i] > 0.2
            })
            .count();
        assert!(
            lowland_green > 50,
            "no water-table forests: {lowland_green} lowland green cells"
        );
    }
}
