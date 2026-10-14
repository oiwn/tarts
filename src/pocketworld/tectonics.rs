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

/// Number of plates and their drift speed (cells per tick). Halved in
/// the 2x-slowdown pass (user, 2026-10-11): the motion cue now comes
/// from crust advection, not raw drift speed.
const PLATE_COUNT: usize = 7;
const PLATE_DRIFT: f32 = 0.04;
/// Membership refresh cadence (ticks) — boundaries migrate at this
/// pace, and the crust advection gathers at the same cadence. Springs
/// ride their plate on it too.
pub(crate) const PLATE_REFRESH: u32 = 15;
/// Uplift per tick on convergent boundaries; rift drop on divergent ones.
/// Raised 2026-10-11 (dynamics round): at the old rates a cell left the
/// front long before accumulating one visible height band. Bumped again
/// when the kernel went square -> disk (smaller weight sum). Halved
/// with the 2x slowdown.
const PLATE_UPLIFT: f32 = 0.0035;
const PLATE_RIFT: f32 = 0.00175;
/// Boundary deformation spreads over this Euclidean radius with a
/// smoothstep bell — round stamps, no square corners.
const DEFORM_RADIUS: f32 = 2.7;
/// Relief (above sea) where uplift efficiency reaches zero — the soft
/// summit limit. World gen maps land into ~[0, 0.42]; orogens may
/// build somewhat above the generated high country.
const RELIEF_MAX: f32 = 0.5;
/// Spatial frequency of the uplift ridge texture (peaks ~5-8 cells apart).
const RIDGE_SCALE: f64 = 0.15;
/// Plate-margin meander: sample positions wander this many cells of
/// low-frequency noise in the Voronoi, so margins curve instead of
/// running ruler-straight between seeds.
const PLATE_WARP_SCALE: f64 = 0.11;
const PLATE_WARP_AMP: f64 = 5.0;
/// Crustal activity memory: deformation stamps it, it decays at this
/// rate per tick (~700-tick half-life), and only active crust pays the
/// isostatic sag — ancient interiors keep their generated character
/// instead of melting into the plate network.
const ACTIVITY_DECAY: f32 = 0.001;
/// Per-plate-pair orogen intensity lives in [OROGEN_MIN, 1]: margins
/// come and go — some segments build ranges, others stay passive.
const OROGEN_MIN: f32 = 0.15;
/// Pace at which orogen intensities evolve (noise z input per tick).
const OROGEN_PACE: f64 = 0.0004;
/// Mantle plumes: how many and their magma rate. The plumes are FIXED
/// in the mantle; the plates slide over them (advection), so their
/// trails are true age-progressive volcanic chains (Hawaii).
const HOTSPOT_COUNT: usize = 3;
const HOTSPOT_UPLIFT: f32 = 0.006;
/// Aging volcanic rock: its load memory decays at this rate per tick,
/// and while it lives the crust subsides under it — old island trails
/// sink back below sea level, so only the young end of each chain
/// stays emergent (Hawaii's seamounts).
const VOLCANIC_DECAY: f32 = 0.0005;
const VOLCANIC_SUBSIDE: f32 = 0.0003;
/// Emergent volcanic rock relaxes toward sea level at this rate while
/// its load memory lives — otherwise every trail cell leaves a
/// permanent island stump and the land grows without bound.
const VOLCANIC_RELAX: f32 = 0.01;
/// Boundary classification threshold on the normal velocity component.
pub(crate) const PLATE_CONV_T: f32 = 0.004;
/// Earthquake chance per transform boundary cell per tick, and the jolt.
const QUAKE_CHANCE: f64 = 0.00005;
const QUAKE_JOLT: f32 = 0.02;

/// A lithospheric plate: its seed position, slow drift velocity, and
/// the crust's sub-cell travel not yet applied by advection.
#[derive(Clone, Copy)]
pub(crate) struct Plate {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) dx: f32,
    pub(crate) dy: f32,
    /// Fractional crust displacement carried between advections.
    pub(crate) carry: (f32, f32),
    /// Whole-cell crust shift applied by the last advection (springs
    /// ride the same shift).
    pub(crate) shift: (i32, i32),
}

