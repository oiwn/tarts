//! World generation (Phase 1): the base map. Two-band fBm with domain
//! warp on a cylinder via 3D Perlin — seamless x-wrap — a post-blend
//! mountain mask, sea level as a height percentile, ocean fill, land
//! stretch, and biome thresholds. Runs once per world, on construction
//! and `reset()` only. The warp drift that animated the earliest sketch
//! is frozen here; from Phase 2 on, the water itself is the motion.

use crate::pocketworld::effect::PocketWorldOptions;
use crate::pocketworld::world::World;

/// Target fraction of the map below sea level. Land-dominant (round 2):
/// a lakes-and-rivers world with three times the terrain for rivers.
const SEA_FRACTION: f64 = 0.28;

/// Mountain-chain spatial frequency for the ridged noise band.
const MOUNTAIN_SCALE: f64 = 0.045;

/// Elevation field + sea level + ocean fill + land stretch.
pub(crate) fn generate(world: &mut World, options: &PocketWorldOptions) {
    let width = world.w;
    let height = world.h;
    let radius = width as f64 / std::f64::consts::TAU;
    let warp_strength = options.warp_strength;
    let warp_scale = options.warp_scale;

    for y in 0..height {
        for x in 0..width {
            let wu = world.noise.octave_noise_3d(
                x as f64 * 0.05,
                y as f64 * 0.05,
                7.3,
                3,
                0.5,
                warp_scale,
            ) * warp_strength;
            let wv = world.noise.octave_noise_3d(
                x as f64 * 0.05 + 77.7,
                y as f64 * 0.05 + 33.3,
                7.3,
                3,
                0.5,
                warp_scale,
            ) * warp_strength;

            let u = x as f64 + wu;
            let v = y as f64 + wv;

            let theta = u / radius;
            let (sin_t, cos_t) = theta.sin_cos();
            let (px, py, pz) = (radius * cos_t, v, radius * sin_t);

            // Two-band base (continent + detail), then a mountain MASK
            // on top: blending ridges into the average compresses them
            // below every threshold (round-1 lesson) — chains must be
            // added after, tall enough to clear the rock/snow bands.
            let continent = world.noise.octave_noise_3d(
                px,
                py,
                pz,
                options.octaves,
                options.persistence,
                options.scale,
            );
            let detail = world.noise.octave_noise_3d(
                px + 500.5,
                py + 123.4,
                pz + 87.2,
                options.detail_octaves,
                options.persistence,
                options.detail_scale,
            );
            let mut sample = 0.75 * continent + 0.25 * detail;
            sample = (sample * 1.8).clamp(-1.0, 1.0);
            let mut h = ((sample + 1.0) / 2.0) as f32;

            // Mountain chains: ridged noise above a mask threshold, added
            // on top and scaled by landness so ranges ride the continents.
            let ridged = 1.0
                - world
                    .noise
                    .octave_noise_3d(
                        px + 250.5,
                        py + 311.4,
                        pz + 97.2,
                        4,
                        0.5,
                        MOUNTAIN_SCALE,
                    )
                    .abs() as f32;
            let mask = ((ridged - 0.72) / 0.28).clamp(0.0, 1.0);
            let landness = (sample as f32).clamp(0.0, 1.0);
            h += mask * mask * 0.6 * (0.4 + 0.6 * landness);

            world.elevation[y * width + x] = h.min(1.0);
        }
    }

    // Sea level as a height percentile (~28% — land-dominant world),
    // then ocean fill.
    let mut sorted = world.elevation.clone();
    let idx = ((sorted.len() as f64 * SEA_FRACTION) as usize).min(sorted.len() - 1);
    select_nth(&mut sorted, idx);
    world.sea_level = sorted[idx];

    for i in 0..world.elevation.len() {
        if world.elevation[i] < world.sea_level {
            world.water[i] = world.sea_level - world.elevation[i];
        }
    }

    // Stretch land heights: compressed distributions read as flat plains
    // (round-4 review). Sea fill is untouched; only above-sea relief
    // expands to actually use the height range.
    for h in world.elevation.iter_mut() {
        if *h > world.sea_level {
            let rel = *h - world.sea_level;
            *h = (world.sea_level + (rel * 1.7).min(0.9)).min(1.0);
        }
    }
}

/// Biome thresholds as percentiles of the land height distribution, so
/// the rock/scree bands exist on every seed.
pub(crate) fn thresholds(world: &mut World) {
    let land: Vec<f32> = world
        .elevation
        .iter()
        .copied()
        .filter(|h| *h > world.sea_level)
        .collect();
    if land.len() > 10 {
        let mut sorted_land = land;
        let rock_idx =
            ((sorted_land.len() as f64 * 0.85) as usize).min(sorted_land.len() - 1);
        let scree_idx =
            ((sorted_land.len() as f64 * 0.7) as usize).min(sorted_land.len() - 1);
        select_nth(&mut sorted_land, rock_idx);
        world.rock_level = sorted_land[rock_idx];
        select_nth(&mut sorted_land, scree_idx);
        world.scree_level = sorted_land[scree_idx];
    }
}

/// Partial selection: reorder so that element `nth` is in its sorted
/// position.
fn select_nth(v: &mut [f32], nth: usize) {
    let mut lo = 0usize;
    let mut hi = v.len() - 1;
    while lo < hi {
        let pivot = v[(lo + hi) / 2];
        let (mut i, mut j) = (lo, hi);
        while i <= j {
            while v[i] < pivot {
                i += 1;
            }
            while v[j] > pivot {
                j -= 1;
            }
            if i <= j {
                v.swap(i, j);
                i += 1;
                if j == 0 {
                    break;
                }
                j -= 1;
            }
        }
        if nth < i {
            hi = i - 1;
        } else {
            lo = i;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pocketworld::sim::test_sim;

    #[test]
    fn heights_are_normalized() {
        let t = test_sim(7, 60, 20);
        for h in &t.world.elevation {
            assert!((0.0..=1.0).contains(h), "height {h} out of range");
        }
        let max = t.world.elevation.iter().copied().fold(0.0f32, f32::max);
        let min = t.world.elevation.iter().copied().fold(1.0f32, f32::min);
        assert!(max - min > 0.3, "world too flat: [{min}, {max}]");
    }

    #[test]
    fn wraps_seamlessly_on_x() {
        let (w, h) = (80usize, 24usize);
        let t = test_sim(3, w, h);
        let at = |x: usize, y: usize| t.world.elevation[y * w + x % w];

        let mut neighbor_delta = 0.0;
        let mut seam_delta = 0.0;
        for y in 0..h {
            for x in 0..w {
                neighbor_delta += (at(x, y) - at(x + 1, y)).abs();
            }
            seam_delta += (at(w, y) - at(w - 1, y)).abs();
        }
        let avg_neighbor = neighbor_delta / (w * h) as f32;
        let avg_seam = seam_delta / h as f32;
        assert!(
            avg_seam < avg_neighbor * 4.0 + 0.05,
            "visible x seam: seam {avg_seam} vs neighbor {avg_neighbor}"
        );
    }
}
