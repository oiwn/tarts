use crate::rain::gradient;
use crate::rain::rain_drop::RainDropStyle;
use crossterm::style;

pub fn pick_style(vw_style: &RainDropStyle, pos: usize) -> style::Attribute {
    match vw_style {
        RainDropStyle::Front => style::Attribute::Bold,
        RainDropStyle::Middle => match pos {
            0..=4 => style::Attribute::Bold,
            _ => style::Attribute::NormalIntensity,
        },
        RainDropStyle::Back => style::Attribute::Bold,
        _ => style::Attribute::NormalIntensity,
    }
}

pub fn pick_color(
    vw_style: &RainDropStyle,
    pos: usize,
    gradients: &[Vec<gradient::Color>],
) -> style::Color {
    match vw_style {
        RainDropStyle::Gradient => match pos {
            0 => style::Color::White,
            _ => style::Color::Rgb {
                r: 0,
                g: 255 - (pos as u16 * 12).clamp(10, 256) as u8,
                b: 0,
            },
        },
        RainDropStyle::Front => match pos {
            0 => style::Color::White,
            _ => style::Color::Rgb {
                r: 0,
                g: 255 - (pos.pow(2) as u16).clamp(10, 256) as u8,
                b: 0,
            },
        },
        RainDropStyle::Back => {
            // clamp to the gradient row: drops can outgrow the palette at
            // degenerate sizes (issue #94)
            let color = gradients[2]
                .get(pos)
                .or_else(|| gradients[2].last())
                .copied();
            match color {
                Some(c) => style::Color::Rgb {
                    r: c.r,
                    g: c.g,
                    b: c.b,
                },
                None => style::Color::DarkGrey,
            }
        }
        _ => style::Color::DarkGrey,
    }
}

#[cfg(test)]
mod tests {
    use crate::rain::digital_rain::{DigitalRain, DigitalRainOptionsBuilder};

    // use crossterm::terminal;
    // use std::io::{self, Write};

    fn get_default_rain() -> DigitalRain {
        let rain_options = DigitalRainOptionsBuilder::default()
            .drops_range((10, 20))
            .speed_range((2, 15))
            .build()
            .unwrap();
        DigitalRain::new(rain_options, (30, 30))
    }

    #[test]
    fn run_loop_10_iterations() {
        let mut stdout = Vec::new();
        let mut digital_rain = get_default_rain();
        let _ = crate::common::run_loop(&mut stdout, &mut digital_rain, Some(10));
    }

    /* // NOTE: this test failed on github CI pipeline
    #[test]
    fn run_loop_fps_gte_0() {
        let mut stdout = Vec::new();
        let mut digital_rain = get_default_rain();
        // let mut fps: f64 = 0.0;
        let fps = crate::common::run_loop(&mut stdout, &mut digital_rain, Some(10))
            .unwrap();

        /*
        for _ in 0..10 {
            let fps_res =
                crate::common::run_loop(&mut stdout, &mut digital_rain, Some(10));
            if let Ok(f) = fps_res {
                fps = f;
                break;
            }
        }
        */
        assert_eq!(fps > 0.0, true);
    } */
}
