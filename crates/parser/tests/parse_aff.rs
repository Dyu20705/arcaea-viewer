use std::panic;

use arcaea_viewer_core::{ArcColor, ArcCurve, ChartEvent, TimingGroupId};
use arcaea_viewer_parser::{DiagnosticCode, parse_chart};

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../../fixtures/{name}",
        env!("CARGO_MANIFEST_DIR").replace('\\', "/")
    );
    std::fs::read_to_string(path).expect("fixture should be readable")
}

#[test]
fn parses_valid_tap() {
    let chart = parse_chart("(1000,1);").expect("tap chart");

    assert!(
        matches!(chart.events()[0], ChartEvent::Tap(tap) if tap.time().as_millis() == 1000 && tap.lane().as_u8() == 1)
    );
}

#[test]
fn parses_valid_hold() {
    let chart = parse_chart("hold(1500,3000,2);").expect("hold chart");

    assert!(
        matches!(chart.events()[0], ChartEvent::Hold(hold) if hold.start_time().as_millis() == 1500 && hold.end_time().as_millis() == 3000 && hold.lane().as_u8() == 2)
    );
}

#[test]
fn parses_valid_timing() {
    let chart = parse_chart("timing(0,120.500,4);").expect("timing chart");

    assert!(
        matches!(chart.events()[0], ChartEvent::Timing(timing) if timing.tempo().as_milli_bpm() == 120_500 && timing.beats_per_measure() == 4)
    );
}

#[test]
fn parses_valid_basic_arc() {
    let chart =
        parse_chart("arc(3200,5000,0.25,0.75,sisi,0.50,1.00,0,none,false);").expect("arc chart");

    assert!(
        matches!(chart.events()[0], ChartEvent::Arc(arc) if arc.id().as_u32() == 0 && arc.curve() == ArcCurve::SineInOut && arc.color() == ArcColor::Blue)
    );
}

#[test]
fn mixed_events_preserve_source_order() {
    let chart = parse_chart(&fixture("mixed_events.aff")).expect("mixed fixture");

    assert!(matches!(chart.events()[0], ChartEvent::Timing(_)));
    assert!(matches!(chart.events()[1], ChartEvent::Tap(_)));
    assert!(matches!(chart.events()[2], ChartEvent::Hold(_)));
    assert!(matches!(chart.events()[3], ChartEvent::Arc(_)));
    assert!(matches!(chart.events()[4], ChartEvent::Tap(_)));
}

#[test]
fn note_ids_are_deterministic_and_skip_timing_events() {
    let chart = parse_chart(&fixture("mixed_events.aff")).expect("mixed fixture");
    let ids: Vec<u32> = chart
        .events()
        .iter()
        .filter_map(|event| match event {
            ChartEvent::Tap(tap) => Some(tap.id().as_u32()),
            ChartEvent::Hold(hold) => Some(hold.id().as_u32()),
            ChartEvent::Arc(arc) => Some(arc.id().as_u32()),
            ChartEvent::ArcTap(arc_tap) => Some(arc_tap.id().as_u32()),
            ChartEvent::Timing(_) => None,
        })
        .collect();

    assert_eq!(ids, [0, 1, 2, 3]);
}

#[test]
fn parses_negative_chart_time() {
    let chart = parse_chart(&fixture("negative_time.aff")).expect("negative fixture");

    assert!(
        matches!(chart.events()[0], ChartEvent::Timing(timing) if timing.time().as_millis() == -500)
    );
    assert!(matches!(chart.events()[1], ChartEvent::Tap(tap) if tap.time().as_millis() == -250));
}

#[test]
fn invalid_lane_becomes_domain_diagnostic() {
    let diagnostics = parse_chart(&fixture("invalid_lane.aff")).expect_err("invalid lane");

    assert_eq!(diagnostics[0].code, DiagnosticCode::DomainValidationError);
}

#[test]
fn reversed_hold_becomes_domain_diagnostic() {
    let diagnostics = parse_chart(&fixture("invalid_hold_interval.aff")).expect_err("invalid hold");

    assert_eq!(diagnostics[0].code, DiagnosticCode::DomainValidationError);
}

#[test]
fn invalid_coordinate_becomes_domain_diagnostic() {
    let diagnostics =
        parse_chart(&fixture("invalid_arc_coordinate.aff")).expect_err("invalid coordinate");

    assert_eq!(diagnostics[0].code, DiagnosticCode::DomainValidationError);
}

#[test]
fn malformed_syntax_includes_correct_span() {
    let diagnostics = parse_chart(&fixture("malformed_syntax.aff")).expect_err("malformed syntax");

    assert_eq!(diagnostics[0].code, DiagnosticCode::SyntaxError);
    assert_eq!(diagnostics[0].span.start, "timing(0,120.000,4".len());
}

