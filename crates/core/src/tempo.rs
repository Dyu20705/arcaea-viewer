use crate::TempoError;

/// Chart tempo stored as thousandths of a BPM.
///
/// Milli-BPM keeps common decimal BPM values deterministic without exposing a
/// raw floating-point tempo as the domain representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tempo(u32);

impl Tempo {
    /// Creates a tempo from thousandths of a BPM.
    pub const fn from_milli_bpm(milli_bpm: u32) -> Result<Self, TempoError> {
        if milli_bpm == 0 {
            Err(TempoError::Zero)
        } else {
            Ok(Self(milli_bpm))
        }
    }

    /// Returns this tempo in thousandths of a BPM.
    #[must_use]
    pub const fn as_milli_bpm(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Tempo;
    use crate::TempoError;

    #[test]
    fn accepts_positive_tempo() {
        let tempo = Tempo::from_milli_bpm(120_500).expect("positive tempo");

        assert_eq!(tempo.as_milli_bpm(), 120_500);
    }

    #[test]
    fn rejects_zero_tempo() {
        assert_eq!(Tempo::from_milli_bpm(0), Err(TempoError::Zero));
    }
}
