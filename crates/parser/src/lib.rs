//! Parsing, validation, and normalization boundary for chart source data.
//!
//! This crate currently implements a thin AFF parser slice for blank lines,
//! `//` comments, tap notes, hold notes, timing events, and basic arc notes.

mod diagnostic;
mod source;
mod syntax;

pub use diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticCode, Severity};
pub use source::{LineColumn, SourceMap, Span};

use arcaea_viewer_core::{
    ArcNote, ArcPath, ArcX, ArcY, Chart, ChartEvent, ChartTime, HoldNote, Lane, NoteId, TapNote,
    Tempo, TimingEvent,
};
use syntax::{SyntaxEvent, SyntaxParser};

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
    let mut next_note_id = 0_u32;

    for event in syntax_events {
        match event {
            SyntaxEvent::Tap { time, lane } => {
                let lane = match Lane::new(lane.value) {
                    Ok(lane) => lane,
                    Err(error) => {
                        diagnostics.push(Diagnostic::domain_validation(
                            "invalid ground lane",
                            lane.span,
                            Some(format!("{error}")),
                            Some("expected a ground lane from 1 to 4"),
                        ));
                        continue;
                    }
                };
                let tap = TapNote::new(
                    NoteId::new(next_note_id),
                    ChartTime::from_millis(time.value),
                    lane,
                );
                next_note_id += 1;
                events.push(ChartEvent::Tap(tap));
            }
            SyntaxEvent::Hold {
                start_time,
                end_time,
                lane,
            } => {
                let lane = match Lane::new(lane.value) {
                    Ok(lane) => lane,
                    Err(error) => {
                        diagnostics.push(Diagnostic::domain_validation(
                            "invalid hold lane",
                            lane.span,
                            Some(format!("{error}")),
                            Some("expected a ground lane from 1 to 4"),
                        ));
                        continue;
                    }
                };
                match HoldNote::new(
                    NoteId::new(next_note_id),
                    ChartTime::from_millis(start_time.value),
                    ChartTime::from_millis(end_time.value),
                    lane,
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
                events.push(ChartEvent::Timing(TimingEvent::new(
                    ChartTime::from_millis(time.value),
                    tempo,
                    beats_per_measure.value,
                )));
            }
            SyntaxEvent::Arc {
                start_time,
                end_time,
                start_x,
                end_x,
                curve,
                start_y,
                end_y,
                color,
                is_trace,
            } => {
                let start_x = match ArcX::new(start_x.value) {
                    Ok(value) => value,
                    Err(error) => {
                        diagnostics.push(Diagnostic::domain_validation(
                            "invalid arc start x coordinate",
                            start_x.span,
                            Some(format!("{error}")),
                            Some("expected normalized coordinate in 0.0..=1.0"),
                        ));
                        continue;
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
                        continue;
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
                        continue;
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
                        continue;
                    }
                };

                match ArcNote::new(
                    NoteId::new(next_note_id),
                    ChartTime::from_millis(start_time.value),
                    ChartTime::from_millis(end_time.value),
                    ArcPath::new(start_x, end_x, start_y, end_y),
                    curve.value,
                    color.value,
                    is_trace.value,
                ) {
                    Ok(arc) => {
                        next_note_id += 1;
                        events.push(ChartEvent::Arc(arc));
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
        Ok(Chart::new(events))
    } else {
        Err(diagnostics)
    }
}
