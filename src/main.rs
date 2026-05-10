#![cfg(not(test))]
use crossterm::{self, cursor, execute, terminal};
use std::{io, process};

mod blank;
mod boids;
mod buffer;
mod check;
mod common;
mod config;
mod crab;
mod cube;
mod donut;
mod error;
mod fire;
mod life;
mod maze;
mod pipes;
mod plasma;
mod rain;
mod terrain;

use crate::config::Config;

const VALID_SAVERS: &[&str] = &[
    "matrix", "life", "maze", "boids", "blank", "cube", "crab", "donut", "pipes",
    "plasma", "fire", "terrain",
];

#[derive(Debug)]
struct AppArgs {
    screen_saver: String,
    check: bool,
    effect: Option<String>,
    frames: Option<usize>,
}

/// Guard to drop out alternate screen in case of errors
struct TerminalGuard {
    stdout: io::Stdout,
}

impl TerminalGuard {
    fn new() -> Result<Self, io::Error> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All)
        )?;

        Ok(Self { stdout })
    }

    // Get mutable access to the stdout
    fn get_stdout(&mut self) -> &mut io::Stdout {
        &mut self.stdout
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Ignore errors during drop - we're doing best effort cleanup
        let _ = execute!(
            self.stdout,
            cursor::Show,
            terminal::Clear(terminal::ClearType::All),
            terminal::LeaveAlternateScreen,
        );
        let _ = terminal::disable_raw_mode();
    }
}

fn main() -> Result<(), error::TartsError> {
    env_logger::init();

    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing args: {}", e);
            process::exit(1);
        }
    };

    if args.check {
        let effect = args.effect.unwrap_or_else(|| "matrix".to_string());
        let frames = args.frames.unwrap_or(1);
        return check::run_test_for_effect(&effect, frames);
    }

    // Check if valid before entering alternate screen
    if !VALID_SAVERS.contains(&args.screen_saver.as_str()) {
        println!("Unknown screen saver: {}", args.screen_saver);
        print_help();
        return Ok(());
    }

    let (config, config_status) = Config::load()?;

    let fps = {
        let mut guard = TerminalGuard::new()?;
        let (width, height) = terminal::size()?;

        match args.screen_saver.as_str() {
            "matrix" => {
                let options = config.get_matrix_options((width, height));
                let mut digital_rain =
                    rain::digital_rain::DigitalRain::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut digital_rain, None)?
            }
            "life" => {
                let options = config.get_life_options((width, height));
                let mut conway_life =
                    life::ConwayLife::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut conway_life, None)?
            }
            "maze" => {
                let options = config.get_maze_options((width, height));
                let mut maze = maze::Maze::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut maze, None)?
            }
            "boids" => {
                let options = config.get_boids_options((width, height));
                let mut boids = boids::Boids::new(options);
                common::run_loop(guard.get_stdout(), &mut boids, None)?
            }
            "blank" => {
                let options = config.get_blank_options();
                let mut blank = blank::Blank::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut blank, None)?
            }
            "cube" => {
                let options = config.get_cube_options();
                let mut cube = cube::Cube::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut cube, None)?
            }
            "crab" => {
                let options = config.get_crab_options((width, height));
                let mut crab = crab::Crab::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut crab, None)?
            }
            "donut" => {
                let options = config.get_donut_options((width, height));
                let mut donut = donut::Donut::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut donut, None)?
            }
            "pipes" => {
                let options = config.get_pipes_options();
                let mut pipes = pipes::Pipes::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut pipes, None)?
            }
            "plasma" => {
                let options = config.get_plasma_options();
                let mut plasma = plasma::Plasma::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut plasma, None)?
            }
            "fire" => {
                let options = config.get_fire_options();
                let mut fire = fire::Fire::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut fire, None)?
            }
            "terrain" => {
                let options = config.get_terrain_options();
                let mut terrain = terrain::Terrain::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut terrain, None)?
            }
            _ => {
                println!(
                    "Pick screensaver: [matrix, life, maze, boids, cube, crab, donut]"
                );
                0.0
            }
        }
    };

    println!("{}", config_status);
    println!("Frames per second: {:.1}", fps);
    Ok(())
}

fn parse_args() -> Result<AppArgs, String> {
    let mut args = std::env::args().skip(1);
    let mut screen_saver = "matrix".to_string();
    let mut check = false;
    let mut effect = None;
    let mut frames = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" | "-v" => {
                print_version();
                std::process::exit(0);
            }
            "--check" => {
                check = true;
            }
            "--print-config" => {
                if let Err(e) = Config::print_default_config() {
                    eprintln!("Failed to print config: {}", e);
                    std::process::exit(1);
                }
                std::process::exit(0);
            }
            "--effect" => {
                effect = args.next();
            }
            "--frames" => {
                if let Some(frame_str) = args.next() {
                    frames = frame_str.parse().ok();
                }
            }
            arg if !arg.starts_with('-') => {
                if check {
                    effect = Some(arg.to_string());
                } else {
                    screen_saver = arg.to_string();
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", arg));
            }
        }
    }

    Ok(AppArgs {
        screen_saver,
        check,
        effect,
        frames,
    })
}

fn print_help() {
    println!("tarts - Terminal screensavers");
    println!();
    println!("USAGE:");
    println!("    tarts [EFFECT] [OPTIONS]");
    println!();
    println!("EFFECTS:");
    println!("    matrix      Matrix digital rain");
    println!("    life        Conway's Game of Life");
    println!("    maze        Maze generation");
    println!("    boids       Boids flocking simulation");
    println!("    cube        3D cube rotation");
    println!("    crab        ASCII crab animation");
    println!("    donut       3D donut rotation");
    println!("    pipes       Pipe maze animation");
    println!("    plasma      Plasma effect");
    println!("    fire        Fire simulation");
    println!("    terrain     Terrain generation");
    println!("    blank       Blank screen");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help              Show help");
    println!("    -v, --version           Show version");
    println!("        --check             Run test mode");
    println!("        --effect <EFFECT>    Effect to test (with --check)");
    println!("        --frames <NUM>       Number of frames to run (with --check)");
    println!("        --print-config       Print default config as TOML to stdout");
    println!();
    println!("CONFIG:");
    println!("    Config file (optional): ~/.config/tarts.toml");
    println!(
        "    Generate one with:      tarts --print-config > ~/.config/tarts.toml"
    );
    println!();
    println!("EXAMPLES:");
    println!("    tarts matrix            Run Matrix effect");
    println!("    tarts --check            Test with default effect");
    println!("    tarts --check life       Test Life effect");
    println!("    tarts --check --frames 100 life");
    println!("    tarts --print-config > ~/.config/tarts.toml");
    println!("    tarts --version          Show version");
}

fn print_version() {
    println!("tarts {}", env!("CARGO_PKG_VERSION"));
}
