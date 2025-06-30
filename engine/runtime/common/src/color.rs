use colorsys::{ColorAlpha, Hsl, Rgb};

pub type ColorTupleA = (u8, u8, u8, u8);

pub fn convert_hsl_to_rgba(
    hue: f64,
    saturation: f64,
    lightness: f64,
    alpha: f64,
) -> crate::Result<ColorTupleA> {
    let mut hsl = Hsl::default();
    hsl.set_hue(hue);
    hsl.set_saturation(saturation);
    hsl.set_lightness(lightness);
    hsl.set_alpha(alpha);
    let rgb = Rgb::from(&hsl);
    let color_code = percent_to_hex_color(rgb.alpha())?;
    Ok((
        rgb.red().round() as u8,
        rgb.green().round() as u8,
        rgb.blue().round() as u8,
        color_code as u8,
    ))
}

fn percent_to_hex_color(percent: f64) -> crate::Result<i64> {
    let hex = (255.0 * percent).round() as u8;
    i64::from_str_radix(format!("{hex:02X}").as_str(), 16)
        .map_err(|e| crate::Error::Color(format!("{e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_hsl_to_rgb() {
        let hue = 62.28448275862069f64;
        let saturation = 100.0f64;
        let lightness = 70.0f64;
        let alpha = 1.0f64;

        let converted_rgb = convert_hsl_to_rgba(hue, saturation, lightness, alpha).unwrap();
        assert_eq!(converted_rgb, (249, 255, 102, 255));
    }
}
