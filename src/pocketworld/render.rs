//! Rendering: pure functions of the simulation state. `Renderer` holds
//! the active display layer (cycled with `j`/`k`, plus the `l` legend
//! screen); every view reads the world and the layer states without
//! owning any of them. Color carries elevation, glyph carries state; in
//! the heightmap view brightness = height, strictly.

use crate::buffer::{Buffer, Cell};
use crate::pocketworld::climate::{Climate, temp_at};
use crate::pocketworld::hydro::{Hydro, LAKE, RIVER_FLUX, WET_MIN};
use crate::pocketworld::life::{ASH, BURNING, FOREST_VEG};
use crate::pocketworld::sim::Sim;
use crate::pocketworld::tectonics::PLATE_CONV_T;
use crate::pocketworld::world::World;
use crossterm::style;

/// Water depth above which water renders — thin rain sheets on land stay
/// invisible instead of painting puddle noise over whole regions.
const RENDER_WET_MIN: f32 = 0.012;

/// Smoothed ocean depth bands: below `OCEAN_OPEN` is the shallow shelf,
/// above `OCEAN_DEEP` the deep ocean, the open sea between.
const OCEAN_OPEN: f32 = 0.08;
const OCEAN_DEEP: f32 = 0.2;

/// Lowland biome gates (relief above sea, moisture on life's scale,
/// derived temperature). Swamp is waterlogged ground at the coast;
/// savanna and desert are the hot belt's dry rings.
const SWAMP_REL: f32 = 0.05;
const SWAMP_MOIST: f32 = 0.5;
const DESERT_MOIST: f32 = 0.12;
const SAVANNA_MOIST: f32 = 0.3;
const TUNDRA_TEMP: f32 = -0.15;

/// Display layers, cycled with `l` (forward) and `k` (backward) — the
/// tuning instruments.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Layer {
    Biome,
    Plates,
    Height,
    Hydro,
    Temp,
    Wind,
    Humidity,
    Aquifer,
}

impl Layer {
    pub(crate) fn next(self) -> Self {
        match self {
            Layer::Biome => Layer::Plates,
            Layer::Plates => Layer::Height,
            Layer::Height => Layer::Hydro,
            Layer::Hydro => Layer::Temp,
            Layer::Temp => Layer::Wind,
            Layer::Wind => Layer::Humidity,
            Layer::Humidity => Layer::Aquifer,
            Layer::Aquifer => Layer::Biome,
        }
    }

    pub(crate) fn prev(self) -> Self {
        match self {
            Layer::Biome => Layer::Aquifer,
            Layer::Aquifer => Layer::Humidity,
            Layer::Humidity => Layer::Wind,
            Layer::Wind => Layer::Temp,
            Layer::Temp => Layer::Hydro,
            Layer::Hydro => Layer::Height,
            Layer::Height => Layer::Plates,
            Layer::Plates => Layer::Biome,
        }
    }
}

/// Display state: which layer view is on screen, whether the legend
/// overlay is showing, and the per-cell land-tier memory that keeps
/// mountain band edges from flickering.
pub(crate) struct Renderer {
    pub(crate) view: Layer,
    pub(crate) legend: bool,
    /// Last land tier per cell (0 lowland, 1 scree, 2 rock) for
    /// hysteresis at the tier thresholds.
    pub(crate) sticky: Vec<u8>,
}

impl Renderer {
    pub(crate) fn new() -> Self {
        Self {
            view: Layer::Biome,
            legend: false,
            sticky: Vec::new(),
        }
    }

    /// Glyph + color for one cell under the active view.
    pub(crate) fn classify(
        &mut self,
        sim: &Sim,
        x: usize,
        y: usize,
    ) -> (char, style::Color) {
        match self.view {
            Layer::Plates => plates_view(sim, x, y),
            Layer::Height => height_view(&sim.world, x, y),
            Layer::Hydro => hydro_view(&sim.world, &sim.hydro, x, y),
            Layer::Temp => temp_view(&sim.world, x, y),
            Layer::Wind => wind_view(&sim.climate, &sim.world, x, y),
            Layer::Humidity => humidity_view(&sim.climate, sim.world.w, x, y),
            Layer::Aquifer => aquifer_view(&sim.world, x, y),
            Layer::Biome => biome_view(self, sim, x, y),
        }
    }

    /// HUD label for the active layer, written on the top row.
    pub(crate) fn hud_label(&self) -> String {
        format!(" LAYER: {} [j/k] legend:l", layer_name(self.view))
    }
}

