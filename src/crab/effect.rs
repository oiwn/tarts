use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};
use crossterm::style;
use derive_builder::Builder;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

// Direction the crab is facing
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
}

// Selected animation frames for the crab
// Each frame has (or no?) the same width/height for consistent rendering
static CRAB_FRAMES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![
        // Frame 0: Standard pose facing right
        r#"    _~^~^~_
\) /  o o  \ (/
  '_   ¬   _'
  \ '-----' /"#,
        // Frame 1: Walking pose facing right
        r#"    _~^~^~_
\) /  o o  \ (/
 '-,   -  _'\
  | '----' "#,
        // Frame 2: Special pose (claws open) facing right
        r#"    _~^~^~_
\/ /  o o  \ \/
  '_   u   _'
  \ '-----' /"#,
        // Frame 3: Standard pose facing left (mirrored)
        r#"    _~^~^~_
(\ /  o o  \ ()
  '_   ¬   _'
  / '-----' \"#,
        // Frame 4: Walking pose facing left (mirrored)
        r#"    _~^~^~_
(\ /  o o  \ ()
 /'_  -   ,-'
    '----' |"#,
        // Frame 5: Special pose (claws open) facing left
        r#"    _~^~^~_
\/ /  o o  \ \/
  '_   u   _'
  / '-----' \"#,
    ]
});

// Individual crab entity
#[derive(Clone)]
struct CrabEntity {
    position: (f32, f32), // Floating point for smooth movement
    velocity: (f32, f32), // Direction and speed
    direction: Direction, // Facing left or right
    current_frame: usize, // Current animation frame
    animation_timer: f32, // Timer for animation
    special_timer: f32,   // Timer for special animations
    is_special: bool,     // Whether doing special animation
    color: style::Color,  // Crab color
    frame_width: usize,   // Cached frame width
    frame_height: usize,  // Cached frame height
}

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct CrabOptions {
    #[builder(default = "5")]
    #[serde(skip)]
    pub crab_count: u16,

    #[builder(default = "0.2")]
    pub animation_speed: f32,

    #[builder(default = "0.05")]
    pub clap_chance: f32, // Random chance for special animation

    #[builder(default = "5.0")]
    pub movement_speed: f32,

    #[builder(default = "1.0")]
    pub crab_coeff: f32,
}

pub struct Crab {
    pub screen_size: (u16, u16),
    options: CrabOptions,
    buffer: Buffer,
    crabs: Vec<CrabEntity>,
    rng: rand::prelude::ThreadRng,
    frame_timer: f32,
}

impl CrabEntity {
    fn new(
        position: (f32, f32),
        velocity: (f32, f32),
        rng: &mut rand::prelude::ThreadRng,
    ) -> Self {
        // Determine initial direction based on velocity
        let direction = if velocity.0 >= 0.0 {
            Direction::Right
        } else {
            Direction::Left
        };

        // Random color with predominantly red tint for crabs
        let color = style::Color::Rgb {
            r: rng.random_range(200..=255),
            g: rng.random_range(50..=150),
            b: rng.random_range(50..=100),
        };

        // Calculate frame dimensions from the first frame
        let frame_lines: Vec<&str> = CRAB_FRAMES[0].lines().collect();
        let frame_height = frame_lines.len();
        let frame_width =
            frame_lines.iter().map(|line| line.len()).max().unwrap_or(0);

        Self {
            position,
            velocity,
            direction,
            current_frame: 0,
            animation_timer: 0.0,
            special_timer: 0.0,
            is_special: false,
            color,
            frame_width,
            frame_height,
        }
    }

    // Get the appropriate frame for the crab's current state
    fn get_frame_index(&self) -> usize {
        if self.is_special {
            // Special animation (open claws)
            if self.direction == Direction::Right {
                2
            } else {
                5
            }
        } else if self.direction == Direction::Right {
            // Walking animation right
            self.current_frame % 2 // Alternate between frames 0 and 1
        } else {
            // Walking animation left
            3 + (self.current_frame % 2) // Alternate between frames 3 and 4
        }
    }

