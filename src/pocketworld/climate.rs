//! Climate and weather (Phase 4): the cloud fleet, the cryosphere, the
//! humidity memory, and the derived temperature/wind fields. Clouds are
//! 1-cell particles spawned as fronts over warm ocean, riding the
//! banded winds with turbulence — the sky is the wind made visible.
//! Temperature and wind are derived, never stored: f(latitude,
//! elevation, time).

use crate::pocketworld::hydro::WET_MIN;
use crate::pocketworld::world::{World, sample_bilinear};
use rand::RngExt;
use rand::rngs::StdRng;

// Deliberately untuned — the user dials it in visually.
/// Ocean evaporation fills a cloud-spawn budget; this is the fill per tick.
const CLOUD_SPAWN: f32 = 0.5;
/// New-cloud water content and the fleet size cap. Raised 2026-10-12:
/// fronts should cross continents and oceans, not die at the coast.
const CLOUD_WATER: f32 = 0.42;
const CLOUD_MAX: usize = 250;
/// Fraction of a cloud rained out per tick over land. Lowered
/// 2026-10-12 (twice): at 0.06 fronts died within ~30 ticks, at 0.035
/// still mid-continent — clouds now cross whole landmasses raining.
const CLOUD_RAIN: f32 = 0.02;
/// Smoothing for the rendered cloud-coverage field (per tick).
const COVER_EMA: f32 = 0.3;
/// Extra rain-out multiplier when mountains block the path.
const OROGRAPHIC: f32 = 2.0;
/// Particles per spawned cloud front (a visible traveling system).
const CLOUD_FRONT: usize = 11;
/// Elevation above sea level that counts as a mountain wall.
const MOUNTAIN_WALL: f32 = 0.22;
/// Soil-humidity memory: written by rain, decays each tick.
const HUMID_DECAY: f32 = 0.0007;
// Winds (2026-10-12 round): a simulated momentum field — thermal band
// forcing, Coriolis turning from the world's rotation, orographic and
// land drag, turbulence, and self-advection. Weather (meandering jets,
// eddies, front pile-ups) emerges; clouds ride the real flow instead
// of drawn-on latitude bands.
/// Relaxation toward the thermal-wind bands (per tick). Must outpace
/// meridional mixing, or the circulation bands smear into mush.
const WIND_RELAX: f32 = 0.08;
/// Coriolis turning strength, scaled by sin(latitude).
const WIND_CORIOLIS: f32 = 0.08;
/// Turbulence injected per tick (slow evolving noise field). The
/// meridional part is damped — it mixes the bands away.
const WIND_JITTER: f32 = 0.12;
const WIND_JITTER_V: f32 = 0.05;
/// Surface drag over land, and extra orographic drag at a full wall.
const WIND_DRAG_LAND: f32 = 0.004;
const WIND_DRAG_WALL: f32 = 0.35;
/// Speed clamp (cells/tick) — stability for the self-advection.
const WIND_MAX: f32 = 1.4;
/// Snow melts above this temperature, at this rate (scaled by temperature).
const SNOW_MELT_T: f32 = 0.05;
const SNOW_MELT: f32 = 0.0015;
/// Polar precipitation rate: the cold zones accumulate steadily —
/// the cloud fleet is a sketch of water transport and never reliably
/// reaches the poles (probed: zero clouds past |lat| 0.5), so the
/// polar cell delivers its own snowfall, scaling with coldness.
const POLAR_SNOW: f32 = 0.0006;
/// Sublimation: snow loss proportional to mass, everywhere. With the
/// polar deposit this gives a stable cap equilibrium (~0.4 ×
/// coldness) — without it, glaciers monotonically swallowed the poles.
const SNOW_SUBLIME: f32 = 0.0015;
/// Year length in seconds of screen time.
const YEAR: f64 = 120.0;
/// Seasonal temperature anomaly amplitude (hemisphere-antisymmetric sine).
const SEASON_AMPLITUDE: f32 = 0.15;
/// Temperature model: equator value, pole gradient, elevation lapse.
/// Pole drop deepened 2026-10-12: at 1.2 the poles hovered near 0 and
/// snowfall (temp < 0) essentially never happened — caps were rendered
/// frost, not accumulated snow.
const TEMP_EQUATOR: f32 = 1.0;
const TEMP_POLE_DROP: f32 = 1.6;
const TEMP_LAPSE: f32 = 0.6;

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
    /// Slow average of cloud coverage — the render field. Particles
    /// are speckle; the average is what fronts are made of.
    pub(crate) cover_ema: Vec<f32>,
    /// The cloud fleet — 1-cell particles; formations emerge from the wind.
    pub(crate) clouds: Vec<Cloud>,
    /// Ocean-evaporation budget toward spawning the next cloud.
    pub(crate) spawn_budget: f32,
    /// Simulated wind field, eastward component (cells/tick).
    pub(crate) wind_u: Vec<f32>,
    /// Simulated wind field, northward component (cells/tick).
    pub(crate) wind_v: Vec<f32>,
}

