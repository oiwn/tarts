//! Terminal adapter for the pocketworld: options, the `TerminalEffect`
//! wiring, and key handling. The simulation lives in `sim` (composed
//! from the layer modules); drawing lives in `render`.

use crate::buffer::{Buffer, Cell};
use crate::common::{DefaultOptions, TerminalEffect};
use crate::pocketworld::render::{Layer, Renderer, draw_legend, rgb};
use crate::pocketworld::sim::{Sim, SimLayer};
use crossterm::style;
use derive_builder::Builder;
use rand::RngExt;
use serde::{Deserialize, Serialize};

#[derive(Builder, Default, Debug, Clone, Serialize, Deserialize)]
#[builder(public, setter(into))]
pub struct PocketWorldOptions {
    #[builder(default = "42")]
    pub seed: u32,
    /// Base frequency of the continent-scale field (lower = bigger landmasses).
    #[builder(default = "0.022")]
    pub scale: f64,
    /// Base frequency of the detail field (coastlines, hills).
    #[builder(default = "0.09")]
    pub detail_scale: f64,
    #[builder(default = "3")]
    pub octaves: i32,
    #[builder(default = "4")]
    pub detail_octaves: i32,
    #[builder(default = "0.5")]
    pub persistence: f64,
    /// Domain-warp displacement in cells.
    #[builder(default = "9.0")]
    pub warp_strength: f64,
    /// Spatial frequency of the warp field itself.
    #[builder(default = "0.025")]
    pub warp_scale: f64,
    /// Simulation layers to skip at runtime (empty = all on). This is
    /// groundwork for perf work, not a display control — layer cycling
    /// stays on the `l`/`k` keys.
    #[serde(default)]
    #[builder(default)]
    pub disabled: Vec<SimLayer>,
}

pub struct PocketWorld {
    pub screen_size: (u16, u16),
    options: PocketWorldOptions,
    buffer: Buffer,
    sim: Sim,
    renderer: Renderer,
    rng: rand::prelude::ThreadRng,
}

impl PocketWorld {
    pub fn new(options: PocketWorldOptions, screen_size: (u16, u16)) -> Self {
        let width = screen_size.0.max(1);
        let height = screen_size.1.max(1);
        Self {
            screen_size: (width, height),
            buffer: Buffer::new(width as usize, height as usize),
            sim: Sim::new(&options, (width, height)),
            options,
            renderer: Renderer::new(),
            rng: rand::rng(),
        }
    }
}

impl TerminalEffect for PocketWorld {
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Char('k') => self.renderer.view = self.renderer.view.next(),
            KeyCode::Char('j') => self.renderer.view = self.renderer.view.prev(),
            KeyCode::Char('l') => self.renderer.legend = !self.renderer.legend,
            _ => {}
        }
    }

    /// Programmatic face of `j`/`k`: the layer cycle as named views
    /// (used by `tarts --analyze`).
    fn views(&self) -> Vec<&'static str> {
        vec![
            "biome", "plates", "height", "hydro", "temp", "wind", "humidity",
            "aquifer",
        ]
    }

    fn set_view(&mut self, name: &str) {
        self.renderer.view = match name {
            "biome" => Layer::Biome,
            "plates" => Layer::Plates,
            "height" => Layer::Height,
            "hydro" => Layer::Hydro,
            "temp" => Layer::Temp,
            "wind" => Layer::Wind,
            "humidity" => Layer::Humidity,
            "aquifer" => Layer::Aquifer,
            _ => return,
        };
    }

    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)> {
        let mut curr_buffer =
            Buffer::new(self.screen_size.0 as usize, self.screen_size.1 as usize);

        if self.renderer.legend {
            // The legend is its own screen, not an overlay.
            draw_legend(&mut curr_buffer);
        } else {
            for y in 0..curr_buffer.height {
                for x in 0..curr_buffer.width {
                    let (character, color) =
                        self.renderer.classify(&self.sim, x, y);
                    curr_buffer.set(
                        x,
                        y,
                        Cell::new(
                            character,
                            color,
                            style::Attribute::NormalIntensity,
                        ),
                    );
                }
            }

            // HUD: always say which layer is on screen.
            let label = self.renderer.hud_label();
            for (n, ch) in label.chars().enumerate() {
                if n < curr_buffer.width {
                    curr_buffer.set(
                        n,
                        0,
                        Cell::new(
                            ch,
                            rgb(210, 212, 216),
                            style::Attribute::NormalIntensity,
                        ),
                    );
                }
            }
        }

        let diff = self.buffer.diff(&curr_buffer);
        self.buffer = curr_buffer;
        diff
    }

    fn update(&mut self) {
        self.sim.tick();
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.screen_size = (width.max(1), height.max(1));
        self.reset();
    }

    fn reset(&mut self) {
        // Re-roll the world from fresh entropy.
        self.options.seed = self.rng.random();
        self.sim = Sim::new(&self.options, self.screen_size);
    }
}

