//! Chunk-aware dense grid storage with double buffering.

/// Chunk edge length in cells. 64x64 chunks = 4096 elements, cache-friendly.
pub const CHUNK_SIZE: u32 = 64;

/// Dense rectangular grid of `T`, row-major.
///
/// Storage is a single `Vec<T>`. Chunk awareness currently shapes the public
/// helpers (chunk counts, chunk ranges) that later parallel phases will use
/// to split work; the layout itself stays row-major to keep neighborhood
/// math simple and deterministic.
#[derive(Debug, Clone)]
pub struct Grid<T> {
    width: u32,
    height: u32,
    data: Vec<T>,
}

impl<T> Grid<T> {
    /// Creates a grid filled with `value` by cloning.
    #[must_use]
    pub fn filled(width: u32, height: u32, value: T) -> Self
    where
        T: Clone,
    {
        Self {
            width,
            height,
            data: vec![value; (width as usize) * (height as usize)],
        }
    }

    /// Creates a grid from a row-major vector.
    ///
    /// Returns `None` when `data.len()` does not match `width * height`.
    #[must_use]
    pub fn from_row_major(width: u32, height: u32, data: Vec<T>) -> Option<Self> {
        if data.len() == (width as usize) * (height as usize) {
            Some(Self {
                width,
                height,
                data,
            })
        } else {
            None
        }
    }

    /// Width in cells.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Height in cells.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Underlying row-major slice.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Mutable underlying row-major slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Element at a coordinate, `None` when out of bounds.
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> Option<&T> {
        if x >= self.width || y >= self.height {
            None
        } else {
            self.data
                .get((y as usize) * (self.width as usize) + (x as usize))
        }
    }

    /// Mutable element at a coordinate, `None` when out of bounds.
    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut T> {
        if x >= self.width || y >= self.height {
            None
        } else {
            let idx = (y as usize) * (self.width as usize) + (x as usize);
            self.data.get_mut(idx)
        }
    }

    /// Number of chunks along each axis (rounded up).
    #[must_use]
    pub const fn chunk_counts(&self) -> (u32, u32) {
        (
            self.width.div_ceil(CHUNK_SIZE),
            self.height.div_ceil(CHUNK_SIZE),
        )
    }

    /// Cell bounds of chunk (cx, cy) as (x0, y0, x1, y1), x1/y1 exclusive.
    #[must_use]
    pub fn chunk_bounds(&self, cx: u32, cy: u32) -> Option<(u32, u32, u32, u32)> {
        let (ccx, ccy) = self.chunk_counts();
        if cx >= ccx || cy >= ccy {
            return None;
        }
        let x0 = cx * CHUNK_SIZE;
        let y0 = cy * CHUNK_SIZE;
        let x1 = ((cx + 1) * CHUNK_SIZE).min(self.width);
        let y1 = ((cy + 1) * CHUNK_SIZE).min(self.height);
        Some((x0, y0, x1, y1))
    }
}

/// Double-buffered pair of grids for snapshot-and-swap phases.
#[derive(Debug, Clone)]
pub struct DoubleBuffer<T> {
    front: Grid<T>,
    back: Grid<T>,
}

impl<T> DoubleBuffer<T>
where
    T: Clone,
{
    /// Creates two identically sized buffers filled with `value`.
    #[must_use]
    pub fn filled(width: u32, height: u32, value: T) -> Self {
        Self {
            front: Grid::filled(width, height, value.clone()),
            back: Grid::filled(width, height, value),
        }
    }

    /// Frozen read side of the current tick.
    #[must_use]
    pub const fn front(&self) -> &Grid<T> {
        &self.front
    }

    /// Write side of the current tick.
    #[must_use]
    pub fn back_mut(&mut self) -> &mut Grid<T> {
        &mut self.back
    }

    /// Swaps front and back so the freshly written buffer becomes the snapshot.
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_row_major_rejects_wrong_len() {
        assert!(Grid::from_row_major(2, 2, vec![1, 2, 3]).is_none());
        assert!(Grid::from_row_major(2, 2, vec![1, 2, 3, 4]).is_some());
    }

    #[test]
    fn get_roundtrip() {
        let mut g = Grid::filled(3, 2, 0u32);
        *g.get_mut(2, 1).unwrap() += 7;
        assert_eq!(g.get(2, 1), Some(&7));
        assert_eq!(g.get(3, 1), None);
    }

    #[test]
    fn chunk_counts_round_up() {
        let g: Grid<u8> = Grid::filled(65, 64, 0);
        assert_eq!(g.chunk_counts(), (2, 1));
        assert_eq!(g.chunk_bounds(1, 0), Some((64, 0, 65, 64)));
        assert_eq!(g.chunk_bounds(2, 0), None);
    }

    #[test]
    fn double_buffer_swap() {
        let mut db = DoubleBuffer::filled(2, 2, 0u32);
        *db.back_mut().get_mut(0, 0).unwrap() += 5;
        assert_eq!(db.front().get(0, 0), Some(&0));
        db.swap();
        assert_eq!(db.front().get(0, 0), Some(&5));
    }
}
