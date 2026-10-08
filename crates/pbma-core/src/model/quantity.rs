//! Non-negative amounts with saturating arithmetic.

use serde::{Deserialize, Serialize};

/// A non-negative quantity (population count, gas amount).
///
/// Named for what it *is* rather than what it carries: an `f64` restricted
/// to non-negative values, with saturating semantics so values never become
/// negative or infinite through the provided operations. Stored as `f64`
/// because diffusion and population dynamics are continuous models.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PositiveF64(f64);

impl PositiveF64 {
    /// The zero amount.
    pub const ZERO: PositiveF64 = PositiveF64(0.0);

    /// Creates an amount. Negative input is clamped to zero.
    #[must_use]
    pub const fn new(value: f64) -> Self {
        PositiveF64(if value < 0.0 { 0.0 } else { value })
    }

    /// Raw value, guaranteed non-negative and finite.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Adds another amount, saturating at [`f64::MAX`].
    #[must_use]
    pub const fn saturating_add(self, other: PositiveF64) -> PositiveF64 {
        let sum = self.0 + other.0;
        if sum < 0.0 || sum.is_infinite() {
            PositiveF64(f64::MAX)
        } else {
            PositiveF64(sum)
        }
    }

    /// Subtracts another amount, clamping at zero.
    #[must_use]
    pub const fn saturating_sub(self, other: PositiveF64) -> PositiveF64 {
        let diff = self.0 - other.0;
        PositiveF64(if diff < 0.0 { 0.0 } else { diff })
    }

    /// Scales by a non-negative factor; negative factors clamp the result to zero.
    #[must_use]
    pub const fn scaled_by(self, factor: f64) -> PositiveF64 {
        let scaled = self.0 * factor;
        PositiveF64(if scaled < 0.0 || scaled.is_infinite() {
            if scaled.is_infinite() && scaled > 0.0 {
                f64::MAX
            } else {
                0.0
            }
        } else {
            scaled
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_input_clamped() {
        assert_eq!(PositiveF64::new(-5.0), PositiveF64::ZERO);
    }

    #[test]
    fn subtract_clamps_at_zero() {
        let a = PositiveF64::new(1.0);
        assert_eq!(a.saturating_sub(PositiveF64::new(2.0)), PositiveF64::ZERO);
    }

    #[test]
    fn add_saturates_at_max() {
        let a = PositiveF64::new(f64::MAX);
        assert_eq!(a.saturating_add(PositiveF64::new(1.0)).value(), f64::MAX);
    }

    #[test]
    fn negative_factor_clamps_to_zero() {
        assert_eq!(PositiveF64::new(3.0).scaled_by(-1.0), PositiveF64::ZERO);
    }
}
