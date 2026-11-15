//! World state and double-buffered, chunk-parallel tick orchestration.

use rayon::prelude::*;

use crate::grid::coords::{MOORE, lat_lon_rad, resolve_neighbor};
use crate::grid::{DoubleBuffer, Grid};
use crate::model::{
    EnvState, GasRegistry, GlobalParams, OrbitalState, Population, PositiveF64, SpeciesId,
    TraitSchema, cell_insolation, orbital_state,
};
use crate::ports::{
    CellContext, EnvironmentStep, EvolutionEngine, Flux, MigrationPolicy, PopulationBehavior,
    Recorder, Rng,
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

/// The world: a chunk-ordered double-buffered grid of cells plus registries
/// for species, gases, and trait dimensions.
pub struct World {
    cells: DoubleBuffer<Cell>,
    pub(crate) registry: SpeciesRegistry,
    pub(crate) gases: GasRegistry,
    pub(crate) trait_schema: TraitSchema,
    params: GlobalParams,
}

impl World {
    /// Creates a world filled with one initial environment state and default
    /// gas and trait registries.
    #[must_use]
    pub fn new(width: u32, height: u32, initial: EnvState, params: GlobalParams) -> Self {
        Self::with_registries(
            width,
            height,
            initial,
            params,
            GasRegistry::with_defaults(),
            TraitSchema::with_defaults(),
        )
    }

    /// Creates a world with custom gas and trait registries, ensuring every
    /// cell's gas vector matches `gases.len()`.
    #[must_use]
    pub fn with_registries(
        width: u32,
        height: u32,
        mut initial: EnvState,
        params: GlobalParams,
        gases: GasRegistry,
        trait_schema: TraitSchema,
    ) -> Self {
        initial.gases.resize(gases.len());
        let cells = DoubleBuffer::filled(width as usize, height as usize, Cell::new(initial));
        Self {
            cells,
            registry: SpeciesRegistry::new(),
            gases,
            trait_schema,
            params,
        }
    }

    /// Width in cells.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.cells.front().width() as u32
    }

    /// Height in cells.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.cells.front().height() as u32
    }

    /// Global planetary and orbital parameters.
    #[must_use]
    pub const fn params(&self) -> &GlobalParams {
        &self.params
    }

    /// Registered atmospheric gases.
    #[must_use]
    pub const fn gases(&self) -> &GasRegistry {
        &self.gases
    }

    /// Registered species trait schema.
    #[must_use]
    pub const fn trait_schema(&self) -> &TraitSchema {
        &self.trait_schema
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

    /// Front-buffer grid view (read-only).
    #[must_use]
    pub const fn grid(&self) -> &Grid<Cell> {
        self.cells.front()
    }

    /// Mutable access to the front-buffer grid for setup between ticks.
    pub fn grid_mut(&mut self) -> &mut Grid<Cell> {
        self.cells.front_mut()
    }

    /// Shared reference to one cell.
    ///
    /// # Errors
    /// [`Error::OutOfBounds`] when outside the grid.
    pub fn cell(&self, x: u32, y: u32) -> Result<&Cell> {
        self.cells
            .front()
            .get(x as usize, y as usize)
            .ok_or(Error::OutOfBounds {
                x,
                y,
                width: self.width(),
                height: self.height(),
            })
    }

    /// Mutable reference to one cell in the front buffer.
    ///
    /// # Errors
    /// [`Error::OutOfBounds`] when outside the grid.
    pub fn cell_mut(&mut self, x: u32, y: u32) -> Result<&mut Cell> {
        let width = self.width();
        let height = self.height();
        self.cells
            .front_mut()
            .get_mut(x as usize, y as usize)
            .ok_or(Error::OutOfBounds {
                x,
                y,
                width,
                height,
            })
    }

    /// All cells in chunk-first storage order (front buffer).
    #[must_use]
    pub const fn cells(&self) -> &[Cell] {
        self.cells.front().as_slice()
    }

    /// Mutable access to all cells in chunk-first storage order (front buffer).
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        self.cells.front_mut().as_mut_slice()
    }
}

/// Drives the tick loop by calling injected phases in fixed order against a
/// double-buffered [`World`] with pre-allocated scratch buffers.
pub struct TickDriver {
    environment: Box<dyn EnvironmentStep>,
    behavior: Box<dyn PopulationBehavior>,
    migration: Box<dyn MigrationPolicy>,
    recorder: Box<dyn Recorder>,
    chunk_fluxes: Vec<Vec<Flux>>,
    extinct_scratch: Vec<SpeciesId>,
    variant_scratch: Vec<(SpeciesId, crate::model::Species)>,
    evolution: Option<Box<dyn EvolutionEngine>>,
    rng: Option<Box<dyn Rng>>,
    parallel: bool,
}

