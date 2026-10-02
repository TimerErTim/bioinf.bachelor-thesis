//! Deterministic seedable random number generator.

use pbma_core::ports::Rng;

/// Golden-ratio additive step constant (the splitmix64 stream increment).
const GOLDEN_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// Splitmix64 finalizer multiplication constant (first round).
const MIX_A: u64 = 0xBF58_476D_1CE4_E5B9;

/// Splitmix64 finalizer multiplication constant (second round).
const MIX_B: u64 = 0x94D0_49BB_1331_11EB;

/// Deterministic, seedable RNG adapter implementing the kernel's [`Rng`] port.
///
/// Uses the splitmix64 construction: a Weyl sequence advanced by the
/// golden-ratio constant, passed through the splitmix64 finalizer before
/// every draw. The full 64-bit state makes distinct seeds produce
/// independent-looking streams; child streams are derived deterministically
/// from the parent state and a salt so runs stay reproducible.
#[derive(Debug, Clone, Copy)]
pub struct SplitMixRng {
    /// Weyl sequence state, advanced by the golden-ratio constant per draw.
    state: u64,
}

impl SplitMixRng {
    /// Creates a generator seeded with `seed`.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl Rng for SplitMixRng {
    fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_add(GOLDEN_GAMMA);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(MIX_A);
        z = (z ^ (z >> 27)).wrapping_mul(MIX_B);
        z ^= z >> 31;
        ((z >> 11) as f64) * (1.0 / (1u64 << 53) as f64)
    }

    fn child(&self, salt: u64) -> Box<dyn Rng> {
        Box::new(SplitMixRng::new(
            self.state.wrapping_add(salt).wrapping_mul(GOLDEN_GAMMA),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const N: usize = 10_000;

    #[test]
    fn same_seed_is_deterministic() {
        let mut a = SplitMixRng::new(42);
        let mut b = SplitMixRng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_f64(), b.next_f64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = SplitMixRng::new(1);
        let mut b = SplitMixRng::new(2);
        for _ in 0..100 {
            assert_ne!(a.next_f64(), b.next_f64());
        }
    }

    #[test]
    fn draws_stay_in_unit_interval() {
        let mut rng = SplitMixRng::new(7);
        for _ in 0..N {
            let value = rng.next_f64();
            assert!((0.0..1.0).contains(&value), "value out of [0, 1): {value}");
        }
    }

    #[test]
    fn child_streams_differ_by_salt() {
        let parent = SplitMixRng::new(42);
        let mut first = parent.child(1);
        let mut second = parent.child(2);
        for _ in 0..100 {
            assert_ne!(first.next_f64(), second.next_f64());
        }
    }

    #[test]
    fn child_stream_differs_from_parent() {
        let mut parent = SplitMixRng::new(42);
        let mut child = parent.child(1);
        for _ in 0..100 {
            assert_ne!(parent.next_f64(), child.next_f64());
        }
    }

    #[test]
    fn child_stream_is_deterministic() {
        let parent_a = SplitMixRng::new(42);
        let parent_b = SplitMixRng::new(42);
        let mut child_a = parent_a.child(3);
        let mut child_b = parent_b.child(3);
        for _ in 0..100 {
            assert_eq!(child_a.next_f64(), child_b.next_f64());
        }
    }

    #[test]
    fn mean_is_near_half() {
        let mut rng = SplitMixRng::new(0xDEAD_BEEF);
        let sum: f64 = (0..N).map(|_| rng.next_f64()).sum();
        let mean = sum / N as f64;
        assert!(
            (0.45..=0.55).contains(&mean),
            "mean {mean} outside [0.45, 0.55]"
        );
    }
}
