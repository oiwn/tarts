use crate::buffer::{Buffer, Cell};
use crate::collision::{self, BoundingBox, Collidable};
use crate::common::{DefaultOptions, TerminalEffect};
use crossterm::style;
use derive_builder::Builder;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static BUTTERFLY_FRAMES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        // Frame 0: wings open
        r#"(\ o /)
 ))¥((
(/ ^ \)"#,
        // Frame 1: wings folded (flap)
        r#" \ o /
  )¥(
 / ^ \ "#,
    ]
});

const PALETTE: [(u8, u8, u8); 5] = [
    (255, 145, 175), // pink
    (255, 200, 90),  // orange
    (150, 210, 255), // sky blue
    (200, 150, 255), // lavender
    (255, 230, 120), // yellow
];

#[derive(Clone)]
struct ButterflyEntity {
    position: (f32, f32),
    velocity: (f32, f32),
    flap_timer: f32,
    frame_idx: usize,
    color: style::Color,
    frame_width: usize,
    frame_height: usize,
}

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct ButterflyOptions {
    #[builder(default = "4")]
    #[serde(skip)]
    pub butterfly_count: u16,

    #[builder(default = "0.35")]
    pub flap_speed: f32,

    #[builder(default = "8.0")]
    pub movement_speed: f32,

    #[builder(default = "0.8")]
    pub wander_strength: f32,

    #[builder(default = "1.0")]
    pub butterfly_coeff: f32,
}

pub struct Butterfly {
    pub screen_size: (u16, u16),
    options: ButterflyOptions,
    buffer: Buffer,
    butterflies: Vec<ButterflyEntity>,
    rng: rand::prelude::ThreadRng,
}

impl ButterflyEntity {
    fn new(position: (f32, f32), rng: &mut rand::prelude::ThreadRng) -> Self {
        let angle = rng.random_range(0.0..(std::f32::consts::PI * 2.0));
        let speed = rng.random_range(1.0..2.5);
        let velocity = (angle.cos() * speed, angle.sin() * speed);

        let pal = PALETTE[rng.random_range(0..PALETTE.len())];
        let color = style::Color::Rgb {
            r: pal.0,
            g: pal.1,
            b: pal.2,
        };

        let frame_lines: Vec<&str> = BUTTERFLY_FRAMES[0].lines().collect();
        let frame_height = frame_lines.len();
        let frame_width = frame_lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);

        Self {
            position,
            velocity,
            flap_timer: 0.0,
            frame_idx: 0,
            color,
            frame_width,
            frame_height,
        }
    }

    fn get_frame_lines(&self) -> Vec<String> {
        BUTTERFLY_FRAMES[self.frame_idx]
            .lines()
            .map(|line| line.to_string())
            .collect()
    }

    fn update(
        &mut self,
        dt: f32,
        screen_size: (u16, u16),
        flap_speed: f32,
        movement_speed: f32,
        wander_strength: f32,
        rng: &mut rand::prelude::ThreadRng,
    ) {
        self.velocity.0 += rng.random_range(-1.0..1.0) * wander_strength * dt;
        self.velocity.1 += rng.random_range(-1.0..1.0) * wander_strength * dt;

        let speed = (self.velocity.0.powi(2) + self.velocity.1.powi(2)).sqrt();
        if speed > 1.2 {
            self.velocity.0 = self.velocity.0 / speed * 1.2;
            self.velocity.1 = self.velocity.1 / speed * 1.2;
        } else if speed < 0.2 {
            let angle = rng.random_range(0.0..(std::f32::consts::PI * 2.0));
            self.velocity.0 = angle.cos() * 0.5;
            self.velocity.1 = angle.sin() * 0.5;
        }

        self.position.0 += self.velocity.0 * movement_speed * dt;
        self.position.1 += self.velocity.1 * movement_speed * dt;

        let width = screen_size.0 as f32;
        let height = screen_size.1 as f32;

        if self.position.0 < 0.0 {
            self.position.0 = 0.0;
            self.velocity.0 = self.velocity.0.abs();
        } else if self.position.0 + self.frame_width as f32 > width {
            self.position.0 = width - self.frame_width as f32;
            self.velocity.0 = -self.velocity.0.abs();
        }

        if self.position.1 < 0.0 {
            self.position.1 = 0.0;
            self.velocity.1 = self.velocity.1.abs();
        } else if self.position.1 + self.frame_height as f32 > height {
            self.position.1 = height - self.frame_height as f32;
            self.velocity.1 = -self.velocity.1.abs();
        }

        self.flap_timer += dt;
        if self.flap_timer >= flap_speed {
            self.flap_timer = 0.0;
            self.frame_idx = (self.frame_idx + 1) % BUTTERFLY_FRAMES.len();
        }
    }
}

