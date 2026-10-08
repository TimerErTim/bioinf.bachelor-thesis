//! Global simulation parameters.

use serde::{Deserialize, Serialize};

/// Parameters that are global to the world and constant during a run.
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
}
