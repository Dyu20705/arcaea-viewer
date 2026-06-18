//! Deterministic chart-time calculation boundary for Arcaea-Viewer.
//!
//! This crate builds timing maps and playback snapshots from normalized core
//! charts. It does not parse AFF source text and does not mutate charts.

use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use arcaea_viewer_core::{Chart, ChartEvent, ChartTime, NoteId, Tempo};

/// Small epsilon used only for tests and presentation checks around `f64` beat math.
pub const BEAT_EPSILON: f64 = 0.000_001;

/// A beat position relative to the first timing event in the chart.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeatPosition(f64);

impl BeatPosition {
    /// Creates a beat position from a finite floating-point value.
    pub fn new(value: f64) -> Result<Self, TimingError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(TimingError::NonFiniteBeatValue)
        }
    }

    /// Returns the beat position as `f64` for reports and visualizations.
    #[must_use]
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

/// A beat delta between two chart timestamps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeatDelta(f64);

impl BeatDelta {
    /// Creates a beat delta from a finite floating-point value.
    pub fn new(value: f64) -> Result<Self, TimingError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(TimingError::NonFiniteBeatValue)
        }
    }

    /// Returns the beat delta as `f64` for reports and visualizations.
    #[must_use]
    pub const fn as_f64(self) -> f64 {
        self.0
    }
}

/// Matchable errors produced by the timing map and SVG demo support.
#[derive(Debug)]
pub enum TimingError {
    /// A chart has no timing event, so tempo and beat position are undefined.
    MissingInitialTiming,
    /// Two timing events use the same timestamp.
    DuplicateTimingAtSameTimestamp { time: ChartTime },
    /// Beat math produced a non-finite value.
    NonFiniteBeatValue,
    /// File-system operation failed.
    Io { path: PathBuf, source: io::Error },
    /// SVG rendering failed before writing to disk.
    SvgRender { message: String },
}

impl fmt::Display for TimingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInitialTiming => {
                write!(f, "MISSING_INITIAL_TIMING: chart has no timing events")
            }
            Self::DuplicateTimingAtSameTimestamp { time } => write!(
                f,
                "DUPLICATE_TIMING_AT_SAME_TIMESTAMP: duplicate timing event at {time}"
            ),
            Self::NonFiniteBeatValue => write!(
                f,
                "NON_FINITE_BEAT_VALUE: beat math produced a non-finite value"
            ),
            Self::Io { path, source } => write!(f, "IO_ERROR: {}: {source}", path.display()),
            Self::SvgRender { message } => write!(f, "SVG_RENDER_ERROR: {message}"),
        }
    }
}

impl Error for TimingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TimingSegment {
    time: ChartTime,
    tempo: Tempo,
}

/// Ordered tempo map built from chart timing events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimingMap {
    segments: Vec<TimingSegment>,
}

impl TimingMap {
    /// Builds a timing map by sorting chart timing events without mutating the chart.
    ///
    /// Semantics for this checkpoint:
    /// - at least one timing event is required;
    /// - duplicate timing timestamps are rejected;
    /// - the first timing event extends backward for queries before it.
    pub fn from_chart(chart: &Chart) -> Result<Self, TimingError> {
        let mut segments: Vec<TimingSegment> = chart
            .events()
            .iter()
            .filter_map(|event| match event {
                ChartEvent::Timing(timing) => Some(TimingSegment {
                    time: timing.time(),
                    tempo: timing.tempo(),
                }),
                _ => None,
            })
            .collect();

        if segments.is_empty() {
            return Err(TimingError::MissingInitialTiming);
        }

        segments.sort_by_key(|segment| segment.time);

        for window in segments.windows(2) {
            if window[0].time == window[1].time {
                return Err(TimingError::DuplicateTimingAtSameTimestamp {
                    time: window[0].time,
                });
            }
        }

        Ok(Self { segments })
    }

    /// Returns the tempo active at `time`.
    #[must_use]
    pub fn tempo_at(&self, time: ChartTime) -> Tempo {
        self.segment_at(time).tempo
    }