/// The legend screen: a full page of glyph meanings, shown instead of
/// the world while toggled.
pub(crate) fn draw_legend(buf: &mut Buffer) {
    let put = |buf: &mut Buffer, x: usize, y: usize, s: &str, c: style::Color| {
        for (n, ch) in s.chars().enumerate() {
            if x + n < buf.width && y < buf.height {
                buf.set(
                    x + n,
                    y,
                    Cell::new(ch, c, style::Attribute::NormalIntensity),
                );
            }
        }
    };
    let title = "POCKETWORLD — LEGEND";
    let tx = buf.width.saturating_sub(title.chars().count()) / 2;
    put(buf, tx, 2, title, rgb(230, 232, 238));

    let rows = legend_rows();
    let cols = if buf.width >= 60 { 2 } else { 1 };
    let col_w = 30;
    let per_col = rows.len().div_ceil(cols);
    for (n, (g, desc)) in rows.iter().enumerate() {
        let col = n / per_col;
        let row = n % per_col;
        let x = 4 + col * col_w;
        let y = 5 + row;
        if y + 2 < buf.height && x + 3 < buf.width {
            buf.set(
                x,
                y,
                Cell::new(
                    *g,
                    rgb(230, 232, 238),
                    style::Attribute::NormalIntensity,
                ),
            );
            put(buf, x + 3, y, desc, rgb(200, 204, 212));
        }
    }
    let foot = "[l] back to the world";
    let fx = buf.width.saturating_sub(foot.chars().count()) / 2;
    put(
        buf,
        fx,
        buf.height.saturating_sub(2),
        foot,
        rgb(160, 164, 172),
    );
}

/// The legend table: glyph -> meaning, drawn as an overlay when the
/// legend toggle is on.
pub(crate) fn legend_rows() -> Vec<(char, &'static str)> {
    vec![
        ('≈', "deep ocean"),
        ('≋', "open sea"),
        ('~', "shelf sea / lake"),
        ('˃', "river, flow dir"),
        ('.', "shore  . ,"),
        ('"', "grass  \" ' ˙ , ` ¸"),
        ('♣', "forest  ♣ ♠ §"),
        ('Ψ', "swamp  Ψ ψ ξ"),
        (';', "savanna  ; :"),
        ('∴', "desert  ∴ ∵ ∷"),
        ('▲', "rock  ▲ ∆ ∧"),
        ('°', "scree  ° ˚ ·"),
        ('*', "snow / frost"),
        ('❄', "snow pack"),
        ('▓', "glacier"),
        ('◉', "mantle plume"),
        ('≡', "plate convergent"),
        ('¦', "plate rift"),
        ('▒', "cloud"),
        ('✱', "fire"),
        ('░', "ash"),
    ]
}

/// Biome view (the default). Priority (top wins): rain on land → cloud
/// → water → land biomes. Rain never draws over the sea; clouds draw
/// over everything (they are the visible sky). Fire burns over biomes.
fn biome_view(
    renderer: &mut Renderer,
    sim: &Sim,
    x: usize,
    y: usize,
) -> (char, style::Color) {
    let world = &sim.world;
    let width = world.w;
    let i = y * width + x;
    // Smoothed depth: the raw field sloshes tick to tick, and coastal
    // cells strobed shore ↔ water across the wet gate every frame
    // (probed 2026-10-14: ~40 coast cells per frame).
    let d = sim.hydro.water_ema[i];
    let rel = world.elevation[i] - world.sea_level;

    // No rain animation (user, 2026-10-14): rain still falls in the
    // sim — it feeds rivers, snow, and humidity — but it renders as
    // nothing. The flickering `|`/`⁄` glyphs read as noise, and the
    // 3×3 footprint made them wallpaper.

    if renderer.sticky.len() != world.w * world.h {
        renderer.sticky.resize(world.w * world.h, 0);
    }
    let tier = sticky_tier(&mut renderer.sticky, world, i, rel);

    let base = biome_base(sim, x, y, i, d, rel, tier);
    cloud_overlay(sim, i, base)
}

/// Land tier with hysteresis: 0 lowland, 1 scree, 2 rock. Erosion and
/// talus jitter elevation ±0.002 per tick, so cells near a threshold
/// used to flip rock↔scree↔grass every few frames (the mountain
/// flicker, 2026-10-14). Entering a tier takes a real margin past its
/// threshold; leaving it takes the same margin back.
fn sticky_tier(sticky: &mut [u8], world: &World, i: usize, rel: f32) -> u8 {
    if world.rock_level <= 0.0 {
        sticky[i] = 0;
        return 0;
    }
    let prev = sticky[i];
    let h_rock = ((world.rock_level - world.scree_level) * 0.3).max(0.01);
    let h_scree = (world.scree_level * 0.3).max(0.01);
    let tier = if rel > world.rock_level + if prev == 2 { -h_rock } else { h_rock }
    {
        2
    } else if rel > world.scree_level + if prev == 1 { -h_scree } else { h_scree } {
        1
    } else {
        0
    };
    sticky[i] = tier;
    tier
}

