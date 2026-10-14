//! World generation (Phase 1): the base map. Two-band fBm with domain
//! warp on a cylinder via 3D Perlin — seamless x-wrap — a post-blend
//! mountain mask, sea level as a height percentile, ocean fill, land
//! stretch, and biome thresholds. Runs once per world, on construction
//! and `reset()` only. The warp drift that animated the earliest sketch
//! is frozen here; from Phase 2 on, the water itself is the motion.

use crate::pocketworld::effect::PocketWorldOptions;
use crate::pocketworld::world::World;

/// Target fraction of the BASE field below sea level. The mountain
/// mask is added after the percentile and lifts ~7% of sea cells into
/// land, so this runs high to land near 40% actual water (user,
/// 2026-10-12). Eustasy tracks this fraction of the live crust.
pub(crate) const SEA_FRACTION: f64 = 0.47;

/// Mountain-chain spatial frequency for the ridged noise band.
const MOUNTAIN_SCALE: f64 = 0.045;

/// Elevation field + sea level + ocean fill + land stretch.
pub(crate) fn generate(world: &mut World, options: &PocketWorldOptions) {
    let width = world.w;
    let height = world.h;
    let radius = width as f64 / std::f64::consts::TAU;
    let warp_strength = options.warp_strength;
    let warp_scale = options.warp_scale;

    // Pass 1: the base field (two-band fBm, domain-warped, on the
    // cylinder). Warped positions and samples are kept for pass 2.
    let cells = width * height;
    let mut warped = vec![(0.0f64, 0.0f64, 0.0f64); cells];
    let mut samples = vec![0.0f32; cells];
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

            // Two-band base (continent + detail). The mountain MASK is
            // added in pass 2, AFTER sea level is taken from the base:
            // sea as a percentile of the masked field rose with the
            // mountains and ate the relief (2026-10-12 lesson).
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
            let sample: f32 =
                ((0.75 * continent + 0.25 * detail) * 1.8).clamp(-1.0, 1.0) as f32;

            warped[y * width + x] = (px, py, pz);
            samples[y * width + x] = sample;
            world.elevation[y * width + x] = (sample + 1.0) / 2.0;
        }
    }

    // Sea level as a percentile of the BASE field, then land stretch.
    let mut sorted = world.elevation.clone();
    let idx = ((sorted.len() as f64 * SEA_FRACTION) as usize).min(sorted.len() - 1);
    select_nth(&mut sorted, idx);
    world.sea_level = sorted[idx];

    // Pass 2: mountain chains on top of the base, scaled by landness
    // so ranges ride the continents; then map above-sea relief over
    // the full headroom with a root curve (compressed distributions
    // read flat). The map normalizes by the pre-map maximum: clamping
    // the masked field first piles every over-1.0 cell onto one
    // elevation atom — the plateau (2026-10-13), under any curve.
    let mut masked = vec![0.0f32; cells];
    let mut max_rel = 1e-6f32;
    for i in 0..cells {
        let (px, py, pz) = warped[i];
        let sample = samples[i];
        let mut h = (sample + 1.0) / 2.0;
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
        let mask = ((ridged - 0.68) / 0.32).clamp(0.0, 1.0);
        let landness = sample.clamp(0.0, 1.0);
        h += mask * mask * 0.85 * (0.5 + 0.5 * landness);
        masked[i] = h;
        let rel = h - world.sea_level;
        if rel > max_rel {
            max_rel = rel;
        }
    }
    for (i, h) in masked.iter().enumerate().take(cells) {
        let h = *h;
        world.elevation[i] = if h > world.sea_level {
            // Toe lift like the old ×2.4 stretch, top spread out,
            // everything in (sea, 1.0] — strictly monotonic, no clamp.
            let rel = h - world.sea_level;
            let head = (1.0 - world.sea_level).max(0.0);
            world.sea_level + head * (rel / max_rel).powf(0.7)
        } else {
            h
        };
    }

    // Ocean fill from the final field.
    for i in 0..world.elevation.len() {
        if world.elevation[i] < world.sea_level {
            world.water[i] = world.sea_level - world.elevation[i];
        }
    }
}

