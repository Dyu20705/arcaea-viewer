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

/// Stable timing group identifier unique within one parsed chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimingGroupId(u32);

impl TimingGroupId {
    /// The AFF root/default timing group.
    pub const ROOT: Self = Self(0);

    /// Creates an identifier from a deterministic chart-local integer.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the root/default timing group.
    #[must_use]
    pub const fn root() -> Self {
        Self::ROOT
    }

    /// Returns the integer backing this identifier.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Mode flags attached to a parsed timing group.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimingGroupProperties {
    no_input: bool,
    no_clip: bool,
}

impl TimingGroupProperties {
    /// Creates timing group properties from supported AFF flags.
    #[must_use]
    pub const fn new(no_input: bool, no_clip: bool) -> Self {
        Self { no_input, no_clip }
    }

    /// Returns true when the group was marked `noinput`.
    #[must_use]
    pub const fn no_input(self) -> bool {
        self.no_input
    }

    /// Returns true when the group was marked `noclip`.
    #[must_use]
    pub const fn no_clip(self) -> bool {
        self.no_clip
    }
}

/// A parser-independent timing group declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimingGroup {
    id: TimingGroupId,
    properties: TimingGroupProperties,
}

impl TimingGroup {
    /// Creates a timing group declaration.
    #[must_use]
    pub const fn new(id: TimingGroupId, properties: TimingGroupProperties) -> Self {
        Self { id, properties }
    }

    /// Returns the group identifier.
    #[must_use]
    pub const fn id(self) -> TimingGroupId {
        self.id
    }

    /// Returns supported group properties.
    #[must_use]
    pub const fn properties(self) -> TimingGroupProperties {
        self.properties
    }
}

/// Ground tap note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TapNote {
    id: NoteId,
    time: ChartTime,
    lane: Lane,
    timing_group: TimingGroupId,
}

impl TapNote {
    /// Creates a root timing-group tap note from validated domain values.
    #[must_use]
    pub const fn new(id: NoteId, time: ChartTime, lane: Lane) -> Self {
        Self::new_in_group(id, time, lane, TimingGroupId::ROOT)
    }

    /// Creates a tap note in a specific timing group.
    #[must_use]
    pub const fn new_in_group(
        id: NoteId,
        time: ChartTime,
        lane: Lane,
        timing_group: TimingGroupId,
    ) -> Self {
        Self {
            id,
            time,
            lane,
            timing_group,
        }
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

    /// Returns the timing group that owns this note.
    #[must_use]
    pub const fn timing_group(self) -> TimingGroupId {
        self.timing_group
    }
}

/// Ground hold note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HoldNote {
    id: NoteId,
    start_time: ChartTime,
    end_time: ChartTime,
    lane: Lane,
    timing_group: TimingGroupId,
}

impl HoldNote {
    /// Creates a root timing-group hold note if its end time does not precede its start time.
    pub const fn new(
        id: NoteId,
        start_time: ChartTime,
        end_time: ChartTime,
        lane: Lane,
    ) -> Result<Self, IntervalError> {
        Self::new_in_group(id, start_time, end_time, lane, TimingGroupId::ROOT)
    }

