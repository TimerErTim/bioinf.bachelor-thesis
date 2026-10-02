//! In-memory metrics recorder.

use pbma_core::ports::Recorder;

/// Recorder collecting (tick, metric, value) rows for later export.
#[derive(Debug, Default)]
pub struct VecRecorder {
    rows: Vec<(u64, String, f64)>,
}

impl VecRecorder {
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
    fn record(&mut self, tick: u64, metric: &str, value: f64) {
        self.rows.push((tick, metric.to_owned(), value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_drains() {
        let mut r = VecRecorder::default();
        r.record(0, "total_gas", 10.0);
        r.record(1, "total_gas", 9.0);
        let drained: Vec<_> = r.drain().collect();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].0, 0);
        assert!(r.drain().next().is_none());
    }
}
