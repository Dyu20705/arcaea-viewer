use crate::CoordinateError;

const MIN_NORMALIZED: f32 = 0.0;
const MAX_NORMALIZED: f32 = 1.0;

/// Normalized horizontal arc coordinate.
///
/// Values are currently constrained to `0.0..=1.0`, matching the known visible
/// Arcaea arc coordinate range. If parser fixtures later prove that meaningful
/// out-of-range coordinates are required, this boundary can widen without
/// changing event ownership or timing APIs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcX(f32);

impl ArcX {
    /// Creates a normalized horizontal arc coordinate.
    pub fn new(value: f32) -> Result<Self, CoordinateError> {
        validate_normalized(value).map(Self)
    }

    /// Returns the normalized coordinate value.
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

/// Normalized vertical arc coordinate.
///
/// Values are currently constrained to `0.0..=1.0`, independent of any later
/// renderer-specific screen-space transformation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcY(f32);

impl ArcY {
    /// Creates a normalized vertical arc coordinate.
    pub fn new(value: f32) -> Result<Self, CoordinateError> {
        validate_normalized(value).map(Self)
    }

    /// Returns the normalized coordinate value.
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self.0
    }
}

fn validate_normalized(value: f32) -> Result<f32, CoordinateError> {
    if !value.is_finite() {
        Err(CoordinateError::NonFinite { value })
    } else if !(MIN_NORMALIZED..=MAX_NORMALIZED).contains(&value) {
        Err(CoordinateError::OutOfRange {
            value,
            min: MIN_NORMALIZED,
            max: MAX_NORMALIZED,
        })
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{ArcX, ArcY};
    use crate::CoordinateError;

    #[test]
    fn accepts_normalized_coordinates() {
        assert_eq!(ArcX::new(0.0).expect("x min").as_f32(), 0.0);
        assert_eq!(ArcY::new(1.0).expect("y max").as_f32(), 1.0);
    }

    #[test]
    fn rejects_non_finite_coordinates() {
        assert!(matches!(
            ArcX::new(f32::NAN),
            Err(CoordinateError::NonFinite { .. })
        ));
        assert!(matches!(
            ArcY::new(f32::INFINITY),
            Err(CoordinateError::NonFinite { .. })
        ));
    }

    #[test]
    fn rejects_out_of_range_coordinates() {
        assert!(matches!(
            ArcX::new(-0.01),
            Err(CoordinateError::OutOfRange { .. })
        ));
        assert!(matches!(
            ArcY::new(1.01),
            Err(CoordinateError::OutOfRange { .. })
        ));
    }
}
