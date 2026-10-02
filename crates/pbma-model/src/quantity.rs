//! Non-negative amounts with saturating arithmetic.

use serde::{Deserialize, Serialize};

/// A non-negative quantity (population count, gas amount).
///
/// Stored as `f64` because diffusion and population dynamics are continuous
/// models. Saturation semantics: values never become negative or infinite
/// through the provided operations.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Amount(f64);

impl Amount {
    /// The zero amount.
    pub const ZERO: Amount = Amount(0.0);

    /// Creates an amount. Negative input is clamped to zero.
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Amount(if value < 0.0 { 0.0 } else { value })
    }

    /// Raw value, guaranteed non-negative and finite.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Adds another amount, saturating at [`f64::MAX`].
    #[must_use]
    pub const fn saturating_add(self, other: Amount) -> Amount {
        let sum = self.0 + other.0;
        if sum < 0.0 || sum.is_infinite() {
            Amount(f64::MAX)
        } else {
            Amount(sum)
        }
    }

    /// Subtracts another amount, clamping at zero.
    #[must_use]
    pub const fn saturating_sub(self, other: Amount) -> Amount {
        let diff = self.0 - other.0;
        Amount(if diff < 0.0 { 0.0 } else { diff })
    }

    /// Scales by a non-negative factor; negative factors clamp the result to zero.
    #[must_use]
    pub const fn scaled_by(self, factor: f64) -> Amount {
        let scaled = self.0 * factor;
        Amount(if scaled < 0.0 || scaled.is_infinite() {
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
        assert_eq!(Amount::new(-5.0), Amount::ZERO);
    }

    #[test]
    fn subtract_clamps_at_zero() {
        let a = Amount::new(1.0);
        assert_eq!(a.saturating_sub(Amount::new(2.0)), Amount::ZERO);
    }

    #[test]
    fn add_saturates_at_max() {
        let a = Amount::new(f64::MAX);
        assert_eq!(a.saturating_add(Amount::new(1.0)).value(), f64::MAX);
    }

    #[test]
    fn negative_factor_clamps_to_zero() {
        assert_eq!(Amount::new(3.0).scaled_by(-1.0), Amount::ZERO);
    }
}
