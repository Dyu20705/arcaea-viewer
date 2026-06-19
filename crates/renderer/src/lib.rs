//! Renderer-facing scene IR and deterministic static SVG preview support.
//!
//! Coordinate model:
//! - `NormalizedPoint.x` is horizontal playfield position, with `0.0` at the
//!   left edge and `1.0` at the right edge.
//! - `NormalizedPoint.y` is depth/screen position, with `0.0` at the judgement
//!   line and `1.0` at the far horizon.
//! - Ground lane centers are `(lane - 0.5) / 4.0`, so lane 1 is leftmost and
//!   lane 4 is rightmost.
//! - Arc `ArcX` maps directly to normalized horizontal playfield `x`; arc
//!   `ArcY` is retained as sky-height metadata on each sampled arc point.
//! - The SVG backend maps normalized points into viewport pixels using a
//!   deterministic trapezoid: the horizon is narrower than the judgement line,
//!   and SVG `y` decreases as normalized depth increases.
//!
//! Projection model:
//! - Playback time maps to the judgement line.
//! - The future edge of the configured window maps to the horizon.
//! - Past notes inside the configured past window are visible but clipped to the
//!   judgement line.
//! - Notes outside the past/future window are hidden.
//!
//! This is a debug preview projection, not official Arcaea scroll-speed logic.

use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use arcaea_viewer_core::{
    ArcColor, ArcCurve, ArcNote, Chart, ChartEvent, ChartTime, HoldNote, Lane, NoteId, TapNote,
};
use arcaea_viewer_timing::{NoteState, TimingMap, snapshot_at};

const LANE_COUNT: u8 = 4;
const DEFAULT_PAST_WINDOW_MS: i64 = 500;
const DEFAULT_FUTURE_WINDOW_MS: i64 = 2_000;
const DEFAULT_VIEWPORT_WIDTH: u32 = 1_280;
const DEFAULT_VIEWPORT_HEIGHT: u32 = 720;
const DEFAULT_ARC_SAMPLE_STEPS: usize = 16;
const EPSILON: f32 = 0.000_001;

/// Matchable errors produced by scene building and SVG rendering.
#[derive(Debug)]
pub enum RenderError {
    /// Viewport dimensions must be non-zero.
    InvalidViewport { width: u32, height: u32 },
    /// Projection windows must have a non-negative past and positive future.
    InvalidProjectionWindow {
        past_window_ms: i64,
        future_window_ms: i64,
    },
    /// A coordinate calculation produced a non-finite value.
    NonFiniteCoordinate { value: f32, context: &'static str },
    /// Arc sampling needs at least one step.
    InvalidArcSampling { steps: usize },
    /// File-system operation failed.
    Io { path: PathBuf, source: io::Error },
    /// SVG rendering failed before writing to disk.
    SvgRender { message: String },
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidViewport { width, height } => write!(
                f,
                "INVALID_VIEWPORT: viewport must be non-zero, got {width}x{height}"
            ),
            Self::InvalidProjectionWindow {
                past_window_ms,
                future_window_ms,
            } => write!(
                f,
                "INVALID_PROJECTION_WINDOW: past={past_window_ms}ms future={future_window_ms}ms"
            ),
            Self::NonFiniteCoordinate { value, context } => {
                write!(f, "NON_FINITE_COORDINATE: {context} produced {value}")
            }
            Self::InvalidArcSampling { steps } => {
                write!(f, "INVALID_ARC_SAMPLING: expected steps > 0, got {steps}")
            }
            Self::Io { path, source } => write!(f, "IO_ERROR: {}: {source}", path.display()),
            Self::SvgRender { message } => write!(f, "SVG_RENDER_ERROR: {message}"),
        }
    }
}

impl Error for RenderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Pixel viewport for a rendered scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    /// SVG or backend viewport width in pixels.
    pub width: u32,
    /// SVG or backend viewport height in pixels.
    pub height: u32,
}

impl Viewport {
    /// Creates a viewport if both dimensions are non-zero.
    pub const fn new(width: u32, height: u32) -> Result<Self, RenderError> {
        if width == 0 || height == 0 {
            Err(RenderError::InvalidViewport { width, height })
        } else {
            Ok(Self { width, height })
        }
    }
}

/// Projection and sampling parameters for scene generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectionConfig {
    /// Visible past window. Passed notes inside this window are clipped to the
    /// judgement line; older notes are hidden.
    pub past_window_ms: i64,
    /// Visible future window. Its edge maps to the horizon.
    pub future_window_ms: i64,
    /// Viewport width in pixels.
    pub viewport_width: u32,
    /// Viewport height in pixels.
    pub viewport_height: u32,
    /// Fixed number of arc sampling segments.
    pub arc_sample_steps: usize,
}

impl ProjectionConfig {
    /// Creates a validated config.
    pub const fn new(
        past_window_ms: i64,
        future_window_ms: i64,
        viewport_width: u32,
        viewport_height: u32,
        arc_sample_steps: usize,
    ) -> Result<Self, RenderError> {
        if past_window_ms < 0 || future_window_ms <= 0 {
            return Err(RenderError::InvalidProjectionWindow {
                past_window_ms,
                future_window_ms,
            });
        }
        if viewport_width == 0 || viewport_height == 0 {
            return Err(RenderError::InvalidViewport {
                width: viewport_width,
                height: viewport_height,
            });
        }
        if arc_sample_steps == 0 {
            return Err(RenderError::InvalidArcSampling {
                steps: arc_sample_steps,
            });
        }
        Ok(Self {
            past_window_ms,
            future_window_ms,
            viewport_width,
            viewport_height,
            arc_sample_steps,
        })
    }

    /// Returns the viewport encoded by this config.
    #[must_use]
    pub const fn viewport(self) -> Viewport {
        Viewport {
            width: self.viewport_width,
            height: self.viewport_height,
        }
    }
}

impl Default for ProjectionConfig {
    fn default() -> Self {
        Self {
            past_window_ms: DEFAULT_PAST_WINDOW_MS,
            future_window_ms: DEFAULT_FUTURE_WINDOW_MS,
            viewport_width: DEFAULT_VIEWPORT_WIDTH,
            viewport_height: DEFAULT_VIEWPORT_HEIGHT,
            arc_sample_steps: DEFAULT_ARC_SAMPLE_STEPS,
        }
    }
}

/// Normalized playfield point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedPoint {
    /// Horizontal playfield coordinate, left `0.0` to right `1.0`.
    pub x: f32,
    /// Depth coordinate, judgement line `0.0` to horizon `1.0`.
    pub y: f32,
}

impl NormalizedPoint {
    /// Creates a finite normalized point.
    pub fn new(x: f32, y: f32) -> Result<Self, RenderError> {
        ensure_finite(x, "point x")?;
        ensure_finite(y, "point y")?;
        Ok(Self { x, y })
    }
}

/// Projection result for one chart timestamp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectedTime {
    /// Clipped normalized depth.
    pub depth: f32,
    /// Visibility against the configured time window.
    pub visibility: TimeVisibility,
    /// Milliseconds relative to playback time.
    pub delta_ms: i64,
    /// True when a past timestamp was retained at the judgement line.
    pub clipped_at_judgement: bool,
}

/// Timestamp visibility against the configured time window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeVisibility {
    /// Timestamp is inside the visible window.
    Visible,
    /// Timestamp is older than the visible past window.
    HiddenPast,
    /// Timestamp is newer than the visible future window.
    HiddenFuture,
}

