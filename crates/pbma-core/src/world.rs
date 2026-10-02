//! World state and tick orchestration.

use pbma_model::{EnvState, GlobalParams, Population, SpeciesId};

use crate::ports::{
    BehaviorDelta, CellContext, EnvironmentStep, MigrationPolicy, PopulationBehavior, Recorder,
};
use crate::registry::SpeciesRegistry;
use crate::{Error, Result};

/// One cell of the world: environment plus the populations present.
#[derive(Debug, Clone)]
pub struct Cell {
    /// Environment state of this cell.
    pub env: EnvState,
    /// Populations present, ascending by species id.
    pub populations: Vec<(SpeciesId, Population)>,
}

impl Cell {
    /// Empty cell with the given environment.
    #[must_use]
    pub fn new(env: EnvState) -> Self {
        Self {
            env,
            populations: Vec::new(),
        }
    }
}

/// The world: a rectangular grid of cells plus the species registry.
pub struct World {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    pub(crate) registry: SpeciesRegistry,
    params: GlobalParams,
}

impl World {
    /// Creates a world filled with one initial environment state.
    #[must_use]
    pub fn new(width: u32, height: u32, initial: EnvState, params: GlobalParams) -> Self {
        let cells = vec![Cell::new(initial); (width as usize) * (height as usize)];
        Self {
            width,
            height,
            cells,
            registry: SpeciesRegistry::new(),
            params,
        }
    }

    /// Width in cells.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Height in cells.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Global parameters.
    #[must_use]
    pub const fn params(&self) -> &GlobalParams {
        &self.params
    }

    /// Species registry.
    #[must_use]
    pub const fn registry(&self) -> &SpeciesRegistry {
        &self.registry
    }

    /// Mutable species registry.
    pub fn registry_mut(&mut self) -> &mut SpeciesRegistry {
        &mut self.registry
    }

    /// Resolves a coordinate to a cell index.
    ///
    /// # Errors
    /// [`Error::OutOfBounds`] when outside the grid.
    pub fn cell_index(&self, x: u32, y: u32) -> Result<usize> {
        if x >= self.width || y >= self.height {
            return Err(Error::OutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }
        Ok((y as usize) * (self.width as usize) + (x as usize))
    }

    /// Shared reference to one cell.
    ///
    /// # Errors
    /// [`Error::OutOfBounds`] when outside the grid.
    pub fn cell(&self, x: u32, y: u32) -> Result<&Cell> {
        self.cells
            .get(self.cell_index(x, y)?)
            .ok_or(Error::OutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            })
    }

    /// Mutable reference to one cell.
    ///
    /// # Errors
    /// [`Error::OutOfBounds`] when outside the grid.
    pub fn cell_mut(&mut self, x: u32, y: u32) -> Result<&mut Cell> {
        let idx = self.cell_index(x, y)?;
        self.cells.get_mut(idx).ok_or(Error::OutOfBounds {
            x,
            y,
            width: self.width,
            height: self.height,
        })
    }

    /// All cells in row-major order.
    #[must_use]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Mutable access to all cells in row-major order.
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }
}

/// Drives the tick loop by calling the injected phases in fixed order.
pub struct TickDriver {
    environment: Box<dyn EnvironmentStep>,
    behavior: Box<dyn PopulationBehavior>,
    migration: Box<dyn MigrationPolicy>,
    recorder: Box<dyn Recorder>,
    flux_scratch: Vec<crate::ports::Flux>,
    evolution: Option<Box<dyn crate::ports::EvolutionEngine>>,
    rng: Option<Box<dyn crate::ports::Rng>>,
}

impl TickDriver {
    /// Assembles a driver from port implementations.
    #[must_use]
    pub fn new(
        environment: Box<dyn EnvironmentStep>,
        behavior: Box<dyn PopulationBehavior>,
        migration: Box<dyn MigrationPolicy>,
        recorder: Box<dyn Recorder>,
    ) -> Self {
        Self {
            environment,
            behavior,
            migration,
            recorder,
            flux_scratch: Vec::new(),
            evolution: None,
            rng: None,
        }
    }

    /// Installs an evolution engine and its deterministic rng source.
    ///
    /// Without an engine, no speciation or extinction pruning happens and
    /// species live for the whole run.
    pub fn with_evolution(
        &mut self,
        evolution: Box<dyn crate::ports::EvolutionEngine>,
        rng: Box<dyn crate::ports::Rng>,
    ) -> &mut Self {
        self.evolution = Some(evolution);
        self.rng = Some(rng);
        self
    }