impl Collidable for ButterflyEntity {
    fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            x: self.position.0,
            y: self.position.1,
            width: self.frame_width as f32,
            height: self.frame_height as f32,
        }
    }
}

impl TerminalEffect for Butterfly {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        let mut curr_buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);

        for butterfly in &self.butterflies {
            let frame_lines = butterfly.get_frame_lines();
            let base_x = butterfly.position.0.round() as usize;
            let base_y = butterfly.position.1.round() as usize;

            for (y_offset, line) in frame_lines.iter().enumerate() {
                let y = base_y + y_offset;
                if y >= curr_buffer.height {
                    continue;
                }

                for (x_offset, ch) in line.chars().enumerate() {
                    let x = base_x + x_offset;
                    if x >= curr_buffer.width || ch == ' ' {
                        continue;
                    }

                    curr_buffer.set(
                        x,
                        y,
                        Cell::new(ch, butterfly.color, style::Attribute::Bold),
                    );
                }
            }
        }

        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    fn update(&mut self) {
        let dt = 0.033;

        for butterfly in &mut self.butterflies {
            butterfly.update(
                dt,
                self.screen_size,
                self.options.flap_speed,
                self.options.movement_speed,
                self.options.wander_strength,
                &mut self.rng,
            );
        }

        self.check_butterfly_collisions();
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
    }

    fn reset(&mut self) {
        *self = Self::new(self.options.clone(), self.screen_size);
    }
}

impl Butterfly {
    pub fn new(options: ButterflyOptions, screen_size: (u16, u16)) -> Self {
        let mut rng = rand::rng();
        let buffer = Buffer::new(screen_size.0 as usize, screen_size.1 as usize);

        let width = screen_size.0 as f32;
        let height = screen_size.1 as f32;

        let mut butterflies = Vec::with_capacity(options.butterfly_count as usize);
        for _ in 0..options.butterfly_count {
            let position = (
                rng.random_range(0.0..(width - 8.0).max(1.0)),
                rng.random_range(0.0..(height - 3.0).max(1.0)),
            );
            butterflies.push(ButterflyEntity::new(position, &mut rng));
        }

        Self {
            screen_size,
            options,
            buffer,
            butterflies,
            rng,
        }
    }

    fn check_butterfly_collisions(&mut self) {
        if self.butterflies.len() < 2 {
            return;
        }

        for (i, j) in collision::find_collisions(&self.butterflies) {
            let dx =
                self.butterflies[i].position.0 - self.butterflies[j].position.0;
            let dy =
                self.butterflies[i].position.1 - self.butterflies[j].position.1;

            let (push_x, push_y) = if dx == 0.0 && dy == 0.0 {
                let angle =
                    self.rng.random_range(0.0..(std::f32::consts::PI * 2.0));
                (angle.cos(), angle.sin())
            } else {
                let len = (dx * dx + dy * dy).sqrt();
                (dx / len, dy / len)
            };

            self.butterflies[i].velocity.0 += push_x * 0.5;
            self.butterflies[i].velocity.1 += push_y * 0.5;
            self.butterflies[j].velocity.0 -= push_x * 0.5;
            self.butterflies[j].velocity.1 -= push_y * 0.5;
        }
    }
}

impl DefaultOptions for Butterfly {
    type Options = ButterflyOptions;

    fn default_options(width: u16, height: u16) -> Self::Options {
        let screen_area = width as f32 * height as f32;
        let butterfly_count = (screen_area / 1000.0).clamp(2.0, 10.0) as u16;

        ButterflyOptionsBuilder::default()
            .butterfly_count(butterfly_count)
            .flap_speed(0.35_f32)
            .movement_speed(8.0_f32)
            .wander_strength(0.8_f32)
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_butterfly_effect() {
        let options = ButterflyOptionsBuilder::default().build().unwrap();
        let effect = Butterfly::new(options, (80, 24));
        assert_eq!(effect.screen_size, (80, 24));
    }
}
