//! Global simulation parameters.

use serde::{Deserialize, Serialize};

/// Parameters that are global to the world and constant during a run.
// TODO(design): grow the global parameter set as the model matures.
// Candidates already discussed: light intensity cycle parameters,
// atmospheric reference pressure, geothermal heat flux, orbital
// eccentricity, magnetic field strength. Keep every parameter global and
// constant for a run (determinism rule).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalParams {
    /// Surface gravity in m/s^2 (affects density-driven flows in later increments).
    pub gravity: f64,
    /// Length of one simulation day in ticks.
    pub day_length_ticks: u32,
    /// Axial tilt in radians (drives seasonal light variation).
    pub axial_tilt: f64,
    /// Diffusion coefficient shared by the default gas CA.
    pub diffusion_coefficient: f64,
    /// Global light intensity scaling the per-cell light field (0..=1).
    pub light_intensity: f64,
}
