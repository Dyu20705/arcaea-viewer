use crate::{ArcX, ArcY, ChartTime, IntervalError, Lane, Tempo};

/// Stable event identifier unique within one parsed chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteId(u32);

impl NoteId {
    /// Creates an identifier from a deterministic chart-local integer.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the integer backing this identifier.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Ground tap note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TapNote {
    id: NoteId,
    time: ChartTime,
    lane: Lane,
}

impl TapNote {
    /// Creates a tap note from validated domain values.
    #[must_use]
    pub const fn new(id: NoteId, time: ChartTime, lane: Lane) -> Self {
        Self { id, time, lane }
    }

    /// Returns the chart-local note identifier.
    #[must_use]
    pub const fn id(self) -> NoteId {
        self.id
    }

    /// Returns the tap time.
    #[must_use]
    pub const fn time(self) -> ChartTime {
        self.time
    }

    /// Returns the tap lane.
    #[must_use]
    pub const fn lane(self) -> Lane {
        self.lane
    }
}

/// Ground hold note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HoldNote {
    id: NoteId,
    start_time: ChartTime,
    end_time: ChartTime,
    lane: Lane,
}

impl HoldNote {
    /// Creates a hold note if its end time does not precede its start time.
    pub const fn new(
        id: NoteId,
        start_time: ChartTime,
        end_time: ChartTime,
        lane: Lane,
    ) -> Result<Self, IntervalError> {
        if end_time.as_millis() < start_time.as_millis() {
            Err(IntervalError)
        } else {
            Ok(Self {
                id,
                start_time,
                end_time,
                lane,
            })
        }
    }

    /// Returns the chart-local note identifier.
    #[must_use]
    pub const fn id(self) -> NoteId {
        self.id
    }

    /// Returns the start time.
    #[must_use]
    pub const fn start_time(self) -> ChartTime {
        self.start_time
    }

    /// Returns the end time.
    #[must_use]
    pub const fn end_time(self) -> ChartTime {
        self.end_time
    }

    /// Returns the hold lane.
    #[must_use]
    pub const fn lane(self) -> Lane {
        self.lane
    }
}

/// Semantic arc interpolation kind.
///
/// Source-text curve tokens are intentionally not represented here; the parser
/// crate will translate those tokens into these domain variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArcCurve {
    /// Linear interpolation between endpoints.
    Straight,
    /// Smooth Bezier-style interpolation.
    Bezier,
    /// Sine-eased interpolation entering the segment.
    SineIn,
    /// Sine-eased interpolation leaving the segment.
    SineOut,
    /// Sine-eased interpolation at both ends.
    SineInOut,
    /// Sine-eased interpolation leaving then entering across a split segment.
    SineOutIn,
}

/// Semantic Arcaea arc color or side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArcColor {
    /// Blue arc side.
    Blue,
    /// Red arc side.
    Red,
    /// Green arc side used by applicable chart events.
    Green,
}

/// Validated arc endpoint coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcPath {
    start_x: ArcX,
    end_x: ArcX,
    start_y: ArcY,
    end_y: ArcY,
}

impl ArcPath {
    /// Creates an arc path from validated normalized coordinates.
    #[must_use]
    pub const fn new(start_x: ArcX, end_x: ArcX, start_y: ArcY, end_y: ArcY) -> Self {
        Self {
            start_x,
            end_x,
            start_y,
            end_y,
        }
    }

    /// Returns the start horizontal coordinate.
    #[must_use]
    pub const fn start_x(self) -> ArcX {
        self.start_x
    }

    /// Returns the end horizontal coordinate.
    #[must_use]
    pub const fn end_x(self) -> ArcX {
        self.end_x
    }

    /// Returns the start vertical coordinate.
    #[must_use]
    pub const fn start_y(self) -> ArcY {
        self.start_y
    }

    /// Returns the end vertical coordinate.
    #[must_use]
    pub const fn end_y(self) -> ArcY {
        self.end_y
    }
}

/// Sky arc note.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcNote {
    id: NoteId,
    start_time: ChartTime,
    end_time: ChartTime,
    path: ArcPath,
    curve: ArcCurve,
    color: ArcColor,
    is_trace: bool,
}

impl ArcNote {
    /// Creates an arc note if its end time does not precede its start time.
    pub const fn new(
        id: NoteId,
        start_time: ChartTime,
        end_time: ChartTime,
        path: ArcPath,
        curve: ArcCurve,
        color: ArcColor,
        is_trace: bool,
    ) -> Result<Self, IntervalError> {
        if end_time.as_millis() < start_time.as_millis() {
            Err(IntervalError)
        } else {
            Ok(Self {
                id,
                start_time,
                end_time,
                path,
                curve,
                color,
                is_trace,
            })
        }
    }

    /// Returns the chart-local note identifier.
    #[must_use]
    pub const fn id(self) -> NoteId {
        self.id
    }

    /// Returns the start time.
    #[must_use]
    pub const fn start_time(self) -> ChartTime {
        self.start_time
    }

    /// Returns the end time.
    #[must_use]
    pub const fn end_time(self) -> ChartTime {
        self.end_time
    }

    /// Returns the start horizontal coordinate.
    #[must_use]
    pub const fn start_x(self) -> ArcX {
        self.path.start_x()
    }

    /// Returns the end horizontal coordinate.
    #[must_use]
    pub const fn end_x(self) -> ArcX {
        self.path.end_x()
    }