    // Get the lines of the current frame for rendering
    fn get_frame_lines(&self) -> Vec<String> {
        let frame_index = self.get_frame_index();
        CRAB_FRAMES[frame_index]
            .lines()
            .map(|line| line.to_string())
            .collect()
    }

    // Update the crab's position and animation state
    fn update(
        &mut self,
        dt: f32,
        screen_size: (u16, u16),
        animation_speed: f32,
        movement_speed: f32,
        clap_chance: f32,
        rng: &mut rand::prelude::ThreadRng,
    ) {
        // Update position based on velocity
        self.position.0 += self.velocity.0 * movement_speed * dt;
        self.position.1 += self.velocity.1 * movement_speed * dt;

        // Screen boundary collision detection
        let width = screen_size.0 as f32;
        let height = screen_size.1 as f32;

        // Detect collision with screen edges and reverse direction
        if self.position.0 < 0.0 {
            self.position.0 = 0.0;
            self.velocity.0 = rng.random_range(0.5..1.5);
            self.direction = Direction::Right;
        } else if self.position.0 + self.frame_width as f32 > width {
            self.position.0 = width - self.frame_width as f32;
            self.velocity.0 = rng.random_range(-1.5..-0.5);
            self.direction = Direction::Left;
        }

        if self.position.1 < 0.0 {
            self.position.1 = 0.0;
            self.velocity.1 = rng.random_range(0.2..0.8);
        } else if self.position.1 + self.frame_height as f32 > height {
            self.position.1 = height - self.frame_height as f32;
            self.velocity.1 = rng.random_range(-0.8..-0.2);
        }

        // Update direction based on velocity
        if self.velocity.0 > 0.0 {
            self.direction = Direction::Right;
        } else if self.velocity.0 < 0.0 {
            self.direction = Direction::Left;
        }

        // Update animation timer
        let is_moving = self.velocity.0.abs() > 0.1 || self.velocity.1.abs() > 0.1;
        if is_moving {
            self.animation_timer += dt;
            if self.animation_timer >= animation_speed {
                self.animation_timer = 0.0;
                self.current_frame = (self.current_frame + 1) % 2;
            }
        } else {
            // Use standing frame when not moving
            self.animation_timer = 0.0;
            self.current_frame = 0;
        }

        // Update special animation
        if self.is_special {
            self.special_timer -= dt;
            if self.special_timer <= 0.0 {
                self.is_special = false;
            }
        } else if rng.random::<f32>() < clap_chance * dt {
            // Random chance to trigger special animation
            self.is_special = true;
            self.special_timer = animation_speed * 5.0; // Duration of special animation
        }

        // Occasionally add slight randomness to movement
        if rng.random::<f32>() < 0.02 {
            self.velocity.0 += rng.random_range(-0.2..0.2);
            self.velocity.1 += rng.random_range(-0.1..0.1);

            // Keep velocity in reasonable bounds
            if self.velocity.0.abs() > 2.0 {
                self.velocity.0 = self.velocity.0.signum() * 1.5;
            }

            if self.velocity.1.abs() > 1.0 {
                self.velocity.1 = self.velocity.1.signum() * 0.5;
            }
        }
    }
}

impl TerminalEffect for Crab {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        let mut curr_buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);

        // Draw each crab
        for crab in &self.crabs {
            let frame_lines = crab.get_frame_lines();
            let base_x = crab.position.0.round() as usize;
            let base_y = crab.position.1.round() as usize;

            // Draw each line of the crab frame
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

                    // Set the character in the buffer
                    curr_buffer.set(
                        x,
                        y,
                        Cell::new(ch, crab.color, style::Attribute::Bold),
                    );
                }
            }
        }

        // Calculate the diff
        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    fn update(&mut self) {
        // Use a fixed delta time for smooth animation
        let dt = 0.033; // ~30 FPS

        // Update frame timer
        self.frame_timer += dt;

        // Update each crab
        for crab in &mut self.crabs {
            crab.update(
                dt,
                self.screen_size,
                self.options.animation_speed,
                self.options.movement_speed,
                self.options.clap_chance,
                &mut self.rng,
            );
        }

        // Check for collisions between crabs
        self.check_crab_collisions();
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
    }

    fn reset(&mut self) {
        *self = Self::new(self.options.clone(), self.screen_size);
    }
}