/// Stable scene primitive layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderLayer {
    /// Lane and playfield guides.
    Lanes = 10,
    /// Timing markers.
    TimingMarkers = 20,
    /// Hold bodies.
    Holds = 30,
    /// Arc paths.
    Arcs = 40,
    /// Tap markers.
    Taps = 50,
    /// Judgement line.
    JudgementLine = 60,
    /// Debug labels.
    DebugLabels = 70,
}

/// Note state retained by renderer primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderNoteState {
    /// Starts after playback time.
    Upcoming,
    /// Contains playback time.
    Active,
    /// Ended at or before playback time.
    Passed,
}

impl From<NoteState> for RenderNoteState {
    fn from(value: NoteState) -> Self {
        match value {
            NoteState::Upcoming => Self::Upcoming,
            NoteState::Active => Self::Active,
            NoteState::Passed => Self::Passed,
        }
    }
}

/// Renderer-facing scene independent of SVG.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderScene {
    /// Pixel viewport.
    pub viewport: Viewport,
    /// Queried playback time.
    pub playback_time: ChartTime,
    /// Stable ordered primitive list.
    pub primitives: Vec<RenderPrimitive>,
    /// Summary and fixture metadata.
    pub metadata: SceneMetadata,
}

/// Scene metadata useful for renderers and demos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneMetadata {
    /// Human-readable fixture name or source label.
    pub fixture_name: String,
    /// Inclusive visible window start in chart time.
    pub visible_time_start: ChartTime,
    /// Inclusive visible window end in chart time.
    pub visible_time_end: ChartTime,
    /// Count summary generated from primitives.
    pub summary: SceneSummary,
}

/// Primitive count summary.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SceneSummary {
    /// Number of lane primitives.
    pub lanes: usize,
    /// Visible tap primitive count.
    pub visible_taps: usize,
    /// Visible hold primitive count.
    pub visible_holds: usize,
    /// Visible arc primitive count.
    pub visible_arcs: usize,
    /// Hidden note count.
    pub hidden_notes: usize,
    /// Total primitive count.
    pub primitive_count: usize,
}

/// Renderer primitive variants.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderPrimitive {
    /// Ground lane guide.
    Lane(LanePrimitive),
    /// Judgement line.
    JudgementLine(JudgementLinePrimitive),
    /// Ground tap marker.
    Tap(TapPrimitive),
    /// Ground hold body.
    Hold(HoldPrimitive),
    /// Sky arc sampled path.
    Arc(ArcPrimitive),
    /// Timing marker.
    TimingMarker(TimingMarkerPrimitive),
    /// Debug or summary label.
    Label(LabelPrimitive),
}

impl RenderPrimitive {
    /// Returns stable render layer.
    #[must_use]
    pub const fn layer(&self) -> RenderLayer {
        match self {
            Self::Lane(value) => value.layer,
            Self::JudgementLine(value) => value.layer,
            Self::Tap(value) => value.layer,
            Self::Hold(value) => value.layer,
            Self::Arc(value) => value.layer,
            Self::TimingMarker(value) => value.layer,
            Self::Label(value) => value.layer,
        }
    }

    fn stable_order(&self) -> usize {
        match self {
            Self::Lane(value) => value.stable_order,
            Self::JudgementLine(value) => value.stable_order,
            Self::Tap(value) => value.stable_order,
            Self::Hold(value) => value.stable_order,
            Self::Arc(value) => value.stable_order,
            Self::TimingMarker(value) => value.stable_order,
            Self::Label(value) => value.stable_order,
        }
    }
}

/// Ground lane primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct LanePrimitive {
    /// Stable lane identifier.
    pub id: String,
    /// One-based lane value.
    pub lane: Lane,
    /// Left boundary in normalized playfield coordinates.
    pub x_start: f32,
    /// Right boundary in normalized playfield coordinates.
    pub x_end: f32,
    /// Lane center in normalized playfield coordinates.
    pub center_x: f32,
    /// Stable layer.
    pub layer: RenderLayer,
    /// Primitive visibility.
    pub visible: bool,
    /// Optional label.
    pub debug_label: Option<String>,
    stable_order: usize,
}

/// Judgement line primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct JudgementLinePrimitive {
    /// Normalized depth, always `0.0`.
    pub y: f32,
    /// Stable layer.
    pub layer: RenderLayer,
    /// Primitive visibility.
    pub visible: bool,
    /// Optional label.
    pub debug_label: Option<String>,
    stable_order: usize,
}

/// Ground tap primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct TapPrimitive {
    /// Chart-local note ID.
    pub note_id: NoteId,
    /// One-based lane.
    pub lane: Lane,
    /// Center position.
    pub center: NormalizedPoint,
    /// Current playback state.
    pub state: RenderNoteState,
    /// Stable layer.
    pub layer: RenderLayer,
    /// Primitive visibility.
    pub visible: bool,
    /// Optional label.
    pub debug_label: Option<String>,
    stable_order: usize,
}

/// Ground hold primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct HoldPrimitive {
    /// Chart-local note ID.
    pub note_id: NoteId,
    /// One-based lane.
    pub lane: Lane,
    /// Left boundary in normalized playfield coordinates.
    pub x_start: f32,
    /// Right boundary in normalized playfield coordinates.
    pub x_end: f32,
    /// Near clipped depth.
    pub y_start: f32,
    /// Far clipped depth.
    pub y_end: f32,
    /// Current playback state.
    pub state: RenderNoteState,
    /// True when clipped at the judgement line.
    pub clipped_at_judgement: bool,
    /// True when clipped at the horizon.
    pub clipped_at_horizon: bool,
    /// Stable layer.
    pub layer: RenderLayer,
    /// Primitive visibility.
    pub visible: bool,
    /// Optional label.
    pub debug_label: Option<String>,
    stable_order: usize,
}

/// One sampled arc point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcSamplePoint {
    /// Normalized playfield position.
    pub position: NormalizedPoint,
    /// Retained normalized sky height from `ArcY`.
    pub sky_y: f32,
    /// Source curve progress `0.0..=1.0`.
    pub progress: f32,
}

/// Sky arc primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct ArcPrimitive {
    /// Chart-local note ID.
    pub note_id: NoteId,
    /// Deterministically sampled points.
    pub points: Vec<ArcSamplePoint>,
    /// Core curve semantic.
    pub curve: ArcCurve,
    /// Core color semantic.
    pub color: ArcColor,
    /// Trace-only flag.
    pub is_trace: bool,
    /// Current playback state.
    pub state: RenderNoteState,
    /// True when clipped at the judgement line.
    pub clipped_at_judgement: bool,
    /// True when clipped at the horizon.
    pub clipped_at_horizon: bool,
    /// Stable layer.
    pub layer: RenderLayer,
    /// Primitive visibility.
    pub visible: bool,
    /// Optional label.
    pub debug_label: Option<String>,
    stable_order: usize,
}

/// Timing marker primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct TimingMarkerPrimitive {
    /// Timing event time.
    pub time: ChartTime,
    /// Position on the playfield center line.
    pub position: NormalizedPoint,
    /// Stable layer.
    pub layer: RenderLayer,
    /// Primitive visibility.
    pub visible: bool,
    /// Optional label.
    pub debug_label: Option<String>,
    stable_order: usize,
}

/// Debug label primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct LabelPrimitive {
    /// Label text.
    pub text: String,
    /// Label anchor.
    pub anchor: NormalizedPoint,
    /// Stable layer.
    pub layer: RenderLayer,
    /// Primitive visibility.
    pub visible: bool,
    stable_order: usize,
}

