//! Pure planetary physics helpers: orbital insolation, ozone UV attenuation,
//! greenhouse radiative balance, and barometric scale height.

use crate::model::env::{EnvState, GasId, GasRegistry};
use crate::model::params::{
    GlobalParams, IDEAL_GAS_CONSTANT, SOLAR_CONSTANT_WM2, STEFAN_BOLTZMANN,
};

/// Instantaneous orbital geometry of the planet at a given tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitalState {
    /// Mean anomaly / orbital phase in `[0, 2*pi)`.
    pub orbital_angle_rad: f64,
    /// True anomaly in `[0, 2*pi)` accounting for eccentricity.
    pub true_anomaly_rad: f64,
    /// Instantaneous planet-star distance in astronomical units.
    pub solar_distance_au: f64,
    /// Solar declination angle in radians in `[-axial_tilt, +axial_tilt]`.
    pub solar_declination_rad: f64,
    /// Subsolar longitude in `[0, 2*pi)` (`None` when diurnal averaging is active).
    pub subsolar_longitude_rad: Option<f64>,
    /// Top-of-atmosphere solar irradiance perpendicular to rays ($\mathrm{W\,m^{-2}}$).
    pub toa_irradiance_wm2: f64,
}

/// Local radiative state at a single grid cell for one tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Insolation {
    /// Cosine of solar zenith angle (or diurnal mean factor), clamped to `[0, 1]`.
    pub cos_zenith: f64,
    /// Top-of-atmosphere horizontal irradiance in $\mathrm{W\,m^{-2}}$.
    pub toa_flux_wm2: f64,
    /// Normalized light intensity in `[0, 1]` scaled by `GlobalParams::light_intensity`.
    pub normalized_light: f64,
    /// Top-of-atmosphere UV irradiance in $\mathrm{W\,m^{-2}}$.
    pub toa_uv_wm2: f64,
}

/// Computes the instantaneous orbital geometry at `tick` by solving Kepler's
/// equation $M = E - e \sin E$ with fixed-iteration Newton-Raphson.
#[must_use]
pub fn orbital_state(tick: u64, params: &GlobalParams) -> OrbitalState {
    let orbit = &params.orbit;
    let mean_anomaly = if orbit.year_length_ticks == 0 {
        0.0
    } else {
        let phase =
            (tick % u64::from(orbit.year_length_ticks)) as f64 / f64::from(orbit.year_length_ticks);
        phase * std::f64::consts::TAU
    };

    // Solve M = E - e sin(E) in 5 Newton iterations (converges < 1e-12 for e < 0.99).
    let e = orbit.eccentricity;
    let mut eccentric_anomaly = mean_anomaly;
    for _ in 0..5 {
        let f = eccentric_anomaly - e * eccentric_anomaly.sin() - mean_anomaly;
        let df = 1.0 - e * eccentric_anomaly.cos();
        eccentric_anomaly -= f / df;
    }

    let cos_e = eccentric_anomaly.cos();
    let sin_e = eccentric_anomaly.sin();
    let solar_distance_au = orbit.distance_au * (1.0 - e * cos_e);

    let true_anomaly_rad = ((1.0 - e * e).sqrt() * sin_e)
        .atan2(cos_e - e)
        .rem_euclid(std::f64::consts::TAU);

    let solar_declination_rad = (orbit.axial_tilt_rad.sin() * true_anomaly_rad.sin()).asin();

    let subsolar_longitude_rad = if orbit.day_length_ticks == 0 {
        None
    } else {
        let rot_phase =
            (tick % u64::from(orbit.day_length_ticks)) as f64 / f64::from(orbit.day_length_ticks);
        Some(rot_phase * std::f64::consts::TAU)
    };

    let toa_irradiance_wm2 =
        SOLAR_CONSTANT_WM2 * orbit.solar_luminosity / (solar_distance_au * solar_distance_au);

    OrbitalState {
        orbital_angle_rad: mean_anomaly,
        true_anomaly_rad,
        solar_distance_au,
        solar_declination_rad,
        subsolar_longitude_rad,
        toa_irradiance_wm2,
    }
}

