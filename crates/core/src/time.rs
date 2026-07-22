use core::fmt;

/// Integer chart time measured in milliseconds.
///
/// Negative values are supported so parser and timing layers can preserve
/// source formats that place timing data or offsets before chart zero. The type
/// does not clamp or reinterpret values.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChartTime(i64);

impl ChartTime {
    /// Creates a chart time from an integer millisecond value.
    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    /// Returns the stored time in milliseconds.
    #[must_use]
    pub const fn as_millis(self) -> i64 {
        self.0
    }
}

impl fmt::Display for ChartTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ChartTime;

    #[test]
    fn chart_time_orders_by_milliseconds() {
        let early = ChartTime::from_millis(-50);
        let start = ChartTime::from_millis(0);
        let later = ChartTime::from_millis(1_250);

        assert!(early < start);
        assert!(later > start);
        assert_eq!(later.as_millis(), 1_250);
    }
}
