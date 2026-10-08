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

/// Weight of a diagonal neighbor relative to an axis neighbor.
///
/// Diagonal cells are farther away by a factor of sqrt(2), so their
/// influence on diffusion-like exchange is scaled by 1/sqrt(2).
pub const DIAGONAL_WEIGHT: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// Per-neighbor weights matching the canonical [`MOORE`] order:
/// 1.0 for the four axis neighbors, [`DIAGONAL_WEIGHT`] for the four
/// diagonals.
pub const MOORE_WEIGHTS: [f64; 8] = [
    1.0,
    DIAGONAL_WEIGHT,
    1.0,
    DIAGONAL_WEIGHT,
    1.0,
    DIAGONAL_WEIGHT,
    1.0,
    DIAGONAL_WEIGHT,
];

/// Sum of all [`MOORE_WEIGHTS`] (4 + 4/sqrt(2) ~ 6.828).
///
/// Weighted exchange schemes must scale their rate by this sum to stay
/// mass-conserving and positivity-preserving.
pub const MOORE_WEIGHT_SUM: f64 = 4.0 + 4.0 * DIAGONAL_WEIGHT;

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

    #[test]
    fn moore_weights_match_offsets() {
        assert_eq!(MOORE_WEIGHTS[0], 1.0);
        assert_eq!(MOORE_WEIGHTS[1], DIAGONAL_WEIGHT);
        assert!((DIAGONAL_WEIGHT - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12);
        assert!((MOORE_WEIGHT_SUM - (4.0 + 4.0 * DIAGONAL_WEIGHT)).abs() < 1e-12);
    }
}