impl Crab {
    pub fn new(options: CrabOptions, screen_size: (u16, u16)) -> Self {
        let mut rng = rand::rng();
        let buffer = Buffer::new(screen_size.0 as usize, screen_size.1 as usize);

        let width = screen_size.0 as f32;
        let height = screen_size.1 as f32;

        // Create initial crabs with random positions and velocities
        let mut crabs = Vec::with_capacity(options.crab_count as usize);
        for _ in 0..options.crab_count {
            let position = (
                rng.random_range(0.0..width * 0.8),
                rng.random_range(0.0..height * 0.8),
            );

            // Random velocity, but ensure it's not too slow
            let velocity = (
                rng.random_range(-1.0f32..1.0f32).signum()
                    * rng.random_range(0.5..1.5),
                rng.random_range(-0.5..0.5),
            );

            crabs.push(CrabEntity::new(position, velocity, &mut rng));
        }

        // ensure crabs won't start to close to each other
        let min_distance_squared = 100.0; // Adjust based on crab size
        let mut i = 0;
        while i < crabs.len() {
            let mut repositioned = false;

            for j in 0..i {
                let dx = crabs[i].position.0 - crabs[j].position.0;
                let dy = crabs[i].position.1 - crabs[j].position.1;
                let distance_squared = dx * dx + dy * dy;

                if distance_squared < min_distance_squared {
                    // Reposition this crab
                    crabs[i].position.0 = rng.random_range(0.0..width * 0.8);
                    crabs[i].position.1 = rng.random_range(0.0..height * 0.8);
                    repositioned = true;
                    break;
                }
            }

            if !repositioned {
                i += 1; // Only advance if no repositioning was needed
            }
        }

        Self {
            screen_size,
            options,
            buffer,
            crabs,
            rng,
            frame_timer: 0.0,
        }
    }

    // Check for collisions between crabs and handle them
    fn check_crab_collisions(&mut self) {
        let crab_count = self.crabs.len();
        if crab_count < 2 {
            return;
        }

        // Simple collision detection based on proximity
        for i in 0..crab_count {
            for j in (i + 1)..crab_count {
                let dx = self.crabs[i].position.0 - self.crabs[j].position.0;
                let dy = self.crabs[i].position.1 - self.crabs[j].position.1;
                let distance_squared = dx * dx + dy * dy;

                // If crabs are close enough, consider it a collision
                if distance_squared < 36.0 {
                    // Trigger special animation for both crabs
                    self.crabs[i].is_special = true;
                    self.crabs[i].special_timer =
                        self.options.animation_speed * 5.0;

                    self.crabs[j].is_special = true;
                    self.crabs[j].special_timer =
                        self.options.animation_speed * 5.0;

                    // Reverse directions
                    self.crabs[i].velocity.0 = -self.crabs[i].velocity.0;
                    self.crabs[j].velocity.0 = -self.crabs[j].velocity.0;

                    // Add some vertical movement to avoid getting stuck
                    self.crabs[i].velocity.1 += self.rng.random_range(-0.5..0.5);
                    self.crabs[j].velocity.1 += self.rng.random_range(-0.5..0.5);

                    // Update directions based on new velocities
                    self.crabs[i].direction = if self.crabs[i].velocity.0 >= 0.0 {
                        Direction::Right
                    } else {
                        Direction::Left
                    };

                    self.crabs[j].direction = if self.crabs[j].velocity.0 >= 0.0 {
                        Direction::Right
                    } else {
                        Direction::Left
                    };
                }
            }
        }
    }
}

impl DefaultOptions for Crab {
    type Options = CrabOptions;

    fn default_options(width: u16, height: u16) -> Self::Options {
        // Adjust crab count based on screen size
        let screen_area = width as f32 * height as f32;
        let crab_count = (screen_area / 800.0).clamp(3.0, 15.0) as u16;

        CrabOptionsBuilder::default()
            .crab_count(crab_count)
            .animation_speed(0.2)
            .movement_speed(5.0)
            .clap_chance(0.05)
            .build()
            .unwrap()
    }
}
