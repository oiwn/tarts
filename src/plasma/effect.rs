use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};
use crossterm::style;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

// Characters to represent different plasma densities (from least to most dense)
static PLASMA_CHARS: [char; 10] =
    [' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct PlasmaOptions {
    #[builder(default = "0.1")]
    pub time_scale: f64,
    #[builder(default = "1.0")]
    pub spatial_scale: f64,
    #[builder(default = "2.0")]
    pub color_cycle_speed: f64,
    #[builder(default = "true")]
    pub use_color: bool,
}

pub struct Plasma {
    pub screen_size: (u16, u16),
    options: PlasmaOptions,
    buffer: Buffer,
    time: f64,
    plasma_lut: Vec<f64>,
}

impl TerminalEffect for Plasma {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        // Clone the previous buffer to work with
        let mut curr_buffer = self.buffer.clone();

        // Update the plasma field
        self.update_plasma(&mut curr_buffer);

        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;

        diff
    }

    fn update(&mut self) {
        // Advance the time for the animation
        self.time += self.options.time_scale;
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
        self.reset();
    }

    fn reset(&mut self) {
        self.buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);
        self.time = 0.0;
        self.plasma_lut = self.create_plasma_lut();
    }
}

impl Plasma {
    pub fn new(options: PlasmaOptions, screen_size: (u16, u16)) -> Self {
        let buffer = Buffer::new(screen_size.0 as usize, screen_size.1 as usize);
        let time = 0.0;

        let mut plasma = Self {
            screen_size,
            options,
            buffer,
            time,
            plasma_lut: Vec::new(),
        };

        // Initialize the plasma lookup table
        plasma.plasma_lut = plasma.create_plasma_lut();

        plasma
    }

    /// Calculate plasma value for a specific point
    fn plasma_pixel(&self, x: f64, y: f64, time: f64) -> f64 {
        let scale = self.options.spatial_scale;

        // Classic plasma formula with time component
        ((x * scale / 16.0).sin()
            + (y * scale / 8.0).sin()
            + ((x + y) * scale / 16.0).sin()
            + ((x * x + y * y).sqrt() * scale / 8.0).sin()
            + time.sin()
            + 4.0)
            / 8.0
    }

    /// Create the plasma lookup table based on current dimensions
    fn create_plasma_lut(&self) -> Vec<f64> {
        let width = self.screen_size.0 as usize;
        let height = self.screen_size.1 as usize;

        let mut plasma = vec![0.0; width * height];

        for y in 0..height {
            for x in 0..width {
                plasma[(y * width) + x] = self.plasma_pixel(
                    x as f64, y as f64, 0.0, // Initial time is 0
                );
            }
        }

        plasma
    }

    /// Convert from HSV float to RGB u8 tuple
    fn hsv_to_rgb(&self, hue: f64, saturation: f64, value: f64) -> (u8, u8, u8) {
        let c = value * saturation;
        let h = hue * 6.0;
        let x = c * (1.0 - (h % 2.0 - 1.0).abs());
        let m = value - c;

        let (red, green, blue) = match (h % 6.0).floor() as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        // Convert back to RGB (where components are integers from 0 to 255)
        (
            ((red + m) * 255.0).round() as u8,
            ((green + m) * 255.0).round() as u8,
            ((blue + m) * 255.0).round() as u8,
        )
    }

    /// Get the character to represent a plasma value
    fn get_plasma_char(&self, value: f64) -> char {
        // Scale the plasma value (0.0 to 1.0) to the index range
        let index = (value * (PLASMA_CHARS.len() - 1) as f64).round() as usize;
        PLASMA_CHARS[index.min(PLASMA_CHARS.len() - 1)]
    }

    /// Convert RGB to a crossterm style color
    fn rgb_to_style_color(&self, rgb: (u8, u8, u8)) -> style::Color {
        style::Color::Rgb {
            r: rgb.0,
            g: rgb.1,
            b: rgb.2,
        }
    }

    /// Update the plasma field in the buffer
    fn update_plasma(&mut self, buffer: &mut Buffer) {
        let width = self.screen_size.0 as usize;
        let height = self.screen_size.1 as usize;

        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;

                // Get the base plasma value and add time component
                let plasma_value =
                    (self.plasma_lut[index] + (self.time * 0.1).sin()) % 1.0;

                // Get character based on plasma value
                let ch = self.get_plasma_char(plasma_value);

                // Calculate color based on time and position
                let color_hue = (plasma_value
                    + self.time * self.options.color_cycle_speed / (2.0 * PI))
                    % 1.0;

                let attr = style::Attribute::Bold;

                // Choose color based on options
                let color = if self.options.use_color {
                    let rgb = self.hsv_to_rgb(color_hue, 1.0, 1.0);
                    self.rgb_to_style_color(rgb)
                } else {
                    style::Color::White
                };

                // Set the cell in the buffer
                buffer.set(x, y, Cell::new(ch, color, attr));
            }
        }
    }
}

impl DefaultOptions for Plasma {
    type Options = PlasmaOptions;

    fn default_options(_width: u16, _height: u16) -> Self::Options {
        PlasmaOptionsBuilder::default()
            .time_scale(0.1)
            .spatial_scale(1.0)
            .color_cycle_speed(1.0)
            .use_color(true)
            .build()
            .unwrap()
    }
}
