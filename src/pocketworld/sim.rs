//! Composition root: the world commons plus the layer objects, ticked
//! in one canonical order. Layers are enabled/disabled at runtime (`on`
//! flags, composed from `PocketWorldOptions::disabled`); a disabled layer
//! keeps its state frozen and its steps are skipped. The tick order is
//! canonical — layers interleave (clouds rain between ocean relaxation
//! and springs) and the single RNG stream depends on it, so all-on
//! replays are deterministic and skipping a layer shifts the stream for
//! the rest.

use crate::pocketworld::climate::Climate;
use crate::pocketworld::effect::PocketWorldOptions;
use crate::pocketworld::elevation;
use crate::pocketworld::erosion::Erosion;
use crate::pocketworld::hydro::Hydro;
use crate::pocketworld::life::Life;
use crate::pocketworld::noise::PerlinNoise;
use crate::pocketworld::tectonics::Tectonics;
use crate::pocketworld::world::World;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

/// Screen-time seconds per simulation tick.
const TIME_STEP: f64 = 0.033;

/// Toggleable simulation layers. `PocketWorldOptions::disabled` names the
/// ones to skip; every layer runs by default. `Elevation` is not here:
/// the height field is generated once and written only by tectonics and
/// erosion.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SimLayer {
    Hydro,
    Climate,
    Erosion,
    Tectonics,
    Life,
}

pub(crate) struct Sim {
    pub(crate) world: World,
    pub(crate) hydro: Hydro,
    pub(crate) climate: Climate,
    pub(crate) erosion: Erosion,
    pub(crate) tectonics: Tectonics,
    pub(crate) life: Life,
    /// The single seeded RNG stream for all stochastic rolls (cloud
    /// spawns, lightning, quakes); same seed replays the same history.
    rng: rand::rngs::StdRng,
}

impl Sim {
    pub(crate) fn new(
        options: &PocketWorldOptions,
        screen_size: (u16, u16),
    ) -> Self {
        let w = (screen_size.0 as usize).max(1);
        let h = (screen_size.1 as usize).max(1);
        let cells = w * h;
        let mut sim = Self {
            world: World::new(w, h, PerlinNoise::new(options.seed)),
            hydro: Hydro::new(cells),
            climate: Climate::new(cells),
            erosion: Erosion::new(cells),
            tectonics: Tectonics::new(cells),
            life: Life::new(cells),
            rng: rand::rngs::StdRng::seed_from_u64(options.seed as u64),
        };
        for layer in &options.disabled {
            sim.set_enabled(*layer, false);
        }
        sim.generate(options);
        sim
    }

    /// Enable or disable a layer at runtime. A disabled layer keeps its
    /// state frozen and its steps are skipped.
    pub(crate) fn set_enabled(&mut self, layer: SimLayer, on: bool) {
        *self.flag_mut(layer) = on;
    }

    pub(crate) fn enabled(&self, layer: SimLayer) -> bool {
        match layer {
            SimLayer::Hydro => self.hydro.on,
            SimLayer::Climate => self.climate.on,
            SimLayer::Erosion => self.erosion.on,
            SimLayer::Tectonics => self.tectonics.on,
            SimLayer::Life => self.life.on,
        }
    }

    fn flag_mut(&mut self, layer: SimLayer) -> &mut bool {
        match layer {
            SimLayer::Hydro => &mut self.hydro.on,
            SimLayer::Climate => &mut self.climate.on,
            SimLayer::Erosion => &mut self.erosion.on,
            SimLayer::Tectonics => &mut self.tectonics.on,
            SimLayer::Life => &mut self.life.on,
        }
    }

    /// World generation, in the historical order: elevation + sea fill +
    /// stretch, springs, plates, biome thresholds, first classification.
    fn generate(&mut self, options: &PocketWorldOptions) {
        elevation::generate(&mut self.world, options);
        self.hydro.place_springs(&self.world);
        self.tectonics.generate(&self.world, &mut self.rng);
        elevation::thresholds(&mut self.world);
        self.hydro.refresh_classification(&self.world);
    }

    /// One simulation tick: drain, weather, springs, melt, life,
    /// tectonics, pipes, erosion — then the lazy classification.
    pub(crate) fn tick(&mut self) {
        self.world.time += TIME_STEP;
        if self.enabled(SimLayer::Hydro) {
            self.hydro.drain(&mut self.world);
        }
        if self.enabled(SimLayer::Climate) {
            self.climate.begin();
        }
        if self.enabled(SimLayer::Climate) {
            self.climate.clouds(&mut self.world, &mut self.rng);
        }
        if self.enabled(SimLayer::Hydro) {
            self.hydro.emit_springs(&mut self.world);
        }
        if self.enabled(SimLayer::Climate) {
            self.climate.snowmelt(&mut self.world);
        }
        if self.enabled(SimLayer::Life) {
            self.life.step(&self.world, &self.climate, &mut self.rng);
        }
        if self.enabled(SimLayer::Tectonics) {
            self.tectonics.step(&mut self.world, &mut self.rng);
        }
        if self.enabled(SimLayer::Hydro) {
            self.hydro.pipes(&mut self.world);
        }
        if self.enabled(SimLayer::Erosion) {
            self.erosion.step(&mut self.world, &self.hydro);
        }
        self.world.tick += 1;
        if self.enabled(SimLayer::Hydro) {
            self.hydro.classify_if_due(&self.world);
        }
    }
}