/// Projects one timestamp into normalized scene depth.
pub fn project_time(
    playback_time: ChartTime,
    time: ChartTime,
    config: ProjectionConfig,
) -> Result<ProjectedTime, RenderError> {
    ProjectionConfig::new(
        config.past_window_ms,
        config.future_window_ms,
        config.viewport_width,
        config.viewport_height,
        config.arc_sample_steps,
    )?;
    let delta_ms = time.as_millis() - playback_time.as_millis();
    if delta_ms < -config.past_window_ms {
        return Ok(ProjectedTime {
            depth: 0.0,
            visibility: TimeVisibility::HiddenPast,
            delta_ms,
            clipped_at_judgement: true,
        });
    }
    if delta_ms > config.future_window_ms {
        return Ok(ProjectedTime {
            depth: 1.0,
            visibility: TimeVisibility::HiddenFuture,
            delta_ms,
            clipped_at_judgement: false,
        });
    }
    let depth = if delta_ms <= 0 {
        0.0
    } else {
        delta_ms as f32 / config.future_window_ms as f32
    };
    ensure_finite(depth, "projected depth")?;
    Ok(ProjectedTime {
        depth,
        visibility: TimeVisibility::Visible,
        delta_ms,
        clipped_at_judgement: delta_ms < 0,
    })
}

/// Returns the center of a ground lane in normalized playfield coordinates.
#[must_use]
pub fn lane_center_x(lane: Lane) -> f32 {
    (f32::from(lane.as_u8()) - 0.5) / f32::from(LANE_COUNT)
}

/// Returns normalized lane boundaries.
#[must_use]
pub fn lane_bounds_x(lane: Lane) -> (f32, f32) {
    let lane_index = f32::from(lane.as_u8() - 1);
    (
        lane_index / f32::from(LANE_COUNT),
        (lane_index + 1.0) / f32::from(LANE_COUNT),
    )
}

/// Builds a renderer-facing scene without depending on parser syntax.
pub fn build_scene(
    chart: &Chart,
    timing_map: &TimingMap,
    playback_time: ChartTime,
    fixture_name: impl Into<String>,
    config: ProjectionConfig,
) -> Result<RenderScene, RenderError> {
    ProjectionConfig::new(
        config.past_window_ms,
        config.future_window_ms,
        config.viewport_width,
        config.viewport_height,
        config.arc_sample_steps,
    )?;

    let snapshot = snapshot_at(chart, timing_map, playback_time);
    let mut primitives = Vec::new();
    let mut hidden_notes = 0_usize;

    for lane_value in Lane::MIN..=Lane::MAX {
        let lane = Lane::new(lane_value).expect("lane constant is valid");
        let (x_start, x_end) = lane_bounds_x(lane);
        primitives.push(RenderPrimitive::Lane(LanePrimitive {
            id: format!("lane-{lane_value}"),
            lane,
            x_start,
            x_end,
            center_x: lane_center_x(lane),
            layer: RenderLayer::Lanes,
            visible: true,
            debug_label: Some(format!("Lane {lane_value}")),
            stable_order: lane_value as usize,
        }));
    }

    for (index, (time, tempo)) in timing_map.timing_events().into_iter().enumerate() {
        let projected = project_time(playback_time, time, config)?;
        if projected.visibility == TimeVisibility::Visible {
            primitives.push(RenderPrimitive::TimingMarker(TimingMarkerPrimitive {
                time,
                position: NormalizedPoint::new(0.5, projected.depth)?,
                layer: RenderLayer::TimingMarkers,
                visible: true,
                debug_label: Some(format!(
                    "Timing {}ms {:.3} BPM",
                    time.as_millis(),
                    f64::from(tempo.as_milli_bpm()) / 1000.0
                )),
                stable_order: index,
            }));
        }
    }

    for (source_index, event) in chart.events().iter().enumerate() {
        let state = snapshot
            .notes
            .iter()
            .find(|note| note.source_index == source_index)
            .map(|note| RenderNoteState::from(note.state));

        match event {
            ChartEvent::Tap(tap) => {
                if let Some(state) = state {
                    match build_tap(*tap, playback_time, state, config, source_index)? {
                        Some(primitive) => primitives.push(RenderPrimitive::Tap(primitive)),
                        None => hidden_notes += 1,
                    }
                }
            }
            ChartEvent::Hold(hold) => {
                if let Some(state) = state {
                    match build_hold(*hold, playback_time, state, config, source_index)? {
                        Some(primitive) => primitives.push(RenderPrimitive::Hold(primitive)),
                        None => hidden_notes += 1,
                    }
                }
            }
            ChartEvent::Arc(arc) => {
                if let Some(state) = state {
                    match build_arc(*arc, playback_time, state, config, source_index)? {
                        Some(primitive) => primitives.push(RenderPrimitive::Arc(primitive)),
                        None => hidden_notes += 1,
                    }
                }
            }
            ChartEvent::Timing(_) => {}
        }
    }

    primitives.push(RenderPrimitive::JudgementLine(JudgementLinePrimitive {
        y: 0.0,
        layer: RenderLayer::JudgementLine,
        visible: true,
        debug_label: Some("Judgement line".to_owned()),
        stable_order: 0,
    }));
    primitives.push(RenderPrimitive::Label(LabelPrimitive {
        text: format!("Playback {}ms", playback_time.as_millis()),
        anchor: NormalizedPoint::new(0.02, 0.94)?,
        layer: RenderLayer::DebugLabels,
        visible: true,
        stable_order: 0,
    }));

    primitives.sort_by_key(|primitive| (primitive.layer(), primitive.stable_order()));

    let summary = summarize(&primitives, hidden_notes);
    Ok(RenderScene {
        viewport: config.viewport(),
        playback_time,
        primitives,
        metadata: SceneMetadata {
            fixture_name: fixture_name.into(),
            visible_time_start: ChartTime::from_millis(
                playback_time.as_millis() - config.past_window_ms,
            ),
            visible_time_end: ChartTime::from_millis(
                playback_time.as_millis() + config.future_window_ms,
            ),
            summary,
        },
    })
}

