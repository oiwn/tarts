//! Climate and weather (Phase 4): the cloud fleet, the cryosphere, the
//! humidity memory, and the derived temperature/wind fields. Clouds are
//! 1-cell particles spawned as fronts over warm ocean, riding the
//! banded winds with turbulence — the sky is the wind made visible.
//! Temperature and wind are derived, never stored: f(latitude,
//! elevation, time).

use crate::terrain::hydro::WET_MIN;
use crate::terrain::world::World;
use rand::RngExt;
use rand::rngs::StdRng;

// Deliberately untuned — the user dials it in visually.
/// Ocean evaporation fills a cloud-spawn budget; this is the fill per tick.
const CLOUD_SPAWN: f32 = 0.5;
/// New-cloud water content and the fleet size cap.
const CLOUD_WATER: f32 = 0.35;
const CLOUD_MAX: usize = 250;
/// Fraction of a cloud rained out per tick over land.
const CLOUD_RAIN: f32 = 0.06;
/// Extra rain-out multiplier when mountains block the path.
const OROGRAPHIC: f32 = 2.0;
/// Particles per spawned cloud front (a visible traveling system).
const CLOUD_FRONT: usize = 11;
/// Elevation above sea level that counts as a mountain wall.
const MOUNTAIN_WALL: f32 = 0.22;
/// Soil-humidity memory: written by rain, decays each tick.
const HUMID_DECAY: f32 = 0.001;
/// Wind slowdown factor at a full mountain wall.
const WIND_BLOCK: f32 = 0.35;
/// Snow melts above this temperature, at this rate (scaled by temperature).
const SNOW_MELT_T: f32 = 0.05;
const SNOW_MELT: f32 = 0.0015;
/// Year length in seconds of screen time.
const YEAR: f64 = 120.0;
/// Seasonal temperature anomaly amplitude (hemisphere-antisymmetric sine).
const SEASON_AMPLITUDE: f32 = 0.15;
/// Temperature model: equator value, pole gradient, elevation lapse.
const TEMP_EQUATOR: f32 = 1.0;
const TEMP_POLE_DROP: f32 = 1.2;
const TEMP_LAPSE: f32 = 0.6;
/// Below this temperature the biome view paints snow on high ground.
pub(crate) const TEMP_SNOW: f32 = 0.15;

/// A drifting rain cloud particle: position and water content.
pub(crate) struct Cloud {
    x: f32,
    y: f32,
    water: f32,
}

pub(crate) struct Climate {
    /// Runtime enable flag (composed from `TerrainOptions`).
    pub(crate) on: bool,
    /// Soil/air humidity memory: written by rainfall, decays; biomes read it.
    pub(crate) humidity: Vec<f32>,
    /// Rain intensity per cell from the last tick (for rendering).
    pub(crate) rain: Vec<f32>,
    /// Snow-pack: accumulates where cold + wet, melts into the hydrosphere.
    pub(crate) snow: Vec<f32>,
    /// Per-cell cloud coverage from the last tick (for rendering).
    pub(crate) cloud_cover: Vec<f32>,
    /// The cloud fleet — 1-cell particles; formations emerge from the wind.
    pub(crate) clouds: Vec<Cloud>,
    /// Ocean-evaporation budget toward spawning the next cloud.
    pub(crate) spawn_budget: f32,
}

impl Climate {
    pub(crate) fn new(cells: usize) -> Self {
        Self {
            on: true,
            humidity: vec![0.0; cells],
            rain: vec![0.0; cells],
            snow: vec![0.0; cells],
            cloud_cover: vec![0.0; cells],
            clouds: Vec::new(),
            spawn_budget: 0.0,
        }
    }

    /// Tick prologue: decay the humidity memory and clear the rain field,
    /// before the clouds write both afresh.
    pub(crate) fn begin(&mut self) {
        for i in 0..self.humidity.len() {
            self.humidity[i] = (self.humidity[i] - HUMID_DECAY).max(0.0);
            self.rain[i] = 0.0;
        }
    }

