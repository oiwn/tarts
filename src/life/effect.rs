//! The state of each cell in the next generation is determined by the states
//! of that cell and its eight neighbors in the current generation:
//!
//! Overpopulation:
//!     If a living cell is surrounded by more than three living cells, it dies.
//! Underpopulation:
//!     If a living cell is surrounded by fewer than two living cells, it dies.
//! Survival:
//!     If a living cell is surrounded by two or three living cells, it survives.
//! Birth:
//!     If a dead cell is surrounded by exactly three living cells,
//!     it becomes a living cell.
use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};
use crossterm::style;
use derive_builder::Builder;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

static DEAD_CELLS_CHARS: LazyLock<Vec<char>> = LazyLock::new(|| {
    let characters = "ﾊﾐﾋｰｳｼﾅﾓﾆｻﾜﾂｵﾘｱﾎﾃﾏｹﾒｴｶｷﾑﾕﾗｾﾈｽﾀﾇﾍ";
    let char_vec: Vec<char> = characters.chars().collect();
    char_vec
});

#[derive(Builder, Default, Debug, Serialize, Deserialize, Clone)]
#[builder(public, setter(into))]
pub struct ConwayLifeOptions {
    #[builder(default = "200")]
    #[serde(skip)]
    pub initial_cells: u32,
    #[builder(default = "1.0")]
    pub cells_coeff: f32,
}

#[derive(Clone)]
pub struct LifeCell {
    pub character: char,
    pub color: style::Color,
}

pub struct ConwayLife {
    pub screen_size: (u16, u16),
    #[allow(dead_code)]
    options: ConwayLifeOptions,
    buffer: Buffer,
    cells: HashMap<(usize, usize), LifeCell>,
    pub rng: rand::prelude::ThreadRng,
    pub current_gen: u8,
}

impl LifeCell {
    pub fn new(character: char) -> Self {
        Self {
            character,
            color: style::Color::Rgb { r: 0, g: 255, b: 0 },
        }
    }

    pub fn update_color_and_char(
        &mut self,
        rng: &mut rand::prelude::ThreadRng,
        current_gen: u8,
    ) {
        let green_color = 255_u8.wrapping_sub(current_gen);
        match current_gen {
            0..=230 => {
                self.color = style::Color::Rgb {
                    r: 0,
                    g: green_color,
                    b: 0,
                }; // Green
                let random_index = rng.random_range(0..DEAD_CELLS_CHARS.len());
                self.character = *DEAD_CELLS_CHARS.get(random_index).unwrap();
            }
            _ => {
                self.color = style::Color::Rgb {
                    r: 0,
                    g: green_color,
                    b: 0,
                };
                let random_index = rng.random_range(0..DEAD_CELLS_CHARS.len());
                self.character = *DEAD_CELLS_CHARS.get(random_index).unwrap();
            }
        }
    }
}

impl TerminalEffect for ConwayLife {
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        let mut curr_buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);

        // fill current buffer
        self.fill_buffer(&mut curr_buffer);

        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    fn update(&mut self) {
        let mut next_cells = HashMap::new();

        // update current generation counter
        self.current_gen = (self.current_gen + 1) % 255;

        for (index, _) in self.buffer.iter().enumerate() {
            let neighbors = get_neighbors_by_index(&self.buffer, index);
            if neighbors.is_empty() {
                continue;
            };
            let (nx, ny) = self.buffer.pos_of(index);
            let alive_neighbors = neighbors.len();

            if let Some(cell) = self.cells.get_mut(&(nx, ny)) {
                cell.update_color_and_char(&mut self.rng, self.current_gen);

                // Survival: an alive cell with 2 or 3 alive neighbors stays alive
                if alive_neighbors == 2 || alive_neighbors == 3 {
                    next_cells.insert((nx, ny), cell.clone());
                }
            } else {
                // Birth: a dead cell with exactly 3 alive neighbors becomes alive
                if alive_neighbors == 3 {
                    let mut new_cell = LifeCell::new('*');
                    new_cell.update_color_and_char(&mut self.rng, self.current_gen); // Initialize generation and update color/char
                    next_cells.insert((nx, ny), new_cell);
                    // Replace 'X' with the desired initial state
                }
                // TODO:  here should process state of dead cell
            };
        }

        // generate new cells, if cell already present, skip
        for _ in 0..9 {
            // Inserting glider at a random position with random rotation
            let glider_size = 3;
            let x = self
                .rng
                .random_range(2..self.buffer.width - glider_size + 1);
            let y = self
                .rng
                .random_range(2..self.buffer.height - glider_size + 1);
            let rotation = [0, 90, 180, 270][self.rng.random_range(0..4)];
            insert_glider(&mut next_cells, x, y, rotation, self.current_gen);
        }
        self.cells = next_cells;
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width, height);
    }

    fn reset(&mut self) {
        todo!();
    }
}

