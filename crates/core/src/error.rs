use core::fmt;

/// Error returned when a ground note lane is outside the supported range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaneError {
    value: u8,
}

impl LaneError {
    pub(crate) const fn new(value: u8) -> Self {
        Self { value }
    }

    /// Returns the rejected lane value.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.value
    }
}

impl fmt::Display for LaneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid ground lane {}; expected 1..=4", self.value)
    }
}

impl std::error::Error for LaneError {}

/// Error returned when a tempo value is not valid for chart timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempoError {
    /// Tempo must be greater than zero.
    Zero,
}

impl fmt::Display for TempoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => f.write_str("tempo must be greater than zero"),
        }
    }
}

impl std::error::Error for TempoError {}

/// Error returned when a normalized coordinate is invalid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateError {
    /// Coordinate was NaN or infinite.
    NonFinite { value: f32 },
    /// Coordinate was finite but outside the normalized range.
    OutOfRange { value: f32, min: f32, max: f32 },
}

impl fmt::Display for CoordinateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { value } => {
                write!(f, "coordinate must be finite; got {value}")
            }
            Self::OutOfRange { value, min, max } => {
                write!(f, "coordinate {value} is outside {min}..={max}")
            }
        }
    }
}

impl std::error::Error for CoordinateError {}

/// Error returned when an event interval ends before it starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntervalError;

impl fmt::Display for IntervalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("event end time must not precede start time")
    }
}

impl std::error::Error for IntervalError {}
