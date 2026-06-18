use std::{env, fs, process};

use arcaea_viewer_core::ChartEvent;
use arcaea_viewer_parser::parse_chart;

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: cargo run -p arcaea-viewer-parser --example parse_aff -- <path>");
        process::exit(2);
    };

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read {path}: {error}");
            process::exit(1);
        }
    };

    println!("AFF Parse Demo");
    println!("==============");
    println!();
    println!("Input: {path}");
    println!();

    match parse_chart(&source) {
        Ok(chart) => {
            println!("Parsed successfully");
            println!("Events: {}", chart.len());
            println!();

            let mut timing_count = 0_usize;
            let mut tap_count = 0_usize;
            let mut hold_count = 0_usize;
            let mut arc_count = 0_usize;

            for (index, event) in chart.events().iter().enumerate() {
                match event {
                    ChartEvent::Timing(timing) => {
                        timing_count += 1;
                        println!(
                            "[{index}] Timing time={}ms bpm={:.3} beats={}",
                            timing.time().as_millis(),
                            f64::from(timing.tempo().as_milli_bpm()) / 1000.0,
                            timing.beats_per_measure()
                        );
                    }
                    ChartEvent::Tap(tap) => {
                        tap_count += 1;
                        println!(
                            "[{index}] Tap id={} time={}ms lane={}",
                            tap.id().as_u32(),
                            tap.time().as_millis(),
                            tap.lane().as_u8()
                        );
                    }
                    ChartEvent::Hold(hold) => {
                        hold_count += 1;
                        println!(
                            "[{index}] Hold id={} start={}ms end={}ms lane={}",
                            hold.id().as_u32(),
                            hold.start_time().as_millis(),
                            hold.end_time().as_millis(),
                            hold.lane().as_u8()
                        );
                    }
                    ChartEvent::Arc(arc) => {
                        arc_count += 1;
                        println!(
                            "[{index}] Arc id={} start={}ms end={}ms curve={:?} color={:?} trace={}",
                            arc.id().as_u32(),
                            arc.start_time().as_millis(),
                            arc.end_time().as_millis(),
                            arc.curve(),
                            arc.color(),
                            arc.is_trace()
                        );
                        println!(
                            "    from=({:.2}, {:.2}) to=({:.2}, {:.2})",
                            arc.start_x().as_f32(),
                            arc.start_y().as_f32(),
                            arc.end_x().as_f32(),
                            arc.end_y().as_f32()
                        );
                    }
                }
            }

            println!();
            println!("Summary");
            println!("-------");
            println!("Timing events: {timing_count}");
            println!("Tap notes:     {tap_count}");
            println!("Hold notes:    {hold_count}");
            println!("Arc notes:     {arc_count}");
            println!("Diagnostics:   0");
        }
        Err(diagnostics) => {
            println!("Parse failed");
            println!("Diagnostics: {}", diagnostics.len());
            println!();
            for diagnostic in diagnostics {
                println!("{}", diagnostic.render(&source, &path));
                println!();
            }
            process::exit(1);
        }
    }
}