/// The terrain itself (water + land biomes), before the sky is drawn
/// over it.
fn biome_base(
    sim: &Sim,
    x: usize,
    y: usize,
    i: usize,
    d: f32,
    rel: f32,
    tier: u8,
) -> (char, style::Color) {
    let world = &sim.world;

    // Rivers of the drainage network: from headwaters to the coast by
    // construction — shallow pipe water never rendered (the depth gate
    // hid it, and it died within ~8 cells of a spring). Only a glacial
    // sheet covers them.
    if let Some(water) = network_river(&sim.hydro, world, i)
        && sim.climate.snow[i] <= 0.36
    {
        return water;
    }

    // The sea is the sea: anything below sea level renders as ocean,
    // decided from geometry rather than the connectivity flag, which
    // flips as drifting coastlines change what is connected — whole
    // basins used to strobe blue/teal on every refresh. Depth bands
    // read the geometric depth (sea level − seafloor, slow) — the
    // smoothed water depth still sloshed ~/≋/≈ across the band edges
    // every frame.
    if rel < 0.0 {
        let depth = -rel;
        return if depth > OCEAN_DEEP {
            ('≈', rgb(30, 84, 148))
        } else if depth > OCEAN_OPEN {
            ('≋', rgb(48, 112, 170))
        } else {
            ('~', rgb(70, 140, 190))
        };
    }
    if d > RENDER_WET_MIN {
        // Standing water on land: a lake. Channels are the drainage
        // network's (drawn above); pipe flux is nonzero on nearly all
        // land and no longer marks rivers.
        return ('~', rgb(78, 178, 178));
    }

    // Land: color carries elevation, glyph carries state — with a
    // per-cell jitter so bands are not flat paint.
    let temp = temp_at(world, y, i, world.time);
    // One moisture for render and life alike (2026-10-14): the
    // desert/savanna/swamp gates used raw rain humidity and painted
    // arid over groundwater jungle.
    let m = crate::pocketworld::life::moisture(&sim.climate, world, i);

    if sim.life.fire[i] == BURNING {
        let frames = ['✱', '❋', '✹'];
        let f = (x + y + (world.time * 8.0) as usize) % 3;
        return (frames[f], rgb(250, 140, 40));
    }
    // The cryosphere: substantial snow owns the cell; light dust lets
    // the terrain show through (rock must stay visible under thin
    // cover — mountains were disappearing under white).
    let s = sim.climate.snow[i];
    if s > 0.16 {
        let (g, col): (char, (u8, u8, u8)) = if s > 0.36 {
            ('▓', (206, 224, 248)) // glacial sheet
        } else if s > 0.26 {
            ('❄', (226, 240, 251)) // heavy pack
        } else {
            ('*', (244, 249, 253)) // packed snow
        };
        // Aging warmth: near-melting snow trends translucent blue.
        let col = if temp > -0.2 {
            (col.0.saturating_sub(20), col.1, col.2)
        } else {
            col
        };
        let jc = jitter(col, x, y, 8);
        return (g, rgb(jc.0, jc.1, jc.2));
    }
    // Permanent frost caps on the summits only: tier 2 (hysteresis
    // keeps the edge still), cold, AND in the upper half of the band
    // between the rock line and the summit level. Capping all cold
    // rock whitened most ranges (user objection 1, 2026-10-14).
    let cap_line = world.rock_level + 0.5 * (world.peak_level - world.rock_level);
    if temp < 0.05 && world.rock_level > 0.0 && tier == 2 && rel > cap_line {
        let jc = jitter((198, 218, 242), x, y, 8);
        return ('*', rgb(jc.0, jc.1, jc.2));
    }
    if sim.life.fire[i] == ASH {
        return ('░', rgb(95, 88, 80)); // burnt ground
    }
    // Per-object variety: a stable position hash picks among narrow
    // glyph variants so biomes are not monocultures. The land tiers
    // come from the sticky (hysteresis) resolver — raw `rel` only
    // where the band is tiny (shore) or climate-gated.
    let result = if rel < 0.02 {
        let jc = jitter((210, 192, 138), x, y, 10);
        (pick(x, y, &['.', ',']), rgb(jc.0, jc.1, jc.2)) // shore
    } else {
        match tier {
            2 => {
                // Rock reads as rock: grey faces brightening toward
                // the peaks — bright, not sooty (2026-10-14 pass).
                let top = (1.0 - world.sea_level - world.rock_level).max(0.05);
                let t = ((rel - world.rock_level) / top).clamp(0.0, 1.0);
                let jc = jitter(
                    (
                        (150.0 + 86.0 * t) as u8,
                        (146.0 + 88.0 * t) as u8,
                        (140.0 + 91.0 * t) as u8,
                    ),
                    x,
                    y,
                    10,
                );
                (pick(x, y, &['▲', '∆', '∧']), rgb(jc.0, jc.1, jc.2))
                // rock — the mountain band
            }
            1 => {
                let jc = jitter((196, 184, 158), x, y, 10);
                (pick(x, y, &['°', '˚', '·']), rgb(jc.0, jc.1, jc.2)) // scree
            }
            _ => {
                // Lowland climate biomes: swamp, desert, savanna,
                // forest, grass/tundra. Gates sit on the moisture scale
                // life grows on (equilibrium canopy crosses ~0.3): the
                // old `m > 0.05` painted forest on any trace of water,
                // even barren frozen ground (2026-10-14: grass extinct,
                // cold seed-42 lowlands drawn as woodland at veg 0).
                if rel < SWAMP_REL && m > SWAMP_MOIST && temp > 0.0 {
                    // Swamp: murky waterlogged green, its own color.
                    let jc = jitter((92, 146, 116), x, y, 10);
                    (pick(x, y, &['Ψ', 'ψ', 'ξ']), rgb(jc.0, jc.1, jc.2))
                } else if temp > 0.65 && m < DESERT_MOIST {
                    // Desert: hot and truly dry — deep aridity, sand.
                    let jc = jitter((228, 206, 132), x, y, 12);
                    (pick(x, y, &['∴', '∵', '∷']), rgb(jc.0, jc.1, jc.2))
                } else if temp > 0.55 && m < SAVANNA_MOIST {
                    // Savanna: hot with seasonal moisture — golden dry
                    // grass ring between the jungle and the desert.
                    let jc = jitter((204, 190, 106), x, y, 12);
                    (pick(x, y, &[';', ':']), rgb(jc.0, jc.1, jc.2))
                } else if temp < TUNDRA_TEMP && sim.life.vegetation[i] <= FOREST_VEG
                {
                    // Tundra: cold open ground, grey-green — no canopy
                    // grows below freezing.
                    let jc = jitter((150, 164, 146), x, y, 10);
                    (
                        pick(x, y, &['"', '\'', '˙', ',', '`', '¸']),
                        rgb(jc.0, jc.1, jc.2),
                    )
                } else if sim.life.vegetation[i] > FOREST_VEG {
                    // Mixed canopy: hash-picked greens make a forest
                    // read as woodland, not one flat color.
                    const CANOPY: [(u8, u8, u8); 5] = [
                        (58, 138, 88),   // dark pine
                        (88, 178, 108),  // forest green
                        (128, 203, 113), // spring green
                        (78, 190, 130),  // tropical
                        (103, 170, 125), // taiga
                    ];
                    let c = pick(y, x, &CANOPY); // swapped hash: decorrelated from the glyph
                    (pick(x, y, &['♣', '♠', '§']), rgb(c.0, c.1, c.2)) // forest
                } else {
                    // Living grass: color follows biomass and the
                    // season — meadows green in after rain and dry out
                    // in droughts. Both inputs move over hundreds of
                    // ticks: visible life, no flicker (glyph stays
                    // position-hashed).
                    let lush = (sim.life.vegetation[i] * 1.6).clamp(0.0, 1.0) * 0.6
                        + (temp * 2.0).clamp(0.0, 1.0) * 0.4;
                    let c = (
                        208.0 + (150.0 - 208.0) * lush,
                        196.0 + (204.0 - 196.0) * lush,
                        110.0 + (92.0 - 110.0) * lush,
                    );
                    let jc = jitter((c.0 as u8, c.1 as u8, c.2 as u8), x, y, 14);
                    (
                        pick(x, y, &['"', '\'', '˙', ',', '`', '¸']),
                        rgb(jc.0, jc.1, jc.2), // grass: lush ↔ dry
                    )
                }
            }
        }
    };
    if s > 0.02 {
        // Light snow: the terrain shows through with a cool tint.
        let t = (s / 0.16).min(1.0) * 0.4;
        (result.0, mix_toward(result.1, (228, 237, 249), t))
    } else {
        result
    }
}

