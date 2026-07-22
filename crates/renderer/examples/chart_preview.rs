use std::{collections::BTreeMap, env, fs, path::PathBuf, process};

use arcaea_viewer_core::{ChartTime, TimingGroupId};
use arcaea_viewer_parser::parse_chart;
use arcaea_viewer_renderer::{
    ProjectionConfig, RenderPrimitive, build_scene_with_timing_context, write_scene_svg,
};
use arcaea_viewer_timing::TimingContext;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match Config::parse(&args) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            eprintln!(
                "usage: cargo run -p arcaea-viewer-renderer --example chart_preview -- <input.aff> --time <millis> --output <preview.svg>"
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

    println!("Chart Preview Demo");
    println!("==================");
    println!();
    println!("Input: {}", config.input.display());
    println!("Playback time: {}ms", config.time.as_millis());
    println!(
        "Viewport: {}x{}",
        config.projection.viewport_width, config.projection.viewport_height
    );
    println!(
        "Visible time window: {}..{}ms",
        config.time.as_millis() - config.projection.past_window_ms,
        config.time.as_millis() + config.projection.future_window_ms
    );

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

    let timing_context = match TimingContext::from_chart(&chart) {
        Ok(context) => context,
        Err(error) => {
            eprintln!("timing context failed: {error}");
            process::exit(1);
        }
    };

    let scene = match build_scene_with_timing_context(
        &chart,
        &timing_context,
        config.time,
        config.input.to_string_lossy(),
        config.projection,
    ) {
        Ok(scene) => scene,
        Err(error) => {
            eprintln!("scene build failed: {error}");
            process::exit(1);
        }
    };

    println!();
    println!("Scene");
    println!("-----");
    println!(
        "Root timing events: {}",
        timing_context.root().timing_events().len()
    );
    for (time, tempo) in timing_context.root().timing_events() {
        println!(
            "  root timing {}ms {:.3} BPM",
            time.as_millis(),
            f64::from(tempo.as_milli_bpm()) / 1000.0
        );
    }
    println!("Timing groups:      {}", chart.timing_groups().len());
    for group in chart.timing_groups() {
        let timing_count = timing_context
            .timing_map_for_group(group.id())
            .map_or(0, |map| map.timing_events().len());
        println!(
            "  group={} noinput={} noclip={} timing_events={}",
            group.id().as_u32(),
            group.properties().no_input(),
            group.properties().no_clip(),
            timing_count
        );
    }
    println!("Lanes:              {}", scene.metadata.summary.lanes);
    println!(
        "Visible ground taps: {}",
        scene.metadata.summary.visible_taps
    );
    println!(
        "Visible holds:      {}",
        scene.metadata.summary.visible_holds
    );
    println!(
        "Visible arcs:       {}",
        scene.metadata.summary.visible_arcs
    );
    println!(
        "Visible arc taps:   {}",
        scene.metadata.summary.visible_arc_taps
    );
    println!(
        "Hidden events:      {}",
        scene.metadata.summary.hidden_notes
    );
    println!(
        "Primitive count: {}",
        scene.metadata.summary.primitive_count
    );
    println!("Per-group visible counts:");
    for (group, counts) in visible_counts_by_group(&scene) {
        println!(
            "  group={} taps={} holds={} arcs={} arctaps={}",
            group.as_u32(),
            counts.taps,
            counts.holds,
            counts.arcs,
            counts.arc_taps
        );
    }

    println!();
    println!("Notes");
    println!("-----");
    for primitive in &scene.primitives {
        match primitive {
            RenderPrimitive::Tap(tap) => println!(
                "Tap  #{:<2} group={} {:?} lane={} y={:.3}",
                tap.note_id.as_u32(),
                tap.timing_group.as_u32(),
                tap.state,
                tap.lane.as_u8(),
                tap.center.y
            ),
            RenderPrimitive::Hold(hold) => println!(
                "Hold #{:<2} group={} {:?} clipped_judgement={} clipped_horizon={} y={:.3}..{:.3}",
                hold.note_id.as_u32(),
                hold.timing_group.as_u32(),
                hold.state,
                hold.clipped_at_judgement,
                hold.clipped_at_horizon,
                hold.y_start,
                hold.y_end
            ),
            RenderPrimitive::Arc(arc) => println!(
                "Arc  #{:<2} group={} {:?} samples={} color={:?} trace={} clipped_judgement={} clipped_horizon={}",
                arc.note_id.as_u32(),
                arc.timing_group.as_u32(),
                arc.state,
                arc.points.len(),
                arc.color,
                arc.is_trace,
                arc.clipped_at_judgement,
                arc.clipped_at_horizon
            ),
            RenderPrimitive::ArcTap(arc_tap) => println!(
                "ArcTap #{:<2} group={} parent_arc={} {:?} x={:.3} y={:.3} sky_y={:.3}",
                arc_tap.note_id.as_u32(),
                arc_tap.timing_group.as_u32(),
                arc_tap.parent_arc_id.as_u32(),
                arc_tap.state,
                arc_tap.center.x,
                arc_tap.center.y,
                arc_tap.sky_y
            ),
            _ => {}
        }
    }

    if let Err(error) = write_scene_svg(&scene, &config.output) {
        eprintln!("failed to write SVG: {error}");
        process::exit(1);
    }

    if !config.output.exists() {
        eprintln!("output path was not created: {}", config.output.display());
        process::exit(1);
    }

    println!();
    println!("SVG written:");
    println!("{}", config.output.display());
}

#[derive(Default)]
struct VisibleCounts {
    taps: usize,
    holds: usize,
    arcs: usize,
    arc_taps: usize,
}

fn visible_counts_by_group(
    scene: &arcaea_viewer_renderer::RenderScene,
) -> BTreeMap<TimingGroupId, VisibleCounts> {
    let mut counts = BTreeMap::new();
    for primitive in &scene.primitives {
        match primitive {
            RenderPrimitive::Tap(tap) if tap.visible => {
                counts
                    .entry(tap.timing_group)
                    .or_insert_with(VisibleCounts::default)
                    .taps += 1
            }
            RenderPrimitive::Hold(hold) if hold.visible => {
                counts
                    .entry(hold.timing_group)
                    .or_insert_with(VisibleCounts::default)
                    .holds += 1
            }
            RenderPrimitive::Arc(arc) if arc.visible => {
                counts
                    .entry(arc.timing_group)
                    .or_insert_with(VisibleCounts::default)
                    .arcs += 1
            }
            RenderPrimitive::ArcTap(arc_tap) if arc_tap.visible => {
                counts
                    .entry(arc_tap.timing_group)
                    .or_insert_with(VisibleCounts::default)
                    .arc_taps += 1
            }
            _ => {}
        }
    }
    counts
}

struct Config {
    input: PathBuf,
    time: ChartTime,
    output: PathBuf,
    projection: ProjectionConfig,
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
            projection: ProjectionConfig::default(),
        })
    }
}
