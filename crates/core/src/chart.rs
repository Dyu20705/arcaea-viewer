use crate::{ChartEvent, TimingGroup, TimingGroupId, TimingGroupProperties};

/// Minimal owned chart container for normalized domain events.
///
/// Event order is preserved exactly as supplied. Cross-event validation such as
/// duplicate identifiers or overlap checks belongs to later parser or analytics
/// checkpoints once the canonical chart rules are fully specified.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Chart {
    events: Vec<ChartEvent>,
    timing_groups: Vec<TimingGroup>,
}

impl Chart {
    /// Creates a chart from events in deterministic source order.
    #[must_use]
    pub fn new(events: Vec<ChartEvent>) -> Self {
        Self {
            events,
            timing_groups: vec![TimingGroup::new(
                TimingGroupId::ROOT,
                TimingGroupProperties::default(),
            )],
        }
    }

    /// Creates a chart from events and explicit timing groups.
    ///
    /// The root group is always present exactly once. Additional groups retain
    /// their supplied deterministic order.
    #[must_use]
    pub fn with_timing_groups(
        events: Vec<ChartEvent>,
        mut timing_groups: Vec<TimingGroup>,
    ) -> Self {
        timing_groups.retain(|group| group.id() != TimingGroupId::ROOT);
        let mut groups = vec![TimingGroup::new(
            TimingGroupId::ROOT,
            TimingGroupProperties::default(),
        )];
        groups.extend(timing_groups);
        Self {
            events,
            timing_groups: groups,
        }
    }

    /// Returns chart events in their stored deterministic order.
    #[must_use]
    pub fn events(&self) -> &[ChartEvent] {
        &self.events
    }

    /// Returns declared timing groups in stable order, including root first.
    #[must_use]
    pub fn timing_groups(&self) -> &[TimingGroup] {
        &self.timing_groups
    }

    /// Finds a declared timing group by ID.
    #[must_use]
    pub fn timing_group(&self, id: TimingGroupId) -> Option<TimingGroup> {
        self.timing_groups
            .iter()
            .copied()
            .find(|group| group.id() == id)
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
    use crate::{
        ChartEvent, ChartTime, Lane, NoteId, TapNote, TimingGroup, TimingGroupId,
        TimingGroupProperties,
    };

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
        assert_eq!(chart.timing_groups()[0].id(), TimingGroupId::ROOT);
    }

    #[test]
    fn chart_preserves_explicit_timing_groups_after_root() {
        let group = TimingGroup::new(
            TimingGroupId::new(1),
            TimingGroupProperties::new(true, false),
        );

        let chart = Chart::with_timing_groups(Vec::new(), vec![group]);

        assert_eq!(chart.timing_groups().len(), 2);
        assert_eq!(chart.timing_groups()[0].id(), TimingGroupId::ROOT);
        assert_eq!(chart.timing_groups()[1], group);
    }
}
