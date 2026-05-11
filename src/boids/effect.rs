use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};
use crossterm::style;
use derive_builder::Builder;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum BoidCharset {
    #[default]
    Braille,
    Arrow,
    Simple,
    Dot,
}

impl BoidCharset {
    pub fn chars(&self) -> [char; 8] {
        match self {
            BoidCharset::Braille => ['⣤', '⢰', '⣰', '⡆', '⡇', '⠇', '⠛', '⠙'],
            BoidCharset::Arrow => ['→', '↘', '↓', '↙', '←', '↖', '↑', '↗'],
            BoidCharset::Simple => ['>', '>', 'v', 'v', '<', '<', '^', '^'],
            BoidCharset::Dot => ['•'; 8],
        }
    }
}

// Individual boid
#[derive(Clone)]
struct Boid {
    position: (f32, f32), // Floating point for smooth movement
    velocity: (f32, f32), // Direction vector
    character: char,      // Visual representation
    color: style::Color,  // Color based on velocity/state
}

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct BoidsOptions {
    #[builder(default)]
    #[serde(skip)]
    pub screen_size: (u16, u16),
    #[builder(default = "100")]
    #[serde(skip)]
    pub boid_count: u16,

    #[builder(default = "1.0")]
    pub boid_coeff: f32,

    // Separation parameters
    #[builder(default = "1.5")]
    separation_weight: f32,
    #[builder(default = "3.0")]
    separation_distance: f32,

    // Alignment parameters
    #[builder(default = "2.0")]
    alignment_weight: f32,
    #[builder(default = "15.0")]
    alignment_distance: f32,

    // Cohesion parameters
    #[builder(default = "1.5")]
    cohesion_weight: f32,
    #[builder(default = "15.0")]
    cohesion_distance: f32,

    // Additional parameters
    #[builder(default = "2.0")]
    drive_factor: f32, // Helps maintain momentum
    #[builder(default = "1.2")]
    swirl_factor: f32, // Adds some rotation to movement
    #[builder(default = "1.8")]
    border_factor: f32, // How strongly to avoid borders

    #[builder(default = "1.8")]
    max_speed: f32,
    #[builder(default = "0.2")]
    min_speed: f32,

    #[builder(default)]
    charset: BoidCharset,
}

pub struct Boids {
    options: BoidsOptions,
    buffer: Buffer,
    boids: Vec<Boid>,
    charset_chars: [char; 8],
}

impl Boid {
    fn new(position: (f32, f32), velocity: (f32, f32)) -> Self {
        Self {
            position,
            velocity,
            character: '•',
            color: style::Color::White,
        }
    }

    fn get_direction_char(&self, charset: &[char; 8]) -> char {
        let (vx, vy) = self.velocity;
        let angle = f32::atan2(vy, vx);

        let idx = ((angle / PI * 4.0).round() as i32 + 8) % 8;
        charset[idx as usize]
    }

    fn update_visual(&mut self, charset: &[char; 8]) {
        self.character = self.get_direction_char(charset);

        // Speed-based color (green to white)
        let speed = (self.velocity.0.powi(2) + self.velocity.1.powi(2)).sqrt();
        let intensity = ((speed * 128.0).clamp(0.0, 255.0)) as u8;
        self.color = style::Color::Rgb {
            r: intensity,
            g: 200,
            b: intensity.saturating_add(20),
        };
    }
}

impl TerminalEffect for Boids {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        let mut curr_buffer = Buffer::new(
            self.options.screen_size.0 as usize,
            self.options.screen_size.1 as usize,
        );

        // Fill current buffer with boids
        for boid in &self.boids {
            let x = boid.position.0.round() as usize
                % self.options.screen_size.0 as usize;
            let y = boid.position.1.round() as usize
                % self.options.screen_size.1 as usize;

            curr_buffer.set(
                x,
                y,
                Cell::new(boid.character, boid.color, style::Attribute::Bold),
            );
        }

        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    fn update(&mut self) {
        // Apply the three boid rules
        self.apply_rules();

        // Update positions and appearance
        self.update_positions();
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.options.screen_size = (width, height);
    }

    fn reset(&mut self) {
        *self = Self::new(self.options.clone());
    }
}