/// Renders a scene to a self-contained SVG document.
pub fn render_scene_to_svg(scene: &RenderScene) -> Result<String, RenderError> {
    Viewport::new(scene.viewport.width, scene.viewport.height)?;
    let width = scene.viewport.width;
    let height = scene.viewport.height;
    let mut svg = String::new();
    svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" role=\"img\" aria-label=\"Arcaea Viewer static chart preview\">\n"
    ));
    svg.push_str("<title>Arcaea-Viewer Static Chart Preview</title>\n");
    svg.push_str("<style>text{font-family:Consolas,monospace;font-size:14px;fill:#14213d}.small{font-size:12px}.playfield{fill:#f7f8fc;stroke:#172033;stroke-width:2}.lane{fill:#ffffff;stroke:#9aa4b2;stroke-width:1.5}.lane.alt{fill:#edf2f7}.timing{stroke:#6b7280;stroke-width:2;stroke-dasharray:6 5}.judgement{stroke:#e11d48;stroke-width:5;stroke-linecap:round}.horizon{stroke:#172033;stroke-width:3}.tap{fill:#20a4f3;stroke:#0f172a;stroke-width:2}.hold{fill:#f7b801;fill-opacity:.65;stroke:#7c4a03;stroke-width:2}.arc-blue{fill:none;stroke:#2563eb;stroke-width:5;stroke-linecap:round;stroke-linejoin:round}.arc-red{fill:none;stroke:#dc2626;stroke-width:5;stroke-linecap:round;stroke-linejoin:round}.arc-green{fill:none;stroke:#16a34a;stroke-width:5;stroke-linecap:round;stroke-linejoin:round}.passed{opacity:.55}.active{filter:url(#activeGlow)}.label{fill:#111827;font-size:12px}.summary{fill:#111827;font-size:13px}</style>\n");
    svg.push_str("<defs><filter id=\"activeGlow\"><feDropShadow dx=\"0\" dy=\"0\" stdDeviation=\"3\" flood-color=\"#111827\" flood-opacity=\"0.35\"/></filter></defs>\n");
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#f4f0e8\"/>\n");
    svg.push_str(&format!(
        "<text id=\"preview-title\" x=\"32\" y=\"34\" font-size=\"22\" font-weight=\"700\">Chart Preview</text>\n<text id=\"fixture-name\" x=\"32\" y=\"58\">Fixture: {}</text>\n<text id=\"playback-time\" x=\"32\" y=\"80\">Playback: {}ms</text>\n<text id=\"visible-window\" x=\"32\" y=\"102\">Visible window: {}..{}ms</text>\n",
        escape_xml(&scene.metadata.fixture_name),
        scene.playback_time.as_millis(),
        scene.metadata.visible_time_start.as_millis(),
        scene.metadata.visible_time_end.as_millis()
    ));

    let converter = SvgConverter::new(scene.viewport);
    let playfield = converter.playfield_polygon();
    svg.push_str(&format!(
        "<g id=\"playfield\" data-viewport=\"{width}x{height}\">\n<polygon id=\"playfield-surface\" class=\"playfield\" points=\"{}\"/>\n",
        polygon_points(&playfield)
    ));
    let horizon = converter.segment(0.0, 1.0, 1.0, 1.0);
    svg.push_str(&format!(
        "<line id=\"horizon\" class=\"horizon\" x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
        horizon.0.x, horizon.0.y, horizon.1.x, horizon.1.y
    ));

    for primitive in &scene.primitives {
        match primitive {
            RenderPrimitive::Lane(lane) => render_lane(&mut svg, &converter, lane),
            RenderPrimitive::TimingMarker(marker) => {
                render_timing_marker(&mut svg, &converter, marker)
            }
            RenderPrimitive::Hold(hold) => render_hold(&mut svg, &converter, hold),
            RenderPrimitive::Arc(arc) => render_arc(&mut svg, &converter, arc),
            RenderPrimitive::Tap(tap) => render_tap(&mut svg, &converter, tap),
            RenderPrimitive::JudgementLine(line) => {
                render_judgement_line(&mut svg, &converter, line)
            }
            RenderPrimitive::Label(label) => render_label(&mut svg, &converter, label),
        }
    }
    svg.push_str("</g>\n");
    svg.push_str(&format!(
        "<g id=\"debug-summary\"><text class=\"summary\" x=\"32\" y=\"{}\">Lanes={} Visible taps={} Visible holds={} Visible arcs={} Hidden notes={} Primitive count={}</text></g>\n",
        height.saturating_sub(30),
        scene.metadata.summary.lanes,
        scene.metadata.summary.visible_taps,
        scene.metadata.summary.visible_holds,
        scene.metadata.summary.visible_arcs,
        scene.metadata.summary.hidden_notes,
        scene.metadata.summary.primitive_count
    ));
    svg.push_str("</svg>\n");
    Ok(svg)
}