    /// The cloud fleet: the wind made visible, carrying water from sea to
    /// land. Spawn from an ocean-evaporation budget (warmer water fills it
    /// faster), drift, rain, refill over warm sea, die empty or at the poles.
    pub(crate) fn clouds(&mut self, world: &mut World, rng: &mut StdRng) {
        let width = world.w;
        let height = world.h;

        self.cloud_cover.iter_mut().for_each(|c| *c = 0.0);

        // Spawn: budget fills with ocean warmth; each unit buys one FRONT —
        // a cluster of particles in a blob, so clouds read as traveling
        // systems instead of lone dots.
        self.spawn_budget += CLOUD_SPAWN;
        while self.spawn_budget >= 1.0 && self.clouds.len() < CLOUD_MAX {
            self.spawn_budget -= 1.0;
            for _ in 0..20 {
                let x = rng.random_range(0..width.max(1));
                let y = rng.random_range(0..height.max(1));
                let i = y * width + x;
                let warm = temp_at(world, y, i, world.time).clamp(0.0, 1.0);
                if world.water[i] > WET_MIN && warm > 0.4 {
                    let room = CLOUD_MAX - self.clouds.len();
                    for _ in 0..CLOUD_FRONT.min(room) {
                        let fx = (x as f32 + rng.random_range(-4.0..4.0))
                            .rem_euclid(width as f32);
                        let fy = (y as f32 + rng.random_range(-3.0..3.0))
                            .clamp(0.0, (height - 1) as f32);
                        self.clouds.push(Cloud {
                            x: fx,
                            y: fy,
                            water: CLOUD_WATER * (0.5 + rng.random_range(0.0..1.0)),
                        });
                    }
                    break;
                }
            }
        }

        let mut expired: Vec<usize> = Vec::new();
        // Take the fleet out so cloud state and world fields can be borrowed
        // freely inside the loop.
        let mut clouds = std::mem::take(&mut self.clouds);
        for (ci, cloud) in clouds.iter_mut().enumerate() {
            let (u, v) = wind_at(world, cloud.x as usize, cloud.y as usize);
            // Light turbulence so particle trains meander.
            cloud.x += u + rng.random_range(-0.15..0.15);
            cloud.y += v + rng.random_range(-0.15..0.15);

            // Poles are a wall: blown past one, the particle rains out.
            if cloud.y < 0.0 || cloud.y >= height as f32 {
                expired.push(ci);
                continue;
            }
            // Wrap the cylinder.
            cloud.x = cloud.x.rem_euclid(width as f32);

            let cell = cloud.y as usize * width + cloud.x as usize;
            let over_ocean = world.water[cell] > WET_MIN;
            let temp = temp_at(world, cloud.y as usize, cell, world.time);

            if over_ocean {
                // Refill over warm water; rain on the sea is reabsorbed by
                // the ocean (boundary condition).
                let warm = temp.clamp(0.0, 1.0);
                cloud.water = (cloud.water + 0.004 * warm).min(CLOUD_WATER);
            } else {
                // Rain the single cell below — concentrated rain is what
                // lets rivers form. Mountain walls squeeze harder.
                let rel = world.elevation[cell] - world.sea_level;
                let wall = if rel > MOUNTAIN_WALL { OROGRAPHIC } else { 1.0 };
                let rain = cloud.water * CLOUD_RAIN * wall;
                cloud.water -= rain;
                if temp < 0.0 {
                    // Cryosphere: cold cells take snow instead of water.
                    self.snow[cell] += rain;
                } else {
                    world.water[cell] += rain;
                    self.humidity[cell] += rain;
                    self.rain[cell] = self.rain[cell].max(rain);
                }
                if cloud.water <= 0.02 {
                    expired.push(ci);
                    continue;
                }
            }

            // One cell per particle; density by water content.
            let density = (cloud.water / CLOUD_WATER).clamp(0.0, 1.0);
            self.cloud_cover[cell] = self.cloud_cover[cell].max(density);
        }

        for ci in expired.into_iter().rev() {
            clouds.remove(ci);
        }
        self.clouds = clouds;
    }

    /// Cryosphere: snow melts where warm, feeding the hydrosphere —
    /// poles and winters hold water as ice, spring releases it.
    pub(crate) fn snowmelt(&mut self, world: &mut World) {
        let width = world.w;
        for i in 0..self.snow.len() {
            if self.snow[i] <= 0.0 {
                continue;
            }
            let y = i / width;
            let temp = temp_at(world, y, i, world.time);
            if temp > SNOW_MELT_T {
                let melt = (SNOW_MELT * (temp - SNOW_MELT_T)).min(self.snow[i]);
                self.snow[i] -= melt;
                world.water[i] += melt;
            }
        }
    }
}

