//! Fluent scenario builder for constructing validated [`World`] instances
//! in tests, experiments, and CLI/GUI runners.

use crate::grid::coords::{Topology, lat_lon_rad};
use crate::model::{
    EnvState, GasDef, GasId, GasRegistry, GlobalParams, Population, PositiveF64, Species,
    SpeciesId, TraitDef, TraitId, TraitSchema,
};
use crate::{Error, Result, World};

type EnvInitFn = Box<dyn Fn(u32, u32, u32, u32, &GlobalParams, &GasRegistry, &mut EnvState)>;
type SpeciesSeedFn = Box<dyn Fn(u32, u32, u32, u32) -> f64>;

/// Fluent builder for constructing a configured [`World`] without manual
/// grid loops.
pub struct WorldBuilder {
    width: u32,
    height: u32,
    params: GlobalParams,
    gases: GasRegistry,
    trait_schema: TraitSchema,
    base_env: EnvState,
    env_initializers: Vec<EnvInitFn>,
    species_seeds: Vec<(Species, SpeciesSeedFn)>,
    pending_error: Option<Error>,
}

impl WorldBuilder {
    /// Starts building a `width x height` world with Earth-like defaults.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let gases = GasRegistry::with_defaults();
        let base_env = EnvState::with_gas_count(gases.len(), 288.0, 0.5);
        Self {
            width,
            height,
            params: GlobalParams::earth_like(),
            gases,
            trait_schema: TraitSchema::with_defaults(),
            base_env,
            env_initializers: Vec::new(),
            species_seeds: Vec::new(),
            pending_error: None,
        }
    }

    /// Replaces the full [`GlobalParams`].
    #[must_use]
    pub fn params(mut self, params: GlobalParams) -> Self {
        self.params = params;
        self
    }

    /// Configures planetary mass (kg) and diameter (m), which derive gravity and area.
    #[must_use]
    pub fn planet(mut self, mass_kg: f64, diameter_m: f64) -> Self {
        self.params.planet.mass_kg = mass_kg;
        self.params.planet.diameter_m = diameter_m;
        self
    }

    /// Configures orbital distance (AU), eccentricity, year length (ticks), and axial tilt (rad).
    #[must_use]
    pub fn orbit(
        mut self,
        distance_au: f64,
        eccentricity: f64,
        year_length_ticks: u32,
        axial_tilt_rad: f64,
    ) -> Self {
        self.params.orbit.distance_au = distance_au;
        self.params.orbit.eccentricity = eccentricity;
        self.params.orbit.year_length_ticks = year_length_ticks;
        self.params.orbit.axial_tilt_rad = axial_tilt_rad;
        self
    }

    /// Sets planetary day length in ticks (`0` for diurnal-mean insolation).
    #[must_use]
    pub fn day_length(mut self, day_length_ticks: u32) -> Self {
        self.params.orbit.day_length_ticks = day_length_ticks;
        self
    }

    /// Sets boundary topology (`Clamped`, `Planetary`, or `Toroidal`).
    #[must_use]
    pub fn topology(mut self, topology: Topology) -> Self {
        self.params.topology = topology;
        self
    }

    /// Sets diffusion coefficient.
    #[must_use]
    pub fn diffusion(mut self, diffusion_coefficient: f64) -> Self {
        self.params.diffusion_coefficient = diffusion_coefficient;
        self
    }

    /// Registers an additional gas species in the world's [`GasRegistry`].
    #[must_use]
    pub fn register_gas(mut self, def: GasDef, out_id: Option<&mut GasId>) -> Self {
        if self.pending_error.is_none() {
            match self.gases.register(def) {
                Ok(id) => {
                    self.base_env.gases.resize(self.gases.len());
                    if let Some(slot) = out_id {
                        *slot = id;
                    }
                }
                Err(e) => self.pending_error = Some(e),
            }
        }
        self
    }

    /// Registers an additional species trait dimension in the world's [`TraitSchema`].
    #[must_use]
    pub fn register_trait(mut self, def: TraitDef, out_id: Option<&mut TraitId>) -> Self {
        if self.pending_error.is_none() {
            match self.trait_schema.register(def) {
                Ok(id) => {
                    if let Some(slot) = out_id {
                        *slot = id;
                    }
                }
                Err(e) => self.pending_error = Some(e),
            }
        }
        self
    }

    /// Sets the baseline uniform [`EnvState`] applied before initializers run.
    #[must_use]
    pub fn base_env(mut self, env: EnvState) -> Self {
        self.base_env = env;
        self
    }

    /// Sets a uniform initial concentration for a single gas on the base environment.
    #[must_use]
    pub fn gas(mut self, id: GasId, amount: f64) -> Self {
        self.base_env.set_gas(id, PositiveF64::new(amount));
        self
    }

    /// Sets the baseline temperature (K) and light (`[0, 1]`).
    #[must_use]
    pub fn temperature_and_light(mut self, temperature_k: f64, light: f64) -> Self {
        self.base_env.temperature = temperature_k;
        self.base_env.light = light;
        self
    }

    /// Applies a smooth latitudinal temperature gradient
    /// $T(\phi) = T_{\mathrm{pole}} + (T_{\mathrm{eq}} - T_{\mathrm{pole}})\cos^2\phi$.
    #[must_use]
    pub fn latitudinal_temperature(mut self, equator_k: f64, pole_k: f64) -> Self {
        self.env_initializers
            .push(Box::new(move |_x, y, width, height, _p, _g, env| {
                let (lat, _) = lat_lon_rad(0, y, width, height);
                let cos_lat = lat.cos();
                env.temperature = pole_k + (equator_k - pole_k) * cos_lat * cos_lat;
            }));
        self
    }

    /// Registers a custom per-cell environment initializer executed in row-major order.
    #[must_use]
    pub fn init_env(
        mut self,
        f: impl Fn(u32, u32, u32, u32, &GlobalParams, &GasRegistry, &mut EnvState) + 'static,
    ) -> Self {
        self.env_initializers.push(Box::new(f));
        self
    }

    /// Registers a species and seeds its initial spatial population across cells.
    #[must_use]
    pub fn seed_species(
        mut self,
        species: Species,
        density: impl Fn(u32, u32, u32, u32) -> f64 + 'static,
    ) -> Self {
        self.species_seeds.push((species, Box::new(density)));
        self
    }

    /// Registers a species seeded uniformly in every cell.
    #[must_use]
    pub fn seed_species_everywhere(self, species: Species, amount: f64) -> Self {
        self.seed_species(species, move |_, _, _, _| amount)
    }

    /// Registers a species seeded in a single cell `(tx, ty)`.
    #[must_use]
    pub fn seed_species_at(self, species: Species, tx: u32, ty: u32, amount: f64) -> Self {
        self.seed_species(
            species,
            move |x, y, _, _| {
                if x == tx && y == ty { amount } else { 0.0 }
            },
        )
    }

    /// Validates parameters, constructs the [`World`], runs environment
    /// initializers, and seeds all species.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDimensions`], [`Error::InvalidParams`],
    /// [`Error::DuplicateGas`], [`Error::DuplicateTrait`], or
    /// [`Error::DuplicateSpecies`].
    pub fn build(self) -> Result<World> {
        if let Some(err) = self.pending_error {
            return Err(err);
        }
        if self.width == 0 || self.height == 0 {
            return Err(Error::InvalidDimensions {
                width: self.width,
                height: self.height,
            });
        }
        self.params.validate()?;

        let mut world = World::with_registries(
            self.width,
            self.height,
            self.base_env,
            self.params,
            self.gases,
            self.trait_schema,
        );

        let w = self.width;
        let h = self.height;
        // Clone params/gases once to satisfy borrow checker while mutating cells.
        let params_copy = *world.params();
        let gases_copy = world.gases().clone();

        for y in 0..h {
            for x in 0..w {
                if let Some(cell) = world.grid_mut().get_mut(x as usize, y as usize) {
                    for init in &self.env_initializers {
                        init(x, y, w, h, &params_copy, &gases_copy, &mut cell.env);
                    }
                }
            }
        }

        let mut registered: Vec<(SpeciesId, SpeciesSeedFn)> =
            Vec::with_capacity(self.species_seeds.len());
        for (species, seed_fn) in self.species_seeds {
            let id = world.registry_mut().insert(species)?;
            registered.push((id, seed_fn));
        }

        for y in 0..h {
            for x in 0..w {
                if let Some(cell) = world.grid_mut().get_mut(x as usize, y as usize) {
                    for (id, seed_fn) in &registered {
                        let amount = seed_fn(x, y, w, h);
                        if amount > 0.0 {
                            cell.populations.push((
                                *id,
                                Population {
                                    amount: PositiveF64::new(amount),
                                },
                            ));
                        }
                    }
                    cell.populations.sort_by_key(|(id, _)| id.raw);
                }
            }
        }

        Ok(world)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SpeciesTraits;

    #[test]
    fn builder_constructs_planetary_world_with_gradient_and_species() {
        let mut custom_gas = GasId(0);
        let world = WorldBuilder::new(8, 5)
            .planet(5.9722e24, 1.2742e7)
            .topology(Topology::Planetary)
            .latitudinal_temperature(305.0, 255.0)
            .register_gas(
                GasDef {
                    symbol: "SO2".into(),
                    name: "Sulfur Dioxide".into(),
                    molar_mass_kg: 0.064_066,
                    greenhouse_factor: 0.2,
                    uv_absorption: 0.5,
                    relative_diffusivity: 0.7,
                },
                Some(&mut custom_gas),
            )
            .gas(GasId::O2, 21.0)
            .seed_species_everywhere(Species::new("cyano", SpeciesTraits::default()), 10.0)
            .build()
            .unwrap();

        assert_eq!(world.width(), 8);
        assert_eq!(world.height(), 5);
        assert_eq!(world.gases().len(), 7);
        assert_eq!(custom_gas, GasId(6));
        let eq_temp = world.cell(0, 2).unwrap().env.temperature;
        let pole_temp = world.cell(0, 0).unwrap().env.temperature;
        assert!(eq_temp > pole_temp + 30.0);
        assert!((world.cell(3, 3).unwrap().env.gas(GasId::O2).value() - 21.0).abs() < 1e-12);
        assert_eq!(world.cell(0, 0).unwrap().populations.len(), 1);
    }

    #[test]
    fn builder_rejects_zero_dimensions() {
        assert!(WorldBuilder::new(0, 4).build().is_err());
    }
}
