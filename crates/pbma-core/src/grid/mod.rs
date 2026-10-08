//! Grid cell management: coordinates, chunked dense storage, neighborhoods.
//!
//! The grid is static and dense for the foreseeable future, so storage is a
//! plain `Vec<T>` with chunk-aligned access. Slotmaps live in the registry
//! module where entity lifetimes are dynamic (species); this module stays
//! index-based.

/// Coordinates and neighborhood offsets.
pub mod coords;

/// Chunk-aware dense grid storage with double buffering.
pub mod storage;

pub use storage::{DoubleBuffer, Grid};
