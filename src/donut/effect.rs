use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};
use crossterm::style;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct DonutOptions {
    #[builder(default = "1.0")]
    pub inner_radius: f32,
    #[builder(default = "2.0")]
    pub outer_radius: f32,
    #[builder(default = "0.07")]
    pub rotation_speed_a: f32,
    #[builder(default = "0.03")]
    pub rotation_speed_b: f32,
    #[builder(default = "5.0")]
    pub distance: f32,
    #[builder(default = "25.0")]
    pub k1: f32,
    #[builder(
        default = "vec!['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@']"
    )]
    pub luminance_chars: Vec<char>,
}

pub struct Donut {
    pub screen_size: (u16, u16),
    options: DonutOptions,
    buffer: Buffer,
    rotation_a: f32,
    rotation_b: f32,
}

impl TerminalEffect for Donut {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        let mut curr_buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);

        self.render_donut(&mut curr_buffer);

        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    fn update(&mut self) {
        self.rotation_a += self.options.rotation_speed_a;
        self.rotation_b += self.options.rotation_speed_b;
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
    }

    fn reset(&mut self) {
        *self = Self::new(self.options.clone(), self.screen_size);
    }
}

impl Donut {
    pub fn new(options: DonutOptions, screen_size: (u16, u16)) -> Self {
        let buffer = Buffer::new(screen_size.0 as usize, screen_size.1 as usize);
        Self {
            screen_size,
            options,
            buffer,
            rotation_a: 0.0,
            rotation_b: 0.0,
        }
    }

    fn render_donut(&self, buffer: &mut Buffer) {
        buffer.fill_with(&Cell::default());

        let width = self.screen_size.0 as usize;
        let height = self.screen_size.1 as usize;

        // Precompute sines and cosines
        let sin_a = self.rotation_a.sin();
        let cos_a = self.rotation_a.cos();
        let sin_b = self.rotation_b.sin();
        let cos_b = self.rotation_b.cos();

        // Create a zbuffer and output buffer
        let mut zbuffer = vec![0.0; width * height];
        let mut output = vec![' '; width * height];

        // gruvbox gradient
        let colors = [
            style::Color::Rgb {
                r: 213,
                g: 196,
                b: 161,
            },
            style::Color::Rgb {
                r: 213,
                g: 196,
                b: 161,
            },
            style::Color::Rgb {
                r: 213,
                g: 196,
                b: 161,
            },
            style::Color::Rgb {
                r: 213,
                g: 196,
                b: 161,
            },
            style::Color::Rgb {
                r: 251,
                g: 241,
                b: 199,
            },
            style::Color::Rgb {
                r: 251,
                g: 241,
                b: 199,
            },
            style::Color::Rgb {
                r: 69,
                g: 133,
                b: 136,
            },
            style::Color::Rgb {
                r: 104,
                g: 157,
                b: 106,
            },
            style::Color::Rgb {
                r: 152,
                g: 151,
                b: 26,
            },
            style::Color::Rgb {
                r: 215,
                g: 153,
                b: 33,
            },
            style::Color::Rgb {
                r: 214,
                g: 93,
                b: 14,
            },
            style::Color::Rgb {
                r: 204,
                g: 36,
                b: 29,
            },
        ];

        // Theta goes around the cross-sectional circle of a torus
        for theta in 0..314 {
            let theta = theta as f32 * 0.02;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            // Phi goes around the center of revolution of a torus
            for phi in 0..628 {
                let phi = phi as f32 * 0.01;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                // Compute the x,y coordinate of the circle before revolving
                let circle_x = self.options.outer_radius
                    + self.options.inner_radius * cos_theta;
                let circle_y = self.options.inner_radius * sin_theta;

                // Final 3D (x,y,z) coordinate after rotations
                let x = circle_x * (cos_b * cos_phi + sin_a * sin_b * sin_phi)
                    - circle_y * cos_a * sin_b;
                let y = circle_x * (sin_b * cos_phi - sin_a * cos_b * sin_phi)
                    + circle_y * cos_a * cos_b;
                let z = self.options.distance
                    + cos_a * circle_x * sin_phi
                    + circle_y * sin_a;
                let z_inv = 1.0 / z;

                // Project into 2D
                let x_proj =
                    (width as f32 / 2.0 + self.options.k1 * z_inv * x) as usize;
                let y_proj = (height as f32 / 2.0
                    + self.options.k1 * z_inv * y * 0.8)
                    as usize;

                // Calculate luminance
                let l = cos_phi * cos_theta * sin_b
                    - cos_a * cos_theta * sin_phi
                    - sin_a * sin_theta
                    + cos_b * (cos_a * sin_theta - cos_theta * sin_a * sin_phi);

                if l > 0.0 {
                    let luminance_index = ((l * 8.0) as usize)
                        .min(self.options.luminance_chars.len() - 1);
                    let c = self.options.luminance_chars[luminance_index];

                    // Check bounds
                    if x_proj < width && y_proj < height {
                        let idx = y_proj * width + x_proj;
                        if z_inv > zbuffer[idx] {
                            zbuffer[idx] = z_inv;
                            output[idx] = c;
                        }
                    }
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                if output[idx] != ' ' {
                    let luminance_index = self
                        .options
                        .luminance_chars
                        .iter()
                        .position(|&r| r == output[idx])
                        .unwrap_or(0);
                    let color = colors[luminance_index % colors.len()];
                    buffer.set(
                        x,
                        y,
                        Cell::new(output[idx], color, style::Attribute::Bold),
                    );
                }
            }
        }
    }
}

impl DefaultOptions for Donut {
    type Options = DonutOptions;

    fn default_options(width: u16, height: u16) -> Self::Options {
        DonutOptionsBuilder::default()
            .inner_radius(1.0)
            .outer_radius(2.0)
            .rotation_speed_a(0.07)
            .rotation_speed_b(0.03)
            .distance(5.5)
            .k1((width.min(height) as f32) * 0.8)
            .luminance_chars(vec![
                '.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@',
            ])
            .build()
            .unwrap()
    }
}
