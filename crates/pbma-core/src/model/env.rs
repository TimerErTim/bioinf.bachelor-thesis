//! Environment state of a grid cell and extensible gas registry.

use serde::{Deserialize, Serialize};

use super::quantity::PositiveF64;
use crate::{Error, Result};

/// Inline capacity for gas amounts per cell before spilling to heap storage.
/// Covers all six built-in gases (O2, CO2, CH4, H2O, O3, N2) plus two custom
/// gases with zero heap allocation and full AoS cache locality.
pub const INLINE_GASES: usize = 8;

/// Identifier for a gas registered in a [`GasRegistry`].
///
/// The index order in the registry defines the canonical deterministic
/// iteration order for all gas computations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GasId(pub u16);

impl GasId {
    /// Molecular oxygen ($\mathrm{O_2}$).
    pub const O2: Self = Self(0);
    /// Carbon dioxide ($\mathrm{CO_2}$).
    pub const CO2: Self = Self(1);
    /// Methane ($\mathrm{CH_4}$).
    pub const CH4: Self = Self(2);
    /// Water vapor ($\mathrm{H_2O}$).
    pub const H2O: Self = Self(3);
    /// Ozone ($\mathrm{O_3}$).
    pub const O3: Self = Self(4);
    /// Molecular nitrogen ($\mathrm{N_2}$).
    pub const N2: Self = Self(5);

    /// Array index for this gas.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Physical and optical properties of a tracked atmospheric gas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GasDef {
    /// Short chemical symbol (unique per registry), e.g. `"O2"`, `"O3"`.
    pub symbol: String,
    /// Human-readable gas name.
    pub name: String,
    /// Molar mass in kg/mol (e.g. `0.032` for $\mathrm{O_2}$).
    pub molar_mass_kg: f64,
    /// Dimensionless radiative greenhouse absorption weight per unit column.
    pub greenhouse_factor: f64,
    /// Dimensionless UV-B/C absorption cross-section weight (high for $\mathrm{O_3}$).
    pub uv_absorption: f64,
    /// Relative diffusivity multiplier (`1.0` baseline).
    pub relative_diffusivity: f64,
}

/// Registry of chemical species tracked in the atmosphere.
///
/// Pre-populates the six canonical atmospheric gases (`O2`, `CO2`, `CH4`,
/// `H2O`, `O3`, `N2`) at fixed [`GasId`] indices and allows downstream
/// consumers to register additional gases at setup time without patching
/// `pbma-core`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasRegistry {
    gases: Vec<GasDef>,
}

impl GasRegistry {
    /// Creates a registry populated with the six built-in gases:
    /// `O2`, `CO2`, `CH4`, `H2O`, `O3`, and `N2`.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self {
            gases: vec![
                GasDef {
                    symbol: "O2".into(),
                    name: "Oxygen".into(),
                    molar_mass_kg: 0.031_998,
                    greenhouse_factor: 0.0,
                    uv_absorption: 0.02,
                    relative_diffusivity: 1.0,
                },
                GasDef {
                    symbol: "CO2".into(),
                    name: "Carbon Dioxide".into(),
                    molar_mass_kg: 0.044_01,
                    greenhouse_factor: 0.35,
                    uv_absorption: 0.0,
                    relative_diffusivity: 0.85,
                },
                GasDef {
                    symbol: "CH4".into(),
                    name: "Methane".into(),
                    molar_mass_kg: 0.016_04,
                    greenhouse_factor: 0.85,
                    uv_absorption: 0.0,
                    relative_diffusivity: 1.25,
                },
                GasDef {
                    symbol: "H2O".into(),
                    name: "Water Vapor".into(),
                    molar_mass_kg: 0.018_015,
                    greenhouse_factor: 0.50,
                    uv_absorption: 0.0,
                    relative_diffusivity: 1.15,
                },
                GasDef {
                    symbol: "O3".into(),
                    name: "Ozone".into(),
                    molar_mass_kg: 0.047_998,
                    greenhouse_factor: 0.15,
                    uv_absorption: 4.5,
                    relative_diffusivity: 0.80,
                },
                GasDef {
                    symbol: "N2".into(),
                    name: "Nitrogen".into(),
                    molar_mass_kg: 0.028_014,
                    greenhouse_factor: 0.0,
                    uv_absorption: 0.0,
                    relative_diffusivity: 1.0,
                },
            ],
        }
    }

    /// Registers a custom gas and returns its [`GasId`].
    ///
    /// # Errors
    /// Returns [`Error::DuplicateGas`] if `def.symbol` is already registered.
    pub fn register(&mut self, def: GasDef) -> Result<GasId> {
        if self.gases.iter().any(|g| g.symbol == def.symbol) {
            return Err(Error::DuplicateGas(def.symbol));
        }
        let id = u16::try_from(self.gases.len())
            .map_err(|_| Error::InvalidParams("gas registry capacity exceeded".into()))?;
        self.gases.push(def);
        Ok(GasId(id))
    }

    /// Looks up gas metadata by id.
    ///
    /// # Errors
    /// Returns [`Error::UnknownGas`] if `id` is out of bounds.
    pub fn get(&self, id: GasId) -> Result<&GasDef> {
        self.gases.get(id.index()).ok_or(Error::UnknownGas(id))
    }

    /// Finds a gas id by chemical symbol.
    #[must_use]
    pub fn by_symbol(&self, symbol: &str) -> Option<GasId> {
        self.gases
            .iter()
            .position(|g| g.symbol == symbol)
            .map(|i| GasId(i as u16))
    }

    /// Number of registered gases.
    #[must_use]
    pub fn len(&self) -> usize {
        self.gases.len()
    }

    /// True if no gases are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.gases.is_empty()
    }

    /// Iterates all registered `(GasId, &GasDef)` pairs in canonical order.
    pub fn iter(&self) -> impl Iterator<Item = (GasId, &GasDef)> {
        self.gases
            .iter()
            .enumerate()
            .map(|(i, def)| (GasId(i as u16), def))
    }

    /// Iterates all registered [`GasId`] values in canonical order.
    pub fn ids(&self) -> impl Iterator<Item = GasId> {
        (0..self.gases.len()).map(|i| GasId(i as u16))
    }
}

