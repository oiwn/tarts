use crate::buffer::Cell;
use crossterm::{
    QueueableCommand, cursor, event,
    style::{self, Stylize},
    terminal,
};
use std::{
    io::{BufWriter, Result, Write},
    time::Duration,
};

pub trait DefaultOptions {
    type Options;

    /// Calculate default options based on terminal dimensions
    #[allow(unused)]
    fn default_options(width: u16, height: u16) -> Self::Options;
}

pub trait TerminalEffect {
    /// get difference between frames, this is used to minimize screen updates
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)>;
    /// Update to next frame
    fn update(&mut self);
    // Update screen size option, each saver should implement it by itself
    fn update_size(&mut self, width: u16, height: u16);
    /// Reset effect, i think it's useful in case of size/options update
    fn reset(&mut self);
}

pub fn process_input() -> Result<bool> {
    if event::poll(Duration::from_millis(10))?
        && let event::Event::Key(keyevent) = event::read()?
        && (keyevent
            == event::KeyEvent::new(
                event::KeyCode::Char('q'),
                event::KeyModifiers::NONE,
            )
            || keyevent
                == event::KeyEvent::new(
                    event::KeyCode::Esc,
                    event::KeyModifiers::NONE,
                )
            || keyevent
                == event::KeyEvent::new(
                    event::KeyCode::Char('c'),
                    event::KeyModifiers::CONTROL,
                ))
    {
        return Ok(false);
    }
    Ok(true)
}

pub fn run_loop<W, TE>(
    stdout: &mut W,
    effect: &mut TE,
    iterations: Option<usize>,
) -> Result<f64>
where
    W: Write,
    TE: TerminalEffect,
{
    let (width, height) = terminal::size()?;

    // #[cfg(test)]
    let mut iters: usize = 0;

    let mut is_running = true;
    let mut frames_per_second = 0.0;
    let target_frame_duration = Duration::from_secs_f64(1.0 / 60.0_f64);

    // wrap in buffer due to tests "run_loop_fps_gte_0" failing on CI/CD
    // NOTE: 12/Dec/2023 issue with tests of CI/CD still not resolved
    let mut buffered_stdout = BufWriter::new(stdout);

    // main loop
    while is_running {
        let started_at: std::time::SystemTime = std::time::SystemTime::now();
        is_running = process_input()?;

        #[allow(clippy::single_match)]
        while event::poll(Duration::from_millis(10))? {
            match event::read()? {
                event::Event::Resize(new_width, new_height) => {
                    // Update size and reset effect
                    effect.update_size(new_width, new_height);
                    effect.reset();
                }
                _ => {}
            }
        }

        // draw diff
        let queue = effect.get_diff();
        for item in queue.iter() {
            let (x, y, cell) = item;
            debug_assert!(*x < width as usize && *y < height as usize);
            buffered_stdout.queue(cursor::MoveTo(*x as u16, *y as u16))?;
            buffered_stdout.queue(style::PrintStyledContent(
                cell.symbol.with(cell.color).attribute(cell.attr),
            ))?;
        }
        buffered_stdout.flush()?;
        effect.update();

        // stabilize fps if requred
        let ended_at = std::time::SystemTime::now();
        let delta = ended_at
            .duration_since(started_at)
            .unwrap_or(std::time::Duration::from_secs(0));
        if delta < target_frame_duration {
            std::thread::sleep(target_frame_duration - delta);
        };

        // calculate actual frame rate
        let ended_at = std::time::SystemTime::now();
        let delta = ended_at
            .duration_since(started_at)
            .unwrap_or(std::time::Duration::from_secs(0));
        frames_per_second = (frames_per_second + (1.0 / delta.as_secs_f64())) / 2.0;

        if delta < target_frame_duration {
            std::thread::sleep(target_frame_duration - delta);
        }

        // #[cfg(test)]
        if let Some(iterations) = iterations {
            iters += 1;
            if iters > iterations {
                is_running = false;
            }
        };
    }
    Ok(frames_per_second)
}