/// Computes local top-of-atmosphere insolation at `(latitude_rad, longitude_rad)`.
///
/// When `orbital.subsolar_longitude_rad` is `None` (`day_length_ticks == 0`),
/// evaluates the exact daily-mean insolation factor
/// $\frac{1}{\pi}(H_0 \sin\phi \sin\delta + \cos\phi \cos\delta \sin H_0)$.
#[must_use]
pub fn cell_insolation(
    latitude_rad: f64,
    longitude_rad: f64,
    orbital: &OrbitalState,
    params: &GlobalParams,
) -> Insolation {
    let sin_lat = latitude_rad.sin();
    let cos_lat = latitude_rad.cos().max(0.0);
    let sin_dec = orbital.solar_declination_rad.sin();
    let cos_dec = orbital.solar_declination_rad.cos();

    let cos_zenith = match orbital.subsolar_longitude_rad {
        Some(sub_lon) => {
            let hour_angle = longitude_rad - sub_lon;
            (sin_lat * sin_dec + cos_lat * cos_dec * hour_angle.cos()).clamp(0.0, 1.0)
        }
        None => {
            let denom = cos_lat * cos_dec;
            if denom <= 1e-12 {
                (sin_lat * sin_dec).clamp(0.0, 1.0)
            } else {
                let cos_h0 = (-sin_lat * sin_dec / denom).clamp(-1.0, 1.0);
                let h0 = cos_h0.acos();
                ((h0 * sin_lat * sin_dec + denom * h0.sin()) / std::f64::consts::PI).clamp(0.0, 1.0)
            }
        }
    };

    let toa_flux_wm2 = orbital.toa_irradiance_wm2 * cos_zenith;
    let rel_irradiance = orbital.toa_irradiance_wm2 / SOLAR_CONSTANT_WM2;
    let normalized_light = (cos_zenith * rel_irradiance * params.light_intensity).clamp(0.0, 1.0);
    let toa_uv_wm2 = toa_flux_wm2 * params.stellar_uv_fraction;

    Insolation {
        cos_zenith,
        toa_flux_wm2,
        normalized_light,
        toa_uv_wm2,
    }
}

/// Beer-Lambert UV transmission fraction in `(0, 1]` through atmospheric gases.
///
/// Driven primarily by ozone (`O3`) optical depth via [`GasDef::uv_absorption`].
#[must_use]
pub fn surface_uv_fraction(env: &EnvState, gases: &GasRegistry) -> f64 {
    let optical_depth: f64 = gases
        .iter()
        .map(|(id, def)| def.uv_absorption * env.gas(id).value())
        .sum();
    (-optical_depth.max(0.0)).exp()
}

/// Effective radiative equilibrium temperature in kelvin for a cell given local
/// TOA flux, planetary Bond albedo, geothermal flux, and greenhouse optical depth.
#[must_use]
pub fn radiative_equilibrium_temperature(
    toa_flux_wm2: f64,
    env: &EnvState,
    gases: &GasRegistry,
    params: &GlobalParams,
) -> f64 {
    let absorbed =
        toa_flux_wm2 * (1.0 - params.orbit.bond_albedo) + params.planet.geothermal_flux_wm2;
    let tau_gh: f64 = gases
        .iter()
        .map(|(id, def)| def.greenhouse_factor * env.gas(id).value())
        .sum();
    // Single-layer grey greenhouse amplification: T_s = T_eff * (1 + 0.75 * tau)^0.25.
    let gh_factor = 1.0 + 0.75 * tau_gh.max(0.0);
    ((absorbed.max(1e-6) * gh_factor) / STEFAN_BOLTZMANN).powf(0.25)
}

/// Chapman-inspired ozone formation/photolysis net tendency per step.
///
/// Production scales with oxygen (`O2`) and UV flux; destruction scales with
/// ozone (`O3`) squared (self-recombination) and UV photolysis, optionally
/// modulated by planetary magnetic field protection against stellar wind loss.
#[must_use]
pub fn ozone_net_delta(
    o2_amount: f64,
    o3_amount: f64,
    uv_flux_wm2: f64,
    magnetic_field_strength: f64,
    rate_scale: f64,
) -> f64 {
    let o2 = o2_amount.max(0.0);
    let o3 = o3_amount.max(0.0);
    let uv = (uv_flux_wm2 / 100.0).max(0.0);
    let production = rate_scale * 0.05 * o2 * uv / (1.0 + 0.1 * o2);
    let shielding = 1.0 / (1.0 + magnetic_field_strength.max(0.0));
    let loss = rate_scale * (0.04 * o3 * o3 + 0.01 * o3 * uv + 0.005 * shielding * o3);
    production - loss
}

