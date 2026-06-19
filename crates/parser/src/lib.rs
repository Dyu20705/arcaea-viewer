//! Parsing, validation, and normalization boundary for chart source data.
//!
//! This crate implements the AFF subset used by the current checkpoints:
//! blank lines, `//` comments, tap notes, hold notes, timing events, basic arc
//! notes, timinggroup blocks, and arc-local arctap extension blocks.

use std::collections::BTreeSet;

mod diagnostic;
mod source;
mod syntax;

pub use diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticCode, Severity};
pub use source::{LineColumn, SourceMap, Span};

use arcaea_viewer_core::{
    ArcNote, ArcPath, ArcTapNote, ArcX, ArcY, Chart, ChartEvent, ChartTime, HoldNote, Lane, NoteId,
    TapNote, Tempo, TimingEvent, TimingGroup, TimingGroupId, TimingGroupProperties,
};
use syntax::{Spanned, SyntaxEvent, SyntaxParser};

/// Parses a supported AFF subset into a core [`Chart`].
///
/// The parser preserves source order, assigns deterministic note IDs in note
/// source order, and delegates semantic invariants to `arcaea-viewer-core`
/// constructors.
pub fn parse_chart(source: &str) -> Result<Chart, Vec<Diagnostic>> {
    let parser = SyntaxParser::new(source);
    let (syntax_events, mut diagnostics) = parser.parse();

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut events = Vec::with_capacity(syntax_events.len());
    let mut timing_groups = Vec::new();
    let mut next_note_id = 0_u32;

    for event in syntax_events {
        match event {
            SyntaxEvent::TimingGroupDefinition {
                timing_group_id,
                properties,
            } => {
                let properties = match timing_group_properties(&properties, &mut diagnostics) {
                    Some(properties) => properties,
                    None => continue,
                };
                timing_groups.push(TimingGroup::new(
                    TimingGroupId::new(timing_group_id),
                    properties,
                ));
            }
            SyntaxEvent::Tap {
                timing_group_id,
                time,
                lane,
            } => {
                let Some(lane) = parse_lane(lane, &mut diagnostics) else {
                    continue;
                };
                let tap = TapNote::new_in_group(
                    NoteId::new(next_note_id),
                    ChartTime::from_millis(time.value),
                    lane,
                    TimingGroupId::new(timing_group_id),
                );
                next_note_id += 1;
                events.push(ChartEvent::Tap(tap));
            }
            SyntaxEvent::Hold {
                timing_group_id,
                start_time,
                end_time,
                lane,
            } => {
                let Some(lane) = parse_lane(lane, &mut diagnostics) else {
                    continue;
                };
                match HoldNote::new_in_group(
                    NoteId::new(next_note_id),
                    ChartTime::from_millis(start_time.value),
                    ChartTime::from_millis(end_time.value),
                    lane,
                    TimingGroupId::new(timing_group_id),
                ) {
                    Ok(hold) => {
                        next_note_id += 1;
                        events.push(ChartEvent::Hold(hold));
                    }
                    Err(error) => diagnostics.push(Diagnostic::domain_validation(
                        "invalid hold interval",
                        start_time.span.join(end_time.span),
                        Some(format!("{error}")),
                        Some("hold end time must be greater than or equal to start time"),
                    )),
                }
            }
            SyntaxEvent::Timing {
                timing_group_id,
                time,
                tempo_milli_bpm,
                beats_per_measure,
            } => {
                let tempo = match Tempo::from_milli_bpm(tempo_milli_bpm.value) {
                    Ok(tempo) => tempo,
                    Err(error) => {
                        diagnostics.push(Diagnostic::domain_validation(
                            "invalid tempo",
                            tempo_milli_bpm.span,
                            Some(format!("{error}")),
                            Some("expected BPM greater than zero"),
                        ));
                        continue;
                    }
                };
                events.push(ChartEvent::Timing(TimingEvent::new_in_group(
                    ChartTime::from_millis(time.value),
                    tempo,
                    beats_per_measure.value,
                    TimingGroupId::new(timing_group_id),
                )));
            }
            SyntaxEvent::Arc {
                timing_group_id,
                start_time,
                end_time,
                start_x,
                end_x,
                curve,
                start_y,
                end_y,
                color,
                is_trace,
                arc_taps,
            } => {
                let Some(path) = arc_path(start_x, end_x, start_y, end_y, &mut diagnostics) else {
                    continue;
                };
                let group = TimingGroupId::new(timing_group_id);
                match ArcNote::new_in_group(
                    NoteId::new(next_note_id),
                    ChartTime::from_millis(start_time.value),
                    ChartTime::from_millis(end_time.value),
                    path,
                    curve.value,
                    color.value,
                    is_trace.value,
                    group,
                ) {
                    Ok(arc) => {
                        let parent_arc_id = arc.id();
                        next_note_id += 1;
                        let mut parsed_arc_taps = Vec::with_capacity(arc_taps.len());
                        let mut seen_times = BTreeSet::new();
                        for arc_tap_time in arc_taps {
                            let time = ChartTime::from_millis(arc_tap_time.value);
                            if !arc.contains_time(time) {
                                diagnostics.push(Diagnostic::domain_validation(
                                    "arc tap outside parent arc interval",
                                    arc_tap_time.span,
                                    Some(format!(
                                        "arctap {} is outside parent arc {}..{}",
                                        time,
                                        arc.start_time(),
                                        arc.end_time()
                                    )),
                                    Some("arc tap time must be inside the parent arc interval, including boundaries"),
                                ));
                                continue;
                            }
                            if !seen_times.insert(time) {
                                diagnostics.push(Diagnostic::domain_validation(
                                    "duplicate arc tap timestamp",
                                    arc_tap_time.span,
                                    Some(format!(
                                        "parent arc {} already has an arctap at {}",
                                        parent_arc_id.as_u32(),
                                        time
                                    )),
                                    Some("remove the duplicate arctap or move it to a distinct timestamp"),
                                ));
                                continue;
                            }
                            parsed_arc_taps.push(ArcTapNote::new(
                                NoteId::new(next_note_id),
                                time,
                                parent_arc_id,
                                group,
                            ));
                            next_note_id += 1;
                        }
                        events.push(ChartEvent::Arc(arc));
                        for arc_tap in parsed_arc_taps {
                            events.push(ChartEvent::ArcTap(arc_tap));
                        }
                    }
                    Err(error) => diagnostics.push(Diagnostic::domain_validation(
                        "invalid arc interval",
                        start_time.span.join(end_time.span),
                        Some(format!("{error}")),
                        Some("arc end time must be greater than or equal to start time"),
                    )),
                }
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(Chart::with_timing_groups(events, timing_groups))
    } else {
        Err(diagnostics)
    }
}

fn timing_group_properties(
    properties: &[Spanned<String>],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<TimingGroupProperties> {
    let mut no_input = false;
    let mut no_clip = false;
    let mut ok = true;
    for property in properties {
        match property.value.as_str() {
            "noinput" => no_input = true,
            "noclip" => no_clip = true,
            unknown => {
                diagnostics.push(Diagnostic::unsupported(
                    "unsupported timinggroup property",
                    property.span,
                    format!("timinggroup property `{unknown}` is not modeled in this checkpoint"),
                ));
                ok = false;
            }
        }
    }
    ok.then_some(TimingGroupProperties::new(no_input, no_clip))
}

fn parse_lane(lane: Spanned<u8>, diagnostics: &mut Vec<Diagnostic>) -> Option<Lane> {
    match Lane::new(lane.value) {
        Ok(lane) => Some(lane),
        Err(error) => {
            diagnostics.push(Diagnostic::domain_validation(
                "invalid ground lane",
                lane.span,
                Some(format!("{error}")),
                Some("expected a ground lane from 1 to 4"),
            ));
            None
        }
    }
}

fn arc_path(
    start_x: Spanned<f32>,
    end_x: Spanned<f32>,
    start_y: Spanned<f32>,
    end_y: Spanned<f32>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ArcPath> {
    let start_x = match ArcX::new(start_x.value) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(Diagnostic::domain_validation(
                "invalid arc start x coordinate",
                start_x.span,
                Some(format!("{error}")),
                Some("expected normalized coordinate in 0.0..=1.0"),
            ));
            return None;
        }
    };
    let end_x = match ArcX::new(end_x.value) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(Diagnostic::domain_validation(
                "invalid arc end x coordinate",
                end_x.span,
                Some(format!("{error}")),
                Some("expected normalized coordinate in 0.0..=1.0"),
            ));
            return None;
        }
    };
    let start_y = match ArcY::new(start_y.value) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(Diagnostic::domain_validation(
                "invalid arc start y coordinate",
                start_y.span,
                Some(format!("{error}")),
                Some("expected normalized coordinate in 0.0..=1.0"),
            ));
            return None;
        }
    };
    let end_y = match ArcY::new(end_y.value) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(Diagnostic::domain_validation(
                "invalid arc end y coordinate",
                end_y.span,
                Some(format!("{error}")),
                Some("expected normalized coordinate in 0.0..=1.0"),
            ));
            return None;
        }
    };
    Some(ArcPath::new(start_x, end_x, start_y, end_y))
}
