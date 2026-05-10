use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};
use crossterm::style;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Represents a 3D point in space
#[derive(Clone, Copy, Debug)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
}

/// Represents a 2D point for screen coordinates
#[derive(Clone, Copy, Debug)]
struct Point2D {
    x: f32,
    y: f32,
}

/// Represents a 3D edge connecting two vertices
struct Edge {
    v1: usize,
    v2: usize,
}

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct CubeOptions {
    #[builder(default = "1.0")]
    pub cube_size: f32,
    #[builder(default = "0.6")]
    pub rotation_speed_x: f32,
    #[builder(default = "0.8")]
    pub rotation_speed_y: f32,
    #[builder(default = "0.4")]
    pub rotation_speed_z: f32,
    #[builder(default = "3.5")]
    pub distance: f32,
    #[builder(default = "true")]
    pub use_braille: bool,
}

pub struct Cube {
    pub screen_size: (u16, u16),
    options: CubeOptions,
    buffer: Buffer,
    vertices: Vec<Point3D>,
    edges: Vec<Edge>,
    rotation: (f32, f32, f32),
    start_time: Instant,
}

impl TerminalEffect for Cube {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        let mut curr_buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);

        // Rotate vertices
        let rotated_vertices = self.rotate_vertices();

        // Project 3D points to 2D
        let projected_vertices: Vec<Point2D> =
            rotated_vertices.iter().map(|v| self.project(*v)).collect();

        // Draw the cube
        if self.options.use_braille {
            self.draw_braille(&projected_vertices, &mut curr_buffer);
        } else {
            self.draw_ascii(&projected_vertices, &mut curr_buffer);
        }

        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    fn update(&mut self) {
        // Update rotation based on elapsed time
        let elapsed = self.start_time.elapsed().as_secs_f32();
        self.rotation.0 = elapsed * self.options.rotation_speed_x;
        self.rotation.1 = elapsed * self.options.rotation_speed_y;
        self.rotation.2 = elapsed * self.options.rotation_speed_z;
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
    }

    fn reset(&mut self) {
        *self = Self::new(self.options.clone(), self.screen_size);
    }
}

impl Cube {
    pub fn new(options: CubeOptions, screen_size: (u16, u16)) -> Self {
        let buffer = Buffer::new(screen_size.0 as usize, screen_size.1 as usize);

        // Define cube vertices
        let size = options.cube_size;
        let vertices = vec![
            Point3D {
                x: -size,
                y: -size,
                z: -size,
            }, // 0: front bottom left
            Point3D {
                x: size,
                y: -size,
                z: -size,
            }, // 1: front bottom right
            Point3D {
                x: size,
                y: size,
                z: -size,
            }, // 2: front top right
            Point3D {
                x: -size,
                y: size,
                z: -size,
            }, // 3: front top left
            Point3D {
                x: -size,
                y: -size,
                z: size,
            }, // 4: back bottom left
            Point3D {
                x: size,
                y: -size,
                z: size,
            }, // 5: back bottom right
            Point3D {
                x: size,
                y: size,
                z: size,
            }, // 6: back top right
            Point3D {
                x: -size,
                y: size,
                z: size,
            }, // 7: back top left
        ];

        // Define cube edges
        let edges = vec![
            // Front face
            Edge { v1: 0, v2: 1 },
            Edge { v1: 1, v2: 2 },
            Edge { v1: 2, v2: 3 },
            Edge { v1: 3, v2: 0 },
            // Back face
            Edge { v1: 4, v2: 5 },
            Edge { v1: 5, v2: 6 },
            Edge { v1: 6, v2: 7 },
            Edge { v1: 7, v2: 4 },
            // Connecting edges
            Edge { v1: 0, v2: 4 },
            Edge { v1: 1, v2: 5 },
            Edge { v1: 2, v2: 6 },
            Edge { v1: 3, v2: 7 },
        ];

        Self {
            screen_size,
            options,
            buffer,
            vertices,
            edges,
            rotation: (0.0, 0.0, 0.0),
            start_time: Instant::now(),
        }
    }

    // Rotate a point using rotation matrices
    fn rotate_point(&self, p: Point3D) -> Point3D {
        let (rx, ry, rz) = self.rotation;

        // X-axis rotation
        let cos_x = rx.cos();
        let sin_x = rx.sin();
        let y1 = p.y * cos_x - p.z * sin_x;
        let z1 = p.y * sin_x + p.z * cos_x;

        // Y-axis rotation
        let cos_y = ry.cos();
        let sin_y = ry.sin();
        let x2 = p.x * cos_y + z1 * sin_y;
        let z2 = -p.x * sin_y + z1 * cos_y;

        // Z-axis rotation
        let cos_z = rz.cos();
        let sin_z = rz.sin();
        let x3 = x2 * cos_z - y1 * sin_z;
        let y3 = x2 * sin_z + y1 * cos_z;

        Point3D {
            x: x3,
            y: y3,
            z: z2,
        }
    }