    /// Creates a hold note in a specific timing group.
    pub const fn new_in_group(
        id: NoteId,
        start_time: ChartTime,
        end_time: ChartTime,
        lane: Lane,
        timing_group: TimingGroupId,
    ) -> Result<Self, IntervalError> {
        if end_time.as_millis() < start_time.as_millis() {
            Err(IntervalError)
        } else {
            Ok(Self {
                id,
                start_time,
                end_time,
                lane,
                timing_group,
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

    /// Returns the timing group that owns this note.
    #[must_use]
    pub const fn timing_group(self) -> TimingGroupId {
        self.timing_group
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
    /// Sine-out interpolation on both axes for the currently supported `soso` token.
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

/// Normalized sky arc position at a chart timestamp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcPosition {
    /// Normalized horizontal playfield coordinate.
    pub x: f32,
    /// Normalized sky-height coordinate.
    pub y: f32,
    /// Normalized arc progress.
    pub progress: f32,
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
    timing_group: TimingGroupId,
}

impl ArcNote {
    /// Creates a root timing-group arc note if its end time does not precede its start time.
    pub const fn new(
        id: NoteId,
        start_time: ChartTime,
        end_time: ChartTime,
        path: ArcPath,
        curve: ArcCurve,
        color: ArcColor,
        is_trace: bool,
    ) -> Result<Self, IntervalError> {
        Self::new_in_group(
            id,
            start_time,
            end_time,
            path,
            curve,
            color,
            is_trace,
            TimingGroupId::ROOT,
        )
    }

    /// Creates an arc note in a specific timing group.
    #[allow(clippy::too_many_arguments)]
    pub const fn new_in_group(
        id: NoteId,
        start_time: ChartTime,
        end_time: ChartTime,
        path: ArcPath,
        curve: ArcCurve,
        color: ArcColor,
        is_trace: bool,
        timing_group: TimingGroupId,
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
                timing_group,
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

    /// Returns the timing group that owns this note.
    #[must_use]
    pub const fn timing_group(self) -> TimingGroupId {
        self.timing_group
    }

    /// Returns true when `time` is inside the arc interval, including boundaries.
    #[must_use]
    pub const fn contains_time(self, time: ChartTime) -> bool {
        self.start_time.as_millis() <= time.as_millis()
            && time.as_millis() <= self.end_time.as_millis()
    }
}

/// Sky arc tap note attached to a parent arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArcTapNote {
    id: NoteId,
    time: ChartTime,
    parent_arc_id: NoteId,
    timing_group: TimingGroupId,
}

impl ArcTapNote {
    /// Creates an arc tap with a stable identity and parent arc relationship.
    #[must_use]
    pub const fn new(
        id: NoteId,
        time: ChartTime,
        parent_arc_id: NoteId,
        timing_group: TimingGroupId,
    ) -> Self {
        Self {
            id,
            time,
            parent_arc_id,
            timing_group,
        }
    }

    /// Returns the chart-local note identifier.
    #[must_use]
    pub const fn id(self) -> NoteId {
        self.id
    }

    /// Returns the arc tap timestamp.
    #[must_use]
    pub const fn time(self) -> ChartTime {
        self.time
    }

    /// Returns the parent arc note identifier.
    #[must_use]
    pub const fn parent_arc_id(self) -> NoteId {
        self.parent_arc_id
    }

    /// Returns the timing group that owns this note.
    #[must_use]
    pub const fn timing_group(self) -> TimingGroupId {
        self.timing_group
    }
}

/// Timing event that changes chart tempo within one timing group.
///
/// `beats_per_measure` is retained as the AFF-like timing parameter needed by
/// later timing-group work, but this type does not perform beat conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimingEvent {
    time: ChartTime,
    tempo: Tempo,
    beats_per_measure: u16,
    timing_group: TimingGroupId,
}

impl TimingEvent {
    /// Creates a root timing-group timing event from validated tempo and chart time values.
    #[must_use]
    pub const fn new(time: ChartTime, tempo: Tempo, beats_per_measure: u16) -> Self {
        Self::new_in_group(time, tempo, beats_per_measure, TimingGroupId::ROOT)
    }

    /// Creates a timing event in a specific timing group.
    #[must_use]
    pub const fn new_in_group(
        time: ChartTime,
        tempo: Tempo,
        beats_per_measure: u16,
        timing_group: TimingGroupId,
    ) -> Self {
        Self {
            time,
            tempo,
            beats_per_measure,
            timing_group,
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

    /// Returns the timing group that owns this event.
    #[must_use]
    pub const fn timing_group(self) -> TimingGroupId {
        self.timing_group
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
    /// Sky arc tap note.
    ArcTap(ArcTapNote),
    /// Timing/tempo event.
    Timing(TimingEvent),
}

/// Returns deterministic axis progress for an arc curve at normalized progress.
#[must_use]
pub fn arc_axis_progress(curve: ArcCurve, progress: f32) -> (f32, f32) {
    let t = progress.clamp(0.0, 1.0);
    match curve {
        ArcCurve::Straight => (linear(t), linear(t)),
        ArcCurve::Bezier => (smoothstep(t), smoothstep(t)),
        ArcCurve::SineIn => (sine_in_axis(t), linear(t)),
        ArcCurve::SineOut => (sine_out_axis(t), linear(t)),
        ArcCurve::SineInOut => (sine_in_axis(t), sine_in_axis(t)),
        ArcCurve::SineOutIn => (sine_out_axis(t), sine_out_axis(t)),
    }
}

/// Returns the normalized arc position at `time`, or `None` outside the arc interval.
#[must_use]
pub fn arc_position_at(arc: ArcNote, time: ChartTime) -> Option<ArcPosition> {
    if !arc.contains_time(time) {
        return None;
    }

    let duration = arc.end_time().as_millis() - arc.start_time().as_millis();
    let progress = if duration == 0 {
        1.0
    } else {
        (time.as_millis() - arc.start_time().as_millis()) as f32 / duration as f32
    }
    .clamp(0.0, 1.0);
    let (x_progress, y_progress) = arc_axis_progress(arc.curve(), progress);
    Some(ArcPosition {
        x: lerp(arc.start_x().as_f32(), arc.end_x().as_f32(), x_progress),
        y: lerp(arc.start_y().as_f32(), arc.end_y().as_f32(), y_progress),
        progress,
    })
}

fn linear(progress: f32) -> f32 {
    progress
}

fn smoothstep(progress: f32) -> f32 {
    progress * progress * (3.0 - 2.0 * progress)
}

fn sine_in_axis(progress: f32) -> f32 {
    (progress * std::f32::consts::FRAC_PI_2).sin()
}

fn sine_out_axis(progress: f32) -> f32 {
    1.0 - (progress * std::f32::consts::FRAC_PI_2).cos()
}

fn lerp(start: f32, end: f32, progress: f32) -> f32 {
    start + ((end - start) * progress)
}

#[cfg(test)]
mod tests {
    use super::{
        ArcColor, ArcCurve, ArcNote, ArcPath, ArcTapNote, ChartEvent, HoldNote, NoteId, TapNote,
        TimingEvent, TimingGroupId, TimingGroupProperties, arc_position_at,
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
        assert_eq!(hold.timing_group(), TimingGroupId::ROOT);
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
    fn timing_group_properties_preserve_supported_flags() {
        let properties = TimingGroupProperties::new(true, true);

        assert!(properties.no_input());
        assert!(properties.no_clip());
    }

    #[test]
    fn notes_can_belong_to_non_root_timing_group() {
        let group = TimingGroupId::new(2);
        let tap = TapNote::new_in_group(
            NoteId::new(1),
            ChartTime::from_millis(100),
            Lane::new(1).expect("lane"),
            group,
        );
        let timing = TimingEvent::new_in_group(
            ChartTime::from_millis(0),
            Tempo::from_milli_bpm(180_000).expect("tempo"),
            4,
            group,
        );

        assert_eq!(tap.timing_group(), group);
        assert_eq!(timing.timing_group(), group);
    }

    #[test]
    fn arctap_has_stable_identity_parent_and_group() {
        let group = TimingGroupId::new(1);
        let arctap = ArcTapNote::new(
            NoteId::new(4),
            ChartTime::from_millis(1_500),
            NoteId::new(3),
            group,
        );

        assert_eq!(arctap.id().as_u32(), 4);
        assert_eq!(arctap.parent_arc_id().as_u32(), 3);
        assert_eq!(arctap.timing_group(), group);
    }

    #[test]
    fn arc_position_supports_boundaries_and_curve() {
        let arc =
            sample_arc(ChartTime::from_millis(1_000), ChartTime::from_millis(2_000)).expect("arc");

        let start = arc_position_at(arc, ChartTime::from_millis(1_000)).expect("start");
        let middle = arc_position_at(arc, ChartTime::from_millis(1_500)).expect("middle");
        let end = arc_position_at(arc, ChartTime::from_millis(2_000)).expect("end");

        assert!((start.x - 0.0).abs() < 0.000_001);
        assert!((middle.x - 0.707_106_77).abs() < 0.000_001);
        assert!((end.x - 1.0).abs() < 0.000_001);
        assert!(arc_position_at(arc, ChartTime::from_millis(999)).is_none());
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
