//! Headless effect analyzer: run ONE effect, ONE view, N frames and
//! report what actually renders — the final-frame glyph census plus
//! the change rate between evenly spaced samples. Dev feedback loop
//! for judging effects without a screen (`tarts --analyze`).
//!
//! Contract: default options and the given seed only — the user's
//! config file is deliberately ignored so reports are reproducible,
//! and the canvas is explicit (`--size WxH`, default 100x40), never
//! the live terminal size. Any other view is a re-run away.

use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};

/// Default analysis canvas — matches the numbers the pocketworld
/// census tests already baseline against.
pub const DEFAULT_WIDTH: u16 = 100;
pub const DEFAULT_HEIGHT: u16 = 40;

/// Per-dimension canvas bound; beyond this a run is slow and the
/// census unreadable.
const MAX_DIM: u16 = 1000;

/// Snapshots taken across the run for the change-rate stat.
const SAMPLES: usize = 10;

/// Default analysis seed — pocketworld's boot world. Recorded seed-42
/// baselines stay reproducible with `--seed 42`.
pub const DEFAULT_SEED: u32 = 7;

/// One analysis run. Default: 1000 frames, seed 7, the effect's
/// first view, no dump, 100x40 canvas.
#[derive(Debug)]
pub struct AnalysisOptions {
    pub frames: usize,
    pub seed: u32,
    pub view: Option<String>,
    pub dump: bool,
    pub width: u16,
    pub height: u16,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            frames: 1000,
            seed: DEFAULT_SEED,
            view: None,
            dump: false,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
        }
    }
}

/// Effects whose options carry a seed (deterministic replays). The
/// rest roll from thread entropy, so their reports are statistical
/// until task 0009 wires seeds through.
fn is_seeded(name: &str) -> bool {
    name == "pocketworld"
}

/// Default options for `E` on the given canvas.
fn opts<E>(width: u16, height: u16) -> E::Options
where
    E: DefaultOptions,
{
    E::default_options(width, height)
}

fn build_effect(
    name: &str,
    seed: u32,
    width: u16,
    height: u16,
) -> Result<Box<dyn TerminalEffect>, String> {
    let size = (width, height);
    let effect: Box<dyn TerminalEffect> = match name {
        "matrix" => Box::new(crate::rain::digital_rain::DigitalRain::new(
            opts::<crate::rain::digital_rain::DigitalRain>(width, height),
            size,
        )),
        "life" => Box::new(crate::life::ConwayLife::new(
            opts::<crate::life::ConwayLife>(width, height),
            size,
        )),
        "maze" => Box::new(crate::maze::Maze::new(
            opts::<crate::maze::Maze>(width, height),
            size,
        )),
        "boids" => Box::new(crate::boids::Boids::new(opts::<crate::boids::Boids>(
            width, height,
        ))),
        "blank" => Box::new(crate::blank::Blank::new(
            crate::blank::BlankOptions {},
            size,
        )),
        "cube" => Box::new(crate::cube::Cube::new(
            opts::<crate::cube::Cube>(width, height),
            size,
        )),
        "crab" => Box::new(crate::crab::Crab::new(
            opts::<crate::crab::Crab>(width, height),
            size,
        )),
        "donut" => Box::new(crate::donut::Donut::new(
            opts::<crate::donut::Donut>(width, height),
            size,
        )),
        "pipes" => Box::new(crate::pipes::Pipes::new(
            opts::<crate::pipes::Pipes>(width, height),
            size,
        )),
        "plasma" => Box::new(crate::plasma::Plasma::new(
            opts::<crate::plasma::Plasma>(width, height),
            size,
        )),
        "fire" => Box::new(crate::fire::Fire::new(
            opts::<crate::fire::Fire>(width, height),
            size,
        )),
        "constellation" => Box::new(crate::constellation::Constellation::new(
            opts::<crate::constellation::Constellation>(width, height),
            size,
        )),
        "butterfly" => Box::new(crate::butterfly::Butterfly::new(
            opts::<crate::butterfly::Butterfly>(width, height),
            size,
        )),
        "pocketworld" => {
            let mut options =
                opts::<crate::pocketworld::PocketWorld>(width, height);
            options.seed = seed;
            Box::new(crate::pocketworld::PocketWorld::new(options, size))
        }
        _ => {
            return Err(format!(
                "unknown effect: {} (run `tarts --help` for the list)",
                name
            ));
        }
    };
    Ok(effect)
}