impl Default for GasRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Inline-first storage for per-cell gas concentrations.
///
/// Stores up to [`INLINE_GASES`] entries inline without heap indirection and
/// spills any additional runtime-registered gases to `extra`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GasAmounts {
    inline: [PositiveF64; INLINE_GASES],
    len: usize,
    extra: Vec<PositiveF64>,
}

impl GasAmounts {
    /// Creates zero-filled storage for `count` gases.
    #[must_use]
    pub fn zeroed(count: usize) -> Self {
        let extra_len = count.saturating_sub(INLINE_GASES);
        Self {
            inline: [PositiveF64::ZERO; INLINE_GASES],
            len: count,
            extra: if extra_len > 0 {
                vec![PositiveF64::ZERO; extra_len]
            } else {
                Vec::new()
            },
        }
    }

    /// Number of tracked gases in this cell.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// True when zero gases are tracked.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Ensures the container can index up to `count` gases, zero-filling any new slots.
    pub fn resize(&mut self, count: usize) {
        if count > INLINE_GASES {
            self.extra.resize(count - INLINE_GASES, PositiveF64::ZERO);
        } else {
            self.extra.clear();
        }
        if count > self.len {
            for slot in self.len..count.min(INLINE_GASES) {
                self.inline[slot] = PositiveF64::ZERO;
            }
        }
        self.len = count;
    }

    /// Overwrites `self` from `src` without reallocating `self.extra` when
    /// capacity already suffices (zero-allocation tick writes).
    pub fn copy_from(&mut self, src: &Self) {
        self.inline = src.inline;
        self.len = src.len;
        self.extra.clear();
        self.extra.extend_from_slice(&src.extra);
    }

    /// Reads the amount of `gas`, returning [`PositiveF64::ZERO`] if out of range.
    #[must_use]
    #[inline]
    pub fn get(&self, gas: GasId) -> PositiveF64 {
        let idx = gas.index();
        if idx >= self.len {
            PositiveF64::ZERO
        } else if idx < INLINE_GASES {
            self.inline[idx]
        } else {
            self.extra[idx - INLINE_GASES]
        }
    }

    /// Sets the amount of `gas`, resizing if `gas.index() >= self.len`.
    #[inline]
    pub fn set(&mut self, gas: GasId, amount: PositiveF64) {
        let idx = gas.index();
        if idx >= self.len {
            self.resize(idx + 1);
        }
        if idx < INLINE_GASES {
            self.inline[idx] = amount;
        } else {
            self.extra[idx - INLINE_GASES] = amount;
        }
    }