    /// Returns beat position relative to the first timing event.
    #[must_use]
    pub fn beat_position_at(&self, time: ChartTime) -> BeatPosition {
        let first_time = self.segments[0].time;
        BeatPosition(self.elapsed_beats_between(first_time, time).as_f64())
    }

    /// Returns elapsed beats between two chart timestamps.
    ///
    /// A negative result is returned when `end` is before `start`.
    #[must_use]
    pub fn elapsed_beats_between(&self, start: ChartTime, end: ChartTime) -> BeatDelta {
        if start == end {
            return BeatDelta(0.0);
        }

        if end < start {
            let forward = self.elapsed_beats_forward(end, start);
            BeatDelta(-forward)
        } else {
            BeatDelta(self.elapsed_beats_forward(start, end))
        }
    }

    /// Returns ordered timing event timestamps and tempos.
    #[must_use]
    pub fn timing_events(&self) -> Vec<(ChartTime, Tempo)> {
        self.segments
            .iter()
            .map(|segment| (segment.time, segment.tempo))
            .collect()
    }

    fn segment_at(&self, time: ChartTime) -> TimingSegment {
        let index = self
            .segments
            .partition_point(|segment| segment.time.as_millis() <= time.as_millis());
        if index == 0 {
            self.segments[0]
        } else {
            self.segments[index - 1]
        }
    }

    fn elapsed_beats_forward(&self, start: ChartTime, end: ChartTime) -> f64 {
        let mut cursor = start;
        let mut total = 0.0;

        while cursor < end {
            let segment = self.segment_at(cursor);
            let next_timing = self
                .segments
                .iter()
                .find(|candidate| candidate.time > cursor)
                .map_or(end, |candidate| candidate.time.min(end));
            total += beats_for_interval(cursor, next_timing, segment.tempo);
            cursor = next_timing;
        }

        total
    }
}

/// Playback state for one note at a queried timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteState {
    /// The note starts after the playback timestamp.
    Upcoming,
    /// The note interval contains the playback timestamp.
    Active,
    /// The note ended at or before the playback timestamp.
    Passed,
}

/// Note kind retained in snapshots for reports and renderers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotNoteKind {
    /// Ground tap note.
    Tap,
    /// Ground hold note.
    Hold,
    /// Sky arc note.
    Arc,
}

impl fmt::Display for SnapshotNoteKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tap => write!(f, "Tap"),
            Self::Hold => write!(f, "Hold"),
            Self::Arc => write!(f, "Arc"),
        }
    }
}

/// Snapshot data for a single note.
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotNote {
    /// Deterministic chart-local note ID.
    pub id: NoteId,
    /// Note kind.
    pub kind: SnapshotNoteKind,
    /// Source-order index of the original chart event.
    pub source_index: usize,
    /// Start time for taps and interval notes.
    pub start_time: ChartTime,
    /// End time for holds/arcs, or tap time for taps.
    pub end_time: ChartTime,
    /// Playback state at the queried timestamp.
    pub state: NoteState,
    /// Normalized progress for active interval notes.
    pub progress: Option<f64>,
    /// Milliseconds until start when upcoming.
    pub starts_in_millis: Option<i64>,
    /// Milliseconds since end/tap time when passed.
    pub since_end_millis: Option<i64>,
}

/// Query result for a chart at one playback timestamp.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackSnapshot {
    /// Queried playback timestamp.
    pub playback_time: ChartTime,
    /// Current tempo at playback timestamp.
    pub tempo: Tempo,
    /// Beat position at playback timestamp.
    pub beat_position: BeatPosition,
    /// Source-order note states.
    pub notes: Vec<SnapshotNote>,
}

impl PlaybackSnapshot {
    /// Counts notes in a given state.
    #[must_use]
    pub fn count_state(&self, state: NoteState) -> usize {
        self.notes.iter().filter(|note| note.state == state).count()
    }
}

