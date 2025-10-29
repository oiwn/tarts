//! # tarts
//!
//! `tarts` or TerminalArts is a collection of terminal-based screen savers written
//! in Rust. This crate provides a variety of screen savers like "Matrix Rain",
//! "Conway's Game of Life", and "Maze Generation" (not yet), all running directly
//! in your terminal.
//!
//! ## Features
//!
//! - Matrix Rain: Simulates the famous "Matrix" digital rain effect in your terminal.
//! - Conway's Game of Life: Implements the classic cellular automaton in the terminal.
//! - Maze Generation: Generates and displays a random maze.
//! - Boids
//!
//! ## Usage
//!
//! To use the screen savers, run the executable with the desired screen saver's
//! name as an argument:
//!
//! ```bash
//! tarts matrix
//! tarts life
//! tarts maze
//! tarts boids
//! tarts cube
//! tarts crab
//! ```
#![cfg(not(test))]
use crossterm::{self, cursor, execute, terminal};
// use tarts::{config, rain};
// use log::info;
use crate::common::DefaultOptions;
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
    "plasma", "fire",
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
        // Ignore errors during drop - we'''re doing best effort cleanup
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
    // let config = Config::load()?;

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

    let fps = {
        let mut guard = TerminalGuard::new()?;
        let (width, height) = terminal::size()?;

        match args.screen_saver.as_str() {
            "matrix" => {
                // let options = config.get_matrix_options((width, height));
                let options =
                    rain::digital_rain::DigitalRain::default_options(width, height);
                let mut digital_rain =
                    rain::digital_rain::DigitalRain::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut digital_rain, None)?
            }
            "life" => {
                // let options = config.get_life_options((width, height));
                let options = life::ConwayLife::default_options(width, height);
                let mut conway_life =
                    life::ConwayLife::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut conway_life, None)?
            }
            "maze" => {
                // let options = config.get_maze_options((width, height));
                let options = maze::Maze::default_options(width, height);
                let mut maze = maze::Maze::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut maze, None)?
            }
            "boids" => {
                // let options = config.get_boids_options((width, height));
                let options = boids::Boids::default_options(width, height);
                let mut boids = boids::Boids::new(options);
                common::run_loop(guard.get_stdout(), &mut boids, None)?
            }
            "blank" => {
                let options =
                    blank::BlankOptionsBuilder::default().build().unwrap();
                let mut check = blank::Blank::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut check, None)?
            }
            "cube" => {
                // let options = config.get_cube_options();
                let options = cube::effect::Cube::default_options(width, height);
                let mut cube = cube::Cube::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut cube, None)?
            }
            "crab" => {
                let options = crab::Crab::default_options(width, height);
                let mut crab = crab::Crab::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut crab, None)?
            }
            "donut" => {
                let options = donut::Donut::default_options(width, height);
                let mut donut = donut::Donut::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut donut, None)?
            }
            "pipes" => {
                let options = pipes::Pipes::default_options(width, height);
                let mut pipes = pipes::Pipes::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut pipes, None)?
            }
            "plasma" => {
                let options = plasma::Plasma::default_options(width, height);
                let mut plasma = plasma::Plasma::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut plasma, None)?
            }
            "fire" => {
                let options = fire::Fire::default_options(width, height);
                let mut fire = fire::Fire::new(options, (width, height));
                common::run_loop(guard.get_stdout(), &mut fire, None)?
            }
            "terrain" => {
                let options = terrain::Terrain::default_options(width, height);
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

    println!("Frames per second: {}", fps);
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
            "--generate-config" => {
                if let Err(e) = Config::save_default_config() {
                    eprintln!("Failed to generate config: {}", e);
                    std::process::exit(1);
                }
                println!("Default configuration generated successfully");
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
    println!("    blank       Blank screen");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help              Show help");
    println!("    -v, --version           Show version");
    println!("        --check             Run test mode");
    println!("        --effect <EFFECT>    Effect to test (with --check)");
    println!("        --frames <NUM>       Number of frames to run (with --check)");
    println!("        --generate-config    Generate default config file");
    println!();
    println!("EXAMPLES:");
    println!("    tarts matrix            Run Matrix effect");
    println!("    tarts --check            Test with default effect");
    println!("    tarts --check life       Test Life effect");
    println!("    tarts --check --frames 100 life");
    println!("    tarts --version          Show version");
}

fn print_version() {
    println!("tarts {}", env!("CARGO_PKG_VERSION"));
}