impl Climate {
    pub(crate) fn new(cells: usize) -> Self {
        Self {
            on: true,
            humidity: vec![0.0; cells],
            rain: vec![0.0; cells],
            snow: vec![0.0; cells],
            cloud_cover: vec![0.0; cells],
            cover_ema: vec![0.0; cells],
            clouds: Vec::new(),
            spawn_budget: 0.0,
            wind_u: vec![0.0; cells],
            wind_v: vec![0.0; cells],
        }
    }

    /// Initial wind condition: the analytic thermal-wind profile, so
    /// the simulated field starts in balance instead of spinning up
    /// from calm.
    pub(crate) fn init_wind(&mut self, world: &World) {
        for y in 0..world.h {
            let lat = ((y as f32 + 0.5) / world.h.max(1) as f32) * 2.0 - 1.0;
            let a = lat.abs();
            let band = if a < 0.35 {
                -1.0
            } else if a < 0.75 {
                1.0
            } else {
                -1.0
            };
            for x in 0..world.w {
                let i = y * world.w + x;
                self.wind_u[i] = band;
                self.wind_v[i] = 0.15 * lat;
            }
        }
    }

    /// One wind step: self-advection (the field carries itself — jets
    /// meander, eddies spin off), then thermal forcing toward the
    /// general circulation, Coriolis turning from the world's
    /// rotation, turbulence, and orographic/land drag.
    pub(crate) fn wind_step(&mut self, world: &World) {
        let w = world.w;
        let h = world.h;
        {
            let su = self.wind_u.clone();
            let sv = self.wind_v.clone();
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    let sx = x as f32 - self.wind_u[i];
                    let sy = (y as f32 - self.wind_v[i]).clamp(0.0, (h - 1) as f32);
                    self.wind_u[i] = sample_bilinear(&su, w, h, sx, sy);
                    self.wind_v[i] = sample_bilinear(&sv, w, h, sx, sy);
                }
            }
        }
        let height = h.max(1) as f32;
        for y in 0..h {
            let lat = ((y as f32 + 0.5) / height) * 2.0 - 1.0;
            let a = lat.abs();
            let band = if a < 0.35 {
                -1.0
            } else if a < 0.75 {
                1.0
            } else {
                -1.0
            };
            let u_target = band;
            // Weak meridional drift (~Hadley-ish tilt): a slab
            // translation here smears the bands, so keep it small.
            let v_target = 0.05 * lat;
            let cor = (lat * std::f32::consts::FRAC_PI_2).sin();
            for x in 0..w {
                let i = y * w + x;
                let (u, v) = (self.wind_u[i], self.wind_v[i]);
                // Mountains kill the flow — windward pile-ups, leeward
                // rain shadows; open ocean barely drags at all.
                let rel = (world.elevation[i] - world.sea_level).max(0.0);
                let wall = ((rel - MOUNTAIN_WALL * 0.5) / (MOUNTAIN_WALL * 0.5))
                    .clamp(0.0, 1.0);
                let drag = if rel > 0.0 { WIND_DRAG_LAND } else { 0.0 }
                    + wall * WIND_DRAG_WALL;
                let jx = world.noise.noise_3d(
                    x as f64 * 0.07,
                    y as f64 * 0.07,
                    world.time * 0.03,
                ) as f32;
                let jy = world.noise.noise_3d(
                    x as f64 * 0.07 + 41.0,
                    y as f64 * 0.07 + 17.0,
                    world.time * 0.03,
                ) as f32;
                let nu = u
                    + (u_target - u) * WIND_RELAX
                    + cor * v * WIND_CORIOLIS
                    + jx * WIND_JITTER;
                let nv = v + (v_target - v) * WIND_RELAX - cor * u * WIND_CORIOLIS
                    + jy * WIND_JITTER_V;
                self.wind_u[i] = (nu * (1.0 - drag)).clamp(-WIND_MAX, WIND_MAX);
                self.wind_v[i] = (nv * (1.0 - drag)).clamp(-WIND_MAX, WIND_MAX);
            }
        }
    }

    /// The simulated wind at a cell.
    pub(crate) fn wind_at(&self, world: &World, x: usize, y: usize) -> (f32, f32) {
        let i = y.min(world.h - 1) * world.w + (x % world.w);
        (self.wind_u[i], self.wind_v[i])
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
            let (u, v) = self.wind_at(world, cloud.x as usize, cloud.y as usize);
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
                cloud.water = (cloud.water + 0.01 * warm).min(CLOUD_WATER);
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
                if cloud.water <= 0.012 {
                    expired.push(ci);
                    continue;
                }
            }

            // Soft footprint: particles stamp a 3x3 blob so fronts
            // read as cohesive bodies, not salt-and-pepper.
            let density = (cloud.water / CLOUD_WATER).clamp(0.0, 1.0);
            let cx = cloud.x as isize;
            let cy = cloud.y as isize;
            for oy in -1isize..=1 {
                let ny = cy + oy;
                if ny < 0 || ny >= height as isize {
                    continue;
                }
                for ox in -1isize..=1 {
                    let nx = (cx + ox).rem_euclid(width as isize) as usize;
                    let w = if ox == 0 && oy == 0 { 1.0 } else { 0.5 };
                    let j = ny as usize * width + nx;
                    self.cloud_cover[j] = self.cloud_cover[j].max(density * w);
                }
            }
        }

        for ci in expired.into_iter().rev() {
            clouds.remove(ci);
        }
        self.clouds = clouds;
        // The render field: the averaged coverage is what fronts are
        // made of.
        for (e, c) in self.cover_ema.iter_mut().zip(self.cloud_cover.iter()) {
            *e += (*c - *e) * COVER_EMA;
        }
    }

    /// Cryosphere: snow falls steadily in the cold zones (the polar
    /// cell) and melts where warm, feeding the hydrosphere — poles and
    /// winters hold water as ice, spring releases it.
    pub(crate) fn snowmelt(&mut self, world: &mut World) {
        let width = world.w;
        for i in 0..self.snow.len() {
            let y = i / width;
            let temp = temp_at(world, y, i, world.time);
            if self.snow[i] > 0.0 {
                self.snow[i] *= 1.0 - SNOW_SUBLIME;
            }
            if temp < -0.05 {
                self.snow[i] += POLAR_SNOW * (-temp).min(1.0);
            } else if temp > SNOW_MELT_T && self.snow[i] > 0.0 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pocketworld::sim::test_sim;

    #[test]
    fn winds_circulation_and_stability() {
        use crate::pocketworld::effect::PocketWorldOptionsBuilder;
        use crate::pocketworld::sim::{Sim, SimLayer};

        // Simulated winds: the general circulation must hold (trades
        // westward in the tropics, westerlies mid-latitude, polar
        // easterlies) while the field stays bounded and meanders.
        let options = PocketWorldOptionsBuilder::default()
            .seed(42u32)
            .disabled(vec![SimLayer::Life])
            .build()
            .unwrap();
        let mut t = Sim::new(&options, (100, 40));
        for _ in 0..600 {
            t.tick();
        }
        let (w, h) = (t.world.w, t.world.h);
        for y in 0..h {
            let lat = ((y as f32 + 0.5) / h as f32) * 2.0 - 1.0;
            let mut mean_u = 0.0f32;
            let mut var_u = 0.0f32;
            let mut free = 0usize;
            // Circulation over the free flow: mountain belts locally
            // block the wind (that's the orography working), so zonal
            // means exclude high-wall cells.
            for x in 0..w {
                let i = y * w + x;
                let rel = (t.world.elevation[i] - t.world.sea_level).max(0.0);
                let wall = ((rel - 0.22 * 0.5) / (0.22 * 0.5)).clamp(0.0, 1.0);
                let (u, v) = t.climate.wind_at(&t.world, x, y);
                assert!(
                    u.abs() <= 1.5 && v.abs() <= 1.5,
                    "wind unstable at ({x},{y}): {u},{v}"
                );
                if wall < 0.5 {
                    mean_u += u;
                    free += 1;
                }
            }
            if free < 10 {
                continue;
            }
            mean_u /= free as f32;
            for x in 0..w {
                let (u, _) = t.climate.wind_at(&t.world, x, y);
                var_u += (u - mean_u).powi(2);
            }
            let std_u = (var_u / w as f32).sqrt();
            let a = lat.abs();
            if a < 0.25 {
                assert!(
                    mean_u < -0.2,
                    "trades missing at lat {lat:.2}: mean_u {mean_u:.2}"
                );
            } else if (0.4..0.7).contains(&a) {
                assert!(
                    mean_u > 0.2,
                    "westerlies missing at lat {lat:.2}: mean_u {mean_u:.2}"
                );
                assert!(
                    std_u > 0.1,
                    "no meander at lat {lat:.2}: std_u {std_u:.2}"
                );
            } else if a > 0.85 {
                assert!(
                    mean_u < -0.1,
                    "polar easterlies missing at lat {lat:.2}: mean_u {mean_u:.2}"
                );
            }
        }
    }

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
