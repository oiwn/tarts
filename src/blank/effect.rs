use crate::buffer::{Buffer, Cell};
use crate::common::TerminalEffect;
use crossterm::style;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct BlankOptions {}

#[allow(dead_code)]
pub struct Blank {
    screen_size: (u16, u16),
    options: BlankOptions,
    buffer: Buffer,
}

impl TerminalEffect for Blank {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        let mut curr_buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);

        curr_buffer.fill_with(&Cell {
            symbol: '#',
            color: style::Color::Green,
            attr: style::Attribute::Reset,
        });

        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    fn update(&mut self) {}

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height)
    }

    fn reset(&mut self) {
        *self = Self::new(self.options.clone(), self.screen_size);
    }
}

impl Blank {
    pub fn new(options: BlankOptions, screen_size: (u16, u16)) -> Self {
        let mut buffer =
            Buffer::new(screen_size.0 as usize, screen_size.1 as usize);

        buffer.fill_with(&Cell {
            symbol: '#',
            color: style::Color::Green,
            attr: style::Attribute::Reset,
        });

        Self {
            screen_size,
            options,
            buffer,
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn blank_test() {}
}
