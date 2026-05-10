use super::draw::{pick_color, pick_style};
use super::gradient;
use super::rain_drop::RainDrop;
use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};

use derive_builder::Builder;
use rand::{self, RngExt};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Builder, Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DigitalRainOptions {
    #[builder(default = "(10, 20)")]
    #[serde(skip)]
    pub drops_range: (u16, u16),
    #[builder(default = "(2, 16)")]
    #[serde(skip)]
    pub speed_range: (u16, u16),
    #[builder(default = "1.0")]
    pub drops_coeff: f32,
    #[builder(default = "1.0")]
    pub speed_coeff: f32,
}

pub struct DigitalRain {
    pub screen_size: (u16, u16),
    options: DigitalRainOptions,
    gradients: Vec<Vec<gradient::Color>>,
    rain_drops: Vec<RainDrop>,
    buffer: Buffer,
    rng: rand::prelude::ThreadRng,
}

impl TerminalEffect for DigitalRain {
    /// Calculate difference between current frame and previous frame
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        let mut curr_buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);

        // fill current buffer
        // first draw drops with bigger fy
        Self::fill_buffer(&mut self.rain_drops, &mut curr_buffer, &self.gradients);

        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    /// Update each rain drop position
    fn update(&mut self) {
        for rain_drop in self.rain_drops.iter_mut() {
            rain_drop.update(
                self.screen_size,
                &self.options,
                Duration::from_millis(50),
                &mut self.rng,
            );
        }

        self.add_one();
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
    }

    fn reset(&mut self) {
        let new_effect = DigitalRain::new(self.options.clone(), self.screen_size);
        *self = new_effect;
    }
}

/// Process digital rain effect.
/// Note that all processing done implying coordinates started from 0, 0
/// and width / height is actual number of columns and rows
impl DigitalRain {
    // Initialize screensaver
    pub fn new(options: DigitalRainOptions, screen_size: (u16, u16)) -> Self {
        let mut rng = rand::rng();
        let mut rain_drops: Vec<RainDrop> = vec![];
        let mut buffer: Buffer =
            Buffer::new(screen_size.0 as usize, screen_size.1 as usize);
        for rain_drop_id in 1..=options.get_min_drops_number() {
            rain_drops.push(RainDrop::new(
                screen_size,
                &options,
                rain_drop_id as usize,
                &mut rng,
            ));
        }

        // fill gradients
        let gradients = vec![
            gradient::two_step_color_gradient(
                gradient::Color {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                gradient::Color { r: 0, g: 255, b: 0 },
                gradient::Color {
                    r: 10,
                    g: 10,
                    b: 10,
                },
                4,
                3 * screen_size.1 as usize / 2,
            ),
            gradient::two_step_color_gradient(
                gradient::Color {
                    r: 200,
                    g: 200,
                    b: 200,
                },
                gradient::Color { r: 0, g: 250, b: 0 },
                gradient::Color {
                    r: 10,
                    g: 10,
                    b: 10,
                },
                6,
                3 * screen_size.1 as usize / 2,
            ),
            gradient::two_step_color_gradient(
                gradient::Color {
                    r: 200,
                    g: 200,
                    b: 200,
                },
                gradient::Color { r: 0, g: 200, b: 0 },
                gradient::Color {
                    r: 10,
                    g: 10,
                    b: 10,
                },
                screen_size.1 as usize / 2,
                3 * screen_size.1 as usize / 2,
            ),
        ];

        Self::fill_buffer(&mut rain_drops, &mut buffer, &gradients);

        Self {
            screen_size,
            options,
            gradients,
            rain_drops,
            buffer,
            rng,
        }
    }

    pub fn fill_buffer(
        rain_drops: &mut [RainDrop],
        buffer: &mut Buffer,
        gradients: &[Vec<gradient::Color>],
    ) {
        rain_drops.sort_by(|a, b| a.speed.partial_cmp(&b.speed).unwrap());
        for rain_drop in rain_drops.iter().rev() {
            let points = rain_drop.to_points_vec();
            for (index, (x, y, character)) in points.iter().enumerate() {
                let (width, height) = buffer.get_size();
                if *x < width as u16 && *y < height as u16 {
                    buffer.set(
                        *x as usize,
                        *y as usize,
                        Cell::new(
                            *character,
                            pick_color(&rain_drop.style, index, gradients),
                            pick_style(&rain_drop.style, index),
                        ),
                    );
                };
            }
        }
    }

    /// Add one more worm with decent chance
    pub fn add_one(&mut self) {
        if self.rain_drops.len() >= self.options.get_max_drops_number() as usize {
            return;
        };
        let mut rng = rand::rng();
        if rng.random_range(0.0..=1.0) <= 0.3 {
            self.rain_drops.push(RainDrop::new(
                self.screen_size,
                &self.options,
                self.rain_drops.len() + 1,
                &mut rng,
            ));
        };
    }
}

impl DigitalRainOptions {
    #[inline]
    pub fn get_min_drops_number(&self) -> u16 {
        self.drops_range.0
    }

    #[inline]
    pub fn get_max_drops_number(&self) -> u16 {
        self.drops_range.1
    }

    #[inline]
    pub fn get_min_speed(&self) -> u16 {
        self.speed_range.0
    }

    #[inline]
    pub fn get_max_speed(&self) -> u16 {
        self.speed_range.1
    }
}

impl DefaultOptions for DigitalRain {
    type Options = DigitalRainOptions;

    fn default_options(width: u16, height: u16) -> Self::Options {
        let drops_range = {
            let min_drops = (width * height) / 160; // Approximately 0.6% of screen space
            let max_drops = (width * height) / 80; // Approximately 1.2% of screen space
            (min_drops.max(10), max_drops.max(20)) // Ensure minimum values
        };

        let speed_range = {
            let min_speed = (height / 20).max(2); // Faster for larger screens
            let max_speed = (height / 10).max(16); // But not too fast
            (min_speed, max_speed)
        };

        DigitalRainOptionsBuilder::default()
            .drops_range(drops_range)
            .speed_range(speed_range)
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_sane_default_options() -> DigitalRainOptions {
        DigitalRainOptionsBuilder::default()
            .drops_range((20, 30))
            .speed_range((10, 20))
            .build()
            .unwrap()
    }

    #[test]
    fn create_new() {
        let foo = DigitalRain::new(get_sane_default_options(), (100, 100));
        assert_eq!(foo.rain_drops.len(), 20);
    }

    #[test]
    fn no_diff() {
        let mut foo = DigitalRain::new(get_sane_default_options(), (100, 100));
        let q = foo.get_diff();
        assert!(q.is_empty());
    }

    #[test]
    fn same_diff_and_update() {
        let mut foo = DigitalRain::new(get_sane_default_options(), (100, 100));
        foo.update();
        let q = foo.get_diff();
        assert!(!q.is_empty());
    }
}