/// Run `effect_name` per `opts` and return the analysis report: the
/// view resolved (first view when `opts.view` is None), optionally
/// the whole final frame (`opts.dump`), then the glyph census and
/// change rate. Errors on unknown effect or view names.
pub fn analyze(
    effect_name: &str,
    opts: &AnalysisOptions,
) -> Result<String, String> {
    let (width, height) = (opts.width, opts.height);
    if width == 0 || height == 0 || width > MAX_DIM || height > MAX_DIM {
        return Err(format!(
            "canvas size must be 1..={} per dimension (got {}x{})",
            MAX_DIM, width, height
        ));
    }
    let mut effect = build_effect(effect_name, opts.seed, width, height)?;
    let views = effect.views();
    let view = match &opts.view {
        Some(name) => {
            if !views.contains(&name.as_str()) {
                return Err(format!(
                    "unknown view '{}' for {} (available: {})",
                    name,
                    effect_name,
                    views.join(", ")
                ));
            }
            name.clone()
        }
        None => views[0].to_string(),
    };
    effect.set_view(&view);

    // Shadow the screen: apply every diff patch to a local buffer the
    // same way run_loop applies them to the terminal.
    let mut shadow = Buffer::new(width as usize, height as usize);
    let every = (opts.frames / SAMPLES).max(1);
    let mut prev: Option<Vec<Cell>> = None;
    let mut changes: Vec<usize> = Vec::new();

    for i in 0..opts.frames {
        effect.update();
        for (x, y, cell) in effect.get_diff() {
            shadow.set(x, y, cell);
        }
        if (i + 1) % every == 0 {
            if let Some(prev) = &prev {
                let changed = shadow
                    .buffer
                    .iter()
                    .zip(prev.iter())
                    .filter(|(a, b)| a != b)
                    .count();
                changes.push(changed);
            }
            prev = Some(shadow.buffer.clone());
        }
    }

    // Glyph census of the final frame.
    let total = shadow.buffer.len();
    let mut counts = std::collections::HashMap::<char, usize>::new();
    for cell in &shadow.buffer {
        *counts.entry(cell.symbol).or_insert(0) += 1;
    }
    let mut census: Vec<(char, usize)> = counts.into_iter().collect();
    census.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let mut report = String::new();
    report.push_str(&format!("== tarts analyze: {}\n", effect_name));
    let seed_note = if is_seeded(effect_name) {
        format!("seed: {} (deterministic)", opts.seed)
    } else {
        format!("seed: {} (ignored)", opts.seed)
    };
    report.push_str(&format!(
        "   view: {} | {} | canvas: {}x{} | frames: {}\n",
        view, seed_note, width, height, opts.frames
    ));
    if !is_seeded(effect_name) {
        report.push_str(
            "   note: no seed support yet (task 0009) — re-runs will differ\n",
        );
    }
    if opts.dump {
        report.push_str("FRAME (final, glyphs only):\n");
        let row_width = width as usize;
        for y in 0..height as usize {
            let row: String = shadow.buffer[y * row_width..(y + 1) * row_width]
                .iter()
                .map(|cell| cell.symbol)
                .collect();
            report.push_str(&row);
            report.push('\n');
        }
    }
    report.push_str(&format!(
        "CENSUS total={} distinct={}\n",
        total,
        census.len()
    ));
    for (glyph, count) in &census {
        report.push_str(&format!(
            "   '{}'  {:>5}  {:5.1}%\n",
            glyph,
            count,
            100.0 * *count as f64 / total as f64
        ));
    }
    match (changes.iter().min(), changes.iter().max()) {
        (Some(min), Some(max)) => {
            let mean = changes.iter().sum::<usize>() as f64 / changes.len() as f64;
            report.push_str(&format!(
                "CHANGE cells/sample: mean {:.0} | min {} | max {} | {:.1}% of canvas\n",
                mean,
                min,
                max,
                100.0 * mean / total as f64
            ));
        }
        _ => report.push_str("CHANGE cells/sample: n/a (too few frames)\n"),
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(
        effect: &str,
        frames: usize,
        seed: u32,
        view: Option<&str>,
        dump: bool,
    ) -> Result<String, String> {
        analyze(
            effect,
            &AnalysisOptions {
                frames,
                seed,
                view: view.map(str::to_string),
                dump,
                ..AnalysisOptions::default()
            },
        )
    }

    #[test]
    fn height_view_report_shape() {
        let report = run("pocketworld", 40, 42, Some("height"), false).unwrap();
        assert!(report.contains("view: height"), "{}", report);
        assert!(report.contains("seed: 42 (deterministic)"));
        assert!(report.contains("canvas: 100x40"));
        assert!(report.contains("CENSUS total=4000"));
        assert!(report.contains("CHANGE cells/sample"));
    }

    #[test]
    fn same_seed_same_report() {
        let a = run("pocketworld", 40, 7, Some("biome"), false).unwrap();
        let b = run("pocketworld", 40, 7, Some("biome"), false).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn unknown_view_lists_available_views() {
        let err = run("pocketworld", 5, 42, Some("spoon"), false).unwrap_err();
        assert!(err.contains("unknown view 'spoon'"), "{}", err);
        assert!(err.contains("height"), "{}", err);
    }

    #[test]
    fn unknown_effect_errors() {
        assert!(run("nope", 5, 1, None, false).is_err());
    }

    #[test]
    fn generic_effect_uses_default_view() {
        let report = run("life", 20, 1, None, false).unwrap();
        assert!(report.contains("view: default"), "{}", report);
        assert!(report.contains("task 0009"), "{}", report);
    }

    #[test]
    fn dump_prints_whole_frame_and_keeps_stats() {
        let report = run("pocketworld", 30, 42, Some("height"), true).unwrap();
        assert!(report.contains("FRAME (final, glyphs only)"), "{}", report);
        let start = report
            .lines()
            .position(|l| l.starts_with("FRAME"))
            .expect("frame marker");
        let frame: Vec<&str> = report
            .lines()
            .skip(start + 1)
            .take(DEFAULT_HEIGHT as usize)
            .collect();
        assert_eq!(frame.len(), DEFAULT_HEIGHT as usize);
        for line in &frame {
            assert_eq!(line.chars().count(), DEFAULT_WIDTH as usize);
        }
        // The frame carries real terrain glyphs, and the stats survive.
        let joined = frame.concat();
        assert!(
            joined.contains('·') || joined.contains('≈') || joined.contains('~')
        );
        assert!(report.contains("CENSUS total=4000"));
        assert!(report.contains("CHANGE cells/sample"));
    }

    #[test]
    fn custom_canvas_size() {
        let report = analyze(
            "pocketworld",
            &AnalysisOptions {
                frames: 20,
                seed: 42,
                view: Some("height".to_string()),
                width: 60,
                height: 20,
                ..AnalysisOptions::default()
            },
        )
        .unwrap();
        assert!(report.contains("canvas: 60x20"), "{}", report);
        assert!(report.contains("CENSUS total=1200"), "{}", report);
    }

    #[test]
    fn out_of_bounds_canvas_errors() {
        let zero = AnalysisOptions {
            width: 0,
            ..AnalysisOptions::default()
        };
        assert!(analyze("life", &zero).is_err());
        let huge = AnalysisOptions {
            width: 2000,
            height: 20,
            ..AnalysisOptions::default()
        };
        assert!(analyze("life", &huge).is_err());
    }
}
