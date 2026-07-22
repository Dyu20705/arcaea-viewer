use std::{env, fs, path::PathBuf, process};

use arcaea_viewer_core::ChartTime;
use arcaea_viewer_parser::parse_chart;
use arcaea_viewer_timing::{
    NoteState, SnapshotNoteKind, TimingMap, snapshot_at, write_timeline_svg,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match Config::parse(&args) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            eprintln!(
                "usage: cargo run -p arcaea-viewer-timing --example timing_snapshot -- <input.aff> --time <millis> --output <timeline.svg>"
            );
            process::exit(2);
        }
    };

    let source = match fs::read_to_string(&config.input) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read {}: {error}", config.input.display());
            process::exit(1);
        }
    };

    println!("Timing Snapshot Demo");
    println!("====================");
    println!();
    println!("Input: {}", config.input.display());
    println!("Playback time: {}ms", config.time.as_millis());

    let chart = match parse_chart(&source) {
        Ok(chart) => chart,
        Err(diagnostics) => {
            println!();
            println!("Parse failed");
            println!("Diagnostics: {}", diagnostics.len());
            println!();
            let input = config.input.to_string_lossy();
            for diagnostic in diagnostics {
                println!("{}", diagnostic.render(&source, &input));
                println!();
            }
            process::exit(1);
        }
    };

    let timing_map = match TimingMap::from_chart(&chart) {
        Ok(map) => map,
        Err(error) => {
            eprintln!("timing map failed: {error}");
            process::exit(1);
        }
    };
    let snapshot = snapshot_at(&chart, &timing_map, config.time);

    println!(
        "Current tempo: {:.3} BPM",
        f64::from(snapshot.tempo.as_milli_bpm()) / 1000.0
    );
    println!("Beat position: {:.3}", snapshot.beat_position.as_f64());
    println!();
    println!("Notes");
    println!("-----");
    for note in &snapshot.notes {
        match note.kind {
            SnapshotNoteKind::Tap => {
                println!(
                    "Tap  #{:<2} {:<8} time={}ms{}",
                    note.id.as_u32(),
                    state_label(note.state),
                    note.start_time.as_millis(),
                    relation_suffix(note)
                );
            }
            SnapshotNoteKind::Hold => {
                println!(
                    "Hold #{:<2} {:<8} {}..{}ms{}{}",
                    note.id.as_u32(),
                    state_label(note.state),
                    note.start_time.as_millis(),
                    note.end_time.as_millis(),
                    progress_suffix(note),
                    relation_suffix(note)
                );
            }
            SnapshotNoteKind::Arc => {
                println!(
                    "Arc  #{:<2} {:<8} {}..{}ms{}{}",
                    note.id.as_u32(),
                    state_label(note.state),
                    note.start_time.as_millis(),
                    note.end_time.as_millis(),
                    progress_suffix(note),
                    relation_suffix(note)
                );
            }
            SnapshotNoteKind::ArcTap => {
                println!(
                    "ArcTap #{:<2} {:<8} time={}ms parent_arc={} group={}{}",
                    note.id.as_u32(),
                    state_label(note.state),
                    note.start_time.as_millis(),
                    note.parent_arc_id.map_or(0, |id| id.as_u32()),
                    note.timing_group.as_u32(),
                    relation_suffix(note)
                );
            }
        }
    }

    println!();
    println!("Summary");
    println!("-------");
    println!("Upcoming: {}", snapshot.count_state(NoteState::Upcoming));
    println!("Active:   {}", snapshot.count_state(NoteState::Active));
    println!("Passed:   {}", snapshot.count_state(NoteState::Passed));

    if let Err(error) = write_timeline_svg(
        &chart,
        &timing_map,
        &snapshot,
        &config.input.to_string_lossy(),
        &config.output,
    ) {
        eprintln!("failed to write SVG: {error}");
        process::exit(1);
    }

    println!();
    println!("SVG written:");
    println!("{}", config.output.display());
}

struct Config {
    input: PathBuf,
    time: ChartTime,
    output: PathBuf,
}

impl Config {
    fn parse(args: &[String]) -> Result<Self, String> {
        let input = args
            .get(1)
            .map(PathBuf::from)
            .ok_or_else(|| "missing input fixture path".to_owned())?;
        let mut time = None;
        let mut output = None;
        let mut index = 2;

        while index < args.len() {
            match args[index].as_str() {
                "--time" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| "missing value for --time".to_owned())?;
                    time = Some(
                        value
                            .parse::<i64>()
                            .map(ChartTime::from_millis)
                            .map_err(|_| format!("invalid --time value: {value}"))?,
                    );
                    index += 2;
                }
                "--output" => {
                    let value = args
                        .get(index + 1)
                        .ok_or_else(|| "missing value for --output".to_owned())?;
                    output = Some(PathBuf::from(value));
                    index += 2;
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        Ok(Self {
            input,
            time: time.ok_or_else(|| "missing --time".to_owned())?,
            output: output.ok_or_else(|| "missing --output".to_owned())?,
        })
    }
}

fn state_label(state: NoteState) -> &'static str {
    match state {
        NoteState::Upcoming => "Upcoming",
        NoteState::Active => "Active",
        NoteState::Passed => "Passed",
    }
}

fn progress_suffix(note: &arcaea_viewer_timing::SnapshotNote) -> String {
    note.progress
        .map(|progress| format!(" progress={:.2}%", progress * 100.0))
        .unwrap_or_default()
}

fn relation_suffix(note: &arcaea_viewer_timing::SnapshotNote) -> String {
    if let Some(starts_in) = note.starts_in_millis {
        format!(" starts_in={starts_in}ms")
    } else if let Some(since) = note.since_end_millis {
        format!(" since={since}ms")
    } else {
        String::new()
    }
}