impl TickDriver {
    /// Assembles a driver from port implementations (parallel execution enabled by default).
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
            chunk_fluxes: Vec::new(),
            extinct_scratch: Vec::new(),
            variant_scratch: Vec::new(),
            evolution: None,
            rng: None,
            parallel: true,
        }
    }

    /// Enables or disables `rayon` chunk parallelism (deterministic in both modes).
    pub fn set_parallel(&mut self, parallel: bool) -> &mut Self {
        self.parallel = parallel;
        self
    }

    /// Builder-style toggle for `rayon` chunk parallelism.
    #[must_use]
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Installs an evolution engine and its deterministic rng source.
    ///
    /// Without an engine, no speciation or extinction pruning happens and
    /// species live for the whole run.
    pub fn with_evolution(
        &mut self,
        evolution: Box<dyn EvolutionEngine>,
        rng: Box<dyn Rng>,
    ) -> &mut Self {
        self.evolution = Some(evolution);
        self.rng = Some(rng);
        self
    }

    /// Runs one tick against `world`.
    ///
    /// Phase order: environment step, population behavior, migration flux
    /// collection and application, double-buffer swap, optional evolution
    /// pruning/spawning, and consumer-defined metric recording.
    ///
    /// All per-cell phases read the immutable `front` grid and write into the
    /// pre-allocated `back` grid or per-chunk flux buffers, avoiding per-tick
    /// grid cloning and keeping the hot loop allocation-free once warmed.
    pub fn tick(&mut self, world: &mut World, tick: u64) {
        let width = world.width();
        let height = world.height();
        if width == 0 || height == 0 {
            self.recorder.record(tick, world);
            return;
        }

        let orbital = orbital_state(tick, world.params());
        let (ccx, ccy) = world.cells.front().chunk_counts();
        let num_chunks = ccx * ccy;
        if self.chunk_fluxes.len() != num_chunks {
            self.chunk_fluxes.resize_with(num_chunks, Vec::new);
        }
        for buf in &mut self.chunk_fluxes {
            buf.clear();
        }

        let params = &world.params;
        let gases = &world.gases;
        let trait_schema = &world.trait_schema;
        let registry = &world.registry;
        let env_step = &*self.environment;
        let behavior = &*self.behavior;
        let migration = &*self.migration;

        let (front, back) = world.cells.split_mut();

        if self.parallel {
            let flux_slices = &mut self.chunk_fluxes;
            let (row_chunks, _) = (ccx, ccy);
            let _ = row_chunks;

            // Split back buffer into chunk rows and flux_slices into matching rows.
            let chunk_size = crate::grid::storage::DEFAULT_CHUNK_SIZE;
            let w_usize = width as usize;
            let h_usize = height as usize;

            back.as_mut_slice()
                .par_chunks_mut(chunk_size * w_usize)
                .zip(flux_slices.par_chunks_mut(ccx))
                .enumerate()
                .for_each(|(cy, (row_cells, row_fluxes))| {
                    let y0 = cy * chunk_size;
                    let chunk_h = (h_usize - y0).min(chunk_size);
                    row_cells
                        .par_chunks_mut(chunk_size * chunk_h)
                        .zip(row_fluxes.par_iter_mut())
                        .enumerate()
                        .for_each(|(cx, (chunk_cells, chunk_flux))| {
                            let x0 = cx * chunk_size;
                            let chunk_w = (w_usize - x0).min(chunk_size);
                            process_chunk(
                                x0,
                                y0,
                                chunk_w,
                                chunk_h,
                                width,
                                height,
                                tick,
                                &orbital,
                                front,
                                chunk_cells,
                                chunk_flux,
                                params,
                                gases,
                                trait_schema,
                                registry,
                                env_step,
                                behavior,
                                migration,
                            );
                        });
                });
        } else {
            let chunk_size = crate::grid::storage::DEFAULT_CHUNK_SIZE;
            let w_usize = width as usize;
            let h_usize = height as usize;
            for (cy, (row_cells, row_fluxes)) in back
                .as_mut_slice()
                .chunks_mut(chunk_size * w_usize)
                .zip(self.chunk_fluxes.chunks_mut(ccx))
                .enumerate()
            {
                let y0 = cy * chunk_size;
                let chunk_h = (h_usize - y0).min(chunk_size);
                for (cx, (chunk_cells, chunk_flux)) in row_cells
                    .chunks_mut(chunk_size * chunk_h)
                    .zip(row_fluxes.iter_mut())
                    .enumerate()
                {
                    let x0 = cx * chunk_size;
                    let chunk_w = (w_usize - x0).min(chunk_size);
                    process_chunk(
                        x0,
                        y0,
                        chunk_w,
                        chunk_h,
                        width,
                        height,
                        tick,
                        &orbital,
                        front,
                        chunk_cells,
                        chunk_flux,
                        params,
                        gases,
                        trait_schema,
                        registry,
                        env_step,
                        behavior,
                        migration,
                    );
                }
            }
        }

        // Apply migration fluxes in canonical chunk order (bit-identical across thread counts).
        let topology = world.params.topology;
        let back_mut = world.cells.back_mut();
        for chunk_buf in &self.chunk_fluxes {
            for flux in chunk_buf {
                let (tx, ty) = resolve_neighbor(
                    flux.from_x as usize,
                    flux.from_y as usize,
                    flux.direction.0,
                    flux.direction.1,
                    width as usize,
                    height as usize,
                    topology,
                );
                if tx == flux.from_x as usize && ty == flux.from_y as usize {
                    continue;
                }
                let moved = PositiveF64::new(flux.amount);
                if moved.value() <= 0.0 {
                    continue;
                }
                if let Some(source) = back_mut.get_mut(flux.from_x as usize, flux.from_y as usize)
                    && let Some(entry) = source
                        .populations
                        .iter_mut()
                        .find(|(pid, _)| pid.raw == flux.species.raw)
                {
                    entry.1.amount = entry.1.amount.saturating_sub(moved);
                }
                if let Some(target) = back_mut.get_mut(tx, ty) {
                    match target
                        .populations
                        .iter_mut()
                        .find(|(pid, _)| pid.raw == flux.species.raw)
                    {
                        Some(entry) => entry.1.amount = entry.1.amount.saturating_add(moved),
                        None => {
                            target
                                .populations
                                .push((flux.species, Population { amount: moved }));
                            target.populations.sort_by_key(|(id, _)| id.raw);
                        }
                    }
                }
            }
        }

        // Clean zeroed populations after migration and publish back -> front.
        for cell in back_mut.as_mut_slice() {
            cell.populations.retain(|(_, p)| p.amount.value() > 0.0);
        }
        world.cells.swap();

        // Evolution: prune extinct species, spawn variants against the new front state.
        if let (Some(evolution), Some(rng)) = (&mut self.evolution, self.rng.as_deref_mut()) {
            self.extinct_scratch.clear();
            evolution.prune_extinct(world, &mut self.extinct_scratch);
            self.variant_scratch.clear();
            evolution.spawn_variants(tick, world, rng, &mut self.variant_scratch);
            for id in self.extinct_scratch.drain(..) {
                world.registry.remove(id);
            }
            for (_parent, species) in self.variant_scratch.drain(..) {
                let _ = world.registry.insert(species);
            }
        }

        // Phase 4: record metrics (consumer-defined).
        self.recorder.record(tick, world);
    }
}

