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

/// Vegetation growth per tick where humid and warm, and its decay elsewhere.
const VEG_GROW: f32 = 0.003;
const VEG_DECAY: f32 = 0.0004;
/// Lightning ignition chance per raining cell with fuel; spread chance to a
/// fueled neighbor per tick; burn-down rate.
const LIGHTNING: f64 = 0.0004;
const FIRE_SPREAD: f64 = 0.35;
const FIRE_BURN: f32 = 0.04;

/// `fire` cell states: burning, and ash left behind.
pub(crate) const BURNING: u8 = 1;
pub(crate) const ASH: u8 = 2;

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
                        && rng.random::<f64>() < FIRE_SPREAD
                    {
                        ignitions.push(j);
                    }
                }
                if self.vegetation[i] <= 0.05 {
                    self.fire[i] = ASH;
                }
                continue;
            }

            // Growth: humid and warm grows; ash briefly fertilizes.
            let grow = VEG_GROW
                * (climate.humidity[i] * 8.0).clamp(0.0, 1.0)
                * (temp * 2.0).clamp(0.0, 1.0)
                * if self.fire[i] == ASH { 2.0 } else { 1.0 };
            self.vegetation[i] =
                (self.vegetation[i] + grow - VEG_DECAY).clamp(0.0, 1.0);
            if self.fire[i] == ASH && self.vegetation[i] > 0.2 {
                self.fire[i] = 0; // recovered
            }

            // Ignition: lightning under rain, or prolonged hot and dry.
            let fuel = self.vegetation[i] > 0.5;
            let lightning =
                fuel && climate.rain[i] > 1e-4 && rng.random::<f64>() < LIGHTNING;
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