/// Position hash with proper mixing (multiply-xorshift chain): a
/// naive `x·31 + y·97` showed visible diagonal lattice patterns in
/// the grass. Never time-based — glyphs must not flicker.
fn cell_hash(x: usize, y: usize) -> u32 {
    let mut h =
        (x as u32).wrapping_mul(0x9E3779B1) ^ (y as u32).wrapping_mul(0x85EBCA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2AE35);
    h ^ (h >> 16)
}

/// Stable per-cell variant pick: a position hash (never time — glyphs
/// must not flicker) chooses among narrow-width alternatives.
fn pick<T: Copy>(x: usize, y: usize, variants: &[T]) -> T {
    variants[(cell_hash(x, y) as usize) % variants.len()]
}

/// Deterministic per-cell color jitter (±`amt` per channel), hashed
/// from position only — varied terrain without flicker.
fn jitter(c: (u8, u8, u8), x: usize, y: usize, amt: u8) -> (u8, u8, u8) {
    let h = cell_hash(x, y);
    let j = |salt: u32| -> i32 {
        let span = 2 * amt as usize + 1;
        ((h.wrapping_mul(salt) >> 12) % span as u32) as i32 - amt as i32
    };
    let cl = |v: i32| v.clamp(0, 255) as u8;
    (
        cl(c.0 as i32 + j(3)),
        cl(c.1 as i32 + j(5)),
        cl(c.2 as i32 + j(7)),
    )
}