impl DefaultOptions for PocketWorld {
    type Options = PocketWorldOptions;

    fn default_options(_width: u16, _height: u16) -> Self::Options {
        PocketWorldOptionsBuilder::default().build().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pocketworld::render::Layer;

    fn pocketworld(seed: u32, w: usize, h: usize) -> PocketWorld {
        let options = PocketWorldOptionsBuilder::default()
            .seed(seed)
            .build()
            .unwrap();
        PocketWorld::new(options, (w as u16, h as u16))
    }

    #[test]
    fn reset_re_rolls_the_world() {
        let mut t = pocketworld(11, 60, 20);
        let before: Vec<f32> = t.sim.world.elevation.clone();
        t.reset();
        let differing = before
            .iter()
            .zip(t.sim.world.elevation.iter())
            .filter(|(a, b)| (**a - **b).abs() > 0.01)
            .count();
        assert!(differing > 100, "reset produced a near-identical world");
    }

    #[test]
    fn survives_degenerate_sizes() {
        for (w, h) in [(0u16, 0u16), (0, 24), (80, 0), (1, 1)] {
            let options = PocketWorldOptionsBuilder::default().build().unwrap();
            let mut t = PocketWorld::new(options, (w, h));
            t.update();
            t.get_diff();
        }
    }

    #[test]
    fn set_view_maps_names() {
        let mut t = pocketworld(31, 60, 20);
        assert_eq!(t.views().len(), 8);
        t.set_view("aquifer");
        assert_eq!(t.renderer.view, Layer::Aquifer);
        // Unknown names are ignored, not fatal.
        t.set_view("nonsense");
        assert_eq!(t.renderer.view, Layer::Aquifer);
    }

    #[test]
    fn layer_cycling_and_legend() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut t = pocketworld(31, 60, 20);
        let next = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let prev = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let legend = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        assert_eq!(t.renderer.view, Layer::Biome);
        // Forward: k walks the whole cycle and wraps.
        for expected in [
            Layer::Plates,
            Layer::Height,
            Layer::Hydro,
            Layer::Temp,
            Layer::Wind,
            Layer::Humidity,
            Layer::Aquifer,
            Layer::Biome,
        ] {
            t.handle_key(next);
            assert_eq!(t.renderer.view, expected);
            // Every layer renders without panicking.
            t.update();
            t.get_diff();
        }
        // Backward: j walks the cycle in reverse.
        for expected in [
            Layer::Aquifer,
            Layer::Humidity,
            Layer::Wind,
            Layer::Temp,
            Layer::Hydro,
            Layer::Height,
            Layer::Plates,
            Layer::Biome,
        ] {
            t.handle_key(prev);
            assert_eq!(t.renderer.view, expected);
            t.update();
            t.get_diff();
        }
        // Legend: l toggles the overlay, rendering stays safe.
        assert!(!t.renderer.legend);
        t.handle_key(legend);
        assert!(t.renderer.legend);
        t.update();
        t.get_diff();
        t.handle_key(legend);
        assert!(!t.renderer.legend);
    }
}
