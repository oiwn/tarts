use crossterm::style;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub symbol: char,
    pub color: style::Color,
    pub attr: style::Attribute,
}

/// Buffer implementation, coordinates unlike in crossterm started from [0, 0]
#[derive(Clone)]
pub struct Buffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Cell>,
}

impl Cell {
    pub fn new(symbol: char, color: style::Color, attr: style::Attribute) -> Self {
        Self {
            symbol,
            color,
            attr,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: ' ',
            color: style::Color::Black,
            attr: style::Attribute::Reset,
        }
    }
}

impl Buffer {
    // Keep in mind!
    // Indexing from 0: 0 1 2 3 4  | Square: 16
    // Indexing from 1: 1 2 3 4 5  | Square: 25
    // Need to check width of height are greater than zero
    pub fn new(width: usize, height: usize) -> Self {
        // fill buffer with default values
        debug_assert!(width > 0 && height > 0);
        Self {
            width,
            height,
            buffer: vec![Cell::default(); width * height],
        }
    }

    pub fn fill_with(&mut self, cell: &Cell) {
        self.buffer.fill(*cell);
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        debug_assert!(x < self.width && y < self.height);
        let index = self.index_of(x, y);
        self.buffer[index] = cell;
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        let index = self.index_of(x, y);
        self.buffer[index]
    }

    #[inline]
    pub fn index_of(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    pub fn pos_of(&self, i: usize) -> (usize, usize) {
        (i % self.width, i / self.width)
    }

    // Return x, y and Cell
    pub fn diff(&self, other: &Buffer) -> Vec<(usize, usize, Cell)> {
        let prev_buffer = &self.buffer;
        let next_buffer = &other.buffer;

        let mut updates: Vec<(usize, usize, Cell)> = vec![];

        for (i, (curr, prev)) in
            next_buffer.iter().zip(prev_buffer.iter()).enumerate()
        {
            if curr != prev {
                let (x, y) = self.pos_of(i);
                debug_assert!(x < self.width && y < self.height);
                updates.push((x, y, next_buffer[i]));
            }
        }

        updates
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> std::slice::Iter<'_, Cell> {
        self.buffer.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_default() {
        let c = Cell::default();
        assert_eq!(c.symbol, ' ');
        assert_eq!(c.color, style::Color::Black);
        assert_eq!(c.attr, style::Attribute::Reset);
    }

    #[test]
    fn cell_new() {
        let c = Cell::new('x', style::Color::Red, style::Attribute::Bold);
        assert_eq!(c.symbol, 'x');
        assert_eq!(c.color, style::Color::Red);
        assert_eq!(c.attr, style::Attribute::Bold);
    }

    #[test]
    fn cell_partial_eq() {
        let a = Cell::new('a', style::Color::Green, style::Attribute::Bold);
        let b = Cell::new('a', style::Color::Green, style::Attribute::Bold);
        let c = Cell::new('b', style::Color::Green, style::Attribute::Bold);
        let d = Cell::new('a', style::Color::Red, style::Attribute::Bold);
        let e = Cell::new('a', style::Color::Green, style::Attribute::Reset);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
        assert_ne!(a, e);
    }

    #[test]
    fn create_new() {
        let buf = Buffer::new(5, 4);
        assert_eq!(buf.width, 5);
        assert_eq!(buf.height, 4);
        assert_eq!(buf.buffer.len(), 20);
    }

    #[test]
    fn new_filled_with_defaults() {
        let buf = Buffer::new(3, 2);
        for cell in &buf.buffer {
            assert_eq!(*cell, Cell::default());
        }
    }

    #[test]
    fn get_size() {
        let buf = Buffer::new(10, 20);
        assert_eq!(buf.get_size(), (10, 20));
    }

    #[test]
    fn index_of() {
        let buf = Buffer::new(4, 3);
        assert_eq!(buf.index_of(0, 0), 0);
        assert_eq!(buf.index_of(3, 0), 3);
        assert_eq!(buf.index_of(0, 1), 4);
        assert_eq!(buf.index_of(3, 1), 7);
        assert_eq!(buf.index_of(0, 2), 8);
        assert_eq!(buf.index_of(3, 2), 11);
    }

    #[test]
    fn pos_of() {
        let buf = Buffer::new(4, 3);
        assert_eq!(buf.pos_of(0), (0, 0));
        assert_eq!(buf.pos_of(3), (3, 0));
        assert_eq!(buf.pos_of(4), (0, 1));
        assert_eq!(buf.pos_of(7), (3, 1));
        assert_eq!(buf.pos_of(11), (3, 2));
    }

    #[test]
    fn pos_of_roundtrip() {
        let buf = Buffer::new(5, 4);
        for y in 0..buf.height {
            for x in 0..buf.width {
                let idx = buf.index_of(x, y);
                let (rx, ry) = buf.pos_of(idx);
                assert_eq!((rx, ry), (x, y));
            }
        }
    }

    #[test]
    fn set_and_get() {
        let mut buf = Buffer::new(3, 3);
        let cell = Cell::new('*', style::Color::Yellow, style::Attribute::Bold);
        buf.set(1, 2, cell);
        assert_eq!(buf.get(1, 2), cell);
        assert_eq!(buf.get(0, 0), Cell::default());
    }

    #[test]
    fn fill_with() {
        let mut buf = Buffer::new(3, 2);
        let cell = Cell::new('.', style::Color::Blue, style::Attribute::Reset);
        buf.fill_with(&cell);
        for c in &buf.buffer {
            assert_eq!(*c, cell);
        }
    }

    #[test]
    fn diff_identical() {
        let buf1 = Buffer::new(3, 3);
        let buf2 = Buffer::new(3, 3);
        assert!(buf1.diff(&buf2).is_empty());
    }

    #[test]
    fn diff_single_cell() {
        let buf1 = Buffer::new(3, 3);
        let mut buf2 = Buffer::new(3, 3);
        let cell = Cell::new('X', style::Color::Red, style::Attribute::Bold);
        buf2.set(1, 1, cell);

        let diff = buf1.diff(&buf2);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0], (1, 1, cell));
    }