/// Temperature by latitude, elevation lapse, and season. Summer in the
/// north while winter in the south (hemisphere-antisymmetric sine).
pub(crate) fn temp_at(world: &World, y: usize, i: usize, t: f64) -> f32 {
    let height = world.h.max(1) as f32;
    let lat = ((y as f32 + 0.5) / height) * 2.0 - 1.0; // -1 .. 1, pole to pole
    let rel = (world.elevation[i] - world.sea_level).max(0.0);
    let season = SEASON_AMPLITUDE
        * (std::f64::consts::TAU * t / YEAR).sin() as f32
        * lat.signum();
    TEMP_EQUATOR - TEMP_POLE_DROP * lat.abs() - TEMP_LAPSE * rel + season
}

/// Latitude-banded wind at a cell: trades / westerlies / polar
/// easterlies, plus a slow noise swirl for meandering fronts.
pub(crate) fn wind_at(world: &World, x: usize, y: usize) -> (f32, f32) {
    let height = world.h.max(1) as f32;
    let lat = ((y as f32 + 0.5) / height) * 2.0 - 1.0;
    let a = lat.abs();
    let band = if a < 0.35 {
        -1.0
    } else if a < 0.75 {
        1.0
    } else {
        -1.0
    };
    let swirl =
        world
            .noise
            .noise_3d(x as f64 * 0.1, y as f64 * 0.1, world.time * 0.05)
            as f32;
    let v_swirl = world.noise.noise_3d(
        x as f64 * 0.1 + 50.0,
        y as f64 * 0.1 + 50.0,
        world.time * 0.05,
    ) as f32;
    // Mountains block the wind: flow slows toward and across high
    // ground — clouds pile up on windward slopes and rain out there,
    // leeward sides dry into rain shadows.
    let width = world.w;
    let cell = y * width + (x % width.max(1));
    let rel = (world.elevation[cell] - world.sea_level).max(0.0);
    let wall =
        ((rel - MOUNTAIN_WALL * 0.5) / (MOUNTAIN_WALL * 0.5)).clamp(0.0, 1.0);
    let factor = 1.0 - wall * (1.0 - WIND_BLOCK);
    (
        (band + 0.3 * swirl) * factor,
        (0.15 * lat + 0.1 * v_swirl) * factor,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::sim::test_sim;

    #[test]
    fn clouds_spawn_drift_and_carry_water() {
        let mut t = test_sim(29, 100, 40);
        // Long enough for the spawn budget to buy clouds many times over.
        for tick in 0..600 {
            t.tick();
            assert!(t.climate.clouds.len() <= CLOUD_MAX, "fleet cap exceeded");
            for (i, d) in t.world.water.iter().enumerate() {
                assert!(
                    d.is_finite() && *d >= 0.0,
                    "tick {tick} cell {i}: water={d} elev={} flux={:?}",
                    t.world.elevation[i],
                    t.hydro.flux[i]
                );
            }
        }
        assert!(
            !t.climate.clouds.is_empty(),
            "no clouds alive after 600 ticks — the sky is empty"
        );
        // Some rain must have fallen somewhere over that span.
        let total_rain: f32 = t.climate.humidity.iter().sum();
        assert!(
            total_rain > 0.0,
            "humidity memory is zero — clouds never rained"
        );
        // Cloud coverage renders somewhere.
        assert!(
            t.climate.cloud_cover.iter().any(|c| *c > 0.15),
            "no renderable cloud coverage"
        );
    }

    #[test]
    fn seasons_are_hemisphere_antisymmetric() {
        let t = test_sim(19, 60, 24);
        let i = 12 * 60; // mid-map column
        let north = temp_at(&t.world, 6, i, 10.0)
            - temp_at(&t.world, 6, i, 10.0 + YEAR / 2.0);
        let south = temp_at(&t.world, 18, i, 10.0)
            - temp_at(&t.world, 18, i, 10.0 + YEAR / 2.0);
        assert!(
            north * south < 0.0,
            "season anomaly must flip between hemispheres: {north} vs {south}"
        );
    }
}
