//! Neighborhood iteration benchmark.

use std::hint::black_box;

use pbma_grid::coords::{VON_NEUMANN, in_bounds, index_of};

fn neighbor_sum(cells: &[f64], width: u32, height: u32) -> f64 {
    let mut total = 0.0;
    for y in 0..height {
        for x in 0..width {
            let mut acc = 0.0;
            for (dx, dy) in VON_NEUMANN {
                let nx = i64::from(x) + i64::from(dx);
                let ny = i64::from(y) + i64::from(dy);
                if in_bounds(nx, ny, width, height) {
                    acc += cells[index_of(nx as u32, ny as u32, width)];
                }
            }
            total += acc;
        }
    }
    total
}

fn main() {
    let width = 1000u32;
    let height = 1000u32;
    let cells: Vec<f64> = (0..(width as usize) * (height as usize))
        .map(|i| i as f64)
        .collect();

    let start = std::time::Instant::now();
    let sum = black_box(neighbor_sum(black_box(&cells), width, height));
    let elapsed = start.elapsed();
    println!("neighbor sum {sum:.0} over 1M cells in {elapsed:?}");
}