/// A mantle plume: FIXED in the mantle, forever. The plates slide over
/// it (crust advection), so whatever it builds is carried away as an
/// age-progressive volcanic trail.
pub(crate) struct Hotspot {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

pub(crate) struct Tectonics {
    /// Runtime enable flag (composed from `PocketWorldOptions`).
    pub(crate) on: bool,
    /// Plate seeds with drift velocities.
    pub(crate) plates: Vec<Plate>,
    /// Plate membership per cell (index into `plates`).
    pub(crate) plate: Vec<u8>,
    /// Decaying deformation memory per cell: ~1 where the crust was
    /// recently deformed, ~0 in ancient interiors. Gates the isostatic
    /// sag so only active belts pay. Advects with the crust.
    activity: Vec<f32>,
    /// Volcanic load memory per cell: stamped by mantle plumes, decays
    /// slowly; the crust subsides under it while it lives. Advects
    /// with the crust — the aging trail rides its plate.
    volcanic: Vec<f32>,
    /// Mantle plumes, seeded mid-latitudes at world gen.
    pub(crate) hotspots: Vec<Hotspot>,
    /// Source cell of each cell in the last crust advection (empty when
    /// the last refresh moved nothing) — other layers carry their
    /// crust-attached memories through the same map.
    pub(crate) gather: Vec<u32>,
}

impl Tectonics {
    pub(crate) fn new(cells: usize) -> Self {
        Self {
            on: true,
            plates: Vec::new(),
            plate: vec![0; cells],
            activity: vec![0.0; cells],
            volcanic: vec![0.0; cells],
            hotspots: Vec::new(),
            gather: Vec::new(),
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
                carry: (0.0, 0.0),
                shift: (0, 0),
            });
        }
        self.refresh_plates(world);