    // Rotate all vertices
    fn rotate_vertices(&self) -> Vec<Point3D> {
        self.vertices
            .iter()
            .map(|v| self.rotate_point(*v))
            .collect()
    }

    // Project a 3D point to 2D screen coordinates
    fn project(&self, p: Point3D) -> Point2D {
        let distance = self.options.distance;
        let z_factor = 1.0 / (distance + p.z);

        // Calculate screen coordinates
        let width = self.screen_size.0 as f32;
        let height = self.screen_size.1 as f32;

        let scale_factor = width.min(height) * 0.8;

        let screen_x = width / 2.0 + p.x * z_factor * scale_factor;
        let screen_y = height / 2.0 + p.y * z_factor * scale_factor * 0.8;

        Point2D {
            x: screen_x,
            y: screen_y,
        }
    }

    // Draw the cube using ASCII characters
    fn draw_ascii(&self, projected: &[Point2D], buffer: &mut Buffer) {
        // Clear the buffer first
        buffer.fill_with(&Cell::default());

        // Draw edges
        for edge in &self.edges {
            self.draw_line(
                projected[edge.v1].x,
                projected[edge.v1].y,
                projected[edge.v2].x,
                projected[edge.v2].y,
                buffer,
            );
        }
    }

    // Draw the cube using braille patterns
    fn draw_braille(&self, projected: &[Point2D], buffer: &mut Buffer) {
        // Clear the buffer first
        buffer.fill_with(&Cell::default());

        // Draw edges with braille
        for edge in &self.edges {
            self.draw_braille_line(
                projected[edge.v1].x,
                projected[edge.v1].y,
                projected[edge.v2].x,
                projected[edge.v2].y,
                buffer,
            );
        }
    }

    // Draw a line using ASCII characters
    fn draw_line(&self, x0: f32, y0: f32, x1: f32, y1: f32, buffer: &mut Buffer) {
        let mut x0 = x0 as i32;
        let mut y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let width = self.screen_size.0 as i32;
        let height = self.screen_size.1 as i32;

        loop {
            if x0 >= 0 && x0 < width && y0 >= 0 && y0 < height {
                buffer.set(
                    x0 as usize,
                    y0 as usize,
                    Cell::new('█', style::Color::Green, style::Attribute::Bold),
                );
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                if x0 == x1 {
                    break;
                }
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                if y0 == y1 {
                    break;
                }
                err += dx;
                y0 += sy;
            }
        }
    }

    // Draw a line using braille patterns for higher resolution
    fn draw_braille_line(
        &self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        buffer: &mut Buffer,
    ) {
        // Scale to braille resolution (2x4 dots per cell)
        let x0 = x0 * 2.0;
        let y0 = y0 * 4.0;
        let x1 = x1 * 2.0;
        let y1 = y1 * 4.0;

        let mut x0 = x0 as i32;
        let mut y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        // Map from braille resolution to terminal cells
        let width = (self.screen_size.0 as i32) * 2;
        let height = (self.screen_size.1 as i32) * 4;

        // Braille dot tracking
        let mut braille_map: std::collections::HashMap<(i32, i32), u8> =
            std::collections::HashMap::new();

        loop {
            if x0 >= 0 && x0 < width && y0 >= 0 && y0 < height {
                // Calculate which terminal cell this braille dot belongs to
                let cell_x = x0 / 2;
                let cell_y = y0 / 4;

                // Calculate which dot within the braille pattern (0-7)
                let dot_x = x0 % 2;
                let dot_y = y0 % 4;
                let dot_index = dot_y * 2 + dot_x;

                // Set the corresponding bit in our braille map
                let key = (cell_x, cell_y);
                let dots = braille_map.entry(key).or_insert(0);
                *dots |= 1 << dot_index;
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                if x0 == x1 {
                    break;
                }
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                if y0 == y1 {
                    break;
                }
                err += dx;
                y0 += sy;
            }
        }

        // Convert our braille map to characters and set them in the buffer
        for ((x, y), dots) in braille_map {
            if x >= 0
                && x < self.screen_size.0 as i32
                && y >= 0
                && y < self.screen_size.1 as i32
            {
                // The Unicode braille patterns start at U+2800
                // Each bit set in our dots variable corresponds to a raised dot
                let braille_char =
                    std::char::from_u32(0x2800 + dots as u32).unwrap_or('?');

                buffer.set(
                    x as usize,
                    y as usize,
                    Cell::new(
                        braille_char,
                        style::Color::Green,
                        style::Attribute::Bold,
                    ),
                );
            }
        }
    }
}

impl DefaultOptions for Cube {
    type Options = CubeOptions;

    fn default_options(_width: u16, _height: u16) -> Self::Options {
        CubeOptionsBuilder::default()
            .cube_size(1.0)
            .rotation_speed_x(0.6)
            .rotation_speed_y(0.8)
            .rotation_speed_z(0.4)
            .distance(3.5)
            .use_braille(true)
            .build()
            .unwrap()
    }
}
