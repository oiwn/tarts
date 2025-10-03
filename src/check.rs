use crate::common::TerminalEffect;
use crate::error::Result;
use crossterm::{
    cursor,
    event::{self, Event},
    execute,
    style::Stylize,
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};
use std::time::Duration;

/// Runs a terminal screensaver effect for a limited number of frames to validate its functionality.
///
/// This function initializes the terminal in alternate screen mode, runs the specified effect
/// for the given number of frames, and waits for user input before restoring the terminal state.
/// Useful for testing and debugging screensaver effects.
///
/// # Arguments
/// * `effect` - The terminal effect implementation to run
/// * `frames` - Number of frames to render before pausing
///
/// # Returns
/// * `Result<(), TartsError>` - Success or error with terminal operations
///
/// # Example
/// ```ignore
/// let options = DigitalRainOptionsBuilder::default()
///     .screen_size((80, 40))
///     .build()?;
/// let mut effect = DigitalRain::new(options);
/// test_effect(&mut effect, 100)?;
/// ```
pub fn test_effect<T: TerminalEffect>(effect: &mut T, frames: usize) -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    for frame in 1..=frames {
        // Clear the screen
        execute!(stdout, Clear(ClearType::All))?;

        // Get the diff for the current frame
        let diff = effect.get_diff();

        // Render the frame
        for (x, y, cell) in diff {
            execute!(
                stdout,
                cursor::MoveTo(x as u16, y as u16),
                crossterm::style::PrintStyledContent(
                    cell.symbol.with(cell.color).attribute(cell.attr)
                )
            )?;
        }

        // Print frame number
        execute!(
            stdout,
            cursor::MoveTo(0, 0),
            crossterm::style::Print(format!("Frame: {}", frame))
        )?;

        stdout.flush()?;

        // Update the effect for the next frame
        effect.update();
    }

    // Wait for any key press
    loop {
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(_) = event::read()? {
                break;
            }
    }

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

/// Run appropirate effect till frame number
pub fn run_test_for_effect(effect_name: &str, frames: usize) -> Result<()> {
    match effect_name {
        "matrix" => {
            let options =
                crate::rain::digital_rain::DigitalRainOptionsBuilder::default()
                    .drops_range((120, 240))
                    .speed_range((2, 16))
                    .build()
                    .unwrap();
            let mut digital_rain = crate::rain::digital_rain::DigitalRain::new(
                options,
                terminal::size()?,
            );
            test_effect(&mut digital_rain, frames)
        }
        "life" => {
            let options = crate::life::ConwayLifeOptionsBuilder::default()
                .build()
                .unwrap();
            let mut conway_life =
                crate::life::ConwayLife::new(options, terminal::size()?);
            test_effect(&mut conway_life, frames)
        }
        "maze" => {
            let options =
                crate::maze::MazeOptionsBuilder::default().build().unwrap();
            let mut maze = crate::maze::Maze::new(options, terminal::size()?);
            test_effect(&mut maze, frames)
        }
        _ => {
            println!(
                "Unknown effect: {}. Available effects are: matrix, life, maze",
                effect_name
            );
            Ok(())
        }
    }
}