/// Writes a scene SVG, creating parent directories if needed.
pub fn write_scene_svg(scene: &RenderScene, path: impl AsRef<Path>) -> Result<(), RenderError> {
    let path = path.as_ref();
    let svg = render_scene_to_svg(scene)?;
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|source| RenderError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(path, svg).map_err(|source| RenderError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn build_tap(
    tap: TapNote,
    playback_time: ChartTime,
    state: RenderNoteState,
    config: ProjectionConfig,
    stable_order: usize,
) -> Result<Option<TapPrimitive>, RenderError> {
    let projected = project_time(playback_time, tap.time(), config)?;
    if projected.visibility != TimeVisibility::Visible {
        return Ok(None);
    }
    Ok(Some(TapPrimitive {
        note_id: tap.id(),
        lane: tap.lane(),
        center: NormalizedPoint::new(lane_center_x(tap.lane()), projected.depth)?,
        state,
        layer: RenderLayer::Taps,
        visible: true,
        debug_label: Some(format!("Tap NoteId={} {:?}", tap.id().as_u32(), state)),
        stable_order,
    }))
}

fn build_hold(
    hold: HoldNote,
    playback_time: ChartTime,
    state: RenderNoteState,
    config: ProjectionConfig,
    stable_order: usize,
) -> Result<Option<HoldPrimitive>, RenderError> {
    let Some(interval) =
        project_interval(playback_time, hold.start_time(), hold.end_time(), config)?
    else {
        return Ok(None);
    };
    let (x_start, x_end) = lane_bounds_x(hold.lane());
    Ok(Some(HoldPrimitive {
        note_id: hold.id(),
        lane: hold.lane(),
        x_start,
        x_end,
        y_start: interval.y_start,
        y_end: interval.y_end,
        state,
        clipped_at_judgement: interval.clipped_at_judgement,
        clipped_at_horizon: interval.clipped_at_horizon,
        layer: RenderLayer::Holds,
        visible: true,
        debug_label: Some(format!("Hold NoteId={} {:?}", hold.id().as_u32(), state)),
        stable_order,
    }))
}

fn build_arc(
    arc: ArcNote,
    playback_time: ChartTime,
    state: RenderNoteState,
    config: ProjectionConfig,
    stable_order: usize,
) -> Result<Option<ArcPrimitive>, RenderError> {
    let Some(interval) = project_interval(playback_time, arc.start_time(), arc.end_time(), config)?
    else {
        return Ok(None);
    };

    let mut points = Vec::new();
    let start_ms = arc.start_time().as_millis();
    let duration = arc.end_time().as_millis() - start_ms;
    for step in 0..=config.arc_sample_steps {
        let progress = step as f32 / config.arc_sample_steps as f32;
        let sample_time =
            ChartTime::from_millis(start_ms + (duration as f32 * progress).round() as i64);
        let projected = project_time(playback_time, sample_time, config)?;
        if projected.visibility != TimeVisibility::Visible {
            continue;
        }
        let (x_progress, y_progress) = arc_axis_progress(arc.curve(), progress);
        let x = lerp(arc.start_x().as_f32(), arc.end_x().as_f32(), x_progress);
        let sky_y = lerp(arc.start_y().as_f32(), arc.end_y().as_f32(), y_progress);
        ensure_finite(x, "arc x")?;
        ensure_finite(sky_y, "arc sky y")?;
        points.push(ArcSamplePoint {
            position: NormalizedPoint::new(x, projected.depth)?,
            sky_y,
            progress,
        });
    }

    if points.is_empty() {
        return Ok(None);
    }

    Ok(Some(ArcPrimitive {
        note_id: arc.id(),
        points,
        curve: arc.curve(),
        color: arc.color(),
        is_trace: arc.is_trace(),
        state,
        clipped_at_judgement: interval.clipped_at_judgement,
        clipped_at_horizon: interval.clipped_at_horizon,
        layer: RenderLayer::Arcs,
        visible: true,
        debug_label: Some(format!("Arc NoteId={} {:?}", arc.id().as_u32(), state)),
        stable_order,
    }))
}

#[derive(Debug, Clone, Copy)]
struct ProjectedInterval {
    y_start: f32,
    y_end: f32,
    clipped_at_judgement: bool,
    clipped_at_horizon: bool,
}

fn project_interval(
    playback_time: ChartTime,
    start_time: ChartTime,
    end_time: ChartTime,
    config: ProjectionConfig,
) -> Result<Option<ProjectedInterval>, RenderError> {
    let start_delta = start_time.as_millis() - playback_time.as_millis();
    let end_delta = end_time.as_millis() - playback_time.as_millis();
    if end_delta < -config.past_window_ms || start_delta > config.future_window_ms {
        return Ok(None);
    }

    let clipped_start = start_delta.clamp(-config.past_window_ms, config.future_window_ms);
    let clipped_end = end_delta.clamp(-config.past_window_ms, config.future_window_ms);
    let y_start = depth_from_delta(clipped_start, config)?;
    let y_end = depth_from_delta(clipped_end, config)?;
    Ok(Some(ProjectedInterval {
        y_start: y_start.min(y_end),
        y_end: y_start.max(y_end),
        clipped_at_judgement: start_delta < 0 || end_delta < 0,
        clipped_at_horizon: start_delta > config.future_window_ms
            || end_delta > config.future_window_ms,
    }))
}

fn depth_from_delta(delta_ms: i64, config: ProjectionConfig) -> Result<f32, RenderError> {
    let depth = if delta_ms <= 0 {
        0.0
    } else {
        delta_ms as f32 / config.future_window_ms as f32
    };
    ensure_finite(depth, "interval depth")?;
    Ok(depth.clamp(0.0, 1.0))
}

fn summarize(primitives: &[RenderPrimitive], hidden_notes: usize) -> SceneSummary {
    let mut summary = SceneSummary {
        hidden_notes,
        primitive_count: primitives.len(),
        ..SceneSummary::default()
    };
    for primitive in primitives {
        match primitive {
            RenderPrimitive::Lane(_) => summary.lanes += 1,
            RenderPrimitive::Tap(tap) if tap.visible => summary.visible_taps += 1,
            RenderPrimitive::Hold(hold) if hold.visible => summary.visible_holds += 1,
            RenderPrimitive::Arc(arc) if arc.visible => summary.visible_arcs += 1,
            _ => {}
        }
    }
    summary
}

fn arc_axis_progress(curve: ArcCurve, progress: f32) -> (f32, f32) {
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

fn ensure_finite(value: f32, context: &'static str) -> Result<(), RenderError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(RenderError::NonFiniteCoordinate { value, context })
    }
}

#[derive(Debug, Clone, Copy)]
struct SvgPoint {
    x: f32,
    y: f32,
}

struct SvgConverter {
    horizon_y: f32,
    judgement_y: f32,
    center_x: f32,
    near_width: f32,
    far_width: f32,
}

impl SvgConverter {
    fn new(viewport: Viewport) -> Self {
        let width = viewport.width as f32;
        let height = viewport.height as f32;
        Self {
            horizon_y: 120.0,
            judgement_y: height - 120.0,
            center_x: width / 2.0,
            near_width: width - 240.0,
            far_width: (width - 240.0) * 0.58,
        }
    }

    fn point(&self, point: NormalizedPoint) -> SvgPoint {
        let depth = point.y.clamp(0.0, 1.0);
        let width_at_depth = self.near_width + ((self.far_width - self.near_width) * depth);
        SvgPoint {
            x: self.center_x + ((point.x.clamp(0.0, 1.0) - 0.5) * width_at_depth),
            y: self.judgement_y - ((self.judgement_y - self.horizon_y) * depth),
        }
    }

    fn segment(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> (SvgPoint, SvgPoint) {
        (
            self.point(NormalizedPoint { x: x1, y: y1 }),
            self.point(NormalizedPoint { x: x2, y: y2 }),
        )
    }

    fn lane_polygon(&self, x_start: f32, x_end: f32) -> [SvgPoint; 4] {
        [
            self.point(NormalizedPoint { x: x_start, y: 0.0 }),
            self.point(NormalizedPoint { x: x_end, y: 0.0 }),
            self.point(NormalizedPoint { x: x_end, y: 1.0 }),
            self.point(NormalizedPoint { x: x_start, y: 1.0 }),
        ]
    }

    fn playfield_polygon(&self) -> [SvgPoint; 4] {
        self.lane_polygon(0.0, 1.0)
    }
}

fn render_lane(svg: &mut String, converter: &SvgConverter, lane: &LanePrimitive) {
    let points = converter.lane_polygon(lane.x_start, lane.x_end);
    let class = if lane.lane.as_u8().is_multiple_of(2) {
        "lane alt"
    } else {
        "lane"
    };
    svg.push_str(&format!(
        "<polygon id=\"{}\" class=\"{}\" data-layer=\"{}\" points=\"{}\"><title>{}</title></polygon>\n",
        lane.id,
        class,
        lane.layer as u8,
        polygon_points(&points),
        escape_xml(lane.debug_label.as_deref().unwrap_or(""))
    ));
}

fn render_timing_marker(
    svg: &mut String,
    converter: &SvgConverter,
    marker: &TimingMarkerPrimitive,
) {
    let (left, right) = converter.segment(0.0, marker.position.y, 1.0, marker.position.y);
    svg.push_str(&format!(
        "<line id=\"timing-marker-{}\" class=\"timing\" data-time=\"{}\" data-layer=\"{}\" x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"><title>{}</title></line>\n",
        marker.time.as_millis(),
        marker.time.as_millis(),
        marker.layer as u8,
        left.x,
        left.y,
        right.x,
        right.y,
        escape_xml(marker.debug_label.as_deref().unwrap_or(""))
    ));
}

fn render_hold(svg: &mut String, converter: &SvgConverter, hold: &HoldPrimitive) {
    let y0 = hold.y_start;
    let y1 = if (hold.y_end - hold.y_start).abs() < EPSILON {
        (hold.y_end + 0.018).min(1.0)
    } else {
        hold.y_end
    };
    let points = [
        converter.point(NormalizedPoint {
            x: hold.x_start,
            y: y0,
        }),
        converter.point(NormalizedPoint {
            x: hold.x_end,
            y: y0,
        }),
        converter.point(NormalizedPoint {
            x: hold.x_end,
            y: y1,
        }),
        converter.point(NormalizedPoint {
            x: hold.x_start,
            y: y1,
        }),
    ];
    svg.push_str(&format!(
        "<polygon id=\"note-{}-hold\" class=\"hold {}\" data-note-id=\"{}\" data-layer=\"{}\" data-state=\"{:?}\" data-clipped-judgement=\"{}\" data-clipped-horizon=\"{}\" points=\"{}\"><title>{}</title></polygon>\n",
        hold.note_id.as_u32(),
        state_class(hold.state),
        hold.note_id.as_u32(),
        hold.layer as u8,
        hold.state,
        hold.clipped_at_judgement,
        hold.clipped_at_horizon,
        polygon_points(&points),
        escape_xml(hold.debug_label.as_deref().unwrap_or(""))
    ));
    render_debug_label(svg, converter, hold.debug_label.as_deref(), hold.x_end, y1);
}

fn render_arc(svg: &mut String, converter: &SvgConverter, arc: &ArcPrimitive) {
    let points: Vec<SvgPoint> = arc
        .points
        .iter()
        .map(|point| converter.point(point.position))
        .collect();
    svg.push_str(&format!(
        "<polyline id=\"note-{}-arc\" class=\"{} {}\" data-note-id=\"{}\" data-layer=\"{}\" data-state=\"{:?}\" data-curve=\"{:?}\" data-color=\"{:?}\" data-trace=\"{}\" data-samples=\"{}\" points=\"{}\"><title>{}</title></polyline>\n",
        arc.note_id.as_u32(),
        arc_color_class(arc.color),
        state_class(arc.state),
        arc.note_id.as_u32(),
        arc.layer as u8,
        arc.state,
        arc.curve,
        arc.color,
        arc.is_trace,
        arc.points.len(),
        polygon_points(&points),
        escape_xml(arc.debug_label.as_deref().unwrap_or(""))
    ));
    if let Some(last) = arc.points.last() {
        render_debug_label(
            svg,
            converter,
            arc.debug_label.as_deref(),
            last.position.x,
            last.position.y,
        );
    }
}

fn render_tap(svg: &mut String, converter: &SvgConverter, tap: &TapPrimitive) {
    let center = converter.point(tap.center);
    svg.push_str(&format!(
        "<rect id=\"note-{}-tap\" class=\"tap {}\" data-note-id=\"{}\" data-layer=\"{}\" data-state=\"{:?}\" x=\"{:.2}\" y=\"{:.2}\" width=\"42\" height=\"18\" rx=\"3\"><title>{}</title></rect>\n",
        tap.note_id.as_u32(),
        state_class(tap.state),
        tap.note_id.as_u32(),
        tap.layer as u8,
        tap.state,
        center.x - 21.0,
        center.y - 9.0,
        escape_xml(tap.debug_label.as_deref().unwrap_or(""))
    ));
    render_debug_label(
        svg,
        converter,
        tap.debug_label.as_deref(),
        tap.center.x,
        tap.center.y,
    );
}

fn render_judgement_line(
    svg: &mut String,
    converter: &SvgConverter,
    line: &JudgementLinePrimitive,
) {
    let (left, right) = converter.segment(0.0, line.y, 1.0, line.y);
    svg.push_str(&format!(
        "<line id=\"judgement-line\" class=\"judgement\" data-layer=\"{}\" data-playback-cursor=\"true\" x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"><title>{}</title></line>\n",
        line.layer as u8,
        left.x,
        left.y,
        right.x,
        right.y,
        escape_xml(line.debug_label.as_deref().unwrap_or(""))
    ));
}

fn render_label(svg: &mut String, converter: &SvgConverter, label: &LabelPrimitive) {
    let point = converter.point(label.anchor);
    svg.push_str(&format!(
        "<text class=\"label\" data-layer=\"{}\" x=\"{:.2}\" y=\"{:.2}\">{}</text>\n",
        label.layer as u8,
        point.x,
        point.y,
        escape_xml(&label.text)
    ));
}

fn render_debug_label(
    svg: &mut String,
    converter: &SvgConverter,
    label: Option<&str>,
    x: f32,
    y: f32,
) {
    if let Some(label) = label {
        let point = converter.point(NormalizedPoint { x, y });
        svg.push_str(&format!(
            "<text class=\"label\" x=\"{:.2}\" y=\"{:.2}\">{}</text>\n",
            point.x + 8.0,
            point.y - 8.0,
            escape_xml(label)
        ));
    }
}

fn polygon_points(points: &[SvgPoint]) -> String {
    points
        .iter()
        .map(|point| format!("{:.2},{:.2}", point.x, point.y))
        .collect::<Vec<_>>()
        .join(" ")
}

fn state_class(state: RenderNoteState) -> &'static str {
    match state {
        RenderNoteState::Upcoming => "upcoming",
        RenderNoteState::Active => "active",
        RenderNoteState::Passed => "passed",
    }
}

fn arc_color_class(color: ArcColor) -> &'static str {
    match color {
        ArcColor::Blue => "arc-blue",
        ArcColor::Red => "arc-red",
        ArcColor::Green => "arc-green",
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use arcaea_viewer_core::{ArcPath, ArcX, ArcY, HoldNote, TapNote, Tempo, TimingEvent};

    #[test]
    fn projection_maps_playback_to_judgement_line() {
        let projected = project_time(time(2_500), time(2_500), config()).expect("projection");
        assert_eq!(projected.visibility, TimeVisibility::Visible);
        assert_eq!(projected.depth, 0.0);
    }

    #[test]
    fn projection_maps_future_edge_to_horizon() {
        let projected = project_time(time(2_500), time(4_500), config()).expect("projection");
        assert_eq!(projected.visibility, TimeVisibility::Visible);
        assert!((projected.depth - 1.0).abs() < EPSILON);
    }

    #[test]
    fn projection_clips_past_window_to_judgement_line() {
        let projected = project_time(time(2_500), time(2_000), config()).expect("projection");
        assert_eq!(projected.visibility, TimeVisibility::Visible);
        assert_eq!(projected.depth, 0.0);
        assert!(projected.clipped_at_judgement);
    }

    #[test]
    fn projection_hides_outside_time_window() {
        assert_eq!(
            project_time(time(2_500), time(1_999), config())
                .expect("projection")
                .visibility,
            TimeVisibility::HiddenPast
        );
        assert_eq!(
            project_time(time(2_500), time(4_501), config())
                .expect("projection")
                .visibility,
            TimeVisibility::HiddenFuture
        );
    }

    #[test]
    fn projection_supports_negative_chart_time() {
        let projected = project_time(time(-250), time(250), config()).expect("projection");
        assert_eq!(projected.visibility, TimeVisibility::Visible);
        assert!((projected.depth - 0.25).abs() < EPSILON);
    }

    #[test]
    fn projection_is_deterministic() {
        let first = project_time(time(100), time(600), config()).expect("projection");
        let second = project_time(time(100), time(600), config()).expect("projection");
        assert_eq!(first, second);
    }

    #[test]
    fn invalid_projection_configuration_returns_structured_error() {
        assert!(matches!(
            ProjectionConfig::new(0, 0, 1280, 720, 16),
            Err(RenderError::InvalidProjectionWindow { .. })
        ));
        assert!(matches!(
            ProjectionConfig::new(0, 1_000, 0, 720, 16),
            Err(RenderError::InvalidViewport { .. })
        ));
        assert!(matches!(
            ProjectionConfig::new(0, 1_000, 1280, 720, 0),
            Err(RenderError::InvalidArcSampling { .. })
        ));
    }

    #[test]
    fn lanes_have_four_ground_primitives() {
        let scene = scene_at(2_500);
        assert_eq!(scene.metadata.summary.lanes, 4);
        assert_eq!(
            scene
                .primitives
                .iter()
                .filter(|primitive| matches!(primitive, RenderPrimitive::Lane(_)))
                .count(),
            4
        );
    }

    #[test]
    fn lane_centers_are_ordered_and_within_viewport() {
        let centers: Vec<f32> = (Lane::MIN..=Lane::MAX)
            .map(|value| lane_center_x(Lane::new(value).expect("lane")))
            .collect();
        assert_eq!(centers, vec![0.125, 0.375, 0.625, 0.875]);
        assert!(centers.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(centers.iter().all(|value| (0.0..=1.0).contains(value)));
    }

    #[test]
    fn lane_mapping_uses_one_based_lane_values() {
        let lane_one = Lane::new(1).expect("lane");
        let lane_four = Lane::new(4).expect("lane");
        assert_eq!(lane_bounds_x(lane_one), (0.0, 0.25));
        assert_eq!(lane_bounds_x(lane_four), (0.75, 1.0));
    }

    #[test]
    fn tap_primitive_includes_note_id_and_lane_position() {
        let scene = scene_at(500);
        let tap = only_tap(&scene);
        assert_eq!(tap.note_id.as_u32(), 0);
        assert_eq!(tap.lane.as_u8(), 1);
        assert!((tap.center.x - 0.125).abs() < EPSILON);
        assert_eq!(tap.state, RenderNoteState::Upcoming);
    }

    #[test]
    fn passed_tap_inside_window_is_visible_at_judgement_line() {
        let scene = scene_at(1_250);
        let tap = only_tap(&scene);
        assert_eq!(tap.state, RenderNoteState::Passed);
        assert_eq!(tap.center.y, 0.0);
    }

    #[test]
    fn tap_outside_time_window_is_hidden() {
        let scene = scene_at(2_500);
        assert!(
            !scene
                .primitives
                .iter()
                .any(|primitive| matches!(primitive, RenderPrimitive::Tap(_)))
        );
        assert_eq!(scene.metadata.summary.hidden_notes, 1);
    }

    #[test]
    fn hold_visible_interval_is_projected() {
        let scene = scene_at(1_000);
        let hold = only_hold(&scene);
        assert_eq!(hold.note_id.as_u32(), 1);
        assert!((hold.y_start - 0.25).abs() < EPSILON);
        assert!((hold.y_end - 1.0).abs() < EPSILON);
    }

    #[test]
    fn hold_clips_at_judgement_line_when_active() {
        let scene = scene_at(2_500);
        let hold = only_hold(&scene);
        assert_eq!(hold.state, RenderNoteState::Active);
        assert_eq!(hold.y_start, 0.0);
        assert!((hold.y_end - 0.25).abs() < EPSILON);
        assert!(hold.clipped_at_judgement);
    }

    #[test]
    fn hold_clips_at_horizon() {
        let scene = scene_at(750);
        let hold = only_hold(&scene);
        assert!(hold.clipped_at_horizon);
        assert_eq!(hold.y_end, 1.0);
    }

    #[test]
    fn passed_hold_inside_past_window_is_visible() {
        let scene = scene_at(3_250);
        let hold = only_hold(&scene);
        assert_eq!(hold.state, RenderNoteState::Passed);
        assert_eq!(hold.y_start, 0.0);
        assert_eq!(hold.y_end, 0.0);
    }

    #[test]
    fn zero_duration_hold_is_safe() {
        let chart = Chart::new(vec![
            ChartEvent::Timing(timing(0)),
            ChartEvent::Hold(
                HoldNote::new(
                    NoteId::new(7),
                    time(1_000),
                    time(1_000),
                    Lane::new(2).expect("lane"),
                )
                .expect("hold"),
            ),
        ]);
        let map = TimingMap::from_chart(&chart).expect("timing");
        let scene = build_scene(&chart, &map, time(1_000), "zero.aff", config()).expect("scene");
        let hold = only_hold(&scene);
        assert_eq!(hold.y_start, 0.0);
        assert_eq!(hold.y_end, 0.0);
    }

    #[test]
    fn zero_duration_arc_is_safe_and_finite() {
        let chart = Chart::new(vec![
            ChartEvent::Timing(timing(0)),
            ChartEvent::Arc(
                ArcNote::new(
                    NoteId::new(8),
                    time(1_000),
                    time(1_000),
                    ArcPath::new(
                        ArcX::new(0.25).expect("x"),
                        ArcX::new(0.75).expect("x"),
                        ArcY::new(0.50).expect("y"),
                        ArcY::new(1.00).expect("y"),
                    ),
                    ArcCurve::Straight,
                    ArcColor::Blue,
                    false,
                )
                .expect("arc"),
            ),
        ]);
        let map = TimingMap::from_chart(&chart).expect("timing");
        let scene = build_scene(&chart, &map, time(1_000), "zero.aff", config()).expect("scene");
        let arc = only_arc(&scene);

        assert_eq!(arc.note_id.as_u32(), 8);
        assert_eq!(arc.state, RenderNoteState::Active);
        assert!(arc.points.iter().all(|point| {
            point.position.x.is_finite() && point.position.y.is_finite() && point.sky_y.is_finite()
        }));
    }

    #[test]
    fn straight_arc_preserves_endpoints() {
        let chart = chart_with_arc_curve(ArcCurve::Straight);
        let map = TimingMap::from_chart(&chart).expect("timing");
        let scene = build_scene(&chart, &map, time(3_200), "arc.aff", config()).expect("scene");
        let arc = only_arc(&scene);
        let first = arc.points.first().expect("first");
        let last = arc.points.last().expect("last");
        assert!((first.position.x - 0.25).abs() < EPSILON);
        assert!((first.sky_y - 0.5).abs() < EPSILON);
        assert!((last.position.x - 0.75).abs() < EPSILON);
        assert!((last.sky_y - 1.0).abs() < EPSILON);
    }

    #[test]
    fn all_supported_arc_curves_generate_finite_points() {
        for curve in [
            ArcCurve::Straight,
            ArcCurve::Bezier,
            ArcCurve::SineIn,
            ArcCurve::SineOut,
            ArcCurve::SineInOut,
            ArcCurve::SineOutIn,
        ] {
            let chart = chart_with_arc_curve(curve);
            let map = TimingMap::from_chart(&chart).expect("timing");
            let scene = build_scene(&chart, &map, time(3_200), "arc.aff", config()).expect("scene");
            let arc = only_arc(&scene);
            assert!(arc.points.iter().all(|point| {
                point.position.x.is_finite()
                    && point.position.y.is_finite()
                    && point.sky_y.is_finite()
            }));
        }
    }

    #[test]
    fn sine_in_arc_eases_horizontal_axis_and_keeps_vertical_axis_linear() {
        let chart = chart_with_arc_curve(ArcCurve::SineIn);
        let map = TimingMap::from_chart(&chart).expect("timing");
        let scene = build_scene(&chart, &map, time(3_200), "arc.aff", config()).expect("scene");
        let arc = only_arc(&scene);
        let middle = arc
            .points
            .iter()
            .find(|point| (point.progress - 0.5).abs() < EPSILON)
            .expect("middle sample");

        assert!((middle.position.x - 0.6035534).abs() < EPSILON);
        assert!((middle.sky_y - 0.75).abs() < EPSILON);
    }

    #[test]
    fn sine_in_out_arc_eases_both_axes() {
        let chart = chart_with_arc_curve(ArcCurve::SineInOut);
        let map = TimingMap::from_chart(&chart).expect("timing");
        let scene = build_scene(&chart, &map, time(3_200), "arc.aff", config()).expect("scene");
        let arc = only_arc(&scene);
        let middle = arc
            .points
            .iter()
            .find(|point| (point.progress - 0.5).abs() < EPSILON)
            .expect("middle sample");

        assert!((middle.position.x - 0.6035534).abs() < EPSILON);
        assert!((middle.sky_y - 0.8535534).abs() < EPSILON);
    }

    #[test]
    fn arc_sample_count_is_deterministic() {
        let scene = scene_at(3_200);
        let arc = only_arc(&scene);
        assert_eq!(arc.points.len(), config().arc_sample_steps + 1);
    }

    #[test]
    fn arc_partial_visibility_is_clipped_by_time_window() {
        let scene = scene_at(4_800);
        let arc = only_arc(&scene);
        assert_eq!(arc.state, RenderNoteState::Active);
        assert!(arc.clipped_at_judgement);
        assert!(arc.points.len() < config().arc_sample_steps + 1);
        assert!(arc.points.iter().all(|point| point.position.y >= 0.0));
    }

    #[test]
    fn arc_color_and_trace_metadata_are_preserved() {
        let chart = Chart::new(vec![
            ChartEvent::Timing(timing(0)),
            ChartEvent::Arc(arc_note(2, ArcCurve::SineInOut, ArcColor::Green, true)),
        ]);
        let map = TimingMap::from_chart(&chart).expect("timing");
        let scene = build_scene(&chart, &map, time(3_200), "arc.aff", config()).expect("scene");
        let arc = only_arc(&scene);
        assert_eq!(arc.color, ArcColor::Green);
        assert!(arc.is_trace);
    }

    #[test]
    fn scene_primitive_ordering_is_deterministic_and_layered() {
        let scene = scene_at(2_500);
        let layers: Vec<RenderLayer> = scene
            .primitives
            .iter()
            .map(RenderPrimitive::layer)
            .collect();
        assert!(layers.windows(2).all(|pair| pair[0] <= pair[1]));
        assert_eq!(scene_at(2_500), scene_at(2_500));
    }

    #[test]
    fn scene_build_does_not_mutate_chart() {
        let (chart, map) = sample_chart();
        let original = chart.clone();
        let _ = build_scene(&chart, &map, time(2_500), "mixed.aff", config()).expect("scene");
        assert_eq!(chart, original);
    }

    #[test]
    fn primitive_layers_are_stable() {
        assert!((RenderLayer::Lanes as u8) < (RenderLayer::Holds as u8));
        assert!((RenderLayer::Holds as u8) < (RenderLayer::Arcs as u8));
        assert!((RenderLayer::Taps as u8) < (RenderLayer::JudgementLine as u8));
    }

    #[test]
    fn svg_contains_required_structure() {
        let scene = scene_at(2_500);
        let svg = render_scene_to_svg(&scene).expect("svg");
        assert!(svg.contains("<svg"));
        assert!(svg.contains("width=\"1280\""));
        assert!(svg.contains("height=\"720\""));
        assert!(svg.contains("lane-1"));
        assert!(svg.contains("lane-2"));
        assert!(svg.contains("lane-3"));
        assert!(svg.contains("lane-4"));
        assert!(svg.contains("judgement-line"));
        assert!(svg.contains("Playback: 2500ms"));
        assert!(svg.contains("note-1-hold"));
        assert!(svg.contains("note-2-arc"));
        assert!(svg.contains("data-note-id=\"1\""));
        assert!(svg.contains("data-playback-cursor=\"true\""));
        assert!(svg.contains("Hold NoteId=1"));
        assert!(svg.contains("Arc NoteId=2"));
        assert!(svg.len() > 1_000);
    }

    #[test]
    fn fixture_style_scene_at_2500_can_show_tap_hold_and_arc_together() {
        let chart = Chart::new(vec![
            ChartEvent::Timing(timing(0)),
            ChartEvent::Tap(tap()),
            ChartEvent::Hold(hold()),
            ChartEvent::Arc(arc_note(2, ArcCurve::SineInOut, ArcColor::Blue, false)),
            ChartEvent::Tap(TapNote::new(
                NoteId::new(3),
                time(2_600),
                Lane::new(4).expect("lane"),
            )),
        ]);
        let map = TimingMap::from_chart(&chart).expect("timing");
        let scene =
            build_scene(&chart, &map, time(2_500), "mixed_events.aff", config()).expect("scene");
        let svg = render_scene_to_svg(&scene).expect("svg");

        assert_eq!(scene.metadata.summary.visible_taps, 1);
        assert_eq!(scene.metadata.summary.visible_holds, 1);
        assert_eq!(scene.metadata.summary.visible_arcs, 1);
        assert!(svg.contains("note-3-tap"));
        assert!(svg.contains("Tap NoteId=3"));
        assert!(svg.contains("note-1-hold"));
        assert!(svg.contains("note-2-arc"));
    }

    #[test]
    fn invalid_svg_output_path_returns_structured_error() {
        let scene = scene_at(2_500);
        let base = std::env::temp_dir().join(format!(
            "arcaea-viewer-renderer-test-{}",
            std::process::id()
        ));
        fs::write(&base, "not a directory").expect("temp marker");
        let invalid_path = base.join("child.svg");
        let error = write_scene_svg(&scene, &invalid_path).expect_err("invalid path");
        assert!(matches!(error, RenderError::Io { .. }));
        let _ = fs::remove_file(base);
    }

    #[test]
    fn generated_svg_file_is_non_empty() {
        let scene = scene_at(2_500);
        let path = std::env::temp_dir().join(format!(
            "arcaea-viewer-renderer-svg-{}.svg",
            std::process::id()
        ));
        write_scene_svg(&scene, &path).expect("write");
        let content = fs::read_to_string(&path).expect("read");
        assert!(content.starts_with("<?xml"));
        assert!(content.contains("<svg"));
        assert!(content.len() > 1_000);
        let _ = fs::remove_file(path);
    }

    fn config() -> ProjectionConfig {
        ProjectionConfig::default()
    }

    fn time(value: i64) -> ChartTime {
        ChartTime::from_millis(value)
    }

    fn timing(value: i64) -> TimingEvent {
        TimingEvent::new(
            time(value),
            Tempo::from_milli_bpm(120_000).expect("tempo"),
            4,
        )
    }

    fn tap() -> TapNote {
        TapNote::new(NoteId::new(0), time(1_000), Lane::new(1).expect("lane"))
    }

    fn hold() -> HoldNote {
        HoldNote::new(
            NoteId::new(1),
            time(1_500),
            time(3_000),
            Lane::new(2).expect("lane"),
        )
        .expect("hold")
    }

    fn arc_note(id: u32, curve: ArcCurve, color: ArcColor, is_trace: bool) -> ArcNote {
        ArcNote::new(
            NoteId::new(id),
            time(3_200),
            time(5_000),
            ArcPath::new(
                ArcX::new(0.25).expect("x"),
                ArcX::new(0.75).expect("x"),
                ArcY::new(0.50).expect("y"),
                ArcY::new(1.00).expect("y"),
            ),
            curve,
            color,
            is_trace,
        )
        .expect("arc")
    }

    fn sample_chart() -> (Chart, TimingMap) {
        let chart = Chart::new(vec![
            ChartEvent::Timing(timing(0)),
            ChartEvent::Tap(tap()),
            ChartEvent::Hold(hold()),
            ChartEvent::Arc(arc_note(2, ArcCurve::SineInOut, ArcColor::Blue, false)),
        ]);
        let map = TimingMap::from_chart(&chart).expect("timing");
        (chart, map)
    }

    fn chart_with_arc_curve(curve: ArcCurve) -> Chart {
        Chart::new(vec![
            ChartEvent::Timing(timing(0)),
            ChartEvent::Arc(arc_note(2, curve, ArcColor::Blue, false)),
        ])
    }

    fn scene_at(millis: i64) -> RenderScene {
        let (chart, map) = sample_chart();
        build_scene(&chart, &map, time(millis), "mixed_events.aff", config()).expect("scene")
    }

    fn only_tap(scene: &RenderScene) -> &TapPrimitive {
        scene
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                RenderPrimitive::Tap(tap) => Some(tap),
                _ => None,
            })
            .expect("tap")
    }

    fn only_hold(scene: &RenderScene) -> &HoldPrimitive {
        scene
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                RenderPrimitive::Hold(hold) => Some(hold),
                _ => None,
            })
            .expect("hold")
    }

    fn only_arc(scene: &RenderScene) -> &ArcPrimitive {
        scene
            .primitives
            .iter()
            .find_map(|primitive| match primitive {
                RenderPrimitive::Arc(arc) => Some(arc),
                _ => None,
            })
            .expect("arc")
    }
}
