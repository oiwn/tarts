use std::collections::HashMap;
use std::sync::LazyLock;
use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};
use crossterm::style;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use rand::Rng;

// Fixed typo in variable name
static STRAIGHT_LINE_CHARS: LazyLock<HashMap<usize, char>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(1, '┃');  // Vertical
    m.insert(2, '┃');  // Vertical (duplicate)
    m.insert(3, '━');  // Horizontal
    m.insert(4, '━');  // Horizontal (duplicate)
    m.insert(5, '┏');  // Top-left corner
    m.insert(6, '┓');  // Top-right corner
    m.insert(7, '┗');  // Bottom-left corner
    m.insert(8, '┛');  // Bottom-right corner
    m
});

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct PipesOptions {
    #[builder(default = "0.1")]
    pub turn_probability: f64,
    #[builder(default = "1")]
    pub line_type: usize,
}

pub struct Pipes {
    pub screen_size: (u16, u16),
    options: PipesOptions,
    buffer: Buffer,
    pipe_made: bool,
    prev_location: (usize, usize),
    prev_node_type: usize,
    next_location: (usize, usize),
    pub colors: Vec<style::Color>,
    curr_color: style::Color,
    pub rng: rand::prelude::ThreadRng,
}

impl TerminalEffect for Pipes {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        // Clone the previous buffer to work with
        let mut curr_buffer = self.buffer.clone();

        // Update the pipe
        if !self.pipe_made {
            // Start a new pipe from an edge
            self.start_new_pipe(&mut curr_buffer);
        } else {
            // Continue the existing pipe
            self.continue_pipe(&mut curr_buffer);
        }

        // Calculate difference between old and new buffer
        let diff = self.buffer.diff(&curr_buffer);

        // Update our reference buffer
        self.buffer = curr_buffer;

        diff
    }

    fn update(&mut self) {
        // No additional state updates needed between frames
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
        self.reset();
    }

    fn reset(&mut self) {
        self.buffer = Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);
        self.pipe_made = false;
    }
}

impl Pipes {
    pub fn new(options: PipesOptions, screen_size: (u16, u16)) -> Self {
        let buffer = Buffer::new(screen_size.0 as usize, screen_size.1 as usize);
        let colors = vec![
            style::Color::Red,
            style::Color::Green,
            style::Color::Blue,
            style::Color::Yellow,
            style::Color::Cyan,
            style::Color::Magenta,
        ];

        Self {
            screen_size,
            options,
            buffer,
            pipe_made: false,
            prev_location: (0, 0),
            prev_node_type: 0,
            next_location: (0, 0),
            colors,
            curr_color: style::Color::White,
            rng: rand::rng(),
        }
    }

    // Start a new pipe from a random edge location
    fn start_new_pipe(&mut self, buffer: &mut Buffer) {
        let width = self.screen_size.0 as usize;
        let height = self.screen_size.1 as usize;

        // Choose a random edge
        let edge = self.rng.random_range(0..4); // 0: top, 1: right, 2: bottom, 3: left

        // Generate a position on that edge and direction inward
        let (pos, direction) = match edge {
            0 => { // Top edge
                let x = self.rng.random_range(0..width);
                ((x, 0), (0, 1))
            },
            1 => { // Right edge
                let y = self.rng.random_range(0..height);
                ((width - 1, y), (-1, 0))
            },
            2 => { // Bottom edge
                let x = self.rng.random_range(0..width);
                ((x, height - 1), (0, -1))
            },
            3 => { // Left edge
                let y = self.rng.random_range(0..height);
                ((0, y), (1, 0))
            },
            _ => unreachable!(),
        };

        // Choose a random color
        self.curr_color = self.colors[self.rng.random_range(0..self.colors.len())];

        // Determine initial node type based on direction
        let node_type = match direction {
            (0, 1) | (0, -1) => 1, // Vertical
            (1, 0) | (-1, 0) => 3, // Horizontal
            _ => unreachable!(),
        };

        // Set the initial node
        buffer.set(
            pos.0,
            pos.1,
            Cell::new(
                *STRAIGHT_LINE_CHARS.get(&node_type).unwrap_or(&'?'),
                self.curr_color,
                style::Attribute::Bold,
            ),
        );

        // Store state for the next iteration
        self.prev_location = pos;
        self.prev_node_type = node_type;

        // Calculate next position based on direction
        self.next_location = (
            (pos.0 as i32 + direction.0) as usize,
            (pos.1 as i32 + direction.1) as usize,
        );

        self.pipe_made = true;
    }