    /// Returns the start vertical coordinate.
    #[must_use]
    pub const fn start_y(self) -> ArcY {
        self.path.start_y()
    }

    /// Returns the end vertical coordinate.
    #[must_use]
    pub const fn end_y(self) -> ArcY {
        self.path.end_y()
    }

    /// Returns the validated arc coordinate path.
    #[must_use]
    pub const fn path(self) -> ArcPath {
        self.path
    }

    /// Returns the arc interpolation kind.
    #[must_use]
    pub const fn curve(self) -> ArcCurve {
        self.curve
    }

    /// Returns the semantic arc color or side.
    #[must_use]
    pub const fn color(self) -> ArcColor {
        self.color
    }

    /// Returns whether this arc is a trace-only arc.
    #[must_use]
    pub const fn is_trace(self) -> bool {
        self.is_trace
    }
}

/// Timing event that changes chart tempo.
///
/// `beats_per_measure` is retained as the AFF-like timing parameter needed by
/// later timing-group work, but this type does not perform beat conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimingEvent {
    time: ChartTime,
    tempo: Tempo,
    beats_per_measure: u16,
}

impl TimingEvent {
    /// Creates a timing event from validated tempo and chart time values.
    #[must_use]
    pub const fn new(time: ChartTime, tempo: Tempo, beats_per_measure: u16) -> Self {
        Self {
            time,
            tempo,
            beats_per_measure,
        }
    }

    /// Returns the timing event time.
    #[must_use]
    pub const fn time(self) -> ChartTime {
        self.time
    }

    /// Returns the tempo active at this timing event.
    #[must_use]
    pub const fn tempo(self) -> Tempo {
        self.tempo
    }

    /// Returns the beats-per-measure parameter.
    #[must_use]
    pub const fn beats_per_measure(self) -> u16 {
        self.beats_per_measure
    }
}

/// Parser-independent chart event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartEvent {
    /// Ground tap note.
    Tap(TapNote),
    /// Ground hold note.
    Hold(HoldNote),
    /// Sky arc note.
    Arc(ArcNote),
    /// Timing/tempo event.
    Timing(TimingEvent),
}

#[cfg(test)]
mod tests {
    use super::{
        ArcColor, ArcCurve, ArcNote, ArcPath, ChartEvent, HoldNote, NoteId, TapNote, TimingEvent,
    };
    use crate::{ArcX, ArcY, ChartTime, IntervalError, Lane, Tempo};

    #[test]
    fn creates_valid_hold_interval() {
        let hold = HoldNote::new(
            NoteId::new(1),
            ChartTime::from_millis(100),
            ChartTime::from_millis(500),
            Lane::new(2).expect("lane"),
        )
        .expect("valid hold");

        assert_eq!(hold.start_time().as_millis(), 100);
        assert_eq!(hold.end_time().as_millis(), 500);
    }

    #[test]
    fn rejects_reversed_hold_interval() {
        let hold = HoldNote::new(
            NoteId::new(1),
            ChartTime::from_millis(500),
            ChartTime::from_millis(100),
            Lane::new(2).expect("lane"),
        );

        assert_eq!(hold, Err(IntervalError));
    }

    #[test]
    fn creates_valid_arc_interval() {
        let arc = sample_arc(ChartTime::from_millis(100), ChartTime::from_millis(700))
            .expect("valid arc");

        assert_eq!(arc.start_time().as_millis(), 100);
        assert_eq!(arc.end_time().as_millis(), 700);
        assert_eq!(arc.curve(), ArcCurve::SineInOut);
        assert_eq!(arc.color(), ArcColor::Blue);
    }

    #[test]
    fn rejects_reversed_arc_interval() {
        let arc = sample_arc(ChartTime::from_millis(700), ChartTime::from_millis(100));

        assert_eq!(arc, Err(IntervalError));
    }

    #[test]
    fn chart_event_variants_wrap_domain_types() {
        let tap = TapNote::new(
            NoteId::new(7),
            ChartTime::from_millis(250),
            Lane::new(3).expect("lane"),
        );
        let timing = TimingEvent::new(
            ChartTime::from_millis(0),
            Tempo::from_milli_bpm(120_000).expect("tempo"),
            4,
        );

        assert!(matches!(
            ChartEvent::Tap(tap),
            ChartEvent::Tap(t) if t.id().as_u32() == tap.id().as_u32()
        ));
        assert!(matches!(
            ChartEvent::Timing(timing),
            ChartEvent::Timing(t) if t.tempo() == timing.tempo()
        ));
    }

    #[test]
    fn arc_curve_variants_are_semantic_not_parser_tokens() {
        let curves = [
            ArcCurve::Straight,
            ArcCurve::Bezier,
            ArcCurve::SineIn,
            ArcCurve::SineOut,
            ArcCurve::SineInOut,
            ArcCurve::SineOutIn,
        ];

        assert_eq!(curves.len(), 6);
    }

    fn sample_arc(start_time: ChartTime, end_time: ChartTime) -> Result<ArcNote, IntervalError> {
        ArcNote::new(
            NoteId::new(2),
            start_time,
            end_time,
            ArcPath::new(
                ArcX::new(0.0).expect("start x"),
                ArcX::new(1.0).expect("end x"),
                ArcY::new(0.25).expect("start y"),
                ArcY::new(0.75).expect("end y"),
            ),
            ArcCurve::SineInOut,
            ArcColor::Blue,
            false,
        )
    }
}