/// Plates view: drift arrows at seeds, boundaries typed by stress (red
/// convergent — ranges grow here, cyan divergent — rift, yellow
/// transform — shear, quakes), interiors in dim per-plate hues.
/// The sky over one cell: light coverage milks the terrain's color
/// (true translucency without glyph dithering), heavy coverage takes
/// over with cloud glyphs by density — fresh clouds near-white,
/// rain-loaded cells steel-blue. Over open water the sea shows
/// through until full storm cover.
fn cloud_overlay(
    sim: &Sim,
    i: usize,
    base: (char, style::Color),
) -> (char, style::Color) {
    let cover = sim.climate.cover_ema[i];
    if cover <= 0.15 {
        return base;
    }
    let over_water = matches!(base.0, '≈' | '≋' | '~');
    if cover > 0.85 {
        return ('▓', rgb(172, 186, 204)); // storm cell, rain-loaded
    }
    if !over_water && cover > 0.65 {
        return ('▒', rgb(233, 238, 244)); // dense cloud
    }
    if !over_water && cover > 0.35 {
        return ('░', rgb(243, 246, 250)); // cloud body
    }
    // Haze: keep the base glyph, milk its color toward cloud-white.
    let t = if over_water {
        ((cover - 0.15) / 0.7) * 0.6
    } else {
        ((cover - 0.15) / 0.2) * 0.7
    };
    (base.0, mix_toward(base.1, (241, 245, 249), t))
}

/// Linear color mix toward a target RGB.
fn mix_toward(c: style::Color, to: (u8, u8, u8), t: f32) -> style::Color {
    match c {
        style::Color::Rgb { r, g, b } => {
            let mix = |a: u8, b_: u8| (a as f32 + (b_ as f32 - a as f32) * t) as u8;
            rgb(mix(r, to.0), mix(g, to.1), mix(b, to.2))
        }
        other => other,
    }
}

fn plates_view(sim: &Sim, x: usize, y: usize) -> (char, style::Color) {
    let world = &sim.world;
    let width = world.w;
    let i = y * width + x;
    // Mantle plumes first: the active vents, feeding the island trails.
    for h in &sim.tectonics.hotspots {
        if h.x as usize == x && h.y as usize == y {
            return ('◉', rgb(255, 120, 40));
        }
    }
    // Drift arrow at each plate's seed cell: where it's going.
    for p in &sim.tectonics.plates {
        if p.x as usize == x && p.y as usize == y {
            let glyph = if p.dx.abs() > p.dy.abs() {
                if p.dx > 0.0 { '˃' } else { '˂' }
            } else if p.dy > 0.0 {
                '˅'
            } else {
                '˄'
            };
            return (glyph, rgb(245, 246, 250));
        }
    }
    let my = sim.tectonics.plate[i];
    let mut kind = 0u8; // 1 conv, 2 div, 3 transform
    for (dx, dy) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
        let ny = y as isize + dy;
        if ny < 0 || ny >= world.h as isize {
            continue;
        }
        let nx = (x as isize + dx).rem_euclid(world.w as isize) as usize;
        let j = ny as usize * width + nx;
        let other = sim.tectonics.plate[j];
        if other == my {
            continue;
        }
        let (pa, pb) = (
            &sim.tectonics.plates[my as usize],
            &sim.tectonics.plates[other as usize],
        );
        let vr = (pa.dx - pb.dx, pa.dy - pb.dy);
        let conv = -(vr.0 * dx as f32 + vr.1 * dy as f32);
        let k = if conv > PLATE_CONV_T {
            1
        } else if conv < -PLATE_CONV_T {
            2
        } else {
            3
        };
        if k == 1 || kind == 0 {
            kind = k;
        }
    }
    if kind > 0 {
        return match kind {
            1 => ('≡', rgb(235, 90, 70)),
            2 => ('¦', rgb(90, 200, 230)),
            _ => ('≈', rgb(230, 210, 80)),
        };
    }
    // Plate interior: its own dim hue.
    ('·', plate_hue(my))
}

/// Heightmap view (round 5a): brightness-monotonic hypsometric bands —
/// luminance IS height, strictly, so the map reads at a glance. Below
/// sea: bathymetry blues. Above: dark-green lowlands climbing to
/// near-white peaks. A fake NW sun (hillshade from the elevation
/// gradient) multiplies in, lighting ridges and shadowing valleys.
fn height_view(world: &World, x: usize, y: usize) -> (char, style::Color) {
    let width = world.w;
    let height = world.h;
    let i = y * width + x;
    let rel = world.elevation[i] - world.sea_level;

    // Hillshade: slope facing the light (up toward NW) is lit.
    let h = |xx: usize, yy: usize| {
        world.elevation[(yy.min(height - 1)) * width + (xx % width)]
    };
    let west = (x + width - 1) % width;
    let dhdx = h(x + 1, y) - h(west, y);
    let dhdy = h(x, y + 1) - h(x, y.saturating_sub(1));
    let shade = (1.0 + 9.0 * (dhdx + dhdy)).clamp(0.55, 1.45);

    let (rgbv, glyph) = if rel < 0.0 {
        // Bathymetry: darker = deeper (4 bands).
        let depth = (-rel / 0.4).clamp(0.0, 1.0);
        match (depth * 4.0) as usize {
            0 => ((36, 78, 122), '~'),
            1 => ((26, 56, 96), '≈'),
            2 => ((18, 38, 74), '≈'),
            _ => ((12, 22, 50), '≈'),
        }
    } else {
        // Land: eight strictly-brightening bands (luminance is height)
        // mapped over the populated relief range — the top is the
        // summit level (98th land percentile, refreshed with eustasy).
        // Topping out at the rock level (85th) painted a third of the
        // land white (user objection 2, 2026-10-14).
        // Convex (t²) so white stays rare on broad orogens: strictly
        // monotonic in height still, but the top band needs ~94% of
        // the summit level.
        let top = world.peak_level.max(0.08);
        let t = (rel / top).clamp(0.0, 1.0).powi(2);
        match (t * 8.0) as usize {
            0 => ((48, 108, 64), '·'),
            1 => ((78, 136, 70), '·'),
            2 => ((110, 160, 74), '░'),
            3 => ((140, 168, 84), '░'),
            4 => ((170, 172, 100), '▒'),
            5 => ((184, 176, 138), '▒'),
            6 => ((200, 194, 176), '▒'),
            _ => ((252, 252, 253), '▓'), // summits only
        }
    };

    let lit = |c: u8| ((c as f32 * shade).clamp(0.0, 255.0)) as u8;
    (
        glyph,
        style::Color::Rgb {
            r: lit(rgbv.0),
            g: lit(rgbv.1),
            b: lit(rgbv.2),
        },
    )
}