        // Mantle plumes: seeded after the plates, mid-latitudes only.
        let y_lo = world.h as f32 * 0.15;
        let y_hi = world.h as f32 * 0.85;
        for _ in 0..HOTSPOT_COUNT {
            self.hotspots.push(Hotspot {
                x: rng.random_range(0.0..width_f),
                y: rng.random_range(y_lo..y_hi),
            });
        }
    }

    /// Drift seeds, lazily refresh membership, then apply boundary stress
    /// and mantle plumes, keeping the isostatic books balanced.
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
            // The crust rides the plates: advect right after the
            // membership refresh, using the fresh plate velocities.
            self.advect(world);
        }

        // The crust's memory of deformation fades: old belts stop
        // paying the isostatic tax, ancient interiors never pay it.
        // Volcanic loads age much slower — islands linger, then sink.
        for a in &mut self.activity {
            *a = (*a - ACTIVITY_DECAY).max(0.0);
        }
        for v in &mut self.volcanic {
            *v = (*v - VOLCANIC_DECAY).max(0.0);
        }

        // Boundary stress. Each plate pair's margin has its own slow
        // orogen intensity: some segments build ranges, some stay
        // passive (Africa's Atlantic coast, not every edge an Andes).
        let mut land_delta = 0.0f32;
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
                    let pa = self.plates[a];
                    let pb = self.plates[b];
                    // Normal from a's cell toward b's cell.
                    let (nx_f, ny_f) = (dx as f32, dy as f32);
                    let vr = ((pa.dx - pb.dx), (pa.dy - pb.dy));
                    let conv = -(vr.0 * nx_f + vr.1 * ny_f);
                    let factor = orogen(world, a, b);
                    if conv > PLATE_CONV_T {
                        // Convergent: uplift both sides of the edge,
                        // spread over the deformation kernel. Ridged
                        // noise along the front gives the belt peaks
                        // and passes — a uniform stamp built a smooth
                        // berm that read as a dome.
                        let lift = PLATE_UPLIFT * conv / PLATE_DRIFT
                            * factor
                            * ridge_texture(world, x, y);
                        land_delta += self.deform(world, x, y, lift, false);
                        land_delta +=
                            self.deform(world, nx, ny as usize, lift, false);
                    } else if conv < -PLATE_CONV_T {
                        // Divergent: the rift opens.
                        let drop = PLATE_RIFT * -conv / PLATE_DRIFT * factor;
                        land_delta += self.deform(world, x, y, -drop, false);
                        land_delta +=
                            self.deform(world, nx, ny as usize, -drop, false);
                    } else if rng.random::<f64>() < QUAKE_CHANCE {
                        // Transform shear: quake jolt on the boundary.
                        let jolt = QUAKE_JOLT
                            * if rng.random::<f64>() < 0.5 { 1.0 } else { -1.0 };
                        world.elevation[i] = (world.elevation[i] + jolt).max(0.0);
                        world.elevation[j] = (world.elevation[j] + jolt).max(0.0);
                    }
                }
            }
        }

        // Mantle plumes: fixed vents. The plates slide over them
        // (advection), so islands surface at the vent and ride away as
        // age-progressive trails.
        for hi in 0..self.hotspots.len() {
            let (hx, hy) = (self.hotspots[hi].x, self.hotspots[hi].y);
            land_delta += self.deform(
                world,
                (hx as usize).min(width - 1),
                (hy as usize).min(height - 1),
                HOTSPOT_UPLIFT,
                true,
            );
        }

        // Aging volcanic loads sink (thermal subsidence of the aging
        // lithosphere): old island trails return below sea level —
        // only the young end of each chain stays emergent. Unbooked:
        // what the plumes raised for free must sink back for free, or
        // the land grows at the ocean's expense.
        for i in 0..self.volcanic.len() {
            let v = self.volcanic[i];
            if v <= 1e-4 {
                continue;
            }
            let rel = (world.elevation[i] - world.sea_level).max(0.0);
            let drop = VOLCANIC_SUBSIDE * v + VOLCANIC_RELAX * v * rel;
            world.elevation[i] = (world.elevation[i] - drop).max(0.0);
        }

        // Balance the books: the active crust sags back under its loads.
        self.isostasy(world, land_delta);
    }

    /// Apply deformation at a cell over a smoothstep-weighted Euclidean
    /// disk: round bell-shaped stamps, so fronts build rounded ridges
    /// instead of square-cornered ones. Wrap on x; poles bound the
    /// cylinder. Continents ride the plates; oceanic crust mostly
    /// subducts — sea cells take a fraction of the deformation so the
    /// ocean survives hot tectonics. Magma (hotspots) ignores the crust
    /// gate: plumes punch through oceanic crust and raise islands.
    /// Stamps the activity memory and returns the delta actually
    /// applied to LAND cells (post-clamp) for the isostatic ledger.
    fn deform(
        &mut self,
        world: &mut World,
        x: usize,
        y: usize,
        delta: f32,
        magma: bool,
    ) -> f32 {
        let width = world.w as isize;
        let height = world.h as isize;
        let ir = DEFORM_RADIUS.ceil() as isize;
        let mut land_delta = 0.0f32;
        for oy in -ir..=ir {
            let ny = y as isize + oy;
            if ny < 0 || ny >= height {
                continue;
            }
            for ox in -ir..=ir {
                let d = ((ox * ox + oy * oy) as f32).sqrt();
                if d >= DEFORM_RADIUS {
                    continue;
                }
                let t = 1.0 - d / DEFORM_RADIUS;
                let w = t * t * (3.0 - 2.0 * t);
                let nx = (x as isize + ox).rem_euclid(width) as usize;
                let i = ny as usize * world.w + nx;
                let land = world.elevation[i] > world.sea_level;
                // Continents ride the plates; oceanic crust subducts —
                // seafloor deformation is nearly zero, or boundary
                // uplift slowly fills the ocean floor (measured: floor
                // rose +0.42 over 900 ticks at the old 0.15 gate).
                let crust = if magma || land { 1.0 } else { 0.04 };
                let before = world.elevation[i];
                // Uplift fades with height instead of hitting a wall: a
                // hard 1.0 clamp parked every orogen's summit on one
                // elevation — the runtime plateau (2026-10-14: the top
                // 15% of land sat at the cap, the rock tier was
                // unreachable). With no limit at all, relief ran away
                // (maxrel 9, sea level adrift). The fade gives each
                // belt its own equilibrium height against isostasy and
                // erosion, so summits spread instead of stacking.
                let fade = if delta > 0.0 {
                    (1.0 - (before - world.sea_level) / RELIEF_MAX).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                world.elevation[i] = (before + delta * w * crust * fade).max(0.0);
                if land {
                    land_delta += world.elevation[i] - before;
                }
                self.activity[i] = self.activity[i].max(w);
                if magma {
                    self.volcanic[i] = self.volcanic[i].max(w);
                }
            }
        }
        land_delta
    }

    /// Isostatic compensation: gravity limits how tall mountains grow.
    /// Whatever the boundaries and plumes added to (or took from) the
    /// land this tick is balanced by sag spread over the ACTIVE land in
    /// proportion to each cell's load (height above sea × activity
    /// memory) — young ranges rise at convergent fronts while their
    /// forelands subside beneath them, and old belts relax back down
    /// for a while once their front has migrated away (activity fades).
    /// Ancient interiors never pay, so the generated terrain keeps its
    /// character instead of melting into the plate network. Land mass
    /// is conserved; relief is not. Earthquake jolts are unbooked noise
    /// (bounded, symmetric).
    fn isostasy(&mut self, world: &mut World, land_delta: f32) {
        if land_delta == 0.0 {
            return;
        }
        let mut total_load = 0.0f32;
        for (e, a) in world.elevation.iter().zip(self.activity.iter()) {
            let rel = e - world.sea_level;
            if rel > 0.0 {
                total_load += rel * a;
            }
        }
        if total_load <= 1e-6 {
            return;
        }
        let share = -land_delta / total_load;
        for (e, a) in world.elevation.iter_mut().zip(self.activity.iter()) {
            let rel = *e - world.sea_level;
            if rel > 0.0 {
                *e = (*e + share * rel * a).max(0.0);
            }
        }
    }

    /// The crust rides the plates: elevation and its activity memory
    /// are carried by each cell's plate velocity (wrap on x).
    /// Continents travel and keep their outlines, ranges ride along
    /// after their boundary moves on (Appalachian behavior), and the
    /// fixed mantle plumes leave true age-progressive trails as the
    /// plates slide over them.
    ///
    /// The gather is lossless: each plate banks its sub-cell travel and
    /// the crust moves by whole cells only. The old bilinear gather at
    /// ~0.6-cell offsets was a low-pass filter applied every refresh —
    /// probed 2026-10-14: each advection cut land roughness by 15× what
    /// all other ticks rebuilt, smearing ranges into domes (rock tier
    /// 2.2% @300 → 0% @1500).
    fn advect(&mut self, world: &mut World) {
        let w = world.w;
        let h = world.h;
        let dt = PLATE_REFRESH as f32;
        for p in &mut self.plates {
            p.carry.0 += p.dx * dt;
            p.carry.1 += p.dy * dt;
            let sx = p.carry.0.round();
            let sy = p.carry.1.round();
            p.carry.0 -= sx;
            p.carry.1 -= sy;
            p.shift = (sx as i32, sy as i32);
        }
        self.gather.clear();
        if self.plates.iter().all(|p| p.shift == (0, 0)) {
            return;
        }
        self.gather.resize(w * h, 0);
        let src = world.elevation.clone();
        let act = self.activity.clone();
        let vol = self.volcanic.clone();
        let mass_before: f32 = src.iter().sum();
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let (sx, sy) = self.plates[self.plate[i] as usize].shift;
                let gx = (x as isize - sx as isize).rem_euclid(w as isize) as usize;
                let gy =
                    (y as isize - sy as isize).clamp(0, h as isize - 1) as usize;
                let j = gy * w + gx;
                self.gather[i] = j as u32;
                world.elevation[i] = src[j];
                self.activity[i] = act[j];
                self.volcanic[i] = vol[j];
            }
        }
        // Renormalize only THIS step's error: the bilinear gather
        // duplicates crust at divergent boundaries and loses it at
        // convergent ones. Never pin to a gen-time target — erosion
        // parks mass in the sediment field, and paying that "deficit"
        // back every advection lifted the whole seafloor above sea
        // (2026-10-12: below_sea collapsed 1600 -> 45).
        let mass_after: f32 = world.elevation.iter().sum();
        let cells = (w * h) as f32;
        let corr = (mass_before - mass_after) / cells;
        if corr != 0.0 {
            for e in world.elevation.iter_mut() {
                *e = (*e + corr).max(0.0);
            }
        }
    }
}