/// Test constructor: a Sim with the given seed and grid.
#[cfg(test)]
pub(crate) fn test_sim(seed: u32, w: usize, h: usize) -> Sim {
    use crate::pocketworld::effect::PocketWorldOptionsBuilder;
    let options = PocketWorldOptionsBuilder::default()
        .seed(seed)
        .build()
        .unwrap();
    Sim::new(&options, (w as u16, h as u16))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pocketworld::effect::PocketWorldOptionsBuilder;

    #[test]
    fn oceans_stay_filled_and_water_stays_sane() {
        let mut t = test_sim(9, 60, 20);
        let fill: Vec<f32> = t
            .world
            .elevation
            .iter()
            .map(|h| (t.world.sea_level - h).max(0.0))
            .collect();
        for _ in 0..600 {
            t.tick();
        }
        assert!(
            t.world.water.iter().all(|d| d.is_finite() && *d >= 0.0),
            "negative or non-finite water"
        );
        // The ocean is a boundary condition: as a body it stays near its
        // fill volume. Individual shallow straits may run through-flow
        // currents and dip below their own fill level.
        let mut fill_total = 0.0f32;
        let mut water_total = 0.0f32;
        for (d, f) in t.world.water.iter().zip(fill.iter()) {
            if *f > 0.0 {
                fill_total += f;
                water_total += d;
            }
        }
        assert!(
            water_total > fill_total * 0.85,
            "ocean body drained: {water_total} vs fill {fill_total}"
        );
        // Land water stays bounded — no runaway sheets.
        for (d, f) in t.world.water.iter().zip(fill.iter()) {
            if *f == 0.0 {
                assert!(*d < 0.5, "land cell flooded: {d}");
            }
        }
    }

    #[test]
    fn water_flows_downhill() {
        let mut t = test_sim(13, 60, 20);
        // Drop a blob on the highest cell and let it run off.
        let peak = t
            .world
            .elevation
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| i)
            .unwrap();
        t.world.water[peak] += 0.5;
        for _ in 0..300 {
            t.tick();
        }
        assert!(
            t.world.water[peak] < 0.35,
            "water did not drain from the peak: {}",
            t.world.water[peak]
        );
        let spread = t.world.water.iter().filter(|d| **d > 1e-4).count();
        assert!(spread > 10, "water did not spread: {spread} wet cells");
    }

    #[test]
    fn deterministic_replay() {
        let mut a = test_sim(21, 60, 20);
        let mut b = test_sim(21, 60, 20);
        for _ in 0..120 {
            a.tick();
            b.tick();
        }
        for (x, y) in a.world.water.iter().zip(b.world.water.iter()) {
            assert!((x - y).abs() < 1e-6, "replay diverged: {x} vs {y}");
        }
    }

    #[test]
    fn erosion_conserves_solid_mass() {
        // Hydraulic + talus erosion only move material between elevation
        // and sediment. Plate tectonics (round 5b) is an external source/
        // sink by design, so the drift bound covers tectonic contribution
        // while still catching unbounded erosion leaks.
        let mut t = test_sim(17, 60, 20);
        let before: f32 = t.world.elevation.iter().sum::<f32>()
            + t.erosion.sediment.iter().sum::<f32>();
        for _ in 0..300 {
            t.tick();
        }
        let after: f32 = t.world.elevation.iter().sum::<f32>()
            + t.erosion.sediment.iter().sum::<f32>();
        assert!(
            (after - before).abs() < 15.0,
            "solid mass drifted beyond tectonics: {before} -> {after}"
        );
    }

    #[test]
    fn layers_can_be_disabled_at_runtime() {
        let options = PocketWorldOptionsBuilder::default()
            .seed(23u32)
            .disabled(vec![SimLayer::Climate, SimLayer::Life])
            .build()
            .unwrap();
        let mut t = Sim::new(&options, (80, 24));
        assert!(!t.enabled(SimLayer::Climate));
        assert!(!t.enabled(SimLayer::Life));
        assert!(t.enabled(SimLayer::Hydro));
        for _ in 0..100 {
            t.tick();
        }
        // Frozen climate: no clouds ever spawn, no humidity memory.
        assert!(t.climate.clouds.is_empty());
        assert!(t.climate.humidity.iter().all(|h| *h == 0.0));
        assert!(t.world.water.iter().all(|d| d.is_finite()));

        // Re-enable: the layer resumes from its frozen state.
        t.set_enabled(SimLayer::Climate, true);
        for _ in 0..200 {
            t.tick();
        }
        assert!(!t.climate.clouds.is_empty());
    }
}
