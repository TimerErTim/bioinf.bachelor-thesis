//! Chunk-aware dense grid storage with double buffering.

use std::ops::Range;

use rayon::prelude::*;

use crate::grid::coords::index_of;

/// Chunk edge length in cells. 64x64 chunks = 4096 elements, cache-friendly.
pub const DEFAULT_CHUNK_SIZE: usize = 64;

/// Dense rectangular grid, chunk-first internal ordering.
///
/// Each `CHUNK_SIZE x CHUNK_SIZE` tile (and partial edge tile) occupies a
/// contiguous slice in memory, enabling zero-copy parallel chunk splitting
/// while preserving deterministic coordinate indexing.
#[derive(Debug, Clone, PartialEq)]
pub struct Grid<T, const CHUNK_SIZE: usize = DEFAULT_CHUNK_SIZE> {
    width: usize,
    height: usize,
    data: Box<[T]>,
}

impl<T, const CHUNK_SIZE: usize> Grid<T, CHUNK_SIZE> {
    /// Creates a grid filled with `value` by cloning.
    ///
    /// # Panics
    /// Panics if `CHUNK_SIZE == 0`.
    #[must_use]
    pub fn filled(width: usize, height: usize, value: T) -> Self
    where
        T: Clone,
    {
        assert!(CHUNK_SIZE > 0, "chunk size must be positive");
        Self {
            width,
            height,
            data: vec![value; width * height].into_boxed_slice(),
        }
    }

    /// Creates a grid by evaluating `f(x, y)` for every cell.
    ///
    /// # Panics
    /// Panics if `CHUNK_SIZE == 0`.
    #[must_use]
    pub fn from_fn(width: usize, height: usize, mut f: impl FnMut(usize, usize) -> T) -> Self
    where
        T: Default + Clone,
    {
        assert!(CHUNK_SIZE > 0, "chunk size must be positive");
        let mut data = vec![T::default(); width * height].into_boxed_slice();
        for y in 0..height {
            for x in 0..width {
                let idx = index_of(x, y, width, height, CHUNK_SIZE);
                data[idx] = f(x, y);
            }
        }
        Self {
            width,
            height,
            data,
        }
    }

    /// Creates a grid from a row-major sequence, scattering elements into
    /// chunk-first storage.
    ///
    /// Returns `None` when `data.len()` does not match `width * height`.
    ///
    /// # Panics
    /// Panics if `CHUNK_SIZE == 0`.
    #[must_use]
    pub fn from_row_major(width: usize, height: usize, data: Vec<T>) -> Option<Self> {
        assert!(CHUNK_SIZE > 0, "chunk size must be positive");
        if data.len() != width * height {
            return None;
        }
        let mut slots: Vec<Option<T>> = (0..data.len()).map(|_| None).collect();
        for (linear, item) in data.into_iter().enumerate() {
            let x = linear % width;
            let y = linear / width;
            let idx = index_of(x, y, width, height, CHUNK_SIZE);
            slots[idx] = Some(item);
        }
        let ordered: Vec<T> = slots
            .into_iter()
            .map(|opt| opt.expect("bijective index_of fills every slot"))
            .collect();
        Some(Self {
            width,
            height,
            data: ordered.into_boxed_slice(),
        })
    }