/// Hydro view: depth ramp + flux direction vectors.
fn hydro_view(
    world: &World,
    hydro: &Hydro,
    x: usize,
    y: usize,
) -> (char, style::Color) {
    let i = y * world.w + x;
    let d = world.water[i];
    if let Some(water) = network_river(hydro, world, i) {
        return water; // the drainage network (terrain rivers + lakes)
    }
    if d > WET_MIN {
        // Below sea level is ocean: depth ramp only, no flow vectors —
        // currents over the sea would draw as pulsing river glyphs.
        // On land, arrows only where the flow is channel-like.
        if world.elevation[i] >= world.sea_level {
            let flux_mag = flux_out(&hydro.flux_ema[i]);
            if flux_mag > RIVER_FLUX && d < 0.08 {
                return (river_glyph(&hydro.flux_ema[i]), rgb(80, 200, 220));
            }
        }
        let t = (hydro.water_ema[i] / 0.2).min(1.0);
        return ('~', ramp_water(t));
    }
    ('·', rgb(40, 40, 45))
}

/// Temperature view: the derived field, in three density bands.
fn temp_view(world: &World, x: usize, y: usize) -> (char, style::Color) {
    let i = y * world.w + x;
    let t = ((temp_at(world, y, i, world.time) + 0.4) / 1.4).clamp(0.0, 1.0);
    let glyph = match (t * 3.0) as usize {
        0 => '░',
        1 => '▒',
        _ => '▓',
    };
    (glyph, ramp_temp(t))
}

/// Wind view: direction glyph, speed ramp — reading the simulated
/// momentum field.
fn wind_view(
    climate: &Climate,
    world: &World,
    x: usize,
    y: usize,
) -> (char, style::Color) {
    let (u, v) = climate.wind_at(world, x, y);
    let glyph = if u.abs() > v.abs() {
        if u > 0.0 { '˃' } else { '˂' }
    } else if v > 0.0 {
        '˅'
    } else {
        '˄'
    };
    let speed = (u * u + v * v).sqrt().min(1.4) / 1.4;
    (glyph, ramp_wind(speed))
}

/// Humidity view: the decaying rainfall memory.
fn humidity_view(
    climate: &Climate,
    width: usize,
    x: usize,
    y: usize,
) -> (char, style::Color) {
    let i = y * width + x;
    let t = (climate.humidity[i] / 0.3).min(1.0);
    let glyph = match (t * 3.0) as usize {
        0 => '·',
        1 => '░',
        _ => '▒',
    };
    (glyph, ramp_humidity(t))
}

/// Aquifer view: the water table — groundwater density, wet ground
/// blue-green, dry uplands warm tan, the sea a dim backdrop.
fn aquifer_view(world: &World, x: usize, y: usize) -> (char, style::Color) {
    let i = y * world.w + x;
    if world.elevation[i] < world.sea_level {
        return (' ', rgb(20, 28, 44)); // the sea: backdrop only
    }
    let a = world.aquifer[i];
    let glyph = if a > 0.6 {
        '▒'
    } else if a > 0.3 {
        '░'
    } else {
        '·'
    };
    let stops: [(f32, (u8, u8, u8)); 3] = [
        (0.0, (176, 164, 112)), // dry tan
        (0.5, (116, 172, 140)), // damp
        (1.0, (58, 150, 190)),  // saturated
    ];
    (glyph, ramp(&stops, a.clamp(0.0, 1.0)))
}

/// Total outflux of a cell (positive pipe fluxes only).
fn flux_out(flux: &[f32; 4]) -> f32 {
    flux.iter().filter(|f| **f > 0.0).sum()
}

/// River direction glyph: the strongest outflux direction, pointing
/// downstream.
fn river_glyph(flux: &[f32; 4]) -> char {
    let dir = flux
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(dir, _)| dir)
        .unwrap_or(3);
    dir_glyph(dir)
}