/// Queries playback state without mutating the chart.
#[must_use]
pub fn snapshot_at(
    chart: &Chart,
    timing_map: &TimingMap,
    playback_time: ChartTime,
) -> PlaybackSnapshot {
    let notes = chart
        .events()
        .iter()
        .enumerate()
        .filter_map(|(source_index, event)| match event {
            ChartEvent::Tap(tap) => Some(snapshot_tap(source_index, *tap, playback_time)),
            ChartEvent::Hold(hold) => Some(SnapshotNote {
                id: hold.id(),
                kind: SnapshotNoteKind::Hold,
                source_index,
                start_time: hold.start_time(),
                end_time: hold.end_time(),
                ..snapshot_interval(playback_time, hold.start_time(), hold.end_time())
            }),
            ChartEvent::Arc(arc) => Some(SnapshotNote {
                id: arc.id(),
                kind: SnapshotNoteKind::Arc,
                source_index,
                start_time: arc.start_time(),
                end_time: arc.end_time(),
                ..snapshot_interval(playback_time, arc.start_time(), arc.end_time())
            }),
            ChartEvent::Timing(_) => None,
        })
        .collect();

    PlaybackSnapshot {
        playback_time,
        tempo: timing_map.tempo_at(playback_time),
        beat_position: timing_map.beat_position_at(playback_time),
        notes,
    }
}

/// Writes a deterministic SVG timeline artifact.
pub fn write_timeline_svg(
    chart: &Chart,
    timing_map: &TimingMap,
    snapshot: &PlaybackSnapshot,
    fixture_name: &str,
    output_path: impl AsRef<Path>,
) -> Result<(), TimingError> {
    let output_path = output_path.as_ref();
    let svg = render_timeline_svg(chart, timing_map, snapshot, fixture_name)?;

    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|source| TimingError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    fs::write(output_path, svg).map_err(|source| TimingError::Io {
        path: output_path.to_path_buf(),
        source,
    })
}

