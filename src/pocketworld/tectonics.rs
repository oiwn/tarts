//! Tectonics (round 5b): lithospheric plates — the mountain factory.
//! ~7 plates seeded at world gen (Voronoi membership, wrap-x) with slow
//! drift velocities; membership refreshes lazily so boundaries migrate.
//! At each boundary cell the plates' relative velocity decomposes on the
//! boundary normal: convergent → uplift (ranges grow ON plate fronts),
//! divergent → rift subsidence, transform → earthquake jolts. Interiors
//! are stable cratons — deformation only at edges.

use crate::pocketworld::world::World;
use rand::RngExt;
use rand::rngs::StdRng;

/// Number of plates and their drift speed (cells per tick).
const PLATE_COUNT: usize = 7;
const PLATE_DRIFT: f32 = 0.04;
/// Membership refresh cadence (ticks) — boundaries migrate at this pace.
const PLATE_REFRESH: u32 = 30;
/// Uplift per tick on convergent boundaries; rift drop on divergent ones.
const PLATE_UPLIFT: f32 = 0.0004;
const PLATE_RIFT: f32 = 0.0003;
/// Boundary classification threshold on the normal velocity component.
pub(crate) const PLATE_CONV_T: f32 = 0.004;
/// Earthquake chance per transform boundary cell per tick, and the jolt.
const QUAKE_CHANCE: f64 = 0.00005;
const QUAKE_JOLT: f32 = 0.02;

/// A lithospheric plate: its seed position and slow drift velocity.
pub(crate) struct Plate {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) dx: f32,
    pub(crate) dy: f32,
}

pub(crate) struct Tectonics {
    /// Runtime enable flag (composed from `TerrainOptions`).
    pub(crate) on: bool,
    /// Plate seeds with drift velocities.
    pub(crate) plates: Vec<Plate>,
    /// Plate membership per cell (index into `plates`).
    pub(crate) plate: Vec<u8>,
}

impl Tectonics {
    pub(crate) fn new(cells: usize) -> Self {
        Self {
            on: true,
            plates: Vec::new(),
            plate: vec![0; cells],
        }
    }

    /// Seed plates at world gen: random positions with slow drift
    /// velocities. Membership via Voronoi.
    pub(crate) fn generate(&mut self, world: &World, rng: &mut StdRng) {
        let width_f = world.w as f32;
        let height_f = world.h as f32;
        for _ in 0..PLATE_COUNT {
            let ang = rng.random_range(0.0..std::f32::consts::TAU);
            self.plates.push(Plate {
                x: rng.random_range(0.0..width_f),
                y: rng.random_range(0.0..height_f),
                dx: ang.cos() * PLATE_DRIFT,
                dy: ang.sin() * PLATE_DRIFT * 0.5,
            });
        }
        self.refresh_plates(world);
    }

    /// Drift seeds, lazily refresh membership, then apply boundary stress:
    /// check the right and down neighbor of each cell.
    pub(crate) fn step(&mut self, world: &mut World, rng: &mut StdRng) {
        let width = world.w;
        let height = world.h;

        // Drift.
        for plate in self.plates.iter_mut() {
            plate.x = (plate.x + plate.dx).rem_euclid(width as f32);
            plate.y = (plate.y + plate.dy).clamp(0.0, (height - 1) as f32);
        }
        if world.tick % PLATE_REFRESH == 0 {
            self.refresh_plates(world);
        }

        // Boundary stress.
        for y in 0..height {
            for x in 0..width {
                let i = y * width + x;
                let a = self.plate[i] as usize;
                for (dx, dy) in [(1isize, 0isize), (0, 1)] {
                    let ny = y as isize + dy;
                    if ny >= height as isize {
                        continue;
                    }
                    let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                    let j = ny as usize * width + nx;
                    let b = self.plate[j] as usize;
                    if a == b {
                        continue;
                    }
                    let (pa, pb) = (&self.plates[a], &self.plates[b]);
                    // Normal from a's cell toward b's cell.
                    let (nx_f, ny_f) = (dx as f32, dy as f32);
                    let vr = ((pa.dx - pb.dx), (pa.dy - pb.dy));
                    let conv = -(vr.0 * nx_f + vr.1 * ny_f);
                    if conv > PLATE_CONV_T {
                        // Convergent: uplift both sides of the edge.
                        let lift = PLATE_UPLIFT * conv / PLATE_DRIFT;
                        world.elevation[i] =
                            (world.elevation[i] + lift).clamp(0.0, 1.0);
                        world.elevation[j] =
                            (world.elevation[j] + lift).clamp(0.0, 1.0);
                    } else if conv < -PLATE_CONV_T {
                        // Divergent: the rift opens.
                        let drop = PLATE_RIFT * -conv / PLATE_DRIFT;
                        world.elevation[i] =
                            (world.elevation[i] - drop).clamp(0.0, 1.0);
                        world.elevation[j] =
                            (world.elevation[j] - drop).clamp(0.0, 1.0);
                    } else if rng.random::<f64>() < QUAKE_CHANCE {
                        // Transform shear: quake jolt on the boundary.
                        let jolt = QUAKE_JOLT
                            * if rng.random::<f64>() < 0.5 { 1.0 } else { -1.0 };
                        world.elevation[i] =
                            (world.elevation[i] + jolt).clamp(0.0, 1.0);
                        world.elevation[j] =
                            (world.elevation[j] + jolt).clamp(0.0, 1.0);
                    }
                }
            }
        }
    }

    /// Voronoi plate membership (nearest seed, torus distance on x).
    pub(crate) fn refresh_plates(&mut self, world: &World) {
        let width = world.w;
        let height = world.h;
        for y in 0..height {
            for x in 0..width {
                let mut best = 0usize;
                let mut best_d = f32::MAX;
                for (pi, p) in self.plates.iter().enumerate() {
                    let mut ddx = (p.x - x as f32).abs().min(
                        (p.x - x as f32 + width as f32)
                            .abs()
                            .min((p.x - x as f32 - width as f32).abs()),
                    );
                    ddx *= ddx;
                    let ddy = p.y - y as f32;
                    let d = ddx + ddy * ddy;
                    if d < best_d {
                        best_d = d;
                        best = pi;
                    }
                }
                self.plate[y * width + x] = best as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pocketworld::sim::test_sim;

    #[test]
    fn plates_partition_and_stress_boundaries() {
        let mut t = test_sim(37, 100, 40);
        // All plates represented.
        let mut seen = std::collections::HashSet::new();
        for p in &t.tectonics.plate {
            seen.insert(*p);
        }
        assert!(
            seen.len() >= 5,
            "only {} plates present in membership",
            seen.len()
        );
        // Boundaries exist between plates.
        let width = 100usize;
        let mut boundaries = 0usize;
        for y in 0..40usize {
            for x in 0..width {
                let i = y * width + x;
                let right = y * width + (x + 1) % width;
                if t.tectonics.plate[i] != t.tectonics.plate[right] {
                    boundaries += 1;
                }
            }
        }
        assert!(boundaries > 20, "too few plate boundaries: {boundaries}");
        // Drift moves boundaries over time (membership changes).
        let initial = t.tectonics.plate.clone();
        for _ in 0..600 {
            t.tick();
        }
        let moved = t
            .tectonics
            .plate
            .iter()
            .zip(initial.iter())
            .filter(|(a, b)| a != b)
            .count();
        assert!(moved > 50, "plate boundaries never migrated: {moved} cells");
    }
}