    #[test]
    fn diff_different_sizes_compares_overlap() {
        let buf1 = Buffer::new(2, 2);
        let mut buf2 = Buffer::new(3, 3);
        buf2.set(
            0,
            0,
            Cell::new('X', style::Color::Red, style::Attribute::Bold),
        );
        // Only compares cells that exist in both buffers (first 4 cells)
        let diff = buf1.diff(&buf2);
        assert_eq!(diff.len(), 1);
    }

    #[test]
    fn diff() {
        let mut buf = Buffer::new(3, 3);
        let point = buf.index_of(0, 0);
        buf.buffer[point] =
            Cell::new('b', style::Color::Green, style::Attribute::NormalIntensity);
        let point = buf.index_of(0, 1);
        buf.buffer[point] =
            Cell::new('a', style::Color::Green, style::Attribute::NormalIntensity);

        let mut next_buf = Buffer::new(3, 3);
        let point = buf.index_of(0, 0);
        next_buf.buffer[point] = Cell::new(
            'c',
            style::Color::DarkGreen,
            style::Attribute::NormalIntensity,
        );
        let point = buf.index_of(0, 1);
        next_buf.buffer[point] =
            Cell::new('b', style::Color::Green, style::Attribute::NormalIntensity);
        let point = buf.index_of(0, 2);
        next_buf.buffer[point] =
            Cell::new('a', style::Color::Green, style::Attribute::NormalIntensity);

        let diff = buf.diff(&next_buf);
        assert_eq!(diff.len(), 3);
    }

    #[test]
    fn iter() {
        let buf = Buffer::new(2, 2);
        let mut count = 0;
        for c in buf.iter() {
            assert_eq!(*c, Cell::default());
            count += 1;
        }
        assert_eq!(count, 4);
    }
}