/// Renders a deterministic SVG timeline string.
pub fn render_timeline_svg(
    chart: &Chart,
    timing_map: &TimingMap,
    snapshot: &PlaybackSnapshot,
    fixture_name: &str,
) -> Result<String, TimingError> {
    let range = timeline_range(chart, snapshot.playback_time)?;
    let width = 920_i64;
    let height = 360_i64;
    let left = 80_i64;
    let right = 860_i64;
    let axis_y = 130_i64;
    let scale = |time: ChartTime| -> i64 {
        let span = (range.end.as_millis() - range.start.as_millis()).max(1);
        let offset = time.as_millis() - range.start.as_millis();
        left + ((offset * (right - left)) / span)
    };

    let mut svg = String::new();
    svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" role=\"img\" aria-label=\"Timing snapshot timeline\">\n"
    ));
    svg.push_str("<title>Arcaea-Viewer Timing Snapshot Timeline</title>\n");
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#fbfbf8\"/>\n");
    svg.push_str("<style>text{font-family:Consolas,monospace;font-size:13px;fill:#1f2933}.small{font-size:11px}.axis{stroke:#25313f;stroke-width:2}.tick{stroke:#6b7280;stroke-width:1}.cursor{stroke:#d21f3c;stroke-width:3}.timing{fill:#6d5dfc}.tap{fill:#1f9d55}.hold{fill:#e5a50a}.arc{fill:#0077b6}.passed{opacity:.45}.active{stroke:#111827;stroke-width:3}.upcoming{stroke:#111827;stroke-width:1;stroke-dasharray:5 3}</style>\n");
    svg.push_str("<text x=\"30\" y=\"32\" font-size=\"20\" font-weight=\"700\">Timing Snapshot Demo</text>\n");
    svg.push_str(&format!(
        "<text x=\"30\" y=\"56\">Fixture: {}</text>\n",
        escape_xml(fixture_name)
    ));
    svg.push_str(&format!(
        "<text x=\"30\" y=\"76\">Playback: {}ms | Tempo: {:.3} BPM | Beat: {:.3}</text>\n",
        snapshot.playback_time.as_millis(),
        bpm(snapshot.tempo),
        snapshot.beat_position.as_f64()
    ));
    svg.push_str(&format!(
        "<text x=\"30\" y=\"96\">Range: {}ms..{}ms</text>\n",
        range.start.as_millis(),
        range.end.as_millis()
    ));

    svg.push_str(&format!(
        "<line class=\"axis\" x1=\"{left}\" y1=\"{axis_y}\" x2=\"{right}\" y2=\"{axis_y}\"/>\n"
    ));
    for tick in ticks(range.start.as_millis(), range.end.as_millis(), 5) {
        let x = scale(ChartTime::from_millis(tick));
        svg.push_str(&format!(
            "<line class=\"tick\" x1=\"{x}\" y1=\"{}\" x2=\"{x}\" y2=\"{}\"/>\n",
            axis_y - 8,
            axis_y + 8
        ));
        svg.push_str(&format!(
            "<text class=\"small\" x=\"{}\" y=\"{}\" text-anchor=\"middle\">{}ms</text>\n",
            x,
            axis_y + 26,
            tick
        ));
    }

    let cursor_x = scale(snapshot.playback_time);
    svg.push_str(&format!(
        "<line id=\"playback-cursor\" class=\"cursor\" x1=\"{cursor_x}\" y1=\"110\" x2=\"{cursor_x}\" y2=\"300\"/>\n"
    ));
    svg.push_str(&format!(
        "<text x=\"{}\" y=\"108\" text-anchor=\"middle\" fill=\"#d21f3c\">cursor</text>\n",
        cursor_x
    ));

    for (time, tempo) in timing_map.timing_events() {
        let x = scale(time);
        svg.push_str(&format!(
            "<circle class=\"timing\" cx=\"{x}\" cy=\"{axis_y}\" r=\"7\"><title>Timing {}ms {:.3} BPM</title></circle>\n",
            time.as_millis(),
            bpm(tempo)
        ));
        svg.push_str(&format!(
            "<text class=\"small\" x=\"{x}\" y=\"{}\" text-anchor=\"middle\">timing {:.0}</text>\n",
            axis_y - 16,
            bpm(tempo)
        ));
    }

    for note in &snapshot.notes {
        let y = 180 + (note.source_index as i64 * 34);
        let class = format!("{} {}", note.kind.svg_class(), note.state.svg_class());
        let label = format!("{} #{} {:?}", note.kind, note.id.as_u32(), note.state);
        match note.kind {
            SnapshotNoteKind::Tap => {
                let x = scale(note.start_time);
                svg.push_str(&format!(
                    "<circle class=\"{class}\" cx=\"{x}\" cy=\"{y}\" r=\"8\"><title>{}</title></circle>\n",
                    escape_xml(&label)
                ));
                svg.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\">{}</text>\n",
                    x + 14,
                    y + 4,
                    escape_xml(&label)
                ));
            }
            SnapshotNoteKind::Hold | SnapshotNoteKind::Arc => {
                let x1 = scale(note.start_time);
                let x2 = scale(note.end_time);
                let bar_width = (x2 - x1).max(3);
                svg.push_str(&format!(
                    "<rect class=\"{class}\" x=\"{x1}\" y=\"{}\" width=\"{bar_width}\" height=\"16\" rx=\"2\"><title>{}</title></rect>\n",
                    y - 8,
                    escape_xml(&label)
                ));
                svg.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\">{}</text>\n",
                    x2 + 10,
                    y + 4,
                    escape_xml(&label)
                ));
            }
        }
    }

    svg.push_str("<text x=\"30\" y=\"315\" font-weight=\"700\">Legend:</text>\n");
    svg.push_str("<text x=\"100\" y=\"315\">Timing dot | Tap circle | Hold/Arc bar | dashed=Upcoming solid=Active faded=Passed</text>\n");
    svg.push_str(&format!(
        "<text x=\"30\" y=\"340\">Summary: Upcoming={} Active={} Passed={}</text>\n",
        snapshot.count_state(NoteState::Upcoming),
        snapshot.count_state(NoteState::Active),
        snapshot.count_state(NoteState::Passed)
    ));
    svg.push_str("</svg>\n");
    Ok(svg)
}

