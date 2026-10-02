//! Pure domain types for the PBMA simulation.
//!
//! This crate has no dependencies on other pbma crates. It defines the
//! vocabulary of the simulation: environment state of a cell, species
//! definitions, population amounts, and global world parameters.

/// Amount of a substance or population. Non-negative, saturating arithmetic.
pub mod quantity;

/// Environment state held by a single grid cell.
pub mod env;

/// Species definition and population bookkeeping.
pub mod species;

/// Global simulation parameters (gravity, day length, ...).
pub mod params;

pub use env::{EnvState, GASES, Gas};
pub use params::GlobalParams;
pub use quantity::Amount;
pub use species::{Population, Species, SpeciesId, SpeciesTraits};
