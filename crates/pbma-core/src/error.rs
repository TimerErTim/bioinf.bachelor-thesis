//! Kernel error types.

use crate::model::{GasId, SpeciesId, TraitId};

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

    /// A gas id is outside the gas registry.
    #[error("unknown gas id {0:?}")]
    UnknownGas(GasId),

    /// A gas symbol or name is already registered.
    #[error("gas already registered: {0}")]
    DuplicateGas(String),

    /// A species trait id is outside the trait schema.
    #[error("unknown trait id {0:?}")]
    UnknownTrait(TraitId),

    /// A species trait key is already registered.
    #[error("trait key already registered: {0}")]
    DuplicateTrait(String),

    /// Grid dimensions must be non-zero.
    #[error("invalid grid dimensions {width}x{height}: both must be > 0")]
    InvalidDimensions {
        /// Requested width.
        width: u32,
        /// Requested height.
        height: u32,
    },

    /// Global or planetary configuration parameter is invalid.
    #[error("invalid simulation parameter: {0}")]
    InvalidParams(String),

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