#[derive(Debug, Clone, Copy)]
struct TimelineRange {
    start: ChartTime,
    end: ChartTime,
}

fn timeline_range(chart: &Chart, playback_time: ChartTime) -> Result<TimelineRange, TimingError> {
    let mut min = playback_time.as_millis();
    let mut max = playback_time.as_millis();

    for event in chart.events() {
        match event {
            ChartEvent::Timing(timing) => {
                min = min.min(timing.time().as_millis());
                max = max.max(timing.time().as_millis());
            }
            ChartEvent::Tap(tap) => {
                min = min.min(tap.time().as_millis());
                max = max.max(tap.time().as_millis());
            }
            ChartEvent::Hold(hold) => {
                min = min.min(hold.start_time().as_millis());
                max = max.max(hold.end_time().as_millis());
            }
            ChartEvent::Arc(arc) => {
                min = min.min(arc.start_time().as_millis());
                max = max.max(arc.end_time().as_millis());
            }
        }
    }

    if min == max {
        return Err(TimingError::SvgRender {
            message: "timeline requires a non-zero time range".to_owned(),
        });
    }

    let padding = ((max - min) / 10).max(250);
    Ok(TimelineRange {
        start: ChartTime::from_millis(min - padding),
        end: ChartTime::from_millis(max + padding),
    })
}

fn snapshot_tap(
    source_index: usize,
    tap: arcaea_viewer_core::TapNote,
    time: ChartTime,
) -> SnapshotNote {
    if time < tap.time() {
        SnapshotNote {
            id: tap.id(),
            kind: SnapshotNoteKind::Tap,
            source_index,
            start_time: tap.time(),
            end_time: tap.time(),
            state: NoteState::Upcoming,
            progress: None,
            starts_in_millis: Some(tap.time().as_millis() - time.as_millis()),
            since_end_millis: None,
        }
    } else {
        SnapshotNote {
            id: tap.id(),
            kind: SnapshotNoteKind::Tap,
            source_index,
            start_time: tap.time(),
            end_time: tap.time(),
            state: NoteState::Passed,
            progress: None,
            starts_in_millis: None,
            since_end_millis: Some(time.as_millis() - tap.time().as_millis()),
        }
    }
}

fn snapshot_interval(time: ChartTime, start: ChartTime, end: ChartTime) -> SnapshotNote {
    if time < start {
        SnapshotNote {
            id: NoteId::new(0),
            kind: SnapshotNoteKind::Hold,
            source_index: 0,
            start_time: start,
            end_time: end,
            state: NoteState::Upcoming,
            progress: None,
            starts_in_millis: Some(start.as_millis() - time.as_millis()),
            since_end_millis: None,
        }
    } else if time > end {
        SnapshotNote {
            id: NoteId::new(0),
            kind: SnapshotNoteKind::Hold,
            source_index: 0,
            start_time: start,
            end_time: end,
            state: NoteState::Passed,
            progress: None,
            starts_in_millis: None,
            since_end_millis: Some(time.as_millis() - end.as_millis()),
        }
    } else {
        SnapshotNote {
            id: NoteId::new(0),
            kind: SnapshotNoteKind::Hold,
            source_index: 0,
            start_time: start,
            end_time: end,
            state: NoteState::Active,
            progress: Some(interval_progress(time, start, end)),
            starts_in_millis: None,
            since_end_millis: None,
        }
    }
}

fn interval_progress(time: ChartTime, start: ChartTime, end: ChartTime) -> f64 {
    let duration = end.as_millis() - start.as_millis();
    if duration == 0 {
        1.0
    } else {
        (time.as_millis() - start.as_millis()) as f64 / duration as f64
    }
}

