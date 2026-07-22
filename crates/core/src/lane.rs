use crate::LaneError;

/// Valid ground-note lane.
///
/// This checkpoint uses the conventional Arcaea ground lane range `1..=4`.
/// Parser-specific numeric tokens should be converted into this type at the
/// parser boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lane(u8);

impl Lane {
    /// Lowest supported ground lane value.
    pub const MIN: u8 = 1;

    /// Highest supported ground lane value.
    pub const MAX: u8 = 4;

    /// Creates a lane if the value is within `1..=4`.
    pub const fn new(value: u8) -> Result<Self, LaneError> {
        if value >= Self::MIN && value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(LaneError::new(value))
        }
    }

    /// Returns the lane as its conventional one-based integer.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Lane;

    #[test]
    fn accepts_valid_lanes() {
        assert_eq!(Lane::new(1).expect("lane 1").as_u8(), 1);
        assert_eq!(Lane::new(4).expect("lane 4").as_u8(), 4);
    }

    #[test]
    fn rejects_invalid_lanes() {
        assert_eq!(Lane::new(0).expect_err("lane 0").value(), 0);
        assert_eq!(Lane::new(5).expect_err("lane 5").value(), 5);
    }
}
