//! Browser-facing Rust and WebAssembly boundary for Arcaea-Viewer.
//!
//! The public exports intentionally return versioned JSON envelopes. Parser,
//! timing, and renderer domain crates remain browser-agnostic; this crate owns
//! the DTO conversion layer used by JavaScript.

use arcaea_viewer_core::{Chart, ChartEvent, ChartTime};
use arcaea_viewer_parser::{Diagnostic, SourceMap, parse_chart};
use arcaea_viewer_renderer::{
    ProjectionConfig, RenderError, RenderLayer, RenderNoteState, RenderPrimitive, RenderScene,
    build_scene_with_timing_context, render_scene_to_svg,
};
use arcaea_viewer_timing::{
    NoteState, PlaybackSnapshot, SnapshotNoteKind, TimingContext, TimingError,
    snapshot_with_context,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

pub const CONTRACT_VERSION: u32 = 1;

const MAX_VIEWPORT_DIMENSION: u32 = 8_192;
const MAX_WINDOW_MS: i64 = 120_000;
const MAX_ARC_SAMPLE_STEPS: usize = 256;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Envelope<T> {
    pub contract_version: u32,
    pub ok: bool,
    pub data: Option<T>,
    pub diagnostics: Vec<DiagnosticDto>,
    pub error: Option<BoundaryErrorDto>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BoundaryErrorDto {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticDto {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub span_start: usize,
    pub span_end: usize,
    pub note: Option<String>,
    pub help: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParseData {
    pub summary: ChartSummaryDto,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChartSummaryDto {
    pub event_count: usize,
    pub timing_group_count: usize,
    pub timing_events: usize,
    pub taps: usize,
    pub holds: usize,
    pub arcs: usize,
    pub arc_taps: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackSnapshotData {
    pub playback_ms: i64,
    pub tempo_milli_bpm: u32,
    pub beat_position: f64,
    pub summary: PlaybackSummaryDto,
    pub notes: Vec<SnapshotNoteDto>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackSummaryDto {
    pub upcoming: usize,
    pub active: usize,
    pub passed: usize,
    pub note_count: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotNoteDto {
    pub id: u32,
    pub kind: String,
    pub source_index: usize,
    pub timing_group_id: u32,
    pub parent_arc_id: Option<u32>,
    pub start_ms: i64,
    pub end_ms: i64,
    pub state: String,
    pub progress: Option<f64>,
    pub starts_in_ms: Option<i64>,
    pub since_end_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RenderRequestDto {
    #[serde(default)]
    pub playback_ms: i64,
    #[serde(default = "default_fixture_name")]
    pub fixture_name: String,
    #[serde(default = "default_past_window_ms")]
    pub past_window_ms: i64,
    #[serde(default = "default_future_window_ms")]
    pub future_window_ms: i64,
    #[serde(default = "default_viewport_width")]
    pub viewport_width: u32,
    #[serde(default = "default_viewport_height")]
    pub viewport_height: u32,
    #[serde(default = "default_arc_sample_steps")]
    pub arc_sample_steps: usize,
    #[serde(default = "default_debug_labels")]
    pub debug_labels: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderSceneData {
    pub request: RenderRequestDto,
    pub scene: SceneDto,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderSvgData {
    pub request: RenderRequestDto,
    pub scene: SceneDto,
    pub svg: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SceneDto {
    pub fixture_name: String,
    pub playback_ms: i64,
    pub visible_time_start_ms: i64,
    pub visible_time_end_ms: i64,
    pub viewport: ViewportDto,
    pub summary: SceneSummaryDto,
    pub primitives: Vec<PrimitiveDto>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ViewportDto {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SceneSummaryDto {
    pub lanes: usize,
    pub visible_taps: usize,
    pub visible_holds: usize,
    pub visible_arcs: usize,
    pub visible_arc_taps: usize,
    pub hidden_notes: usize,
    pub primitive_count: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrimitiveDto {
    pub index: usize,
    pub kind: String,
    pub layer: u8,
    pub layer_name: String,
    pub visible: bool,
    pub note_id: Option<u32>,
    pub parent_arc_id: Option<u32>,
    pub timing_group_id: Option<u32>,
    pub lane: Option<u8>,
    pub state: Option<String>,
    pub point_count: Option<usize>,
}

#[wasm_bindgen]
pub fn parse_chart_json(source: &str) -> String {
    to_json(&parse_chart_envelope(source))
}

#[wasm_bindgen]
pub fn build_playback_snapshot_json(source: &str, playback_ms: i64) -> String {
    to_json(&build_playback_snapshot_envelope(source, playback_ms))
}

#[wasm_bindgen]
pub fn build_render_scene_json(source: &str, request_json: &str) -> String {
    to_json(&build_render_scene_envelope(source, request_json))
}

#[wasm_bindgen]
pub fn render_chart_svg(source: &str, request_json: &str) -> String {
    to_json(&render_chart_svg_envelope(source, request_json))
}

pub fn parse_chart_envelope(source: &str) -> Envelope<ParseData> {
    match parse_chart(source) {
        Ok(chart) => success(ParseData {
            summary: chart_summary(&chart),
        }),
        Err(diagnostics) => failure_with_diagnostics(diagnostic_dtos(source, &diagnostics)),
    }
}

pub fn build_playback_snapshot_envelope(
    source: &str,
    playback_ms: i64,
) -> Envelope<PlaybackSnapshotData> {
    match parse_chart_and_timing(source) {
        Ok((chart, timing_context)) => {
            let snapshot =
                snapshot_with_context(&chart, &timing_context, ChartTime::from_millis(playback_ms));
            success(playback_snapshot_dto(&snapshot))
        }
        Err(PipelineError::Parse(diagnostics)) => {
            failure_with_diagnostics(diagnostic_dtos(source, &diagnostics))
        }
        Err(error) => failure(error.into_boundary_error()),
    }
}

pub fn build_render_scene_envelope(source: &str, request_json: &str) -> Envelope<RenderSceneData> {
    let request = match parse_render_request(request_json) {
        Ok(request) => request,
        Err(error) => return failure(error.into_boundary_error()),
    };
    match build_scene_pipeline(source, &request) {
        Ok(scene) => success(RenderSceneData {
            request,
            scene: scene_dto(&scene),
        }),
        Err(PipelineError::Parse(diagnostics)) => {
            failure_with_diagnostics(diagnostic_dtos(source, &diagnostics))
        }
        Err(error) => failure(error.into_boundary_error()),
    }
}

pub fn render_chart_svg_envelope(source: &str, request_json: &str) -> Envelope<RenderSvgData> {
    let request = match parse_render_request(request_json) {
        Ok(request) => request,
        Err(error) => return failure(error.into_boundary_error()),
    };
    match build_scene_pipeline(source, &request) {
        Ok(scene) => match render_scene_to_svg(&scene) {
            Ok(svg) => success(RenderSvgData {
                request,
                scene: scene_dto(&scene),
                svg,
            }),
            Err(error) => failure(PipelineError::Render(error).into_boundary_error()),
        },
        Err(PipelineError::Parse(diagnostics)) => {
            failure_with_diagnostics(diagnostic_dtos(source, &diagnostics))
        }
        Err(error) => failure(error.into_boundary_error()),
    }
}

fn parse_chart_and_timing(source: &str) -> Result<(Chart, TimingContext), PipelineError> {
    let chart = parse_chart(source).map_err(PipelineError::Parse)?;
    let timing_context = TimingContext::from_chart(&chart).map_err(PipelineError::Timing)?;
    Ok((chart, timing_context))
}

fn build_scene_pipeline(
    source: &str,
    request: &RenderRequestDto,
) -> Result<RenderScene, PipelineError> {
    let config = projection_config(request)?;
    let (chart, timing_context) = parse_chart_and_timing(source)?;
    build_scene_with_timing_context(
        &chart,
        &timing_context,
        ChartTime::from_millis(request.playback_ms),
        request.fixture_name.clone(),
        config,
    )
    .map_err(PipelineError::Render)
}

fn parse_render_request(request_json: &str) -> Result<RenderRequestDto, PipelineError> {
    let request: RenderRequestDto =
        serde_json::from_str(request_json).map_err(|source| PipelineError::Boundary {
            code: "MALFORMED_JSON",
            message: format!("render request is not valid JSON: {source}"),
        })?;
    validate_request(&request)?;
    Ok(request)
}

fn validate_request(request: &RenderRequestDto) -> Result<(), PipelineError> {
    if request.fixture_name.trim().is_empty() {
        return Err(PipelineError::Boundary {
            code: "INVALID_FIXTURE_NAME",
            message: "fixtureName must not be empty".to_owned(),
        });
    }
    if request.past_window_ms < 0 || request.past_window_ms > MAX_WINDOW_MS {
        return Err(PipelineError::Boundary {
            code: "INVALID_PAST_WINDOW",
            message: format!(
                "pastWindowMs must be in 0..={MAX_WINDOW_MS}, got {}",
                request.past_window_ms
            ),
        });
    }
    if request.future_window_ms <= 0 || request.future_window_ms > MAX_WINDOW_MS {
        return Err(PipelineError::Boundary {
            code: "INVALID_FUTURE_WINDOW",
            message: format!(
                "futureWindowMs must be in 1..={MAX_WINDOW_MS}, got {}",
                request.future_window_ms
            ),
        });
    }
    if request.viewport_width == 0 || request.viewport_height == 0 {
        return Err(PipelineError::Boundary {
            code: "INVALID_VIEWPORT",
            message: "viewportWidth and viewportHeight must be greater than zero".to_owned(),
        });
    }
    if request.viewport_width > MAX_VIEWPORT_DIMENSION
        || request.viewport_height > MAX_VIEWPORT_DIMENSION
    {
        return Err(PipelineError::Boundary {
            code: "INVALID_VIEWPORT",
            message: format!(
                "viewport dimensions must be <= {MAX_VIEWPORT_DIMENSION}, got {}x{}",
                request.viewport_width, request.viewport_height
            ),
        });
    }
    if request.arc_sample_steps == 0 || request.arc_sample_steps > MAX_ARC_SAMPLE_STEPS {
        return Err(PipelineError::Boundary {
            code: "INVALID_ARC_SAMPLE_STEPS",
            message: format!(
                "arcSampleSteps must be in 1..={MAX_ARC_SAMPLE_STEPS}, got {}",
                request.arc_sample_steps
            ),
        });
    }
    Ok(())
}

fn projection_config(request: &RenderRequestDto) -> Result<ProjectionConfig, PipelineError> {
    ProjectionConfig::new(
        request.past_window_ms,
        request.future_window_ms,
        request.viewport_width,
        request.viewport_height,
        request.arc_sample_steps,
    )
    .map_err(PipelineError::Render)
}

fn chart_summary(chart: &Chart) -> ChartSummaryDto {
    let mut summary = ChartSummaryDto {
        event_count: chart.len(),
        timing_group_count: chart.timing_groups().len(),
        timing_events: 0,
        taps: 0,
        holds: 0,
        arcs: 0,
        arc_taps: 0,
    };
    for event in chart.events() {
        match event {
            ChartEvent::Timing(_) => summary.timing_events += 1,
            ChartEvent::Tap(_) => summary.taps += 1,
            ChartEvent::Hold(_) => summary.holds += 1,
            ChartEvent::Arc(_) => summary.arcs += 1,
            ChartEvent::ArcTap(_) => summary.arc_taps += 1,
        }
    }
    summary
}

fn diagnostic_dtos(source: &str, diagnostics: &[Diagnostic]) -> Vec<DiagnosticDto> {
    let source_map = SourceMap::new(source);
    diagnostics
        .iter()
        .map(|diagnostic| {
            let location = source_map.line_column(diagnostic.span.start);
            DiagnosticDto {
                code: diagnostic.code.to_string(),
                severity: diagnostic.severity.to_string(),
                message: diagnostic.message.clone(),
                line: location.line,
                column: location.column,
                span_start: diagnostic.span.start,
                span_end: diagnostic.span.end,
                note: diagnostic.note.clone(),
                help: diagnostic.help.clone(),
            }
        })
        .collect()
}

fn playback_snapshot_dto(snapshot: &PlaybackSnapshot) -> PlaybackSnapshotData {
    PlaybackSnapshotData {
        playback_ms: snapshot.playback_time.as_millis(),
        tempo_milli_bpm: snapshot.tempo.as_milli_bpm(),
        beat_position: snapshot.beat_position.as_f64(),
        summary: PlaybackSummaryDto {
            upcoming: snapshot.count_state(NoteState::Upcoming),
            active: snapshot.count_state(NoteState::Active),
            passed: snapshot.count_state(NoteState::Passed),
            note_count: snapshot.notes.len(),
        },
        notes: snapshot
            .notes
            .iter()
            .map(|note| SnapshotNoteDto {
                id: note.id.as_u32(),
                kind: snapshot_kind_label(note.kind).to_owned(),
                source_index: note.source_index,
                timing_group_id: note.timing_group.as_u32(),
                parent_arc_id: note.parent_arc_id.map(|id| id.as_u32()),
                start_ms: note.start_time.as_millis(),
                end_ms: note.end_time.as_millis(),
                state: note_state_label(note.state).to_owned(),
                progress: note.progress,
                starts_in_ms: note.starts_in_millis,
                since_end_ms: note.since_end_millis,
            })
            .collect(),
    }
}

fn scene_dto(scene: &RenderScene) -> SceneDto {
    SceneDto {
        fixture_name: scene.metadata.fixture_name.clone(),
        playback_ms: scene.playback_time.as_millis(),
        visible_time_start_ms: scene.metadata.visible_time_start.as_millis(),
        visible_time_end_ms: scene.metadata.visible_time_end.as_millis(),
        viewport: ViewportDto {
            width: scene.viewport.width,
            height: scene.viewport.height,
        },
        summary: SceneSummaryDto {
            lanes: scene.metadata.summary.lanes,
            visible_taps: scene.metadata.summary.visible_taps,
            visible_holds: scene.metadata.summary.visible_holds,
            visible_arcs: scene.metadata.summary.visible_arcs,
            visible_arc_taps: scene.metadata.summary.visible_arc_taps,
            hidden_notes: scene.metadata.summary.hidden_notes,
            primitive_count: scene.metadata.summary.primitive_count,
        },
        primitives: scene
            .primitives
            .iter()
            .enumerate()
            .map(primitive_dto)
            .collect(),
    }
}

fn primitive_dto((index, primitive): (usize, &RenderPrimitive)) -> PrimitiveDto {
    let base = PrimitiveDto {
        index,
        kind: primitive_kind(primitive).to_owned(),
        layer: primitive.layer() as u8,
        layer_name: layer_label(primitive.layer()).to_owned(),
        visible: primitive_visible(primitive),
        note_id: None,
        parent_arc_id: None,
        timing_group_id: None,
        lane: None,
        state: None,
        point_count: None,
    };
    match primitive {
        RenderPrimitive::Lane(lane) => PrimitiveDto {
            lane: Some(lane.lane.as_u8()),
            ..base
        },
        RenderPrimitive::JudgementLine(_) | RenderPrimitive::TimingMarker(_) => base,
        RenderPrimitive::Tap(tap) => PrimitiveDto {
            note_id: Some(tap.note_id.as_u32()),
            timing_group_id: Some(tap.timing_group.as_u32()),
            lane: Some(tap.lane.as_u8()),
            state: Some(render_state_label(tap.state).to_owned()),
            ..base
        },
        RenderPrimitive::Hold(hold) => PrimitiveDto {
            note_id: Some(hold.note_id.as_u32()),
            timing_group_id: Some(hold.timing_group.as_u32()),
            lane: Some(hold.lane.as_u8()),
            state: Some(render_state_label(hold.state).to_owned()),
            ..base
        },
        RenderPrimitive::Arc(arc) => PrimitiveDto {
            note_id: Some(arc.note_id.as_u32()),
            timing_group_id: Some(arc.timing_group.as_u32()),
            state: Some(render_state_label(arc.state).to_owned()),
            point_count: Some(arc.points.len()),
            ..base
        },
        RenderPrimitive::ArcTap(arc_tap) => PrimitiveDto {
            note_id: Some(arc_tap.note_id.as_u32()),
            parent_arc_id: Some(arc_tap.parent_arc_id.as_u32()),
            timing_group_id: Some(arc_tap.timing_group.as_u32()),
            state: Some(render_state_label(arc_tap.state).to_owned()),
            ..base
        },
        RenderPrimitive::Label(_) => base,
    }
}

fn primitive_visible(primitive: &RenderPrimitive) -> bool {
    match primitive {
        RenderPrimitive::Lane(value) => value.visible,
        RenderPrimitive::JudgementLine(value) => value.visible,
        RenderPrimitive::Tap(value) => value.visible,
        RenderPrimitive::Hold(value) => value.visible,
        RenderPrimitive::Arc(value) => value.visible,
        RenderPrimitive::ArcTap(value) => value.visible,
        RenderPrimitive::TimingMarker(value) => value.visible,
        RenderPrimitive::Label(value) => value.visible,
    }
}

fn success<T>(data: T) -> Envelope<T> {
    Envelope {
        contract_version: CONTRACT_VERSION,
        ok: true,
        data: Some(data),
        diagnostics: Vec::new(),
        error: None,
    }
}

fn failure<T>(error: BoundaryErrorDto) -> Envelope<T> {
    Envelope {
        contract_version: CONTRACT_VERSION,
        ok: false,
        data: None,
        diagnostics: Vec::new(),
        error: Some(error),
    }
}

fn failure_with_diagnostics<T>(diagnostics: Vec<DiagnosticDto>) -> Envelope<T> {
    Envelope {
        contract_version: CONTRACT_VERSION,
        ok: false,
        data: None,
        diagnostics,
        error: None,
    }
}

fn to_json<T: Serialize>(value: &T) -> String {
    match serde_json::to_string(value) {
        Ok(json) => json,
        Err(error) => format!(
            "{{\"contractVersion\":{CONTRACT_VERSION},\"ok\":false,\"data\":null,\"diagnostics\":[],\"error\":{{\"code\":\"SERIALIZATION_ERROR\",\"message\":\"failed to serialize WASM response: {error}\"}}}}"
        ),
    }
}

#[derive(Debug)]
enum PipelineError {
    Boundary { code: &'static str, message: String },
    Parse(Vec<Diagnostic>),
    Timing(TimingError),
    Render(RenderError),
}

impl PipelineError {
    fn into_boundary_error(self) -> BoundaryErrorDto {
        match self {
            Self::Boundary { code, message } => BoundaryErrorDto {
                code: code.to_owned(),
                message,
            },
            Self::Parse(_) => BoundaryErrorDto {
                code: "PARSE_FAILED".to_owned(),
                message: "chart source produced parser diagnostics".to_owned(),
            },
            Self::Timing(error) => BoundaryErrorDto {
                code: timing_error_code(&error).to_owned(),
                message: error.to_string(),
            },
            Self::Render(error) => BoundaryErrorDto {
                code: render_error_code(&error).to_owned(),
                message: error.to_string(),
            },
        }
    }
}

fn timing_error_code(error: &TimingError) -> &'static str {
    match error {
        TimingError::MissingInitialTiming => "MISSING_INITIAL_TIMING",
        TimingError::DuplicateTimingAtSameTimestamp { .. } => "DUPLICATE_TIMING_AT_SAME_TIMESTAMP",
        TimingError::MissingTimingForGroup { .. } => "MISSING_TIMING_FOR_GROUP",
        TimingError::NonFiniteBeatValue => "NON_FINITE_BEAT_VALUE",
        TimingError::Io { .. } => "IO_ERROR",
        TimingError::SvgRender { .. } => "SVG_RENDER_ERROR",
    }
}

fn render_error_code(error: &RenderError) -> &'static str {
    match error {
        RenderError::InvalidViewport { .. } => "INVALID_VIEWPORT",
        RenderError::InvalidProjectionWindow { .. } => "INVALID_PROJECTION_WINDOW",
        RenderError::NonFiniteCoordinate { .. } => "NON_FINITE_COORDINATE",
        RenderError::InvalidArcSampling { .. } => "INVALID_ARC_SAMPLING",
        RenderError::Io { .. } => "IO_ERROR",
        RenderError::SvgRender { .. } => "SVG_RENDER_ERROR",
    }
}

const fn default_past_window_ms() -> i64 {
    500
}

const fn default_future_window_ms() -> i64 {
    2_000
}

const fn default_viewport_width() -> u32 {
    1_280
}

const fn default_viewport_height() -> u32 {
    720
}

const fn default_arc_sample_steps() -> usize {
    16
}

const fn default_debug_labels() -> bool {
    true
}

fn default_fixture_name() -> String {
    "browser-fixture.aff".to_owned()
}

fn snapshot_kind_label(kind: SnapshotNoteKind) -> &'static str {
    match kind {
        SnapshotNoteKind::Tap => "tap",
        SnapshotNoteKind::Hold => "hold",
        SnapshotNoteKind::Arc => "arc",
        SnapshotNoteKind::ArcTap => "arcTap",
    }
}

fn note_state_label(state: NoteState) -> &'static str {
    match state {
        NoteState::Upcoming => "upcoming",
        NoteState::Active => "active",
        NoteState::Passed => "passed",
    }
}

fn render_state_label(state: RenderNoteState) -> &'static str {
    match state {
        RenderNoteState::Upcoming => "upcoming",
        RenderNoteState::Active => "active",
        RenderNoteState::Passed => "passed",
    }
}

fn primitive_kind(primitive: &RenderPrimitive) -> &'static str {
    match primitive {
        RenderPrimitive::Lane(_) => "lane",
        RenderPrimitive::JudgementLine(_) => "judgementLine",
        RenderPrimitive::Tap(_) => "tap",
        RenderPrimitive::Hold(_) => "hold",
        RenderPrimitive::Arc(_) => "arc",
        RenderPrimitive::ArcTap(_) => "arcTap",
        RenderPrimitive::TimingMarker(_) => "timingMarker",
        RenderPrimitive::Label(_) => "label",
    }
}

fn layer_label(layer: RenderLayer) -> &'static str {
    match layer {
        RenderLayer::Lanes => "lanes",
        RenderLayer::TimingMarkers => "timingMarkers",
        RenderLayer::Holds => "holds",
        RenderLayer::Arcs => "arcs",
        RenderLayer::ArcTaps => "arcTaps",
        RenderLayer::Taps => "taps",
        RenderLayer::JudgementLine => "judgementLine",
        RenderLayer::DebugLabels => "debugLabels",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const MIXED: &str = include_str!("../../../fixtures/checkpoint9_mixed.aff");
    const INVALID: &str = include_str!("../../../fixtures/invalid_lane.aff");

    fn render_request(playback_ms: i64) -> String {
        serde_json::json!({
            "playbackMs": playback_ms,
            "fixtureName": "checkpoint9_mixed.aff",
            "pastWindowMs": 500,
            "futureWindowMs": 2000,
            "viewportWidth": 1280,
            "viewportHeight": 720,
            "arcSampleSteps": 16,
            "debugLabels": true
        })
        .to_string()
    }

    #[test]
    fn valid_fixture_parse_returns_success_envelope() {
        let envelope = parse_chart_envelope(MIXED);

        assert!(envelope.ok);
        let data = envelope.data.expect("parse data");
        assert_eq!(data.summary.timing_group_count, 2);
        assert_eq!(data.summary.arc_taps, 3);
    }

    #[test]
    fn invalid_aff_returns_structured_diagnostics() {
        let envelope = parse_chart_envelope(INVALID);

        assert!(!envelope.ok);
        assert!(envelope.error.is_none());
        assert!(!envelope.diagnostics.is_empty());
        let diagnostic = &envelope.diagnostics[0];
        assert_eq!(diagnostic.code, "DOMAIN_VALIDATION_ERROR");
        assert_eq!(diagnostic.severity, "error");
        assert!(!diagnostic.message.is_empty());
        assert!(diagnostic.line > 0);
        assert!(diagnostic.column > 0);
    }

    #[test]
    fn malformed_json_returns_stable_boundary_error() {
        let envelope = build_render_scene_envelope(MIXED, "{");

        assert!(!envelope.ok);
        let error = envelope.error.expect("error");
        assert_eq!(error.code, "MALFORMED_JSON");
    }

    #[test]
    fn invalid_projection_config_does_not_panic() {
        let request = serde_json::json!({
            "playbackMs": 2500,
            "fixtureName": "bad.aff",
            "futureWindowMs": 0
        })
        .to_string();

        let envelope = build_render_scene_envelope(MIXED, &request);

        assert!(!envelope.ok);
        assert_eq!(envelope.error.expect("error").code, "INVALID_FUTURE_WINDOW");
    }

    #[test]
    fn scene_summary_and_svg_are_generated_from_checkpoint_fixture() {
        let envelope = render_chart_svg_envelope(MIXED, &render_request(2_500));

        assert!(envelope.ok);
        let data = envelope.data.expect("svg data");
        assert!(data.svg.contains("<svg"));
        assert_eq!(data.scene.summary.visible_holds, 2);
        assert_eq!(data.scene.summary.visible_arcs, 1);
        assert_eq!(data.scene.summary.visible_arc_taps, 2);
        assert!(data.scene.summary.primitive_count > 0);
    }

    #[test]
    fn render_output_is_deterministic_for_same_source_and_request() {
        let request = render_request(2_500);

        let first = render_chart_svg(MIXED, &request);
        let second = render_chart_svg(MIXED, &request);

        assert_eq!(first, second);
    }

    #[test]
    fn dto_conversion_preserves_primitive_order() {
        let envelope = build_render_scene_envelope(MIXED, &render_request(2_500));
        let data = envelope.data.expect("scene data");

        for (expected, primitive) in data.scene.primitives.iter().enumerate() {
            assert_eq!(primitive.index, expected);
        }
        assert!(
            data.scene
                .primitives
                .windows(2)
                .all(|pair| pair[0].layer <= pair[1].layer)
        );
    }

    #[test]
    fn native_pipeline_and_wasm_wrapper_have_equivalent_logic_data() {
        let request = render_request(2_500);
        let native = render_chart_svg_envelope(MIXED, &request);
        let wrapper_json = render_chart_svg(MIXED, &request);
        let wrapper: Envelope<RenderSvgData> =
            serde_json::from_str(&wrapper_json).expect("wrapper json");

        assert_eq!(native.ok, wrapper.ok);
        assert_eq!(
            native.data.as_ref().map(|data| &data.scene.summary),
            wrapper.data.as_ref().map(|data| &data.scene.summary)
        );
        assert_eq!(
            native.data.as_ref().map(|data| data.scene.primitives.len()),
            wrapper
                .data
                .as_ref()
                .map(|data| data.scene.primitives.len())
        );
    }

    #[test]
    fn timing_groups_and_arc_taps_are_present() {
        let envelope = build_playback_snapshot_envelope(MIXED, 1_900);

        assert!(envelope.ok);
        let data = envelope.data.expect("snapshot");
        assert!(data.notes.iter().any(|note| note.timing_group_id == 1));
        assert!(
            data.notes
                .iter()
                .any(|note| note.kind == "arcTap" && note.parent_arc_id.is_some())
        );
    }

    #[test]
    fn output_json_and_svg_do_not_contain_non_finite_values() {
        let request = render_request(2_500);
        let json = render_chart_svg(MIXED, &request);
        let parsed: Value = serde_json::from_str(&json).expect("json");

        assert!(parsed["ok"].as_bool().unwrap_or(false));
        assert!(!json.contains("NaN"));
        assert!(!json.contains("Infinity"));
        assert!(!json.contains("-Infinity"));
    }
}
