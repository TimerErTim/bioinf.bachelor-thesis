//! Pure domain types for the PBMA simulation.
//!
//! These types are intentionally behavior-free: they define the vocabulary
//! of the simulation (environment state of a cell, species definitions,
//! population amounts, global world parameters) without any simulation
//! machinery.

/// Non-negative amounts with saturating arithmetic.
pub mod quantity;

/// Environment state held by a single grid cell.
pub mod env;

/// Species definition and population bookkeeping.
pub mod species;

/// Global simulation parameters (gravity, day length, ...).
pub mod params;

pub use env::{EnvState, GASES, Gas};
pub use params::GlobalParams;
pub use quantity::PositiveF64;
pub use species::{Population, Species, SpeciesId, SpeciesTraits};