    /// Width in cells.
    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    /// Height in cells.
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Total number of cells (`width * height`).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    /// True when the grid contains zero cells.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Raw slice of all cells in chunk-first storage order.
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Mutable raw slice of all cells in chunk-first storage order.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Element at a coordinate, `None` when out of bounds.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x >= self.width || y >= self.height {
            None
        } else {
            self.data
                .get(index_of(x, y, self.width, self.height, CHUNK_SIZE))
        }
    }

    /// Mutable element at a coordinate, `None` when out of bounds.
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x >= self.width || y >= self.height {
            None
        } else {
            self.data
                .get_mut(index_of(x, y, self.width, self.height, CHUNK_SIZE))
        }
    }

    /// Number of chunks along each axis (rounded up).
    #[must_use]
    pub const fn chunk_counts(&self) -> (usize, usize) {
        (
            self.width.div_ceil(CHUNK_SIZE),
            self.height.div_ceil(CHUNK_SIZE),
        )
    }

    /// Cell bounds of chunk `(cx, cy)` as `(x0, y0, x1, y1)`, `x1`/`y1` exclusive.
    #[must_use]
    pub fn chunk_bounds(&self, cx: usize, cy: usize) -> Option<(usize, usize, usize, usize)> {
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

    /// Contiguous range in [`Self::as_slice`] corresponding to chunk `(cx, cy)`.
    #[must_use]
    pub fn chunk_range(&self, cx: usize, cy: usize) -> Option<Range<usize>> {
        let (x0, y0, x1, y1) = self.chunk_bounds(cx, cy)?;
        let chunk_w = x1 - x0;
        let chunk_h = y1 - y0;
        let start = cy * CHUNK_SIZE * self.width + cx * CHUNK_SIZE * chunk_h;
        Some(start..(start + chunk_w * chunk_h))
    }

    /// Iterates every cell as `(x, y, &T)` in row-major coordinate order.
    pub fn iter_row_major(&self) -> impl Iterator<Item = (usize, usize, &T)> {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| {
                let idx = index_of(x, y, self.width, self.height, CHUNK_SIZE);
                (x, y, &self.data[idx])
            })
        })
    }

    /// Visits disjoint mutable chunk slices sequentially in canonical `(cy, cx)` order.
    ///
    /// Closure receives `(chunk_index, x0, y0, chunk_w, chunk_h, &mut [T])`, where
    /// within the slice cell `(x0 + rx, y0 + ry)` sits at `ry * chunk_w + rx`.
    pub fn for_each_chunk_mut(
        &mut self,
        mut f: impl FnMut(usize, usize, usize, usize, usize, &mut [T]),
    ) {
        if self.width == 0 || self.height == 0 {
            return;
        }
        let (ccx, _) = self.chunk_counts();
        let width = self.width;
        let height = self.height;
        for (cy, row_slice) in self.data.chunks_mut(CHUNK_SIZE * width).enumerate() {
            let y0 = cy * CHUNK_SIZE;
            let chunk_h = (height - y0).min(CHUNK_SIZE);
            for (cx, chunk_slice) in row_slice.chunks_mut(CHUNK_SIZE * chunk_h).enumerate() {
                let x0 = cx * CHUNK_SIZE;
                let chunk_w = (width - x0).min(CHUNK_SIZE);
                let chunk_idx = cy * ccx + cx;
                f(chunk_idx, x0, y0, chunk_w, chunk_h, chunk_slice);
            }
        }
    }

    /// Visits disjoint mutable chunk slices in parallel across `rayon` workers.
    ///
    /// Closure receives `(chunk_index, x0, y0, chunk_w, chunk_h, &mut [T])`.
    pub fn par_for_each_chunk_mut<F>(&mut self, f: F)
    where
        T: Send,
        F: Fn(usize, usize, usize, usize, usize, &mut [T]) + Sync,
    {
        if self.width == 0 || self.height == 0 {
            return;
        }
        let (ccx, _) = self.chunk_counts();
        let width = self.width;
        let height = self.height;
        let f_ref = &f;
        self.data
            .par_chunks_mut(CHUNK_SIZE * width)
            .enumerate()
            .for_each(|(cy, row_slice)| {
                let y0 = cy * CHUNK_SIZE;
                let chunk_h = (height - y0).min(CHUNK_SIZE);
                row_slice
                    .par_chunks_mut(CHUNK_SIZE * chunk_h)
                    .enumerate()
                    .for_each(|(cx, chunk_slice)| {
                        let x0 = cx * CHUNK_SIZE;
                        let chunk_w = (width - x0).min(CHUNK_SIZE);
                        let chunk_idx = cy * ccx + cx;
                        f_ref(chunk_idx, x0, y0, chunk_w, chunk_h, chunk_slice);
                    });
            });
    }
}

/// Double-buffered pair of grids for snapshot-and-swap phases.
#[derive(Debug, Clone)]
pub struct DoubleBuffer<T, const CHUNK_SIZE: usize = DEFAULT_CHUNK_SIZE> {
    front: Grid<T, CHUNK_SIZE>,
    back: Grid<T, CHUNK_SIZE>,
}