    // Continue an existing pipe
    fn continue_pipe(&mut self, buffer: &mut Buffer) {
        let width = self.screen_size.0 as usize;
        let height = self.screen_size.1 as usize;

        // Check if we've reached an edge
        if self.next_location.0 >= width || self.next_location.1 >= height {
            // End the current pipe and start a new one
            self.pipe_made = false;
            self.start_new_pipe(buffer);
            return;
        }

        // Determine current direction based on previous node and locations
        let current_dir = self.get_direction();

        // Decide whether to turn or continue straight
        let turn = self.rng.random_range(0.0..1.0) < self.options.turn_probability;

        // Calculate the next direction and node type
        let (next_dir, node_type) = if turn {
            self.get_turn_direction_and_node(current_dir)
        } else {
            // Continue in the same direction
            (current_dir, match current_dir {
                (0, 1) | (0, -1) => 1, // Vertical
                (1, 0) | (-1, 0) => 3, // Horizontal
                _ => unreachable!(),
            })
        };

        // Draw the current node
        buffer.set(
            self.next_location.0,
            self.next_location.1,
            Cell::new(
                *STRAIGHT_LINE_CHARS.get(&node_type).unwrap_or(&'?'),
                self.curr_color,
                style::Attribute::Bold,
            ),
        );

        // Update state for the next iteration
        self.prev_location = self.next_location;
        self.prev_node_type = node_type;

        // Calculate next position based on direction
        self.next_location = (
            (self.next_location.0 as i32 + next_dir.0) as usize,
            (self.next_location.1 as i32 + next_dir.1) as usize,
        );
    }

    // Get the current direction based on previous node and location
    fn get_direction(&self) -> (i32, i32) {
        match self.prev_node_type {
            1 | 2 => { // Vertical pipe
                if self.next_location.1 > self.prev_location.1 {
                    (0, 1) // Down
                } else {
                    (0, -1) // Up
                }
            },
            3 | 4 => { // Horizontal pipe
                if self.next_location.0 > self.prev_location.0 {
                    (1, 0) // Right
                } else {
                    (-1, 0) // Left
                }
            },
            5 => { // ┏ Top-left corner
                if self.next_location.0 > self.prev_location.0 {
                    (1, 0) // Right
                } else {
                    (0, 1) // Down
                }
            },
            6 => { // ┓ Top-right corner
                if self.next_location.0 < self.prev_location.0 {
                    (-1, 0) // Left
                } else {
                    (0, 1) // Down
                }
            },
            7 => { // ┗ Bottom-left corner
                if self.next_location.0 > self.prev_location.0 {
                    (1, 0) // Right
                } else {
                    (0, -1) // Up
                }
            },
            8 => { // ┛ Bottom-right corner
                if self.next_location.0 < self.prev_location.0 {
                    (-1, 0) // Left
                } else {
                    (0, -1) // Up
                }
            },
            _ => (0, 0), // Shouldn't happen
        }
    }

    // Get a new direction and node type when turning
    fn get_turn_direction_and_node(&mut self, current_dir: (i32, i32)) -> ((i32, i32), usize) {
        match current_dir {
            (0, 1) => { // Moving down
                if self.rng.random_bool(0.5) {
                    ((1, 0), 7) // Turn right -> ┗
                } else {
                    ((-1, 0), 8) // Turn left -> ┛
                }
            },
            (0, -1) => { // Moving up
                if self.rng.random_bool(0.5) {
                    ((1, 0), 5) // Turn right -> ┏
                } else {
                    ((-1, 0), 6) // Turn left -> ┓
                }
            },
            (1, 0) => { // Moving right
                if self.rng.random_bool(0.5) {
                    ((0, 1), 6) // Turn down -> ┓
                } else {
                    ((0, -1), 8) // Turn up -> ┛
                }
            },
            (-1, 0) => { // Moving left
                if self.rng.random_bool(0.5) {
                    ((0, 1), 5) // Turn down -> ┓
                } else {
                    ((0, -1), 7) // Turn up -> ┗
                }
            },
            _ => ((0, 0), 1), // Shouldn't happen
        }
    }
}

impl DefaultOptions for Pipes {
    type Options = PipesOptions;

    fn default_options(_width: u16, _height: u16) -> Self::Options {
        PipesOptionsBuilder::default()
            .turn_probability(0.1)
            .line_type(1usize)
            .build()
            .unwrap()
    }
}