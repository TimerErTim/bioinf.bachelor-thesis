//! Global, planetary, and orbital simulation parameters.

use serde::{Deserialize, Serialize};

use crate::grid::coords::Topology;
use crate::{Error, Result};

/// Newtonian gravitational constant in $\mathrm{m^3\,kg^{-1}\,s^{-2}}$.
pub const GRAVITATIONAL_CONSTANT: f64 = 6.674_30e-11;

/// Solar constant at $1\ \mathrm{AU}$ in $\mathrm{W\,m^{-2}}$.
pub const SOLAR_CONSTANT_WM2: f64 = 1_361.0;

/// Stefan-Boltzmann constant in $\mathrm{W\,m^{-2}\,K^{-4}}$.
pub const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;

/// Ideal gas constant in $\mathrm{J\,mol^{-1}\,K^{-1}}$.
pub const IDEAL_GAS_CONSTANT: f64 = 8.314_462_618;

/// Physical bulk properties of the simulated planet.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlanetParams {
    /// Total planetary mass in kilograms (Earth $\approx 5.9722 \times 10^{24}$ kg).
    pub mass_kg: f64,
    /// Mean planetary diameter in meters (Earth $\approx 1.2742 \times 10^{7}$ m).
    pub diameter_m: f64,
    /// Relative global dipole magnetic field strength (`1.0` = modern Earth).
    pub magnetic_field_strength: f64,
    /// Geothermal heat flux in $\mathrm{W\,m^{-2}}$ (Earth $\approx 0.087$).
    pub geothermal_flux_wm2: f64,
}

impl PlanetParams {
    /// Modern Earth physical parameters.
    #[must_use]
    pub const fn earth_like() -> Self {
        Self {
            mass_kg: 5.972_2e24,
            diameter_m: 1.274_2e7,
            magnetic_field_strength: 1.0,
            geothermal_flux_wm2: 0.087,
        }
    }

    /// Planetary radius $R = D / 2$ in meters.
    #[must_use]
    pub fn radius_m(&self) -> f64 {
        0.5 * self.diameter_m
    }

    /// Surface gravity $g = G M / R^2$ in $\mathrm{m\,s^{-2}}$.
    #[must_use]
    pub fn surface_gravity(&self) -> f64 {
        let r = self.radius_m();
        GRAVITATIONAL_CONSTANT * self.mass_kg / (r * r)
    }

    /// Escape velocity $v_e = \sqrt{2 G M / R}$ in $\mathrm{m\,s^{-1}}$.
    #[must_use]
    pub fn escape_velocity_ms(&self) -> f64 {
        (2.0 * GRAVITATIONAL_CONSTANT * self.mass_kg / self.radius_m()).sqrt()
    }

    /// Total spherical surface area $4 \pi R^2$ in $\mathrm{m^2}$.
    #[must_use]
    pub fn surface_area_m2(&self) -> f64 {
        let r = self.radius_m();
        4.0 * std::f64::consts::PI * r * r
    }

    /// Equatorial circumference $\pi D$ in meters.
    #[must_use]
    pub fn circumference_m(&self) -> f64 {
        std::f64::consts::PI * self.diameter_m
    }

    /// Mean surface area per grid cell in $\mathrm{m^2}$.
    #[must_use]
    pub fn mean_cell_area_m2(&self, width: u32, height: u32) -> f64 {
        let count = f64::from(width) * f64::from(height);
        if count <= 0.0 {
            0.0
        } else {
            self.surface_area_m2() / count
        }
    }

    /// Validates that mass and diameter are strictly positive and finite.
    ///
    /// # Errors
    /// Returns [`Error::InvalidParams`] on non-positive or non-finite values.
    pub fn validate(&self) -> Result<()> {
        if !self.mass_kg.is_finite() || self.mass_kg <= 0.0 {
            return Err(Error::InvalidParams("planet mass_kg must be > 0".into()));
        }
        if !self.diameter_m.is_finite() || self.diameter_m <= 0.0 {
            return Err(Error::InvalidParams("planet diameter_m must be > 0".into()));
        }
        if !self.magnetic_field_strength.is_finite() || self.magnetic_field_strength < 0.0 {
            return Err(Error::InvalidParams(
                "magnetic_field_strength must be >= 0".into(),
            ));
        }
        Ok(())
    }
}

impl Default for PlanetParams {
    fn default() -> Self {
        Self::earth_like()
    }
}

/// Orbital and rotational parameters driving insolation and seasonal cycles.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OrbitParams {
    /// Semi-major axis in astronomical units (`1.0` = Earth-Sun distance).
    pub distance_au: f64,
    /// Orbital eccentricity in `[0, 1)` (`0.0167` for modern Earth).
    pub eccentricity: f64,
    /// Host star luminosity relative to the Sun (`1.0` = modern Sun).
    pub solar_luminosity: f64,
    /// Axial obliquity in radians (`0.4091` $\approx 23.44^\circ$ for Earth).
    pub axial_tilt_rad: f64,
    /// Length of one planetary rotation (day) in ticks. `0` yields diurnal-mean light.
    pub day_length_ticks: u32,
    /// Length of one orbital period (year) in ticks. `0` freezes seasonal phase at equinox.
    pub year_length_ticks: u32,
    /// Planetary Bond albedo in `[0, 1]` used by radiative equilibrium helpers.
    pub bond_albedo: f64,
}

