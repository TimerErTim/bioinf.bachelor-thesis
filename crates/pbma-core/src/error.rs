//! Kernel error types.

use pbma_model::SpeciesId;

/// Result alias for kernel operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors raised by the kernel.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A species id does not match any live species (stale or foreign key).
    #[error("unknown species id {0:?}")]
    UnknownSpecies(SpeciesId),

    /// A species name is already registered.
    #[error("species name already registered: {0}")]
    DuplicateSpecies(String),

    /// Grid coordinate is outside the world bounds.
    #[error("cell {x}, {y} outside world bounds {width}x{height}")]
    OutOfBounds {
        /// X coordinate that was out of bounds.
        x: u32,
        /// Y coordinate that was out of bounds.
        y: u32,
        /// World width in cells.
        width: u32,
        /// World height in cells.
        height: u32,
    },
}
