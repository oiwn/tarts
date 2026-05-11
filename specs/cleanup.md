# Cleanup / Technical Debt Notes

## Config vs Runtime Calculations

The current config structs (e.g. `DigitalRainOptions`, `BoidsOptions`, `CrabOptions`) serve double duty: they store both user-facing config fields and runtime-computed values that depend on screen size. This creates two problems:

1. **`#[serde(skip)]` hack** — Fields like `drops_range`, `speed_range`, `initial_cells`, `screen_size`, `boid_count`, and `crab_count` are hidden from TOML serialization because they're computed at runtime. The struct carries dead weight for no reason.

2. **Leaky abstraction** — Effect constructors read from options fields that must be pre-populated before use. The config getter methods (`get_matrix_options()`, `get_boids_options()`, `get_crab_options()`, etc.) are responsible for filling them in. Direct construction of an options struct (outside the config path) would use wrong builder defaults.

### Affected effects

- **matrix**: `drops_range`, `speed_range` are hidden; `drops_coeff`, `speed_coeff` are user-facing
- **life**: `initial_cells` is hidden; `cells_coeff` is user-facing
- **boids**: `screen_size`, `boid_count` are hidden; `boid_coeff` is user-facing
- **crab**: `crab_count` is hidden; `crab_coeff` is user-facing

### Ideal design

Separate config from runtime state:

```rust
// Clean, serializable config — lives in tarts.toml
#[derive(Debug, Serialize, Deserialize)]
pub struct DigitalRainConfig {
    pub drops_coeff: f32,  // default 1.0
    pub speed_coeff: f32,  // default 1.0
}

// Runtime state — computed from config + screen size
pub struct DigitalRainOptions {
    pub drops_range: (u16, u16),
    pub speed_range: (u16, u16),
}

impl DigitalRainConfig {
    pub fn to_runtime(self, screen_size: (u16, u16)) -> DigitalRainOptions {
        let (w, h) = screen_size;
        let area = w as f32 * h as f32;
        DigitalRainOptions {
            drops_range: {
                let min = (area / 160.0 * self.drops_coeff) as u16;
                let max = (area / 80.0 * self.drops_coeff) as u16;
                (min.max(10), max.max(20))
            },
            speed_range: {
                let min = ((h as f32 / 20.0 * self.speed_coeff) as u16).max(2);
                let max = ((h as f32 / 10.0 * self.speed_coeff) as u16).max(16);
                (min, max)
            },
        }
    }
}
```

Same pattern applies to `ConwayLifeConfig`/`ConwayLifeOptions`, `BoidsConfig`/`BoidsOptions`, and `CrabConfig`/`CrabOptions`.

This eliminates the `#[serde(skip)]` hacks, makes the TOML schema self-documenting, and ensures runtime options are always computed correctly regardless of construction path.
