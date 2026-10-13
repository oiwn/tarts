//! The commons: state written or read by more than one simulation layer.
//!
//! Only the multi-writer fields live here (`elevation`: tectonics +
//! erosion; `water`: pipes, rain, snowmelt, springs) plus the world
//! clock and the height-field parameters derived at generation.
//! Layer-owned grids live in their layer structs (`hydro`, `climate`,
//! `erosion`, `tectonics`, `life`); see `sim.rs` for composition.

use crate::pocketworld::noise::PerlinNoise;

/// Shared world state, one instance per `Sim`.
pub(crate) struct World {
    /// Grid width in cells (the cylinder's circumference).
    pub(crate) w: usize,
    /// Grid height in cells (pole to pole).
    pub(crate) h: usize,
    /// Simulation clock, in seconds of screen time.
    pub(crate) time: f64,
    /// Tick counter; incremented once per `Sim::tick`.
    pub(crate) tick: u32,
    /// The shared noise field (world generation + wind swirl).
    pub(crate) noise: PerlinNoise,
    /// Sea level as a height percentile of the generated field.
    pub(crate) sea_level: f32,
    /// Normalized heights in [0, 1], row-major. Frozen at world
    /// generation — erosion and tectonics are the only writers later.
    pub(crate) elevation: Vec<f32>,
    /// Water depth per cell.
    pub(crate) water: Vec<f32>,
    /// Rock-band threshold (85th percentile of land heights).
    pub(crate) rock_level: f32,
    /// Scree-band threshold (70th percentile of land heights).
    pub(crate) scree_level: f32,
}

impl World {
    pub(crate) fn new(w: usize, h: usize, noise: PerlinNoise) -> Self {
        Self {
            w,
            h,
            time: 0.0,
            tick: 0,
            noise,
            sea_level: 0.0,
            elevation: vec![0.0; w * h],
            water: vec![0.0; w * h],
            rock_level: 0.0,
            scree_level: 0.0,
        }
    }
}

/// Bilinear sample on a wrap-x, pole-clamped grid — the gather used by
/// all semi-Lagrangian advection (crust, wind).
pub(crate) fn sample_bilinear(
    field: &[f32],
    w: usize,
    h: usize,
    x: f32,
    y: f32,
) -> f32 {
    let x = x.rem_euclid(w as f32);
    let y = y.clamp(0.0, (h - 1) as f32);
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1) % w;
    let y1 = (y0 + 1).min(h - 1);
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    let top = field[y0 * w + x0] * (1.0 - fx) + field[y0 * w + x1] * fx;
    let bot = field[y1 * w + x0] * (1.0 - fx) + field[y1 * w + x1] * fx;
    top * (1.0 - fy) + bot * fy
}