/// Atmospheric barometric scale height $H = R T / (M g)$ in meters.
///
/// Connects cell temperature, gas composition (mean molar mass), and planet
/// gravity ($g = G M / R^2$).
#[must_use]
pub fn barometric_scale_height_m(env: &EnvState, gases: &GasRegistry, gravity: f64) -> f64 {
    if gravity <= 0.0 {
        return 0.0;
    }
    let total_mass = env.total_mass();
    let mean_molar_mass = if total_mass <= 1e-12 {
        0.029 // dry air baseline in kg/mol
    } else {
        let weighted: f64 = gases
            .iter()
            .map(|(id, def)| env.gas(id).value() * def.molar_mass_kg)
            .sum();
        (weighted / total_mass).max(1e-4)
    };
    IDEAL_GAS_CONSTANT * env.temperature.max(0.0) / (mean_molar_mass * gravity)
}

/// Mean molar mass of the cell's atmosphere in kg/mol.
#[must_use]
pub fn mean_molar_mass_kg(env: &EnvState, gases: &GasRegistry) -> f64 {
    let total = env.total_mass();
    if total <= 1e-12 {
        return 0.0;
    }
    gases
        .iter()
        .map(|(id, def)| env.gas(id).value() * def.molar_mass_kg)
        .sum::<f64>()
        / total
}

/// Helper to read ozone concentration directly from an [`EnvState`].
#[must_use]
pub fn ozone_amount(env: &EnvState) -> f64 {
    env.gas(GasId::O3).value()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::quantity::PositiveF64;

    #[test]
    fn ozone_shields_surface_uv_exponentially() {
        let gases = GasRegistry::with_defaults();
        let mut anoxic = EnvState::new(288.0, 1.0);
        let mut oxic = anoxic.clone();
        oxic.set_gas(GasId::O3, PositiveF64::new(1.0));
        anoxic.set_gas(GasId::O3, PositiveF64::ZERO);

        let uv_anoxic = surface_uv_fraction(&anoxic, &gases);
        let uv_oxic = surface_uv_fraction(&oxic, &gases);
        assert!((uv_anoxic - 1.0).abs() < 1e-12);
        assert!(
            uv_oxic < 0.02,
            "1 unit O3 should strongly shield UV, got {uv_oxic}"
        );
    }

    #[test]
    fn eccentricity_modulates_solar_distance_and_irradiance() {
        let mut params = GlobalParams::earth_like();
        params.orbit.eccentricity = 0.2;
        params.orbit.year_length_ticks = 100;
        let peri = orbital_state(0, &params);
        let apo = orbital_state(50, &params);
        assert!(peri.solar_distance_au < apo.solar_distance_au);
        assert!(peri.toa_irradiance_wm2 > apo.toa_irradiance_wm2);
    }

    #[test]
    fn equator_receives_more_diurnal_light_than_poles_at_equinox() {
        let mut params = GlobalParams::earth_like();
        params.orbit.day_length_ticks = 0; // diurnal average
        let orb = orbital_state(0, &params);
        let eq = cell_insolation(0.0, 0.0, &orb, &params);
        let pole = cell_insolation(std::f64::consts::FRAC_PI_2 - 0.05, 0.0, &orb, &params);
        assert!(eq.normalized_light > pole.normalized_light);
    }

    #[test]
    fn ozone_has_positive_equilibrium_under_oxygen_and_uv() {
        let mut o3 = 0.0;
        for _ in 0..500 {
            o3 = (o3 + ozone_net_delta(21.0, o3, 100.0, 1.0, 0.5)).max(0.0);
        }
        assert!(o3 > 0.5 && o3 < 5.0, "unexpected equilibrium O3 = {o3}");
    }

    #[test]
    fn scale_height_matches_earth_order_of_magnitude() {
        let gases = GasRegistry::with_defaults();
        let mut env = EnvState::new(288.0, 1.0);
        env.set_gas(GasId::N2, PositiveF64::new(78.0));
        env.set_gas(GasId::O2, PositiveF64::new(21.0));
        let h = barometric_scale_height_m(&env, &gases, 9.81);
        assert!(
            h > 7_500.0 && h < 9_500.0,
            "Earth scale height ~8.5km, got {h}"
        );
    }
}
