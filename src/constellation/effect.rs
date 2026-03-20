use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};
use crossterm::style;
use derive_builder::Builder;
use rand::Rng;
use serde::{Deserialize, Serialize};

const STAR_GLYPHS: [char; 4] = ['○', '◦', '*', '✦'];
const DT: f64 = 1.0 / 60.0;

const PALETTE: [(u8, u8, u8); 4] = [
    (110, 150, 240),
    (170, 110, 230),
    (90, 210, 230),
    (190, 150, 255),
];

const DIM_PALETTE: [(u8, u8, u8); 4] =
    [(33, 43, 78), (48, 30, 68), (26, 58, 68), (53, 38, 78)];

const BRIGHT: (u8, u8, u8) = (238, 243, 255);

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct ConstellationOptions {
    #[builder(default = "65")]
    pub star_count: usize,
    #[builder(default = "0.18")]
    pub connect_radius: f64,
    #[builder(default = "4")]
    pub max_connections: usize,
    #[builder(default = "true")]
    pub twinkle: bool,
    #[builder(default = "0.3")]
    pub min_speed: f64,
    #[builder(default = "1.5")]
    pub max_speed: f64,
}

#[derive(Clone, Debug)]
struct Star {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    twinkle: f64,
    twinkle_freq: f64,
    glyph_idx: usize,
    palette_idx: usize,
}

pub struct Constellation {
    screen_size: (u16, u16),
    options: ConstellationOptions,
    buffer: Buffer,
    stars: Vec<Star>,
    connect_dist: f64,
}

impl TerminalEffect for Constellation {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        let curr_buffer = self.draw();
        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    fn update(&mut self) {
        let width = self.screen_size.0 as f64;
        let height = self.screen_size.1 as f64;

        for star in &mut self.stars {
            star.x += star.vx * DT;
            star.y += star.vy * DT;

            if self.options.twinkle {
                star.twinkle += star.twinkle_freq * DT;
            }

            if star.x < 0.0 {
                star.x = -star.x;
                star.vx = -star.vx;
            } else if star.x >= width {
                star.x = 2.0 * width - star.x - 0.01;
                star.vx = -star.vx;
            }

            if star.y < 0.0 {
                star.y = -star.y;
                star.vy = -star.vy;
            } else if star.y >= height {
                star.y = 2.0 * height - star.y - 0.01;
                star.vy = -star.vy;
            }
        }
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
        self.reset();
    }

    fn reset(&mut self) {
        self.buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);
        self.connect_dist = Self::calc_connect_dist(
            self.screen_size.0,
            self.screen_size.1,
            self.options.connect_radius,
        );

        self.stars.clear();
        self.stars.reserve(self.options.star_count);
        for _ in 0..self.options.star_count {
            self.stars.push(self.random_star(true));
        }
    }
}

impl Constellation {
    pub fn new(options: ConstellationOptions, screen_size: (u16, u16)) -> Self {
        let mut effect = Self {
            screen_size,
            options,
            buffer: Buffer::new(screen_size.0 as usize, screen_size.1 as usize),
            stars: Vec::new(),
            connect_dist: 0.0,
        };

        effect.reset();
        effect
    }

    fn calc_connect_dist(width: u16, height: u16, radius_factor: f64) -> f64 {
        ((width as f64).powi(2) + (height as f64).powi(2)).sqrt() * radius_factor
    }

    fn random_star(&self, scattered: bool) -> Star {
        let mut rng = rand::rng();
        let speed =
            rng.random_range(self.options.min_speed..self.options.max_speed);
        let angle = rng.random_range(0.0..(std::f64::consts::PI * 2.0));

        let (x, y) = if scattered {
            (
                rng.random_range(0.0..self.screen_size.0 as f64),
                rng.random_range(0.0..self.screen_size.1 as f64),
            )
        } else {
            match rng.random_range(0..4) {
                0 => (rng.random_range(0.0..self.screen_size.0 as f64), 0.0),
                1 => (
                    rng.random_range(0.0..self.screen_size.0 as f64),
                    self.screen_size.1.saturating_sub(1) as f64,
                ),
                2 => (0.0, rng.random_range(0.0..self.screen_size.1 as f64)),
                _ => (
                    self.screen_size.0.saturating_sub(1) as f64,
                    rng.random_range(0.0..self.screen_size.1 as f64),
                ),
            }
        };

        Star {
            x,
            y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed,
            twinkle: rng.random_range(0.0..(std::f64::consts::PI * 2.0)),
            twinkle_freq: rng.random_range(0.4..1.2),
            glyph_idx: rng.random_range(0..STAR_GLYPHS.len()),
            palette_idx: rng.random_range(0..PALETTE.len()),
        }
    }

    fn draw(&self) -> Buffer {
        let mut next =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);

        self.draw_connections(&mut next);
        self.draw_stars(&mut next);