#[test]
fn unsupported_event_is_not_ignored() {
    let diagnostics = parse_chart(&fixture("unsupported_event.aff")).expect_err("unsupported");

    assert_eq!(diagnostics[0].code, DiagnosticCode::UnsupportedEvent);
}

#[test]
fn parses_timing_group_with_local_timing_and_note_group_ids() {
    let chart = parse_chart(&fixture("timing_group.aff")).expect("timinggroup fixture");

    assert_eq!(chart.timing_groups().len(), 2);
    assert_eq!(chart.timing_groups()[1].id(), TimingGroupId::new(1));
    assert!(chart.timing_groups()[1].properties().no_input());
    assert!(chart.timing_groups()[1].properties().no_clip());
    assert!(
        matches!(chart.events()[2], ChartEvent::Tap(tap) if tap.timing_group() == TimingGroupId::new(1))
    );
}

#[test]
fn parses_arc_taps_with_parent_relationship_and_boundary_times() {
    let chart = parse_chart(&fixture("arc_taps.aff")).expect("arc taps fixture");
    let arc_id = match chart.events()[1] {
        ChartEvent::Arc(arc) => arc.id(),
        _ => panic!("expected parent arc"),
    };
    let arc_taps: Vec<_> = chart
        .events()
        .iter()
        .filter_map(|event| match event {
            ChartEvent::ArcTap(arc_tap) => Some(*arc_tap),
            _ => None,
        })
        .collect();

    assert_eq!(arc_taps.len(), 3);
    assert_eq!(arc_taps[0].time().as_millis(), 1000);
    assert_eq!(arc_taps[2].time().as_millis(), 3000);
    assert!(
        arc_taps
            .iter()
            .all(|arc_tap| arc_tap.parent_arc_id() == arc_id)
    );
}

#[test]
fn mixed_root_and_grouped_events_parse_together() {
    let chart = parse_chart(&fixture("checkpoint9_mixed.aff")).expect("mixed checkpoint9");

    assert!(chart.events().iter().any(|event| {
        matches!(event, ChartEvent::Tap(tap) if tap.timing_group() == TimingGroupId::ROOT)
    }));
    assert!(chart.events().iter().any(|event| {
        matches!(event, ChartEvent::ArcTap(arc_tap) if arc_tap.timing_group() == TimingGroupId::new(1))
    }));
}

#[test]
fn malformed_timing_group_is_syntax_diagnostic() {
    let diagnostics =
        parse_chart(&fixture("malformed_timing_group.aff")).expect_err("malformed group");

    assert_eq!(diagnostics[0].code, DiagnosticCode::SyntaxError);
}

#[test]
fn unknown_timing_group_property_is_not_ignored() {
    let diagnostics = parse_chart(&fixture("unknown_timing_group_property.aff"))
        .expect_err("unknown group property");

    assert_eq!(diagnostics[0].code, DiagnosticCode::UnsupportedEvent);
}

#[test]
fn arc_tap_without_parent_is_not_accepted() {
    let diagnostics =
        parse_chart(&fixture("arctap_without_parent.aff")).expect_err("arctap without parent");

    assert_eq!(diagnostics[0].code, DiagnosticCode::UnsupportedEvent);
}

#[test]
fn arc_tap_outside_parent_interval_is_domain_diagnostic() {
    let diagnostics =
        parse_chart(&fixture("arctap_outside_parent.aff")).expect_err("arctap outside interval");

    assert_eq!(diagnostics[0].code, DiagnosticCode::DomainValidationError);
}

#[test]
fn duplicate_arc_tap_timestamp_is_domain_diagnostic() {
    let diagnostics =
        parse_chart(&fixture("duplicate_boundary_arctaps.aff")).expect_err("duplicate arctap");

    assert_eq!(diagnostics[0].code, DiagnosticCode::DomainValidationError);
}

#[test]
fn parser_does_not_panic_on_malformed_input() {
    let result = panic::catch_unwind(|| parse_chart("arc(,,,,,,,,,"));

    assert!(result.is_ok());
    assert!(result.expect("no panic").is_err());
}

#[test]
fn repeated_parsing_gives_equivalent_output() {
    let source = fixture("mixed_events.aff");
    let first = parse_chart(&source).expect("first parse");
    let second = parse_chart(&source).expect("second parse");

    assert_eq!(first, second);
}

#[test]
fn integration_parses_minimal_fixture() {
    let chart = parse_chart(&fixture("minimal.aff")).expect("minimal fixture");

    assert_eq!(chart.len(), 2);
}