impl Boids {
    pub fn new(options: BoidsOptions) -> Self {
        let mut rng = rand::rng();
        let buffer = Buffer::new(
            options.screen_size.0 as usize,
            options.screen_size.1 as usize,
        );

        let width = options.screen_size.0 as f32;
        let height = options.screen_size.1 as f32;

        // Create initial boids with random positions and velocities
        let charset_chars = options.charset.chars();

        let mut boids = Vec::with_capacity(options.boid_count as usize);
        for _ in 0..options.boid_count {
            let position =
                (rng.random_range(0.0..width), rng.random_range(0.0..height));

            let velocity =
                (rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0));

            let mut boid = Boid::new(position, velocity);
            boid.update_visual(&charset_chars);
            boids.push(boid);
        }

        Self {
            options,
            buffer,
            boids,
            charset_chars,
        }
    }

    // Calculate toroidal difference between two positions
    fn toroidal_diff(&self, a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
        let width = self.options.screen_size.0 as f32;
        let height = self.options.screen_size.1 as f32;

        let mut dx = a.0 - b.0;
        let mut dy = a.1 - b.1;

        if dx > width / 2.0 {
            dx -= width;
        } else if dx < -width / 2.0 {
            dx += width;
        }

        if dy > height / 2.0 {
            dy -= height;
        } else if dy < -height / 2.0 {
            dy += height;
        }

        (dx, dy)
    }

    fn apply_rules(&mut self) {
        let num_boids = self.boids.len();
        let mut separation_adjustments = vec![(0.0, 0.0); num_boids];
        let mut alignment_adjustments = vec![(0.0, 0.0); num_boids];
        let mut cohesion_adjustments = vec![(0.0, 0.0); num_boids];
        let mut border_adjustments = vec![(0.0, 0.0); num_boids];

        // Pre-calculate all adjustments
        for i in 0..num_boids {
            // Apply separation rule
            let mut separation = (0.0, 0.0);
            let mut sep_count = 0;

            // Apply alignment rule
            let mut avg_velocity = (0.0, 0.0);
            let mut align_count = 0;

            // Apply cohesion rule
            let mut center = (0.0, 0.0);
            let mut cohesion_count = 0;

            for j in 0..num_boids {
                if i == j {
                    continue;
                }

                let diff = self
                    .toroidal_diff(self.boids[j].position, self.boids[i].position);
                let distance = (diff.0.powi(2) + diff.1.powi(2)).sqrt();

                // Separation
                #[allow(clippy::collapsible_if)]
                if distance < self.options.separation_distance {
                    if distance > 0.0 {
                        let factor = 1.0 / distance;
                        separation.0 -= diff.0 * factor;
                        separation.1 -= diff.1 * factor;
                        sep_count += 1;
                    }
                }

                // Alignment
                if distance < self.options.alignment_distance {
                    avg_velocity.0 += self.boids[j].velocity.0;
                    avg_velocity.1 += self.boids[j].velocity.1;
                    align_count += 1;
                }

                // Cohesion
                if distance < self.options.cohesion_distance {
                    center.0 += self.boids[j].position.0;
                    center.1 += self.boids[j].position.1;
                    cohesion_count += 1;
                }
            }

            // Finalize separation
            if sep_count > 0 {
                separation_adjustments[i] = (
                    separation.0 * self.options.separation_weight,
                    separation.1 * self.options.separation_weight,
                );
            }

            // Finalize alignment
            if align_count > 0 {
                let avg_vel = (
                    avg_velocity.0 / align_count as f32,
                    avg_velocity.1 / align_count as f32,
                );

                alignment_adjustments[i] = (
                    (avg_vel.0 - self.boids[i].velocity.0)
                        * self.options.alignment_weight
                        * 0.05,
                    (avg_vel.1 - self.boids[i].velocity.1)
                        * self.options.alignment_weight
                        * 0.05,
                );
            }

            // Finalize cohesion
            if cohesion_count > 0 {
                let perceived_center = (
                    center.0 / cohesion_count as f32,
                    center.1 / cohesion_count as f32,
                );

                let toward_center =
                    self.toroidal_diff(perceived_center, self.boids[i].position);

                // Calculate perpendicular (swirl) vector
                let swirl = (-toward_center.1, toward_center.0);
                let swirl_len = (swirl.0.powi(2) + swirl.1.powi(2)).sqrt();
                let swirl_normalized = if swirl_len > 0.0 {
                    (swirl.0 / swirl_len, swirl.1 / swirl_len)
                } else {
                    (0.0, 0.0)
                };

                cohesion_adjustments[i] = (
                    toward_center.0 * self.options.cohesion_weight * 0.03
                        + swirl_normalized.0 * self.options.swirl_factor * 0.02,
                    toward_center.1 * self.options.cohesion_weight * 0.03
                        + swirl_normalized.1 * self.options.swirl_factor * 0.02,
                );
            }

            // Apply border avoidance
            let width = self.options.screen_size.0 as f32;
            let height = self.options.screen_size.1 as f32;
            let border_margin = 5.0;
            let border_strength = self.options.border_factor;

            let mut border_force = (0.0, 0.0);
            let pos = self.boids[i].position;

            // Left edge
            if pos.0 < border_margin {
                border_force.0 += border_strength * (1.0 - pos.0 / border_margin);
            }
            // Right edge
            else if pos.0 > width - border_margin {
                border_force.0 -=
                    border_strength * (1.0 - (width - pos.0) / border_margin);
            }

            // Top edge
            if pos.1 < border_margin {
                border_force.1 += border_strength * (1.0 - pos.1 / border_margin);
            }
            // Bottom edge
            else if pos.1 > height - border_margin {
                border_force.1 -=
                    border_strength * (1.0 - (height - pos.1) / border_margin);
            }

            border_adjustments[i] = border_force;
        }

        // Apply all forces to boids
        for i in 0..num_boids {
            // Get current velocity
            let mut new_vx = self.boids[i].velocity.0;
            let mut new_vy = self.boids[i].velocity.1;

            // Apply rules
            new_vx += separation_adjustments[i].0;
            new_vy += separation_adjustments[i].1;

            new_vx += alignment_adjustments[i].0;
            new_vy += alignment_adjustments[i].1;

            new_vx += cohesion_adjustments[i].0;
            new_vy += cohesion_adjustments[i].1;

            new_vx += border_adjustments[i].0;
            new_vy += border_adjustments[i].1;

            // Apply drive factor
            let speed = (new_vx * new_vx + new_vy * new_vy).sqrt();
            if speed > 0.0 {
                let normalized_vx = new_vx / speed;
                let normalized_vy = new_vy / speed;
                new_vx += normalized_vx * self.options.drive_factor * 0.1;
                new_vy += normalized_vy * self.options.drive_factor * 0.1;
            }

            // Apply damping for smoother movement
            new_vx = self.boids[i].velocity.0 * 0.7 + new_vx * 0.3;
            new_vy = self.boids[i].velocity.1 * 0.7 + new_vy * 0.3;

            // Apply speed limits
            let speed = (new_vx * new_vx + new_vy * new_vy).sqrt();
            if speed > self.options.max_speed {
                let scale = self.options.max_speed / speed;
                new_vx *= scale;
                new_vy *= scale;
            } else if speed < self.options.min_speed && speed > 0.0 {
                let scale = self.options.min_speed / speed;
                new_vx *= scale;
                new_vy *= scale;
            }

            // Update velocity
            self.boids[i].velocity = (new_vx, new_vy);
        }
    }

    fn update_positions(&mut self) {
        let width = self.options.screen_size.0 as f32;
        let height = self.options.screen_size.1 as f32;

        for boid in &mut self.boids {
            // Update position
            boid.position.0 += boid.velocity.0;
            boid.position.1 += boid.velocity.1;

            // Wrap around screen boundaries
            if boid.position.0 < 0.0 {
                boid.position.0 += width;
            } else if boid.position.0 >= width {
                boid.position.0 -= width;
            }

            if boid.position.1 < 0.0 {
                boid.position.1 += height;
            } else if boid.position.1 >= height {
                boid.position.1 -= height;
            }

            // Update visual representation
            boid.update_visual(&self.charset_chars);
        }
    }
}

impl DefaultOptions for Boids {
    type Options = BoidsOptions;

    fn default_options(width: u16, height: u16) -> Self::Options {
        let boid_count = ((width * height) as f32 * 0.5) as u16; // About 1% of screen space

        BoidsOptionsBuilder::default()
            .screen_size((width, height))
            .boid_count(boid_count.clamp(50, 300))
            .build()
            .unwrap()
    }
}
