use crate::ChartEvent;

/// Minimal owned chart container for normalized domain events.
///
/// Event order is preserved exactly as supplied. Cross-event validation such as
/// duplicate identifiers or overlap checks belongs to later parser or analytics
/// checkpoints once the canonical chart rules are fully specified.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Chart {
    events: Vec<ChartEvent>,
}

impl Chart {
    /// Creates a chart from events in deterministic source order.
    #[must_use]
    pub fn new(events: Vec<ChartEvent>) -> Self {
        Self { events }
    }

    /// Returns chart events in their stored deterministic order.
    #[must_use]
    pub fn events(&self) -> &[ChartEvent] {
        &self.events
    }

    /// Returns the number of events stored in this chart.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns true when the chart contains no events.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Chart;
    use crate::{ChartEvent, ChartTime, Lane, NoteId, TapNote};

    #[test]
    fn chart_preserves_deterministic_event_order() {
        let first = ChartEvent::Tap(TapNote::new(
            NoteId::new(1),
            ChartTime::from_millis(200),
            Lane::new(1).expect("lane"),
        ));
        let second = ChartEvent::Tap(TapNote::new(
            NoteId::new(2),
            ChartTime::from_millis(100),
            Lane::new(4).expect("lane"),
        ));

        let chart = Chart::new(vec![first, second]);

        assert_eq!(chart.events(), &[first, second]);
        assert_eq!(chart.len(), 2);
    }
}
