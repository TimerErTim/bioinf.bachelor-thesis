//! Coordinates and neighborhood definitions.

/// Von Neumann neighborhood offsets in canonical order: north, east, south, west.
pub const VON_NEUMANN: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// Moore neighborhood offsets in canonical order: N, NE, E, SE, S, SW, W, NW.
pub const MOORE: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// Checks whether an offset coordinate is inside a `width x height` grid.
#[must_use]
pub fn in_bounds(x: i64, y: i64, width: u32, height: u32) -> bool {
    x >= 0 && y >= 0 && x < i64::from(width) && y < i64::from(height)
}

/// Row-major index of an in-bounds coordinate. Caller must check bounds first.
#[must_use]
pub fn index_of(x: u32, y: u32, width: u32) -> usize {
    (y as usize) * (width as usize) + (x as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_check() {
        assert!(in_bounds(0, 0, 3, 3));
        assert!(in_bounds(2, 2, 3, 3));
        assert!(!in_bounds(3, 0, 3, 3));
        assert!(!in_bounds(-1, 0, 3, 3));
        assert!(!in_bounds(0, -1, 3, 3));
    }

    #[test]
    fn canonical_orders_are_rotational() {
        // Von Neumann must start north and rotate clockwise.
        assert_eq!(VON_NEUMANN[0], (0, -1));
        assert_eq!(VON_NEUMANN[1], (1, 0));
        assert_eq!(VON_NEUMANN[2], (0, 1));
        assert_eq!(VON_NEUMANN[3], (-1, 0));
        assert_eq!(MOORE.len(), 8);
    }
}