impl ConwayLife {
    pub fn new(options: ConwayLifeOptions, screen_size: (u16, u16)) -> Self {
        let mut rng = rand::rng();
        let buffer = Buffer::new(screen_size.0 as usize, screen_size.1 as usize);

        let mut cells = HashMap::new();
        for _ in 0..options.initial_cells {
            let lc = LifeCell::new('*');
            let x = rng.random_range(0..screen_size.0) as usize;
            let y = rng.random_range(0..screen_size.1) as usize;

            cells.insert((x, y), lc);
        }

        Self {
            screen_size,
            options,
            buffer,
            cells,
            rng,
            current_gen: 0,
        }
    }

    pub fn fill_buffer(&mut self, buffer: &mut Buffer) {
        for ((x, y), cell) in self.cells.iter() {
            buffer.set(
                *x,
                *y,
                Cell::new(cell.character, cell.color, style::Attribute::Bold),
            )
        }
    }
}

fn insert_glider(
    cells: &mut HashMap<(usize, usize), LifeCell>,
    x: usize,
    y: usize,
    rotation: i32,
    current_gen: u8,
) {
    let base_glider = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];

    let rotated_glider = base_glider.iter().map(|&(dx, dy)| {
        match rotation {
            0 => (x + dx, y + dy),
            90 => (x + dy, y - dx + 2), // Adjusted for rotation
            180 => (x - dx + 2, y - dy + 2), // Adjusted for rotation
            270 => (x - dy, y + dx),    // Adjusted for rotation
            _ => (x + dx, y + dy),      // Default case, no rotation
        }
    });

    let green_color = 255_u8.wrapping_sub(current_gen);

    for coords in rotated_glider {
        cells.insert(
            coords,
            LifeCell {
                character: '0',
                color: style::Color::Rgb {
                    r: 0,
                    g: green_color,
                    b: 0,
                },
            },
        );
    }
}

pub fn get_neighbors_by_index(buf: &Buffer, index: usize) -> Vec<(usize, Cell)> {
    let mut neighbors = Vec::new();
    let (x, y) = buf.pos_of(index);
    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 {
                continue; // Skip the cell itself
            }
            let nx = x as i32 + i;
            let ny = y as i32 + j;
            // Check if the coordinates are within the buffer bounds
            if nx >= 0 && nx < buf.width as i32 && ny >= 0 && ny < buf.height as i32
            {
                let idx = nx as usize + ny as usize * buf.width;
                let cell = buf.get(nx as usize, ny as usize);
                if cell.symbol != ' ' {
                    neighbors.push((idx, cell));
                }
            }
        }
    }
    neighbors
}

#[allow(dead_code)]
pub fn get_neighbors_by_coords(
    buf: &Buffer,
    x: usize,
    y: usize,
) -> Vec<(usize, usize, &Cell)> {
    let mut neighbors = Vec::new();
    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 {
                continue; // Skip the cell itself
            }
            let nx = x as i32 + i;
            let ny = y as i32 + j;
            // Check if the coordinates are within the buffer bounds
            if nx >= 0 && nx < buf.width as i32 && ny >= 0 && ny < buf.height as i32
            {
                neighbors.push((
                    nx as usize,
                    ny as usize,
                    &buf.buffer[nx as usize + ny as usize * buf.width],
                ));
            }
        }
    }
    neighbors
}

impl DefaultOptions for ConwayLife {
    type Options = ConwayLifeOptions;

    fn default_options(width: u16, height: u16) -> Self::Options {
        let initial_cells = ((width as u32 * height as u32) as f32 * 0.3) as u32; // 30% of screen space

        ConwayLifeOptionsBuilder::default()
            .initial_cells(initial_cells)
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_neighbors_by_index() {
        let buf = Buffer::new(3, 3);

        for x in 0..3 {
            for y in 0..3 {
                let res = get_neighbors_by_index(&buf, buf.index_of(x, y));
                assert!(res.is_empty());
            }
        }
    }

    #[test]
    fn survive_neighbors_by_index() {
        let mut buf = Buffer::new(3, 3);
        let cell = Cell::new('*', style::Color::Blue, style::Attribute::Bold);
        buf.set(0, 0, cell);
        buf.set(0, 1, cell);
        buf.set(0, 2, cell);

        let res = get_neighbors_by_index(&buf, buf.index_of(1, 1));
        assert_eq!(res.len(), 3);

        let res = get_neighbors_by_index(&buf, buf.index_of(0, 0));
        assert_eq!(res.len(), 1);
    }
}