impl OrbitParams {
    /// Modern Earth orbital configuration with 24-tick days and 365-day years.
    #[must_use]
    pub const fn earth_like() -> Self {
        Self {
            distance_au: 1.0,
            eccentricity: 0.016_7,
            solar_luminosity: 1.0,
            axial_tilt_rad: 0.409_092_8,
            day_length_ticks: 24,
            year_length_ticks: 24 * 365,
            bond_albedo: 0.306,
        }
    }

    /// Validates orbital bounds.
    ///
    /// # Errors
    /// Returns [`Error::InvalidParams`] if orbital distance, eccentricity,
    /// luminosity, or albedo are out of physical range.
    pub fn validate(&self) -> Result<()> {
        if !self.distance_au.is_finite() || self.distance_au <= 0.0 {
            return Err(Error::InvalidParams(
                "orbital distance_au must be > 0".into(),
            ));
        }
        if !self.eccentricity.is_finite() || !(0.0..1.0).contains(&self.eccentricity) {
            return Err(Error::InvalidParams(
                "orbital eccentricity must be in [0, 1)".into(),
            ));
        }
        if !self.solar_luminosity.is_finite() || self.solar_luminosity < 0.0 {
            return Err(Error::InvalidParams("solar_luminosity must be >= 0".into()));
        }
        if !self.bond_albedo.is_finite() || !(0.0..=1.0).contains(&self.bond_albedo) {
            return Err(Error::InvalidParams("bond_albedo must be in [0, 1]".into()));
        }
        Ok(())
    }
}

impl Default for OrbitParams {
    fn default() -> Self {
        Self::earth_like()
    }
}

/// Parameters that are global to the world and constant during a run.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GlobalParams {
    /// Physical bulk planetary parameters (mass -> gravity, diameter -> area).
    pub planet: PlanetParams,
    /// Orbital and rotational parameters driving insolation and seasons.
    pub orbit: OrbitParams,
    /// Grid edge topology (clamped box, east-west planetary wrap, or torus).
    pub topology: Topology,
    /// Diffusion coefficient shared by the default gas CA.
    pub diffusion_coefficient: f64,
    /// Global light intensity multiplier scaling normalized cell light.
    pub light_intensity: f64,
    /// Baseline UV fraction of total stellar irradiance (`~0.08` for G-type stars).
    pub stellar_uv_fraction: f64,
}

impl GlobalParams {
    /// Earth-like planetary simulation parameters with planetary topology.
    #[must_use]
    pub const fn earth_like() -> Self {
        Self {
            planet: PlanetParams::earth_like(),
            orbit: OrbitParams::earth_like(),
            topology: Topology::Planetary,
            diffusion_coefficient: 0.14,
            light_intensity: 1.0,
            stellar_uv_fraction: 0.08,
        }
    }

    /// Surface gravity derived from planet mass and diameter ($\mathrm{m\,s^{-2}}$).
    #[must_use]
    pub fn gravity(&self) -> f64 {
        self.planet.surface_gravity()
    }

    /// Validates all global parameters.
    ///
    /// # Errors
    /// Returns [`Error::InvalidParams`] if any physical or orbital bound fails.
    pub fn validate(&self) -> Result<()> {
        self.planet.validate()?;
        self.orbit.validate()?;
        if !self.diffusion_coefficient.is_finite() || self.diffusion_coefficient < 0.0 {
            return Err(Error::InvalidParams(
                "diffusion_coefficient must be >= 0".into(),
            ));
        }
        if !self.light_intensity.is_finite() || self.light_intensity < 0.0 {
            return Err(Error::InvalidParams("light_intensity must be >= 0".into()));
        }
        if !self.stellar_uv_fraction.is_finite() || !(0.0..=1.0).contains(&self.stellar_uv_fraction)
        {
            return Err(Error::InvalidParams(
                "stellar_uv_fraction must be in [0, 1]".into(),
            ));
        }
        Ok(())
    }
}

impl Default for GlobalParams {
    fn default() -> Self {
        Self::earth_like()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earth_gravity_matches_standard_value() {
        let earth = PlanetParams::earth_like();
        let g = earth.surface_gravity();
        assert!((g - 9.82).abs() < 0.02, "unexpected g = {g}");
    }

    #[test]
    fn mass_and_diameter_scale_gravity_correctly() {
        let mut heavy = PlanetParams::earth_like();
        heavy.mass_kg *= 2.0;
        assert!(
            (heavy.surface_gravity() / PlanetParams::earth_like().surface_gravity() - 2.0).abs()
                < 1e-12
        );

        let mut wide = PlanetParams::earth_like();
        wide.diameter_m *= 2.0;
        assert!(
            (wide.surface_gravity() / PlanetParams::earth_like().surface_gravity() - 0.25).abs()
                < 1e-12
        );
    }
}
