//! Terminal-based screensavers and visual effects.
//!
//! Modules:
//!
//! | Module   | Description                              |
//! |----------|------------------------------------------|
//! | `blank`  | Blank screen — no-op placeholder         |
//! | `boids`  | Boids flocking simulation                |
//! | `buffer` | Terminal cell buffer for colored output  |
//! | `butterfly` | Butterflies floating around           |
//! | `check`  | Terminal event checking (input, resize)  |
//! | `collision` | Shared bounding-box collision detection |
//! | `common` | Shared traits and types (TerminalEffect) |
//! | `config` | CLI configuration and argument parsing   |
//! | `constellation` | Drifting stars and dotted connections |
//! | `crab`   | ASCII crab walking animation             |
//! | `cube`   | 3D rotating cube in ASCII                |
//! | `donut`  | 3D rotating donut in ASCII               |
//! | `error`  | Error types for the crate                |
//! | `fire`   | Fire simulation effect                   |
//! | `life`   | Conway's Game of Life                    |
//! | `maze`   | Maze generation and animation            |
//! | `pipes`  | Pipe maze animation                      |
//! | `plasma` | Plasma color wave effect                 |
//! | `rain`   | Matrix-style digital rain                |
//! | `terrain`| Terrain generation — scrolling landscape |

pub mod blank;
pub mod boids;
pub mod buffer;
pub mod butterfly;
pub mod check;
pub mod collision;
pub mod common;
pub mod config;
pub mod constellation;
pub mod crab;
pub mod cube;
pub mod donut;
pub mod error;
pub mod fire;
pub mod life;
pub mod maze;
pub mod pipes;
pub mod plasma;
pub mod rain;
pub mod terrain;
