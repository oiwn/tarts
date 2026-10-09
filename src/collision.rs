//! Shared bounding-box collision detection for entity-based effects.
//! Entities implement [`Collidable`] to expose their axis-aligned bounding
//! box; [`find_collisions`] returns the index pairs whose boxes overlap so
//! each effect can apply its own reaction (bounce, repel, animate, ...).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox {
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

pub trait Collidable {
    fn bounding_box(&self) -> BoundingBox;
}

/// Returns index pairs `(i, j)` with `i < j` whose bounding boxes intersect.
pub fn find_collisions<T: Collidable>(entities: &[T]) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            if entities[i]
                .bounding_box()
                .intersects(&entities[j].bounding_box())
            {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Point(f32, f32);
    impl Collidable for Point {
        fn bounding_box(&self) -> BoundingBox {
            BoundingBox {
                x: self.0,
                y: self.1,
                width: 2.0,
                height: 2.0,
            }
        }
    }

    #[test]
    fn detects_overlap() {
        let entities = vec![Point(0.0, 0.0), Point(1.0, 1.0), Point(10.0, 10.0)];
        assert_eq!(find_collisions(&entities), vec![(0, 1)]);
    }

    #[test]
    fn no_overlap() {
        let entities = vec![Point(0.0, 0.0), Point(5.0, 5.0)];
        assert!(find_collisions(&entities).is_empty());
    }

    #[test]
    fn touching_edges_do_not_count() {
        let entities = vec![Point(0.0, 0.0), Point(2.0, 0.0)];
        assert!(find_collisions(&entities).is_empty());
    }
}
