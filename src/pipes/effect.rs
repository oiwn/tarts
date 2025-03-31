use std::collections::HashMap;
use std::sync::LazyLock;
use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};
use crossterm::style;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

static STRAIGHT_LINE_CHARS: LazyLock<HashMap<usize, char>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(1, '┃');
    m.insert(2, '┃');
    m.insert(3, '━');
    m.insert(4, '━');
    m.insert(5, '┏');
    m.insert(6, '┓');
    m.insert(7, '┗');
    m.insert(8, '┛');
    m
});

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct PipesOptions {
    #[builder(default = "0.3")]
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
        let mut curr_buffer = Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);

        // Draw all pipes
        for pipe in &self.pipes {
            self.draw_pipe(pipe, &mut curr_buffer);
        }

        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    fn update(&mut self) {
        for pipe in &mut self.pipes {
            self.update_pipe(pipe);
        }
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
        self.reset();
    }

    fn reset(&mut self) {
        self.buffer = Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);
        self.init_pipes();
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
}

impl DefaultOptions for Pipes {
    type Options = PipesOptions;

    fn default_options(_width: u16, _height: u16) -> Self::Options {
        PipesOptionsBuilder::default()
            .turn_probability(0.3)
            .line_type(1usize)
            .build()
            .unwrap()
    }
}