        next
    }

    fn draw_connections(&self, buffer: &mut Buffer) {
        let mut conn_count = vec![0usize; self.stars.len()];

        for i in 0..self.stars.len() {
            let mut neighbors: Vec<(usize, f64)> = Vec::new();
            for j in (i + 1)..self.stars.len() {
                let dx = self.stars[j].x - self.stars[i].x;
                let dy = self.stars[j].y - self.stars[i].y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance <= self.connect_dist {
                    neighbors.push((j, distance));
                }
            }

            neighbors.sort_by(|a, b| a.1.total_cmp(&b.1));

            for (j, distance) in neighbors {
                if conn_count[i] >= self.options.max_connections
                    || conn_count[j] >= self.options.max_connections
                {
                    continue;
                }

                conn_count[i] += 1;
                conn_count[j] += 1;

                let alpha = (1.0 - distance / self.connect_dist) * 0.55;
                let color = self.connection_color(self.stars[i].palette_idx, alpha);

                self.draw_dotted_line(
                    buffer,
                    self.stars[i].x.round() as i32,
                    self.stars[i].y.round() as i32,
                    self.stars[j].x.round() as i32,
                    self.stars[j].y.round() as i32,
                    color,
                );
            }
        }
    }

    fn draw_stars(&self, buffer: &mut Buffer) {
        for star in &self.stars {
            let brightness = if self.options.twinkle {
                0.55 + 0.45 * star.twinkle.sin()
            } else {
                0.85
            };

            let mut color = self.star_color(star.palette_idx, brightness);
            if brightness > 0.9 {
                let pal = PALETTE[star.palette_idx % PALETTE.len()];
                color = lerp_color(
                    style::Color::Rgb {
                        r: pal.0,
                        g: pal.1,
                        b: pal.2,
                    },
                    style::Color::Rgb {
                        r: BRIGHT.0,
                        g: BRIGHT.1,
                        b: BRIGHT.2,
                    },
                    (brightness - 0.9) * 10.0,
                );
            }

            let x = star.x.round() as i32;
            let y = star.y.round() as i32;
            if self.in_bounds(x, y) {
                buffer.set(
                    x as usize,
                    y as usize,
                    Cell::new(
                        STAR_GLYPHS[star.glyph_idx],
                        color,
                        style::Attribute::Bold,
                    ),
                );
            }
        }
    }

    fn draw_dotted_line(
        &self,
        buffer: &mut Buffer,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: style::Color,
    ) {
        let dx = x1 - x0;
        let dy = y1 - y0;

        let steps = dx.abs().max(dy.abs());
        if steps < 2 {
            return;
        }

        for i in 1..steps {
            let t = i as f64 / steps as f64;
            let x = x0 + (dx as f64 * t + 0.5) as i32;
            let y = y0 + (dy as f64 * t + 0.5) as i32;

            if self.in_bounds(x, y) {
                buffer.set(
                    x as usize,
                    y as usize,
                    Cell::new('·', color, style::Attribute::NormalIntensity),
                );
            }
        }
    }

    fn connection_color(&self, palette_idx: usize, alpha: f64) -> style::Color {
        let dim = DIM_PALETTE[palette_idx % DIM_PALETTE.len()];
        let pal = PALETTE[palette_idx % PALETTE.len()];
        lerp_color(
            style::Color::Rgb {
                r: dim.0,
                g: dim.1,
                b: dim.2,
            },
            style::Color::Rgb {
                r: pal.0,
                g: pal.1,
                b: pal.2,
            },
            alpha,
        )
    }

    fn star_color(&self, palette_idx: usize, brightness: f64) -> style::Color {
        let dim = DIM_PALETTE[palette_idx % DIM_PALETTE.len()];
        let pal = PALETTE[palette_idx % PALETTE.len()];
        lerp_color(
            style::Color::Rgb {
                r: dim.0,
                g: dim.1,
                b: dim.2,
            },
            style::Color::Rgb {
                r: pal.0,
                g: pal.1,
                b: pal.2,
            },
            brightness,
        )
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0
            && x < self.screen_size.0 as i32
            && y >= 0
            && y < self.screen_size.1 as i32
    }
}

fn lerp_color(a: style::Color, b: style::Color, t: f64) -> style::Color {
    let (ar, ag, ab) = as_rgb(a);
    let (br, bg, bb) = as_rgb(b);

    let clamped_t = t.clamp(0.0, 1.0);
    style::Color::Rgb {
        r: (ar as f64 + (br as f64 - ar as f64) * clamped_t) as u8,
        g: (ag as f64 + (bg as f64 - ag as f64) * clamped_t) as u8,
        b: (ab as f64 + (bb as f64 - ab as f64) * clamped_t) as u8,
    }
}

fn as_rgb(color: style::Color) -> (u8, u8, u8) {
    match color {
        style::Color::Rgb { r, g, b } => (r, g, b),
        _ => (255, 255, 255),
    }
}

impl DefaultOptions for Constellation {
    type Options = ConstellationOptions;

    fn default_options(_width: u16, _height: u16) -> Self::Options {
        ConstellationOptionsBuilder::default()
            .star_count(65_usize)
            .connect_radius(0.18)
            .max_connections(4_usize)
            .twinkle(true)
            .min_speed(0.3)
            .max_speed(1.5)
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_constellation_effect() {
        let options = ConstellationOptionsBuilder::default().build().unwrap();
        let effect = Constellation::new(options, (80, 24));
        assert_eq!(effect.screen_size, (80, 24));
    }
}
