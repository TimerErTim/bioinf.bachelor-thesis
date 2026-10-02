//! Default simulation adapters implementing the `pbma-core` ports.
//!
//! Everything here is replaceable: downstream consumers implement their own
//! `EnvironmentStep`, `PopulationBehavior`, or `MigrationPolicy` and the
//! kernel treats them identically.

#![deny(missing_docs)]

/// Mass-conserving gas diffusion cellular automaton.
pub mod diffusion;

/// Population growth/decline behavior.
pub mod behavior;

/// Overcrowding-driven migration.
pub mod migration;

/// Metrics recorder.
pub mod recorder;

pub use behavior::LogisticBehavior;
pub use diffusion::DiffusionStep;
pub use migration::OvercrowdingMigration;
pub use recorder::VecRecorder;