fn beats_for_interval(start: ChartTime, end: ChartTime, tempo: Tempo) -> f64 {
    let millis = end.as_millis() - start.as_millis();
    millis as f64 * f64::from(tempo.as_milli_bpm()) / 60_000_000.0
}

fn bpm(tempo: Tempo) -> f64 {
    f64::from(tempo.as_milli_bpm()) / 1000.0
}

fn ticks(start: i64, end: i64, count: usize) -> Vec<i64> {
    if count <= 1 {
        return vec![start];
    }
    (0..count)
        .map(|index| start + ((end - start) * index as i64) / (count as i64 - 1))
        .collect()
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

impl SnapshotNoteKind {
    const fn svg_class(self) -> &'static str {
        match self {
            Self::Tap => "tap",
            Self::Hold => "hold",
            Self::Arc => "arc",
        }
    }
}

impl NoteState {
    const fn svg_class(self) -> &'static str {
        match self {
            Self::Upcoming => "upcoming",
            Self::Active => "active",
            Self::Passed => "passed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arcaea_viewer_core::{
        ArcColor, ArcCurve, ArcNote, ArcPath, ArcX, ArcY, HoldNote, Lane, TapNote, TimingEvent,
    };

    #[test]
    fn timing_events_are_sorted_without_mutating_chart() {
        let chart = chart_with_events(vec![
            ChartEvent::Timing(timing(2_000, 180_000)),
            ChartEvent::Tap(tap(0, 1_000)),
            ChartEvent::Timing(timing(0, 120_000)),
        ]);
        let original = chart.clone();

        let map = TimingMap::from_chart(&chart).expect("timing map");

        assert_eq!(chart, original);
        assert_eq!(map.timing_events()[0].0.as_millis(), 0);
        assert_eq!(map.timing_events()[1].0.as_millis(), 2_000);
    }

    #[test]
    fn tempo_before_first_timing_uses_first_timing() {
        let chart = chart_with_events(vec![ChartEvent::Timing(timing(1_000, 120_000))]);
        let map = TimingMap::from_chart(&chart).expect("timing map");

        assert_eq!(
            map.tempo_at(ChartTime::from_millis(-500)).as_milli_bpm(),
            120_000
        );
    }

    #[test]
    fn tempo_at_timing_boundary_uses_new_tempo() {
        let map = multi_tempo_map();

        assert_eq!(
            map.tempo_at(ChartTime::from_millis(2_000)).as_milli_bpm(),
            180_000
        );
    }

    #[test]
    fn tempo_after_multiple_timing_events_uses_latest_tempo() {
        let map = multi_tempo_map();

        assert_eq!(
            map.tempo_at(ChartTime::from_millis(3_500)).as_milli_bpm(),
            180_000
        );
    }

    #[test]
    fn beat_accumulation_crosses_tempo_change() {
        let map = multi_tempo_map();

        let beats = map
            .elapsed_beats_between(ChartTime::from_millis(1_000), ChartTime::from_millis(3_000))
            .as_f64();

        assert!((beats - 5.0).abs() < BEAT_EPSILON);
    }

    #[test]
    fn negative_chart_time_has_negative_beat_position() {
        let chart = chart_with_events(vec![ChartEvent::Timing(timing(0, 120_000))]);
        let map = TimingMap::from_chart(&chart).expect("timing map");

        assert!(
            (map.beat_position_at(ChartTime::from_millis(-500)).as_f64() + 1.0).abs()
                < BEAT_EPSILON
        );
    }

    #[test]
    fn duplicate_timing_timestamps_are_rejected() {
        let chart = chart_with_events(vec![
            ChartEvent::Timing(timing(0, 120_000)),
            ChartEvent::Timing(timing(0, 180_000)),
        ]);

        assert!(matches!(
            TimingMap::from_chart(&chart),
            Err(TimingError::DuplicateTimingAtSameTimestamp { .. })
        ));
    }

