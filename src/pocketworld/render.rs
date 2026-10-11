//! Rendering: pure functions of the simulation state. `Renderer` holds
//! the active display layer (cycled with `l`/`k`); every view reads the
//! world and the layer states without owning any of them. Color carries
//! elevation, glyph carries state; in the heightmap view brightness =
//! height, strictly.

use crate::pocketworld::climate::{Climate, TEMP_SNOW, temp_at, wind_at};
use crate::pocketworld::hydro::{Hydro, OCEAN, RIVER_FLUX, WET_MIN};
use crate::pocketworld::life::{ASH, BURNING};
use crate::pocketworld::sim::Sim;
use crate::pocketworld::tectonics::PLATE_CONV_T;
use crate::pocketworld::world::World;
use crossterm::style;

/// Water depth above which water renders — thin rain sheets on land stay
/// invisible instead of painting puddle noise over whole regions.
const RENDER_WET_MIN: f32 = 0.012;

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
            Layer::Humidity => Layer::Biome,
        }
    }

    pub(crate) fn prev(self) -> Self {
        match self {
            Layer::Biome => Layer::Humidity,
            Layer::Humidity => Layer::Wind,
            Layer::Wind => Layer::Temp,
            Layer::Temp => Layer::Hydro,
            Layer::Hydro => Layer::Height,
            Layer::Height => Layer::Plates,
            Layer::Plates => Layer::Biome,
        }
    }
}

/// Display state: which layer view is on screen.
pub(crate) struct Renderer {
    pub(crate) view: Layer,
}

impl Renderer {
    pub(crate) fn new() -> Self {
        Self { view: Layer::Biome }
    }

    /// Glyph + color for one cell under the active view.
    pub(crate) fn classify(
        &self,
        sim: &Sim,
        x: usize,
        y: usize,
    ) -> (char, style::Color) {
        match self.view {
            Layer::Plates => plates_view(sim, x, y),
            Layer::Height => height_view(&sim.world, x, y),
            Layer::Hydro => hydro_view(&sim.world, &sim.hydro, x, y),
            Layer::Temp => temp_view(&sim.world, x, y),
            Layer::Wind => wind_view(&sim.world, x, y),
            Layer::Humidity => humidity_view(&sim.climate, sim.world.w, x, y),
            Layer::Biome => biome_view(sim, x, y),
        }
    }

    /// HUD label for the active layer, written on the top row.
    pub(crate) fn hud_label(&self) -> String {
        format!(" LAYER: {} [l/k]", layer_name(self.view))
    }
}

/// Biome view (the default). Priority (top wins): rain on land → cloud
/// → water → land biomes. Rain never draws over the sea; clouds draw
/// over everything (they are the visible sky). Fire burns over biomes.
fn biome_view(sim: &Sim, x: usize, y: usize) -> (char, style::Color) {
    let world = &sim.world;
    let width = world.w;
    let i = y * width + x;
    let d = world.water[i];
    let rel = world.elevation[i] - world.sea_level;

    let dry_land = d <= RENDER_WET_MIN;
    if sim.climate.rain[i] > 1e-5
        && dry_land
        && (x + y + (world.time * 6.0) as usize) % 3 == 0
    {
        let glyph = if (x + y + (world.time * 6.0) as usize) % 2 == 0 {
            '|'
        } else {
            '⁄'
        };
        return (glyph, rgb(170, 200, 230));
    }
    if sim.climate.cloud_cover[i] > 0.15
        && (x + y + (world.time * 3.0) as usize) % 2 == 0
    {
        // Dithered translucency: cloud draws on the parity checkerboard,
        // terrain shows through the other cells — fronts shimmer.
        return if sim.climate.cloud_cover[i] > 0.5 {
            ('▓', rgb(228, 231, 236))
        } else {
            ('▒', rgb(198, 204, 212))
        };
    }

    if d > RENDER_WET_MIN {
        let flux_mag = flux_out(&sim.hydro.flux[i]);
        if flux_mag > RIVER_FLUX {
            // River: motion, not a body — glyph points downstream.
            return (river_glyph(&sim.hydro.flux[i]), rgb(80, 200, 220));
        }
        if sim.hydro.class[i] & OCEAN != 0 {
            return if d > 0.15 {
                ('≈', rgb(18, 60, 110))
            } else {
                ('~', rgb(45, 110, 160))
            };
        }
        // Unflagged standing water: lake.
        return ('~', rgb(60, 150, 150));
    }

    // Land: color carries elevation, glyph carries state.
    let temp = temp_at(world, y, i, world.time);
    let m = sim.climate.humidity[i];
    let height_t = (rel / 0.8).clamp(0.0, 1.0);
    let color = ramp_height(height_t);

    if sim.life.fire[i] == BURNING {
        let frames = ['✱', '❋', '✹'];
        let f = (x + y + (world.time * 8.0) as usize) % 3;
        return (frames[f], rgb(250, 140, 40));
    }
    if sim.climate.snow[i] > 0.02 {
        return ('*', rgb(235, 240, 245)); // snow-pack (renders over cold land)
    }
    if temp < TEMP_SNOW && rel > 0.3 {
        return ('*', rgb(235, 240, 245)); // high cold peaks stay white
    }
    if sim.life.fire[i] == ASH {
        return ('░', rgb(70, 65, 60)); // burnt ground
    }
    match rel {
        r if r < 0.02 => ('.', rgb(194, 178, 128)), // shore
        r if world.rock_level > 0.0 && r > world.rock_level => {
            ('▲', color) // rock — the mountain band
        }
        r if world.scree_level > 0.0 && r > world.scree_level => {
            ('°', color) // scree
        }
        r if r < 0.08 && m > 0.08 => ('Ψ', color), // swamp
        _ if temp > 0.75 && m < 0.005 => ('∴', color), // hot + dry desert
        _ => {
            if sim.life.vegetation[i] > 0.5 || m > 0.02 {
                ('♣', color) // forest
            } else {
                ('"', color) // grass
            }
        }
    }
}