/// Flow-direction glyph in flux order: 0 left, 1 right, 2 up, 3 down.
fn dir_glyph(dir: usize) -> char {
    match dir {
        0 => '˂',
        1 => '˃',
        2 => '˄',
        _ => '˅',
    }
}

/// Drainage-network water at a land cell (the hydro layer's terrain
/// rivers and river-fed lakes): glyph and color.
fn network_river(
    hydro: &Hydro,
    world: &World,
    i: usize,
) -> Option<(char, style::Color)> {
    let r = hydro.river[i];
    if r == 0 || world.elevation[i] < world.sea_level {
        return None;
    }
    Some(if r == LAKE {
        ('~', rgb(78, 178, 178))
    } else {
        (dir_glyph(r as usize - 1), rgb(96, 214, 232))
    })
}

/// Elevation color ramp: quantized bright bands — green lowlands →
/// olive → tan → grey rock → white peaks. Hard bands read as contour
/// lines. Brightness pass (round C, 2026-10-12): luminous greens and
/// tans, sea included.
fn ramp_water(t: f32) -> style::Color {
    let stops: [(f32, (u8, u8, u8)); 3] = [
        (0.0, (150, 205, 235)),
        (0.5, (80, 150, 205)),
        (1.0, (36, 90, 160)),
    ];
    ramp(&stops, t.clamp(0.0, 1.0))
}

fn ramp_temp(t: f32) -> style::Color {
    let stops: [(f32, (u8, u8, u8)); 4] = [
        (0.0, (60, 90, 200)),
        (0.4, (80, 160, 170)),
        (0.7, (220, 190, 90)),
        (1.0, (220, 80, 50)),
    ];
    ramp(&stops, t.clamp(0.0, 1.0))
}

fn ramp_wind(t: f32) -> style::Color {
    let stops: [(f32, (u8, u8, u8)); 3] = [
        (0.0, (50, 60, 70)),
        (0.5, (110, 140, 170)),
        (1.0, (220, 235, 250)),
    ];
    ramp(&stops, t.clamp(0.0, 1.0))
}

fn ramp_humidity(t: f32) -> style::Color {
    let stops: [(f32, (u8, u8, u8)); 3] = [
        (0.0, (200, 170, 80)),
        (0.5, (150, 170, 100)),
        (1.0, (60, 140, 170)),
    ];
    ramp(&stops, t.clamp(0.0, 1.0))
}

fn ramp(stops: &[(f32, (u8, u8, u8))], t: f32) -> style::Color {
    let t = t.clamp(0.0, 1.0);
    let mut lo = stops[0];
    let mut hi = stops[stops.len() - 1];
    for w in stops.windows(2) {
        if t >= w[0].0 && t <= w[1].0 {
            lo = w[0];
            hi = w[1];
            break;
        }
    }
    let span = (hi.0 - lo.0).max(1e-6);
    let f = ((t - lo.0) / span).clamp(0.0, 1.0);
    let lerp_c = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * f) as u8;
    style::Color::Rgb {
        r: lerp_c(lo.1.0, hi.1.0),
        g: lerp_c(lo.1.1, hi.1.1),
        b: lerp_c(lo.1.2, hi.1.2),
    }
}

/// HUD name for a display layer.
fn layer_name(layer: Layer) -> &'static str {
    match layer {
        Layer::Biome => "BIOME",
        Layer::Plates => "PLATES (red=convergent, cyan=rift, yellow=shear)",
        Layer::Height => "HEIGHTMAP",
        Layer::Hydro => "HYDRO (depth + flow)",
        Layer::Temp => "TEMPERATURE",
        Layer::Wind => "WIND",
        Layer::Humidity => "HUMIDITY",
        Layer::Aquifer => "AQUIFER (· dry, ░ damp, ▒ saturated)",
    }
}

/// Distinct dim hue per plate index.
fn plate_hue(idx: u8) -> style::Color {
    let hues = [
        (96, 62, 124),
        (58, 100, 122),
        (124, 92, 50),
        (50, 112, 62),
        (124, 60, 60),
        (68, 70, 132),
        (112, 102, 70),
    ];
    let (r, g, b) = hues[(idx as usize) % hues.len()];
    style::Color::Rgb { r, g, b }
}

