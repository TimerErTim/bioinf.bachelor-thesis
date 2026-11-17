//! Coordinates, boundary topologies, and neighborhood definitions.

use serde::{Deserialize, Serialize};

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

/// Boundary topology controlling how coordinates beyond grid edges resolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Topology {
    /// Box world: out-of-bounds neighbors mirror the boundary cell (no flux across edges).
    #[default]
    Clamped,
    /// Cylindrical planet: east-west (`x`) wraps longitudinally, north-south (`y`) clamps at poles.
    Planetary,
    /// Torus: both `x` and `y` wrap around modulo grid dimensions.
    Toroidal,
}

/// Checks whether an offset coordinate is inside a `width x height` grid.
#[must_use]
pub fn in_bounds(x: i64, y: i64, width: u32, height: u32) -> bool {
    x >= 0 && y >= 0 && x < i64::from(width) && y < i64::from(height)
}

/// Resolves a neighbor coordinate `(x + dx, y + dy)` under the given [`Topology`].
///
/// Always returns an in-bounds coordinate when `width > 0` and `height > 0`.
/// Pairwise neighbor symmetry is preserved across all three topologies,
/// keeping symmetric exchange rules mass-conserving.
#[must_use]
#[inline]
pub fn resolve_neighbor(
    x: usize,
    y: usize,
    dx: i32,
    dy: i32,
    width: usize,
    height: usize,
    topology: Topology,
) -> (usize, usize) {
    let raw_x = x as isize + dx as isize;
    let raw_y = y as isize + dy as isize;
    match topology {
        Topology::Clamped => {
            let nx = if raw_x < 0 || raw_x >= width as isize {
                x
            } else {
                raw_x as usize
            };
            let ny = if raw_y < 0 || raw_y >= height as isize {
                y
            } else {
                raw_y as usize
            };
            (nx, ny)
        }
        Topology::Planetary => {
            let nx = raw_x.rem_euclid(width as isize) as usize;
            let ny = if raw_y < 0 || raw_y >= height as isize {
                y
            } else {
                raw_y as usize
            };
            (nx, ny)
        }
        Topology::Toroidal => (
            raw_x.rem_euclid(width as isize) as usize,
            raw_y.rem_euclid(height as isize) as usize,
        ),
    }
}

/// Chunk-first index of an in-bounds coordinate.
///
/// Caller must ensure `x < width`, `y < height`, and `chunk_size > 0`.
#[must_use]
#[inline(always)]
pub const fn index_of(x: usize, y: usize, width: usize, height: usize, chunk_size: usize) -> usize {
    let cx = x / chunk_size;
    let rx = x % chunk_size;
    let cy = y / chunk_size;
    let ry = y % chunk_size;
    let rem_w = width - cx * chunk_size;
    let chunk_w = if rem_w < chunk_size {
        rem_w
    } else {
        chunk_size
    };
    let rem_h = height - cy * chunk_size;
    let chunk_h = if rem_h < chunk_size {
        rem_h
    } else {
        chunk_size
    };
    cy * chunk_size * width + cx * chunk_size * chunk_h + ry * chunk_w + rx
}

/// Linear chunk index of an in-bounds coordinate in row-major chunk order.
///
/// Caller must ensure `x < width` and `chunk_size > 0`.
#[must_use]
#[inline(always)]
pub const fn chunk_index_of(x: usize, y: usize, width: usize, chunk_size: usize) -> usize {
    let chunks_x = width.div_ceil(chunk_size);
    (y / chunk_size) * chunks_x + (x / chunk_size)
}

/// Converts cell `(x, y)` center into geographic `(latitude_rad, longitude_rad)`.
///
/// `y = 0` is the northern edge (`+pi/2`), `y = height - 1` is the southern edge (`-pi/2`).
/// `x` spans `[0, 2*pi)` eastward.
#[must_use]
pub fn lat_lon_rad(x: u32, y: u32, width: u32, height: u32) -> (f64, f64) {
    let lat = if height == 0 {
        0.0
    } else {
        (0.5 - (f64::from(y) + 0.5) / f64::from(height)) * std::f64::consts::PI
    };
    let lon = if width == 0 {
        0.0
    } else {
        (f64::from(x) / f64::from(width)) * std::f64::consts::TAU
    };
    (lat, lon)
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

    #[test]
    fn chunk_first_index_is_bijective_on_unaligned_grid() {
        let width = 65;
        let height = 67;
        let chunk_size = 64;
        let mut seen = vec![false; width * height];
        for y in 0..height {
            for x in 0..width {
                let idx = index_of(x, y, width, height, chunk_size);
                assert!(!seen[idx], "collision at ({x}, {y}) -> {idx}");
                seen[idx] = true;
            }
        }
        assert!(seen.into_iter().all(|s| s));
    }

    #[test]
    fn planetary_topology_wraps_east_west_and_clamps_poles() {
        assert_eq!(
            resolve_neighbor(0, 0, -1, 0, 10, 5, Topology::Planetary),
            (9, 0)
        );
        assert_eq!(
            resolve_neighbor(9, 0, 1, 0, 10, 5, Topology::Planetary),
            (0, 0)
        );
        assert_eq!(
            resolve_neighbor(3, 0, 0, -1, 10, 5, Topology::Planetary),
            (3, 0)
        );
        assert_eq!(
            resolve_neighbor(3, 4, 0, 1, 10, 5, Topology::Planetary),
            (3, 4)
        );
    }

    #[test]
    fn toroidal_topology_wraps_both_axes() {
        assert_eq!(
            resolve_neighbor(0, 0, -1, -1, 10, 5, Topology::Toroidal),
            (9, 4)
        );
        assert_eq!(
            resolve_neighbor(9, 4, 1, 1, 10, 5, Topology::Toroidal),
            (0, 0)
        );
    }

    #[test]
    fn lat_lon_spans_poles_and_equator() {
        let (north, _) = lat_lon_rad(0, 0, 8, 9);
        let (eq, _) = lat_lon_rad(0, 4, 8, 9);
        let (south, _) = lat_lon_rad(0, 8, 8, 9);
        assert!(north > 1.0);
        assert!(eq.abs() < 1e-12);
        assert!(south < -1.0);
    }
}