/// Biome thresholds as percentiles of the land height distribution, so
/// the rock/scree bands exist on every seed. `relax` is how far each
/// level moves toward its new percentile: 1.0 at world gen, a fraction
/// at runtime — jumping straight to the fresh percentile flipped a
/// dozen-plus cells across the rock line at once on every refresh
/// (probed 2026-10-14: the "sudden mountains" pops).
pub(crate) fn thresholds(world: &mut World, relax: f32) {
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
        let peak_idx =
            ((sorted_land.len() as f64 * 0.98) as usize).min(sorted_land.len() - 1);
        let toward =
            |level: &mut f32, target: f32| *level += (target - *level) * relax;
        // Stored RELATIVE to sea level: readers compare `elevation -
        // sea_level` against these. The 2026-10-13 frame mismatch
        // (absolute levels vs relative relief) made `rel > rock_level`
        // unreachable — rock and scree never rendered, on any seed.
        select_nth(&mut sorted_land, peak_idx);
        toward(
            &mut world.peak_level,
            sorted_land[peak_idx] - world.sea_level,
        );
        select_nth(&mut sorted_land, rock_idx);
        toward(
            &mut world.rock_level,
            sorted_land[rock_idx] - world.sea_level,
        );
        select_nth(&mut sorted_land, scree_idx);
        toward(
            &mut world.scree_level,
            sorted_land[scree_idx] - world.sea_level,
        );
    }
}

/// Partial selection: reorder so that element `nth` is in its sorted
/// position.
pub(crate) fn select_nth(v: &mut [f32], nth: usize) {
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
    fn fresh_world_has_real_relief() {
        // Relief floor (2026-10-12 round): the 40%-sea world had
        // quietly starved the mountain mask (landness dropped) —
        // worlds booted flat. Lock: populated height bands and a
        // real high-country cell count at t=0.
        let t = test_sim(42, 100, 40);
        let sea = t.world.sea_level;
        let top = t.world.rock_level.max(0.08);
        let mut bands = std::collections::HashSet::new();
        let mut high = 0usize;
        for &e in &t.world.elevation {
            if e > sea {
                let rel = e - sea;
                bands.insert((((rel / top).clamp(0.0, 1.0) * 8.0) as usize).min(7));
                if rel > 0.3 {
                    high += 1;
                }
            }
        }
        eprintln!(
            "RELIEF bands={} high={high} sea={:.3} rock={:.3} maxh={:.3}",
            bands.len(),
            t.world.sea_level,
            t.world.rock_level,
            t.world.elevation.iter().cloned().fold(0.0f32, f32::max)
        );
        assert!(bands.len() >= 4, "height bands collapsed: {}", bands.len());
        assert!(high > 150, "no mountains at boot: {high} cells above 0.3");
    }

    #[test]
    fn mountains_survive_long_runs() {
        // 2026-10-13 round regression: with flat-rate erosion the mid
        // band went from 9% of land to zero within 3000 ticks. Rock
        // strength must keep the high country alive: cells above the
        // boot-time rock level must survive a long run.
        let mut t = test_sim(42, 100, 40);
        let boot_rock = t.world.rock_level; // relative to sea level
        let rel_above = |t: &crate::pocketworld::sim::Sim, rock: f32| {
            t.world
                .elevation
                .iter()
                .filter(|&&e| e - t.world.sea_level > rock)
                .count()
        };
        let boot_high = rel_above(&t, boot_rock);
        for _ in 0..1500 {
            t.tick();
        }
        let late_high = rel_above(&t, boot_rock);
        eprintln!("MOUNTAINS boot={boot_high} late={late_high}");
        assert!(boot_high >= 300, "no high country at boot: {boot_high}");
        assert!(
            late_high * 3 > boot_high,
            "high country eroded away: {boot_high} -> {late_high}"
        );
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