impl<T, const CHUNK_SIZE: usize> DoubleBuffer<T, CHUNK_SIZE> {
    /// Creates two identically sized buffers filled with `value`.
    #[must_use]
    pub fn filled(width: usize, height: usize, value: T) -> Self
    where
        T: Clone,
    {
        Self {
            front: Grid::filled(width, height, value.clone()),
            back: Grid::filled(width, height, value),
        }
    }

    /// Creates a double buffer from an initialized front grid, cloning it
    /// into the back buffer.
    #[must_use]
    pub fn from_grid(front: Grid<T, CHUNK_SIZE>) -> Self
    where
        T: Clone,
    {
        let back = front.clone();
        Self { front, back }
    }

    /// Frozen read side of the current tick.
    #[must_use]
    pub const fn front(&self) -> &Grid<T, CHUNK_SIZE> {
        &self.front
    }

    /// Mutable access to the front buffer (for world setup between ticks).
    pub fn front_mut(&mut self) -> &mut Grid<T, CHUNK_SIZE> {
        &mut self.front
    }

    /// Read side of the back buffer.
    #[must_use]
    pub const fn back(&self) -> &Grid<T, CHUNK_SIZE> {
        &self.back
    }

    /// Write side of the current tick.
    #[must_use]
    pub fn back_mut(&mut self) -> &mut Grid<T, CHUNK_SIZE> {
        &mut self.back
    }

    /// Splits the double buffer into an immutable front grid and a mutable
    /// back grid so phases can read `front` while writing `back` without
    /// cloning.
    #[must_use]
    pub fn split_mut(&mut self) -> (&Grid<T, CHUNK_SIZE>, &mut Grid<T, CHUNK_SIZE>) {
        (&self.front, &mut self.back)
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
        assert!(Grid::<i32>::from_row_major(2, 2, vec![1, 2, 3]).is_none());
        let g = Grid::<i32>::from_row_major(2, 2, vec![1, 2, 3, 4]).unwrap();
        assert_eq!(g.get(0, 0), Some(&1));
        assert_eq!(g.get(1, 0), Some(&2));
        assert_eq!(g.get(0, 1), Some(&3));
        assert_eq!(g.get(1, 1), Some(&4));
    }

    #[test]
    fn get_roundtrip() {
        let mut g: Grid<u32> = Grid::filled(3, 2, 0u32);
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
    fn chunk_slices_cover_unaligned_grid_exactly() {
        let mut g: Grid<usize, 4> = Grid::from_fn(10, 7, |x, y| y * 10 + x);
        let mut visited = vec![false; 70];
        g.for_each_chunk_mut(|_idx, x0, y0, cw, ch, slice| {
            assert_eq!(slice.len(), cw * ch);
            for ry in 0..ch {
                for rx in 0..cw {
                    let x = x0 + rx;
                    let y = y0 + ry;
                    let val = slice[ry * cw + rx];
                    assert_eq!(val, y * 10 + x);
                    assert!(!visited[val]);
                    visited[val] = true;
                }
            }
        });
        assert!(visited.into_iter().all(|v| v));
    }

    #[test]
    fn parallel_chunk_mutation_matches_sequential() {
        let mut seq: Grid<usize, 4> = Grid::filled(13, 11, 0);
        let mut par: Grid<usize, 4> = Grid::filled(13, 11, 0);

        seq.for_each_chunk_mut(|idx, x0, y0, cw, ch, slice| {
            for ry in 0..ch {
                for rx in 0..cw {
                    slice[ry * cw + rx] = idx * 1000 + (y0 + ry) * 13 + (x0 + rx);
                }
            }
        });
        par.par_for_each_chunk_mut(|idx, x0, y0, cw, ch, slice| {
            for ry in 0..ch {
                for rx in 0..cw {
                    slice[ry * cw + rx] = idx * 1000 + (y0 + ry) * 13 + (x0 + rx);
                }
            }
        });

        assert_eq!(seq, par);
    }

    #[test]
    fn double_buffer_swap_and_split() {
        let mut db: DoubleBuffer<u32> = DoubleBuffer::filled(2, 2, 0u32);
        {
            let (front, back) = db.split_mut();
            assert_eq!(front.get(0, 0), Some(&0));
            *back.get_mut(0, 0).unwrap() += 5;
        }
        assert_eq!(db.front().get(0, 0), Some(&0));
        db.swap();
        assert_eq!(db.front().get(0, 0), Some(&5));
    }
}