/// Plates view: drift arrows at seeds, boundaries typed by stress (red
/// convergent — ranges grow here, cyan divergent — rift, yellow
/// transform — shear, quakes), interiors in dim per-plate hues.
fn plates_view(sim: &Sim, x: usize, y: usize) -> (char, style::Color) {
    let world = &sim.world;
    let width = world.w;
    let i = y * width + x;
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
        // Bathymetry: darker = deeper.
        let depth = (-rel / 0.35).clamp(0.0, 1.0);
        match (depth * 3.0) as usize {
            0 => ((30, 64, 104), '~'),
            1 => ((20, 42, 76), '≈'),
            _ => ((13, 22, 46), '≈'),
        }
    } else {
        // Land: six strictly-brightening bands.
        let t = (rel / 0.9).clamp(0.0, 1.0);
        match (t * 6.0) as usize {
            0 => ((38, 94, 56), '·'),
            1 => ((72, 132, 62), '·'),
            2 => ((122, 152, 68), '░'),
            3 => ((172, 162, 86), '▒'),
            4 => ((204, 200, 192), '▓'),
            _ => ((252, 252, 253), '▓'),
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
    if d > WET_MIN {
        let flux_mag = flux_out(&hydro.flux[i]);
        if flux_mag > RIVER_FLUX {
            return (river_glyph(&hydro.flux[i]), rgb(80, 200, 220));
        }
        let t = (d / 0.2).min(1.0);
        return ('~', ramp_water(t));
    }
    ('·', rgb(40, 40, 45))
}

/// Temperature view: the derived field, in three density bands.
fn temp_view(world: &World, x: usize, y: usize) -> (char, style::Color) {
    let i = y * world.w + x;
    let t = ((temp_at(world, y, i, world.time) + 0.4) / 1.4).clamp(0.0, 1.0);
    let glyph = match t as usize {
        0 => '░',
        1 => '▒',
        _ => '▓',
    };
    (glyph, ramp_temp(t))
}

/// Wind view: direction glyph, speed ramp.
fn wind_view(world: &World, x: usize, y: usize) -> (char, style::Color) {
    let (u, v) = wind_at(world, x, y);
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
    let glyph = match t as usize {
        0 => '·',
        1 => '░',
        _ => '▒',
    };
    (glyph, ramp_humidity(t))
}

/// Total outflux of a cell (positive pipe fluxes only).
fn flux_out(flux: &[f32; 4]) -> f32 {
    flux.iter().filter(|f| **f > 0.0).sum()
}

/// River direction glyph: the strongest outflux direction, pointing
/// downstream.
fn river_glyph(flux: &[f32; 4]) -> char {
    match flux
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(dir, _)| dir)
    {
        Some(0) => '˂',
        Some(1) => '˃',
        Some(2) => '˄',
        _ => '˅',
    }
}

/// Elevation color ramp: quantized bright bands — green lowlands → olive →
/// tan → grey rock → white peaks. Hard bands read as contour lines.
fn ramp_height(t: f32) -> style::Color {
    let band = (t.clamp(0.0, 1.0) * 5.0) as usize;
    let (r, g, b) = match band {
        0 => (98, 158, 66),
        1 => (140, 168, 74),
        2 => (188, 158, 92),
        3 => (176, 170, 163),
        _ => (240, 244, 248),
    };
    style::Color::Rgb { r, g, b }
}

fn ramp_water(t: f32) -> style::Color {
    let stops: [(f32, (u8, u8, u8)); 3] = [
        (0.0, (125, 185, 220)),
        (0.5, (60, 130, 190)),
        (1.0, (22, 62, 120)),
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
    fn late_world_glyph_census() {
        // After long simulation, the visible map must still read as a world:
        // water and land dominate; rain is weather, not wallpaper.
        let mut t = test_sim(42, 100, 40);
        for _ in 0..900 {
            t.tick();
        }
        let renderer = Renderer::new();
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
                    '≈' | '~' => water += 1,
                    _ => {}
                }
            }
        }
        eprintln!(
            "CENSUS rain={rain}/4000 river={river}/4000 standing={water}/4000 distinct={}",
            kinds.len()
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