#[allow(clippy::too_many_arguments)]
fn process_chunk(
    x0: usize,
    y0: usize,
    chunk_w: usize,
    chunk_h: usize,
    width: u32,
    height: u32,
    tick: u64,
    orbital: &OrbitalState,
    front: &Grid<Cell>,
    back_chunk: &mut [Cell],
    flux_out: &mut Vec<Flux>,
    params: &GlobalParams,
    gases: &GasRegistry,
    trait_schema: &TraitSchema,
    registry: &SpeciesRegistry,
    env_step: &dyn EnvironmentStep,
    behavior: &dyn PopulationBehavior,
    migration: &dyn MigrationPolicy,
) {
    let w_usize = width as usize;
    let h_usize = height as usize;
    let topology = params.topology;

    for ry in 0..chunk_h {
        let y = (y0 + ry) as u32;
        let (lat, _) = lat_lon_rad(0, y, width, height);
        for rx in 0..chunk_w {
            let x = (x0 + rx) as u32;
            let front_cell = front
                .get(x as usize, y as usize)
                .expect("chunk bounds lie within grid");
            let back_cell = &mut back_chunk[ry * chunk_w + rx];

            let (_, lon) = lat_lon_rad(x, y, width, height);
            let insolation = cell_insolation(lat, lon, orbital, params);
            let neighbors =
                resolve_neighbors(front, x as usize, y as usize, w_usize, h_usize, topology);

            let ctx = CellContext {
                x,
                y,
                width,
                height,
                tick,
                latitude_rad: lat,
                longitude_rad: lon,
                insolation,
                env: &front_cell.env,
                neighbors,
                populations: &front_cell.populations,
                params,
                gases,
                trait_schema,
                registry,
            };

            // Phase 1: environment step into back buffer (reuses existing allocation).
            env_step.step(&ctx, &mut back_cell.env);

            // Phase 2: population behavior deltas applied into back buffer.
            back_cell.populations.clear();
            back_cell
                .populations
                .extend_from_slice(&front_cell.populations);
            for (idx, (id, _)) in front_cell.populations.iter().enumerate() {
                let delta = behavior.update(*id, &ctx).delta;
                let current = back_cell.populations[idx].1.amount.value();
                back_cell.populations[idx].1.amount = PositiveF64::new(current + delta);
            }
            back_cell
                .populations
                .retain(|(_, p)| p.amount.value() > 0.0);

            // Phase 3: emit migration fluxes from frozen snapshot.
            migration.fluxes(&ctx, flux_out);
        }
    }
}

#[inline]
fn resolve_neighbors(
    grid: &Grid<Cell>,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    topology: crate::grid::coords::Topology,
) -> [&EnvState; 8] {
    MOORE.map(|(dx, dy)| {
        let (nx, ny) = resolve_neighbor(x, y, dx, dy, width, height, topology);
        &grid
            .get(nx, ny)
            .expect("resolve_neighbor guarantees in-bounds coordinates")
            .env
    })
}
