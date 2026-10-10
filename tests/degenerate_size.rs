//! Regression tests for degenerate terminal sizes (issue #94).
//!
//! Effects must survive construction and a few update/render cycles at 0x0
//! and 1x1 — ptys, CI, and some terminal panes report such sizes.

use tarts::boids::{Boids, BoidsOptionsBuilder};
use tarts::common::TerminalEffect;
use tarts::crab::{Crab, CrabOptionsBuilder};
use tarts::life::{ConwayLife, ConwayLifeOptionsBuilder};
use tarts::rain::digital_rain::{DigitalRain, DigitalRainOptionsBuilder};
use tarts::terrain::{Terrain, TerrainOptionsBuilder};

type EffectExercise = fn((u16, u16));

fn exercise_at_degenerate_size() -> Vec<(&'static str, EffectExercise)> {
    vec![
        ("crab", |size| {
            let options = CrabOptionsBuilder::default().build().unwrap();
            let mut effect = Crab::new(options, size);
            for _ in 0..3 {
                effect.update();
                effect.get_diff();
            }
        }),
        ("matrix", |size| {
            let options = DigitalRainOptionsBuilder::default().build().unwrap();
            let mut effect = DigitalRain::new(options, size);
            for _ in 0..3 {
                effect.update();
                effect.get_diff();
            }
        }),
        ("life", |size| {
            let options = ConwayLifeOptionsBuilder::default().build().unwrap();
            let mut effect = ConwayLife::new(options, size);
            for _ in 0..3 {
                effect.update();
                effect.get_diff();
            }
        }),
        ("boids", |size| {
            let options = BoidsOptionsBuilder::default().build().unwrap();
            let mut effect = Boids::new(options);
            effect.update_size(size.0, size.1);
            for _ in 0..3 {
                effect.update();
                effect.get_diff();
            }
        }),
        ("terrain", |size| {
            let options = TerrainOptionsBuilder::default().build().unwrap();
            let mut effect = Terrain::new(options, size);
            for _ in 0..3 {
                effect.update();
                effect.get_diff();
            }
        }),
    ]
}

#[test]
fn effects_survive_zero_size() {
    for (name, run) in exercise_at_degenerate_size() {
        for size in [(0, 0), (0, 24), (80, 0)] {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(size)))
                .unwrap_or_else(|e| {
                    panic!("{name} panicked at {}x{}: {e:?}", size.0, size.1)
                });
        }
    }
}

#[test]
fn effects_survive_one_by_one() {
    for (name, run) in exercise_at_degenerate_size() {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run((1, 1))))
            .unwrap_or_else(|e| panic!("{name} panicked at 1x1: {e:?}"));
    }
}