/// Uplift texture along convergent fronts, in [0.3, 1.5]: ridged noise
/// fixed in the mantle frame, so migrating fronts sweep across peaks
/// and saddles. Cylinder-sampled (seamless x-wrap).
fn ridge_texture(world: &World, x: usize, y: usize) -> f32 {
    let radius = world.w as f64 / std::f64::consts::TAU;
    let theta = x as f64 / radius;
    let (s, c) = theta.sin_cos();
    let n = world.noise.octave_noise_3d(
        radius * c + 913.1,
        y as f64,
        radius * s + 41.7,
        2,
        0.5,
        RIDGE_SCALE,
    );
    let ridged = (1.0 - n.abs()) as f32;
    0.3 + 1.2 * ridged * ridged
}

/// Per-plate-pair orogen intensity: slow deterministic noise in
/// [OROGEN_MIN, 1] — margins come and go over minutes, so belts
/// localize into a few major orogens instead of the whole network
/// glowing.
fn orogen(world: &World, a: usize, b: usize) -> f32 {
    let salt = (a.min(b)) as f64 * 17.3 + (a.max(b)) as f64 * 7.7;
    let n = world.noise.noise_3d(salt, 0.5, world.time * OROGEN_PACE);
    let t = (0.5 + 0.5 * n).clamp(0.0, 1.0) as f32;
    OROGEN_MIN + (1.0 - OROGEN_MIN) * t
}