/// RGB color shorthand.
pub(crate) fn rgb(r: u8, g: u8, b: u8) -> style::Color {
    style::Color::Rgb { r, g, b }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pocketworld::sim::test_sim;

    #[test]
    fn temp_view_shows_latitude_bands() {
        // Regression: the glyph bands are thirds of [0,1] (`t * 3.0`),
        // never a bare truncation that dumps everything into band 0.
        let t = test_sim(42, 100, 40);
        let mut renderer = Renderer::new();
        renderer.view = Layer::Temp;
        let mut glyphs = std::collections::HashSet::new();
        for y in 0..40 {
            for x in 0..100 {
                let (ch, _) = renderer.classify(&t, x, y);
                glyphs.insert(ch);
            }
        }
        assert!(glyphs.len() >= 3, "temp view collapsed to {glyphs:?}");
    }

    #[test]
    fn tier_hysteresis_resists_jitter() {
        // A cell hovering at a tier threshold must not flip with
        // ±0.002 erosion noise (the mountain flicker, 2026-10-14).
        let mut t = test_sim(42, 20, 10);
        t.world.rock_level = 0.30;
        t.world.scree_level = 0.20;
        let mut sticky = vec![0u8; t.world.w * t.world.h];
        let i = 5usize;
        // Entering rock takes the entry margin (0.30 + 0.03).
        assert_eq!(sticky_tier(&mut sticky, &t.world, i, 0.335), 2);
        // Hovering and jittering across the raw threshold stays rock.
        assert_eq!(sticky_tier(&mut sticky, &t.world, i, 0.3001), 2);
        assert_eq!(sticky_tier(&mut sticky, &t.world, i, 0.2995), 2);
        // A real drop past the leave-margin lands in scree.
        assert_eq!(sticky_tier(&mut sticky, &t.world, i, 0.265), 1);
        assert_eq!(sticky_tier(&mut sticky, &t.world, i, 0.15), 1);
        // And out of scree only past its leave-margin.
        assert_eq!(sticky_tier(&mut sticky, &t.world, i, 0.10), 0);
    }

    #[test]
    fn late_world_glyph_census() {
        // After long simulation, the visible map must still read as a world:
        // water and land dominate; rain is weather, not wallpaper.
        let mut t = test_sim(42, 100, 40);
        for _ in 0..900 {
            t.tick();
        }
        let mut renderer = Renderer::new();
        let mut rain = 0usize;
        let mut river = 0usize;
        let mut water = 0usize;
        let mut kinds = std::collections::HashSet::new();
        for y in 0..40 {
            for x in 0..100 {
                let (ch, _) = renderer.classify(&t, x, y);
                kinds.insert(ch);
                match ch {
                    '|' | '⁄' => rain += 1,
                    '˂' | '˃' | '˄' | '˅' => river += 1,
                    '≈' | '≋' | '~' => water += 1,
                    _ => {}
                }
            }
        }
        eprintln!(
            "CENSUS rain={rain}/4000 river={river}/4000 standing={water}/4000 distinct={} below_sea={} wet={}",
            kinds.len(),
            t.world
                .elevation
                .iter()
                .filter(|e| **e < t.world.sea_level)
                .count(),
            t.world.water.iter().filter(|d| **d > 0.004).count()
        );
        assert!(
            rain < 800,
            "rain covers {rain}/4000 cells — wallpaper, not weather"
        );
        assert!(
            water + river > 800,
            "hydrosphere nearly vanished: {water} standing + {river} flowing"
        );
        assert!(river > 0, "no flowing water formed");
        assert!(
            kinds.len() >= 6,
            "biome diversity too low: {} kinds",
            kinds.len()
        );
    }

    #[test]
    fn lowlands_keep_a_biome_mix() {
        // Biome-round lock (2026-10-14): `m > 0.05` painted forest on
        // any trace of water — grass went extinct by 3000 ticks and
        // cold barren ground drew as woodland. The default world must
        // keep forest, open grass/tundra, and a dry hot ring together.
        let mut t = test_sim(7, 100, 40);
        for _ in 0..3000 {
            t.tick();
        }
        let mut renderer = Renderer::new();
        let (mut land, mut forest, mut open, mut dry) =
            (0usize, 0usize, 0usize, 0usize);
        for y in 1..40 {
            for x in 0..100 {
                let (ch, _) = renderer.classify(&t, x, y);
                match ch {
                    '≈' | '≋' | '~' => continue,
                    '♣' | '♠' | '§' => forest += 1,
                    '"' | '\'' | '˙' | '`' | '¸' => open += 1,
                    ';' | ':' | '∴' | '∵' | '∷' => dry += 1,
                    _ => {}
                }
                land += 1;
            }
        }
        let pct = |v: usize| 100.0 * v as f32 / land as f32;
        eprintln!(
            "BIOMES land {land}: forest {:.0}% open {:.0}% dry {:.0}%",
            pct(forest),
            pct(open),
            pct(dry)
        );
        assert!(pct(forest) > 15.0, "forests collapsed: {:.0}%", pct(forest));
        assert!(
            pct(forest) < 60.0,
            "forest monoculture: {:.0}%",
            pct(forest)
        );
        assert!(pct(open) > 5.0, "grass/tundra extinct: {:.0}%", pct(open));
        assert!(pct(dry) > 2.0, "no savanna/desert ring: {:.0}%", pct(dry));
    }

    #[test]
    fn every_view_renders_without_panicking() {
        let mut t = test_sim(31, 60, 20);
        let mut renderer = Renderer::new();
        let views = [
            Layer::Biome,
            Layer::Plates,
            Layer::Height,
            Layer::Hydro,
            Layer::Temp,
            Layer::Wind,
            Layer::Humidity,
        ];
        // Cycle through all views a couple of times, ticking between.
        for _ in 0..2 {
            for _view in views {
                for y in 0..t.world.h {
                    for x in 0..t.world.w {
                        let _ = renderer.classify(&t, x, y);
                    }
                }
                t.tick();
                renderer.view = renderer.view.next();
            }
        }
    }
}