    /// Total gas mass summed in canonical index order.
    #[must_use]
    pub fn total_mass(&self) -> f64 {
        let inline_sum: f64 = self.inline[..self.len.min(INLINE_GASES)]
            .iter()
            .map(|g| g.value())
            .sum();
        let extra_sum: f64 = self.extra.iter().map(|g| g.value()).sum();
        inline_sum + extra_sum
    }

    /// Iterates `(GasId, PositiveF64)` in canonical order.
    pub fn iter(&self) -> impl Iterator<Item = (GasId, PositiveF64)> + '_ {
        (0..self.len).map(|i| {
            let id = GasId(i as u16);
            (id, self.get(id))
        })
    }
}

impl Default for GasAmounts {
    fn default() -> Self {
        Self::zeroed(6)
    }
}

/// Environment state of one grid cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvState {
    /// Per-gas amounts indexed by [`GasId`].
    pub gases: GasAmounts,
    /// Temperature in kelvin.
    pub temperature: f64,
    /// Photosynthetically active normalized light intensity in `[0, 1]`.
    pub light: f64,
    /// Surface ultraviolet flux fraction / intensity (modulated by $\mathrm{O_3}$).
    pub uv: f64,
}

impl EnvState {
    /// Creates a cell environment sized for `gas_count` gases.
    #[must_use]
    pub fn with_gas_count(gas_count: usize, temperature: f64, light: f64) -> Self {
        Self {
            gases: GasAmounts::zeroed(gas_count),
            temperature,
            light,
            uv: 0.0,
        }
    }

    /// Creates a cell environment sized for the six default gases.
    #[must_use]
    pub fn new(temperature: f64, light: f64) -> Self {
        Self::with_gas_count(6, temperature, light)
    }

    /// Overwrites `self` from `src` without allocating when capacities match.
    pub fn copy_from(&mut self, src: &Self) {
        self.gases.copy_from(&src.gases);
        self.temperature = src.temperature;
        self.light = src.light;
        self.uv = src.uv;
    }

    /// Reads the amount of one gas.
    #[must_use]
    #[inline]
    pub fn gas(&self, gas: GasId) -> PositiveF64 {
        self.gases.get(gas)
    }

    /// Writes the amount of one gas.
    #[inline]
    pub fn set_gas(&mut self, gas: GasId, amount: PositiveF64) {
        self.gases.set(gas, amount);
    }

    /// Total gas mass in this cell, summed in canonical order.
    #[must_use]
    pub fn total_mass(&self) -> f64 {
        self.gases.total_mass()
    }
}

impl Default for EnvState {
    fn default() -> Self {
        Self::new(288.0, 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_mass_sums_builtin_and_extended_gases() {
        let mut reg = GasRegistry::with_defaults();
        let ar = reg
            .register(GasDef {
                symbol: "Ar".into(),
                name: "Argon".into(),
                molar_mass_kg: 0.039_948,
                greenhouse_factor: 0.0,
                uv_absorption: 0.0,
                relative_diffusivity: 0.9,
            })
            .unwrap();
        let he = reg
            .register(GasDef {
                symbol: "He".into(),
                name: "Helium".into(),
                molar_mass_kg: 0.004_002_6,
                greenhouse_factor: 0.0,
                uv_absorption: 0.0,
                relative_diffusivity: 2.5,
            })
            .unwrap();
        let xe = reg
            .register(GasDef {
                symbol: "Xe".into(),
                name: "Xenon".into(),
                molar_mass_kg: 0.131_29,
                greenhouse_factor: 0.0,
                uv_absorption: 0.0,
                relative_diffusivity: 0.4,
            })
            .unwrap();

        let mut env = EnvState::with_gas_count(reg.len(), 288.0, 0.5);
        env.set_gas(GasId::O2, PositiveF64::new(2.0));
        env.set_gas(GasId::O3, PositiveF64::new(0.5));
        env.set_gas(ar, PositiveF64::new(1.0));
        env.set_gas(he, PositiveF64::new(0.25));
        env.set_gas(xe, PositiveF64::new(0.75));
        assert!((env.total_mass() - 4.5).abs() < 1e-12);
    }

    #[test]
    fn duplicate_gas_symbol_is_rejected() {
        let mut reg = GasRegistry::with_defaults();
        let err = reg.register(GasDef {
            symbol: "O2".into(),
            name: "Duplicate".into(),
            molar_mass_kg: 0.032,
            greenhouse_factor: 0.0,
            uv_absorption: 0.0,
            relative_diffusivity: 1.0,
        });
        assert!(err.is_err());
    }
}
