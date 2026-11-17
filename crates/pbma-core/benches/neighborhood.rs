//! Neighborhood iteration benchmark over both canonical search patterns.

use std::hint::black_box;

use pbma_core::grid::coords::{MOORE, VON_NEUMANN, in_bounds, index_of};
use pbma_core::grid::storage::DEFAULT_CHUNK_SIZE;

fn neighbor_sum(cells: &[f64], width: usize, height: usize, offsets: &[(i32, i32)]) -> f64 {
    let mut total = 0.0;
    for y in 0..height {
        for x in 0..width {
            let mut acc = 0.0;
            for &(dx, dy) in offsets {
                let nx = x as i64 + i64::from(dx);
                let ny = y as i64 + i64::from(dy);
                if in_bounds(nx, ny, width as u32, height as u32) {
                    acc += cells
                        [index_of(nx as usize, ny as usize, width, height, DEFAULT_CHUNK_SIZE)];
                }
            }
            total += acc;
        }
    }
    total
}

fn main() {
    let width = 1000usize;
    let height = 1000usize;
    let cells: Vec<f64> = (0..width * height).map(|i| i as f64).collect();

    let patterns: [(&str, &[(i32, i32)]); 2] = [("VON_NEUMANN", &VON_NEUMANN), ("MOORE", &MOORE)];
    for (name, offsets) in patterns {
        let start = std::time::Instant::now();
        let sum = black_box(neighbor_sum(
            black_box(&cells),
            width,
            height,
            black_box(offsets),
        ));
        let elapsed = start.elapsed();
        println!("{name}: neighbor sum {sum:.0} over 1M cells in {elapsed:?}");
    }
}