    #[test]
    fn missing_timing_is_rejected() {
        let chart = chart_with_events(vec![ChartEvent::Tap(tap(0, 1_000))]);

        assert!(matches!(
            TimingMap::from_chart(&chart),
            Err(TimingError::MissingInitialTiming)
        ));
    }

    #[test]
    fn tap_is_upcoming_before_timestamp_and_passed_at_boundary() {
        let (chart, map) = snapshot_chart();

        let before = snapshot_at(&chart, &map, ChartTime::from_millis(999));
        let boundary = snapshot_at(&chart, &map, ChartTime::from_millis(1_000));

        assert_eq!(before.notes[0].state, NoteState::Upcoming);
        assert_eq!(before.notes[0].starts_in_millis, Some(1));
        assert_eq!(boundary.notes[0].state, NoteState::Passed);
        assert_eq!(boundary.notes[0].since_end_millis, Some(0));
    }

    #[test]
    fn hold_states_cover_boundaries() {
        let (chart, map) = snapshot_chart();

        assert_eq!(
            snapshot_at(&chart, &map, ChartTime::from_millis(1_499)).notes[1].state,
            NoteState::Upcoming
        );
        assert_eq!(
            snapshot_at(&chart, &map, ChartTime::from_millis(1_500)).notes[1].state,
            NoteState::Active
        );
        assert_eq!(
            snapshot_at(&chart, &map, ChartTime::from_millis(2_250)).notes[1].state,
            NoteState::Active
        );
        assert_eq!(
            snapshot_at(&chart, &map, ChartTime::from_millis(3_000)).notes[1].state,
            NoteState::Active
        );
        assert_eq!(
            snapshot_at(&chart, &map, ChartTime::from_millis(3_001)).notes[1].state,
            NoteState::Passed
        );
    }

    #[test]
    fn arc_states_cover_upcoming_active_and_passed() {
        let (chart, map) = snapshot_chart();

        assert_eq!(
            snapshot_at(&chart, &map, ChartTime::from_millis(3_199)).notes[2].state,
            NoteState::Upcoming
        );
        assert_eq!(
            snapshot_at(&chart, &map, ChartTime::from_millis(4_000)).notes[2].state,
            NoteState::Active
        );
        assert_eq!(
            snapshot_at(&chart, &map, ChartTime::from_millis(5_001)).notes[2].state,
            NoteState::Passed
        );
    }

    #[test]
    fn active_progress_is_normalized() {
        let (chart, map) = snapshot_chart();

        let start = snapshot_at(&chart, &map, ChartTime::from_millis(1_500)).notes[1]
            .progress
            .expect("start progress");
        let middle = snapshot_at(&chart, &map, ChartTime::from_millis(2_250)).notes[1]
            .progress
            .expect("middle progress");
        let end = snapshot_at(&chart, &map, ChartTime::from_millis(3_000)).notes[1]
            .progress
            .expect("end progress");

        assert!((start - 0.0).abs() < BEAT_EPSILON);
        assert!((middle - 0.5).abs() < BEAT_EPSILON);
        assert!((end - 1.0).abs() < BEAT_EPSILON);
    }

    #[test]
    fn zero_duration_interval_is_active_at_boundary_with_complete_progress() {
        let chart = chart_with_events(vec![
            ChartEvent::Timing(timing(0, 120_000)),
            ChartEvent::Hold(hold(0, 1_000, 1_000)),
        ]);
        let map = TimingMap::from_chart(&chart).expect("timing map");

        assert_eq!(
            snapshot_at(&chart, &map, ChartTime::from_millis(999)).notes[0].state,
            NoteState::Upcoming
        );
        let boundary = snapshot_at(&chart, &map, ChartTime::from_millis(1_000));
        assert_eq!(boundary.notes[0].state, NoteState::Active);
        assert_eq!(boundary.notes[0].progress, Some(1.0));
        assert_eq!(
            snapshot_at(&chart, &map, ChartTime::from_millis(1_001)).notes[0].state,
            NoteState::Passed
        );
    }

