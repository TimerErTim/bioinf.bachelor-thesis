//! Environment state of a grid cell.

use serde::{Deserialize, Serialize};

use crate::quantity::Amount;

/// Chemical species tracked in the environment.
///
/// The set is fixed for the vertical slice; new gases extend this enum.
/// Determinism rule: the discriminant order is the canonical iteration order
/// for all environment fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gas {
    /// Oxygen.
    O2,
    /// Carbon dioxide.
    Co2,
    /// Methane.
    Ch4,
    /// Water vapor.
    H2o,
}

/// All gases in canonical order.
pub const GASES: [Gas; 4] = [Gas::O2, Gas::Co2, Gas::Ch4, Gas::H2o];

impl Gas {
    /// Index of this gas in per-cell fixed-size arrays.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Gas::O2 => 0,
            Gas::Co2 => 1,
            Gas::Ch4 => 2,
            Gas::H2o => 3,
        }
    }
}

/// Environment state of one grid cell.
///
/// Fixed-size arrays keep cells cache-friendly and make iteration order
/// deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EnvState {
    /// Gas amounts indexed by [`Gas::index`].
    pub gases: [Amount; 4],
    /// Temperature in kelvin.
    pub temperature: f64,
    /// Light intensity, unitless normalized [0, 1].
    pub light: f64,
}

impl EnvState {
    /// Reads the amount of one gas.
    #[must_use]
    pub const fn gas(&self, gas: Gas) -> Amount {
        self.gases[gas.index()]
    }

    /// Writes the amount of one gas.
    pub fn set_gas(&mut self, gas: Gas, amount: Amount) {
        self.gases[gas.index()] = amount;
    }

    /// Total gas mass in this cell, summed in canonical order.
    #[must_use]
    pub fn total_mass(&self) -> f64 {
        self.gases.iter().map(|g| g.value()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_mass_sums_all_gases() {
        let mut env = EnvState {
            gases: [Amount::ZERO; 4],
            temperature: 288.0,
            light: 0.5,
        };
        env.set_gas(Gas::O2, Amount::new(2.0));
        env.set_gas(Gas::Co2, Amount::new(3.0));
        assert!((env.total_mass() - 5.0).abs() < 1e-12);
    }
}
