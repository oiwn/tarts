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
}

pub struct Terrain {
    pub screen_size: (u16, u16),
    options: TerrainOptions,
    buffer: Buffer,
    noise: PerlinNoise,
    generated: bool, // Only generate once
}

impl TerminalEffect for Terrain {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        if !self.generated {
            let mut curr_buffer = Buffer::new(
                self.screen_size.0 as usize,
                self.screen_size.1 as usize,
            );

            self.generate_noise(&mut curr_buffer);

            let diff = self.buffer.diff(&curr_buffer);
            self.buffer = curr_buffer;
            self.generated = true;
            diff
        } else {
            Vec::new() // No changes after initial generation
        }
    }

    fn update(&mut self) {
        // No updates needed for static noise
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
        self.reset();
    }

    fn reset(&mut self) {
        self.buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);
        self.generated = false;
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
            generated: false,
        }
    }

    fn generate_noise(&self, buffer: &mut Buffer) {
        let width = self.screen_size.0 as usize;
        let height = self.screen_size.1 as usize;

        for y in 0..height {
            for x in 0..width {
                // Generate noise value
                let noise_value = self.noise.octave_noise_2d(
                    x as f64,
                    y as f64,
                    self.options.octaves,
                    self.options.persistence,
                    self.options.scale,
                );

                // Normalize to 0-1 range
                let normalized = (noise_value + 1.0) / 2.0;

                let (character, color) = self.get_noise_visualization(normalized);

                buffer.set(
                    x,
                    y,
                    Cell::new(character, color, style::Attribute::Bold),
                );
            }
        }
    }

    fn get_noise_visualization(&self, value: f64) -> (char, style::Color) {
        // Simple grayscale visualization of noise
        let intensity = (value * 255.0) as u8;

        let character = match value {
            v if v < 0.1 => ' ',
            v if v < 0.2 => '.',
            v if v < 0.3 => ':',
            v if v < 0.4 => '-',
            v if v < 0.5 => '=',
            v if v < 0.6 => '+',
            v if v < 0.7 => '*',
            v if v < 0.8 => '#',
            v if v < 0.9 => '%',
            _ => '@',
        };

        (
            character,
            style::Color::Rgb {
                r: intensity,
                g: intensity,
                b: intensity,
            },
        )
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
            .build()
            .unwrap()
    }
}
