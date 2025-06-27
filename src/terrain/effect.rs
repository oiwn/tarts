use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};
use crate::terrain::noise::PerlinNoise;
use crossterm::style;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct TerrainOptions {
    #[builder(default = "42")]
    pub seed: u32,
    #[builder(default = "0.05")]
    pub scale: f64,
    #[builder(default = "4")]
    pub octaves: i32,
    #[builder(default = "0.5")]
    pub persistence: f64,
    #[builder(default = "0.01")]
    pub animation_speed: f64,
}

pub struct Terrain {
    pub screen_size: (u16, u16),
    options: TerrainOptions,
    buffer: Buffer,
    noise: PerlinNoise,
    time_offset: f64,
}

impl TerminalEffect for Terrain {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        let mut curr_buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);

        self.generate_terrain(&mut curr_buffer);

        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    fn update(&mut self) {
        self.time_offset += self.options.animation_speed;
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
        self.reset();
    }

    fn reset(&mut self) {
        self.buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);
        self.time_offset = 0.0;
    }
}

impl Terrain {
    pub fn new(options: TerrainOptions, screen_size: (u16, u16)) -> Self {
        let buffer = Buffer::new(screen_size.0 as usize, screen_size.1 as usize);
        let noise = PerlinNoise::new(options.seed);

        Self {
            screen_size,
            options,
            buffer,
            noise,
            time_offset: 0.0,
        }
    }

    fn generate_terrain(&self, buffer: &mut Buffer) {
        let width = self.screen_size.0 as usize;
        let height = self.screen_size.1 as usize;

        for y in 0..height {
            for x in 0..width {
                // Generate noise value with time animation
                let noise_value = self.noise.octave_noise_2d(
                    x as f64,
                    y as f64 + self.time_offset * 50.0, // Animate vertically
                    self.options.octaves,
                    self.options.persistence,
                    self.options.scale,
                );

                // Normalize to 0-1 range
                let normalized = (noise_value + 1.0) / 2.0;

                let (character, color) =
                    self.get_terrain_char_and_color(normalized);

                buffer.set(
                    x,
                    y,
                    Cell::new(character, color, style::Attribute::Bold),
                );
            }
        }
    }

    fn get_terrain_char_and_color(&self, height: f64) -> (char, style::Color) {
        match height {
            h if h < 0.2 => (
                '~',
                style::Color::Rgb {
                    r: 0,
                    g: 100,
                    b: 200,
                },
            ), // Deep water
            h if h < 0.35 => (
                '≈',
                style::Color::Rgb {
                    r: 50,
                    g: 150,
                    b: 255,
                },
            ), // Shallow water
            h if h < 0.4 => (
                '▒',
                style::Color::Rgb {
                    r: 194,
                    g: 178,
                    b: 128,
                },
            ), // Beach/sand
            h if h < 0.6 => (
                '▓',
                style::Color::Rgb {
                    r: 34,
                    g: 139,
                    b: 34,
                },
            ), // Grassland
            h if h < 0.75 => ('♦', style::Color::Rgb { r: 0, g: 100, b: 0 }), // Forest
            h if h < 0.85 => (
                '▲',
                style::Color::Rgb {
                    r: 139,
                    g: 69,
                    b: 19,
                },
            ), // Hills
            h if h < 0.95 => (
                '⛰',
                style::Color::Rgb {
                    r: 105,
                    g: 105,
                    b: 105,
                },
            ), // Mountains
            _ => (
                '❄',
                style::Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            ), // Snow peaks
        }
    }
}

impl DefaultOptions for Terrain {
    type Options = TerrainOptions;

    fn default_options(_width: u16, _height: u16) -> Self::Options {
        TerrainOptionsBuilder::default()
            .seed(42u32)
            .scale(0.02)
            .octaves(4)
            .persistence(0.5)
            .animation_speed(0.01)
            .build()
            .unwrap()
    }
}