    /// Runs one tick against `world`.
    ///
    /// Phase order: environment step, population behavior, migration apply,
    /// recorder. All phases read the frozen previous snapshot; writes land
    /// in fresh buffers first (determinism rule 1). If an evolution engine
    /// is installed, variant species spawn and extinct species are pruned
    /// after the migration phase.
    pub fn tick(&mut self, world: &mut World, tick: u64) {
        let width = world.width();
        let height = world.height();

        // Frozen snapshot of the previous tick, shared by all phases.
        let snapshot: Vec<Cell> = world.cells().to_vec();

        // Phase 1: environment step into fresh buffer.
        let next_env: Vec<EnvState> = snapshot
            .iter()
            .enumerate()
            .map(|(idx, cell)| {
                let x = (idx as u32) % width;
                let y = (idx as u32) / width;
                let neighbors = self.neighbor_envs(&snapshot, x, y, width, height);
                let ctx = CellContext {
                    x,
                    y,
                    env: &cell.env,
                    neighbors,
                    populations: &cell.populations,
                    params: world.params(),
                    registry: &world.registry,
                };
                self.environment.step(&ctx)
            })
            .collect();

        // Phase 2: population behavior deltas per cell.
        let deltas: Vec<Vec<(SpeciesId, BehaviorDelta)>> = snapshot
            .iter()
            .enumerate()
            .map(|(idx, cell)| {
                let x = (idx as u32) % width;
                let y = (idx as u32) / width;
                let neighbors = self.neighbor_envs(&snapshot, x, y, width, height);
                let ctx = CellContext {
                    x,
                    y,
                    env: &cell.env,
                    neighbors,
                    populations: &cell.populations,
                    params: world.params(),
                    registry: &world.registry,
                };
                cell.populations
                    .iter()
                    .map(|(id, _)| {
                        let delta = self.behavior.update(*id, &ctx).delta;
                        (*id, BehaviorDelta { delta })
                    })
                    .collect()
            })
            .collect();

        // Phase 3: migration fluxes.
        self.flux_scratch.clear();
        for (idx, cell) in snapshot.iter().enumerate() {
            let x = (idx as u32) % width;
            let y = (idx as u32) / width;
            let neighbors = self.neighbor_envs(&snapshot, x, y, width, height);
            let ctx = CellContext {
                x,
                y,
                env: &cell.env,
                neighbors,
                populations: &cell.populations,
                params: world.params(),
                registry: &world.registry,
            };
            self.migration.fluxes(&ctx, &mut self.flux_scratch);
        }

        // Apply behavior deltas.
        for (idx, cell_deltas) in deltas.iter().enumerate() {
            let cell = &mut world.cells[idx];
            for (id, delta) in cell_deltas {
                if let Some(entry) = cell
                    .populations
                    .iter_mut()
                    .find(|(pid, _)| pid.raw == id.raw)
                {
                    let new_value = entry.1.amount.value() + delta.delta;
                    entry.1.amount = pbma_model::Amount::new(new_value);
                }
            }
            cell.populations.retain(|(_, p)| p.amount.value() > 0.0);
        }

        // Apply migration fluxes after all computation.
        let fluxes: Vec<crate::ports::Flux> = std::mem::take(&mut self.flux_scratch);
        for flux in fluxes {
            let target_x = flux.from_x as i64 + flux.direction.0 as i64;
            let target_y = flux.from_y as i64 + flux.direction.1 as i64;
            if target_x < 0 || target_y < 0 {
                continue;
            }
            let (target_x, target_y) = (target_x as u32, target_y as u32);
            let moved = pbma_model::Amount::new(flux.amount);
            if let Ok(source) = world.cell_mut(flux.from_x, flux.from_y)
                && let Some(entry) = source
                    .populations
                    .iter_mut()
                    .find(|(pid, _)| pid.raw == flux.species.raw)
            {
                entry.1.amount = entry.1.amount.saturating_sub(moved);
            }
            if target_x >= width || target_y >= height {
                continue;
            }
            if let Ok(target) = world.cell_mut(target_x, target_y) {
                match target
                    .populations
                    .iter_mut()
                    .find(|(pid, _)| pid.raw == flux.species.raw)
                {
                    Some(entry) => entry.1.amount = entry.1.amount.saturating_add(moved),
                    None => target
                        .populations
                        .push((flux.species, Population { amount: moved })),
                }
            }
        }

        // Sort populations by species id to restore the determinism invariant.
        for cell in &mut world.cells {
            cell.populations.sort_by_key(|(id, _)| id.raw);
        }

        // Publish the environment buffer.
        for (cell, env) in world.cells.iter_mut().zip(next_env) {
            cell.env = env;
        }

        // Evolution: prune extinct species, spawn variants. Order fixed for
        // determinism; both steps see the post-migration world.
        if let (Some(evolution), Some(rng)) = (&mut self.evolution, self.rng.as_deref_mut()) {
            let mut extinct = Vec::new();
            evolution.prune_extinct(world, &mut extinct);
            let mut variants = Vec::new();
            evolution.spawn_variants(world, rng, &mut variants);
            for id in extinct {
                world.registry.remove(id);
            }
            for (parent, species) in variants {
                // registry insert errors (duplicate name) are skipped:
                // engine contract says names are unique per spawn batch.
                let _ = world.registry.insert(species).map(|id| (parent, id));
            }
        }

        // Phase 4: record metrics.
        let total_gas: f64 = world.cells.iter().map(|c| c.env.total_mass()).sum();
        let total_population: f64 = world
            .cells
            .iter()
            .flat_map(|c| c.populations.iter())
            .map(|(_, p)| p.amount.value())
            .sum();
        self.recorder.record(tick, "total_gas", total_gas);
        self.recorder
            .record(tick, "total_population", total_population);
    }

    /// Neighbor environments in fixed order: north, east, south, west.
    /// Edge neighbors fall back to the cell's own environment (clamped
    /// boundary), keeping the array shape fixed for deterministic reads.
    fn neighbor_envs<'a>(
        &self,
        cells: &'a [Cell],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> [&'a EnvState; 4] {
        let idx = |x: u32, y: u32| -> Option<usize> {
            if x >= width || y >= height {
                None
            } else {
                Some((y as usize) * (width as usize) + (x as usize))
            }
        };
        let own = &cells[(y as usize) * (width as usize) + (x as usize)].env;
        let pick = |dx: i32, dy: i32| -> Option<&'a EnvState> {
            let nx = x as i64 + dx as i64;
            let ny = y as i64 + dy as i64;
            if nx < 0 || ny < 0 {
                None
            } else {
                idx(nx as u32, ny as u32)
                    .and_then(|i| cells.get(i))
                    .map(|c| &c.env)
            }
        };
        [pick(0, -1), pick(1, 0), pick(0, 1), pick(-1, 0)].map(|opt| opt.unwrap_or(own))
    }
}