impl Tectonics {
    /// Voronoi plate membership (nearest seed, torus distance on x),
    /// sampled through a low-frequency noise warp so margins meander
    /// instead of running ruler-straight between seeds — round plates,
    /// curved convergent fronts.
    pub(crate) fn refresh_plates(&mut self, world: &World) {
        let width = world.w;
        let height = world.h;
        for y in 0..height {
            for x in 0..width {
                let fx = x as f64 * PLATE_WARP_SCALE;
                let fy = y as f64 * PLATE_WARP_SCALE;
                // Two warp octaves: big bends plus fine roughness —
                // nature rarely runs straight.
                let wx = (world.noise.noise_3d(fx, fy, 7.7)
                    + 0.5
                        * world.noise.noise_3d(
                            fx * 2.7 + 9.1,
                            fy * 2.7 + 4.3,
                            7.7,
                        ))
                    * PLATE_WARP_AMP;
                let wy = (world.noise.noise_3d(fx + 31.4, fy + 11.7, 7.7)
                    + 0.5
                        * world.noise.noise_3d(
                            fx * 2.7 + 40.2,
                            fy * 2.7 + 20.8,
                            7.7,
                        ))
                    * PLATE_WARP_AMP;
                let sx = x as f32 + wx as f32;
                let sy = y as f32 + wy as f32;
                let mut best = 0usize;
                let mut best_d = f32::MAX;
                for (pi, p) in self.plates.iter().enumerate() {
                    let mut ddx = (p.x - sx).abs().min(
                        (p.x - sx + width as f32)
                            .abs()
                            .min((p.x - sx - width as f32).abs()),
                    );
                    ddx *= ddx;
                    let ddy = p.y - sy;
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
    fn rock_tier_stays_reachable_and_relief_bounded() {
        // Mountain-round lock (2026-10-14). Two failure modes: a hard
        // elevation ceiling parked the top 15% of land on one value,
        // so the rock percentile sat at the cap and the rock tier
        // (entry margin above it) was unreachable; with no limit,
        // relief ran away (maxrel 9). After a long run, mid-latitude
        // land must clear the rock entry and relief must stay sane.
        for seed in [7u32, 42] {
            let mut t = test_sim(seed, 100, 40);
            for _ in 0..1500 {
                t.tick();
            }
            let (w, h) = (t.world.w, t.world.h);
            let rl = t.world.rock_level;
            let entry = rl + ((rl - t.world.scree_level) * 0.3).max(0.01);
            let (mut land, mut rock) = (0usize, 0usize);
            for y in 0..h {
                let lat = ((y as f32 + 0.5) / h as f32 * 2.0 - 1.0).abs();
                if lat >= 0.6 {
                    continue;
                }
                for x in 0..w {
                    let rel = t.world.elevation[y * w + x] - t.world.sea_level;
                    if rel > 0.0 {
                        land += 1;
                        if rel > entry {
                            rock += 1;
                        }
                    }
                }
            }
            let maxrel = t.world.elevation.iter().cloned().fold(0.0f32, f32::max)
                - t.world.sea_level;
            eprintln!(
                "ROCK seed {seed}: mid-lat land {land} rock {rock} maxrel {maxrel:.3}"
            );
            assert!(rock * 100 > land * 3, "no mid-latitude rock: {rock}/{land}");
            assert!(maxrel < 0.7, "relief ran away: maxrel {maxrel}");
        }
    }

    #[test]
    fn tectonics_alone_moves_ground_visibly() {
        use crate::pocketworld::effect::PocketWorldOptionsBuilder;
        use crate::pocketworld::sim::{Sim, SimLayer};

        // Focus mode (2026-10-11 dynamics round): tectonics only, the
        // rest frozen — the user watches Plates + Height views.
        let options = PocketWorldOptionsBuilder::default()
            .seed(37u32)
            .disabled(vec![
                SimLayer::Hydro,
                SimLayer::Climate,
                SimLayer::Erosion,
                SimLayer::Life,
            ])
            .build()
            .unwrap();
        let mut t = Sim::new(&options, (100, 40));
        let before = t.world.elevation.clone();
        for _ in 0..600 {
            t.tick();
        }
        // One heightmap band is ~0.15 of elevation; >0.1 per cell over
        // ~10 s of screen time is the "clearly visible change" bar.
        let (mut big, mut up, mut down) = (0usize, 0usize, 0usize);
        let mut max_delta = 0.0f32;
        for (b, a) in t.world.elevation.iter().zip(before.iter()) {
            let d = *b - *a;
            if d.abs() > 0.1 {
                big += 1;
            }
            if d > 0.05 {
                up += 1;
            } else if d < -0.05 {
                down += 1;
            }
            max_delta = max_delta.max(d.abs());
        }
        // Isostasy invariants (anti-"all yellow"): the living land's
        // mean stays in a band around its start (no slide to uniform
        // mid-heights), land area survives coast churn, and relief
        // stays alive while the surface keeps morphing. Probed
        // 2026-10-11: equilibrium at mean~0.77, area~2500, std~0.19.
        let sea = t.world.sea_level;
        let land_stats = |field: &[f32]| {
            let land: Vec<f32> =
                field.iter().filter(|h| **h > sea).copied().collect();
            let n = land.len().max(1) as f32;
            let m = land.iter().sum::<f32>() / n;
            let sd = (land.iter().map(|h| (h - m).powi(2)).sum::<f32>() / n).sqrt();
            (land.len(), m, sd)
        };
        let (n_before, mean_before, std_before) = land_stats(&before);
        let (n_after, mean_after, std_after) = land_stats(&t.world.elevation);
        eprintln!(
            "TECTONICS big={big}/4000 up={up} down={down} max={max_delta:.3} land {n_before}->{n_after} mean {mean_before:.3}->{mean_after:.3} std {std_before:.3}->{std_after:.3}"
        );
        assert!(
            big > 20,
            "changes not visible: {big} cells moved >0.1, max {max_delta:.3}"
        );
        assert!(
            up > 10 && down > 10,
            "need both uplift and rift: up={up} down={down}"
        );
        assert!(
            (mean_after - mean_before).abs() < 0.1,
            "land mean slid toward uniform: {mean_before:.3} -> {mean_after:.3}"
        );
        assert!(
            n_after > 1900,
            "land area collapsed: {n_before} -> {n_after}"
        );
        assert!(
            std_after > std_before * 0.8,
            "relief collapsed: std {std_before:.3} -> {std_after:.3}"
        );
    }

    #[test]
    fn hotspots_build_volcanic_islands() {
        use crate::pocketworld::effect::PocketWorldOptionsBuilder;
        use crate::pocketworld::sim::{Sim, SimLayer};

        let options = PocketWorldOptionsBuilder::default()
            .seed(37u32)
            .disabled(vec![
                SimLayer::Hydro,
                SimLayer::Climate,
                SimLayer::Erosion,
                SimLayer::Life,
            ])
            .build()
            .unwrap();
        let mut t = Sim::new(&options, (100, 40));
        let sea = t.world.sea_level;
        let before = t.world.elevation.clone();
        for _ in 0..900 {
            t.tick();
        }
        // Islands surface from below-sea cells along the plume tracks —
        // interior features aligned with plate motion, not boundaries.
        let emerged = before
            .iter()
            .zip(t.world.elevation.iter())
            .filter(|(b, a)| **b <= sea && **a > sea)
            .count();
        eprintln!("HOTSPOTS emerged={emerged}/4000");
        assert!(emerged >= 3, "no volcanic islands surfaced: {emerged}");
    }

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
