//! In-memory metrics recorder.

use pbma_core::World;
use pbma_core::ports::Recorder;

/// Recorder collecting `(tick, metric, value)` rows for later export.
///
/// This default adapter extracts two world aggregates per tick — total gas
/// mass and total population — via [`Recorder::record`]; custom rows can be
/// pushed through [`VecRecorder::push`]. Downstream consumers wanting other
/// metrics implement [`Recorder`] themselves against the full [`World`].
#[derive(Debug, Default)]
pub struct VecRecorder {
    rows: Vec<(u64, String, f64)>,
}

impl VecRecorder {
    /// Appends one custom metric row.
    pub fn push(&mut self, tick: u64, metric: &str, value: f64) {
        self.rows.push((tick, metric.to_owned(), value));
    }

    /// Drains all recorded rows in insertion order.
    pub fn drain(&mut self) -> std::vec::Drain<'_, (u64, String, f64)> {
        self.rows.drain(..)
    }

    /// Borrowed view of all recorded rows in insertion order.
    #[must_use]
    pub fn snapshot(&self) -> &[(u64, String, f64)] {
        &self.rows
    }
}

impl Recorder for VecRecorder {
    fn record(&mut self, tick: u64, world: &World) {
        let total_gas: f64 = world.cells().iter().map(|c| c.env.total_mass()).sum();
        let total_population: f64 = world
            .cells()
            .iter()
            .flat_map(|c| c.populations.iter())
            .map(|(_, p)| p.amount.value())
            .sum();
        self.push(tick, "total_gas", total_gas);
        self.push(tick, "total_population", total_population);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_drains() {
        let mut r = VecRecorder::default();
        r.push(0, "total_gas", 10.0);
        r.push(1, "total_gas", 9.0);
        let drained: Vec<_> = r.drain().collect();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].0, 0);
        assert!(r.drain().next().is_none());
    }
}