    #[test]
    fn repeated_snapshot_is_deterministic_and_does_not_mutate_chart() {
        let (chart, map) = snapshot_chart();
        let original = chart.clone();

        let first = snapshot_at(&chart, &map, ChartTime::from_millis(2_500));
        let second = snapshot_at(&chart, &map, ChartTime::from_millis(2_500));

        assert_eq!(first, second);
        assert_eq!(chart, original);
    }

    #[test]
    fn svg_output_contains_required_labels() {
        let (chart, map) = snapshot_chart();
        let snapshot = snapshot_at(&chart, &map, ChartTime::from_millis(2_500));

        let svg = render_timeline_svg(&chart, &map, &snapshot, "mixed_events.aff").expect("svg");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("playback-cursor"));
        assert!(svg.contains("Tap #0"));
        assert!(svg.contains("Hold #1"));
        assert!(svg.contains("Arc #2"));
        assert!(svg.contains("Active"));
        assert!(svg.contains("Summary"));
    }

    #[test]
    fn invalid_output_path_returns_structured_error() {
        let (chart, map) = snapshot_chart();
        let snapshot = snapshot_at(&chart, &map, ChartTime::from_millis(2_500));
        let base =
            std::env::temp_dir().join(format!("arcaea-viewer-timing-test-{}", std::process::id()));
        fs::write(&base, "not a directory").expect("temp marker");
        let invalid_path = base.join("child.svg");

        let error = write_timeline_svg(&chart, &map, &snapshot, "fixture.aff", &invalid_path)
            .expect_err("invalid path");

        assert!(matches!(error, TimingError::Io { .. }));
        let _ = fs::remove_file(base);
    }

    fn multi_tempo_map() -> TimingMap {
        let chart = chart_with_events(vec![
            ChartEvent::Timing(timing(0, 120_000)),
            ChartEvent::Timing(timing(2_000, 180_000)),
        ]);
        TimingMap::from_chart(&chart).expect("timing map")
    }

    fn snapshot_chart() -> (Chart, TimingMap) {
        let chart = chart_with_events(vec![
            ChartEvent::Timing(timing(0, 120_000)),
            ChartEvent::Tap(tap(0, 1_000)),
            ChartEvent::Hold(hold(1, 1_500, 3_000)),
            ChartEvent::Arc(arc(2, 3_200, 5_000)),
        ]);
        let map = TimingMap::from_chart(&chart).expect("timing map");
        (chart, map)
    }

    fn chart_with_events(events: Vec<ChartEvent>) -> Chart {
        Chart::new(events)
    }

    fn timing(time: i64, milli_bpm: u32) -> TimingEvent {
        TimingEvent::new(
            ChartTime::from_millis(time),
            Tempo::from_milli_bpm(milli_bpm).expect("tempo"),
            4,
        )
    }

    fn tap(id: u32, time: i64) -> TapNote {
        TapNote::new(
            NoteId::new(id),
            ChartTime::from_millis(time),
            Lane::new(1).expect("lane"),
        )
    }

    fn hold(id: u32, start: i64, end: i64) -> HoldNote {
        HoldNote::new(
            NoteId::new(id),
            ChartTime::from_millis(start),
            ChartTime::from_millis(end),
            Lane::new(2).expect("lane"),
        )
        .expect("hold")
    }

    fn arc(id: u32, start: i64, end: i64) -> ArcNote {
        ArcNote::new(
            NoteId::new(id),
            ChartTime::from_millis(start),
            ChartTime::from_millis(end),
            ArcPath::new(
                ArcX::new(0.25).expect("start x"),
                ArcX::new(0.75).expect("end x"),
                ArcY::new(0.5).expect("start y"),
                ArcY::new(1.0).expect("end y"),
            ),
            ArcCurve::SineInOut,
            ArcColor::Blue,
            false,
        )
        .expect("arc")
    }
}
