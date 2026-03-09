use crate::compare_attributes::make_feature_key;
use crate::conv_mvt::tinymvt_value_to_json;
use tinymvt::geometry::GeometryDecoder;
use tinymvt::tag::TagsDecoder;
use tinymvt::vector_tile::Tile;

pub const RASTER_SIZE: usize = 1024;
pub const STROKE_PIXELS: f64 = (RASTER_SIZE / 256) as f64;

/// Rasterizes all geometry in a tile for a given feature ident into a RASTER_SIZE x RASTER_SIZE f32 raster.
/// Lines: capsule SDF per segment (analytically correct round joins/caps).
/// Polygon interiors: scanline fill. Polygon boundaries: Wu line. Points: circle SDF.
pub fn rasterize_tile_feature(tile: &Tile, ident: &str) -> Vec<f32> {
    let mut raster = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];

    for layer in &tile.layers {
        let tags_decoder = TagsDecoder::new(&layer.keys, &layer.values);
        let extent = layer.extent.unwrap_or(4096);
        let scale = 1.0 / extent as f64;

        for feature in &layer.features {
            // Check if this feature matches the ident
            let tags = match tags_decoder.decode(&feature.tags) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let mut props = serde_json::Map::new();
            for (key, value) in tags {
                use crate::conv_mvt::tinymvt_value_to_json;
                props.insert(key.to_string(), tinymvt_value_to_json(&value));
            }
            let props_value = serde_json::Value::Object(props);
            let feature_key = crate::compare_attributes::make_feature_key(&props_value, None);
            if feature_key != ident {
                continue;
            }

            let geom_type = feature.r#type.unwrap_or(0);
            let mut decoder = GeometryDecoder::new(&feature.geometry);

            match geom_type {
                1 => {
                    // Point
                    if let Ok(points) = decoder.decode_points() {
                        for point in &points {
                            let x = point[0] as f64 * scale * RASTER_SIZE as f64;
                            let y = point[1] as f64 * scale * RASTER_SIZE as f64;
                            draw_aa_circle(&mut raster, x, y, STROKE_PIXELS);
                        }
                    }
                }
                2 => {
                    // LineString: capsule SDF per segment for correct round joins/caps
                    if let Ok(linestrings) = decoder.decode_linestrings() {
                        for ls in &linestrings {
                            for window in ls.windows(2) {
                                let x0 = window[0][0] as f64 * scale * RASTER_SIZE as f64;
                                let y0 = window[0][1] as f64 * scale * RASTER_SIZE as f64;
                                let x1 = window[1][0] as f64 * scale * RASTER_SIZE as f64;
                                let y1 = window[1][1] as f64 * scale * RASTER_SIZE as f64;
                                draw_capsule(&mut raster, x0, y0, x1, y1, STROKE_PIXELS);
                            }
                        }
                    }
                }
                3 => {
                    // Polygon: scanline fill interior + Wu boundary
                    if let Ok(polygons) = decoder.decode_polygons() {
                        for rings in &polygons {
                            if rings.is_empty() {
                                continue;
                            }
                            // Convert all rings to pixel coords
                            let pixel_rings: Vec<Vec<(f64, f64)>> = rings
                                .iter()
                                .map(|ring| {
                                    ring.iter()
                                        .map(|p| {
                                            (
                                                p[0] as f64 * scale * RASTER_SIZE as f64,
                                                p[1] as f64 * scale * RASTER_SIZE as f64,
                                            )
                                        })
                                        .collect()
                                })
                                .collect();

                            // Scanline fill using even-odd rule across all rings
                            scanline_fill(&mut raster, &pixel_rings);

                            // Wu 1px boundary for all rings
                            for ring in &pixel_rings {
                                for window in ring.windows(2) {
                                    draw_wu_line(
                                        &mut raster,
                                        window[0].0,
                                        window[0].1,
                                        window[1].0,
                                        window[1].1,
                                    );
                                }
                                // Close the ring
                                if ring.len() >= 2 {
                                    let last = ring[ring.len() - 1];
                                    let first = ring[0];
                                    draw_wu_line(&mut raster, last.0, last.1, first.0, first.1);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    raster
}

fn set_pixel(raster: &mut [f32], x: i32, y: i32, alpha: f32) {
    if x >= 0 && x < RASTER_SIZE as i32 && y >= 0 && y < RASTER_SIZE as i32 {
        let idx = y as usize * RASTER_SIZE + x as usize;
        raster[idx] = f32::max(raster[idx], alpha);
    }
}

/// Capsule SDF renderer. Draws a thick line segment from (ax,ay) to (bx,by) with radius r.
/// alpha = clamp(r + 0.5 - dist_to_segment, 0, 1): boundary pixels (dist=r) get alpha=0.5,
/// interior pixels get 1.0. Adjacent segments share overlapping caps forming round joins via max().
pub fn draw_capsule(raster: &mut [f32], ax: f64, ay: f64, bx: f64, by: f64, r: f64) {
    let dx = bx - ax;
    let dy = by - ay;
    let len2 = dx * dx + dy * dy;

    let pad = (r + 1.0).ceil() as i32;
    let x0 = (ax.min(bx).floor() as i32 - pad).max(0);
    let y0 = (ay.min(by).floor() as i32 - pad).max(0);
    let x1 = (ax.max(bx).ceil() as i32 + pad).min(RASTER_SIZE as i32 - 1);
    let y1 = (ay.max(by).ceil() as i32 + pad).min(RASTER_SIZE as i32 - 1);

    for py in y0..=y1 {
        for px in x0..=x1 {
            let dist = if len2 < 1e-20 {
                (px as f64 - ax).hypot(py as f64 - ay)
            } else {
                let t = (((px as f64 - ax) * dx + (py as f64 - ay) * dy) / len2).clamp(0.0, 1.0);
                (px as f64 - (ax + t * dx)).hypot(py as f64 - (ay + t * dy))
            };
            let alpha = (r + 0.5 - dist).clamp(0.0, 1.0) as f32;
            if alpha > 0.0 {
                set_pixel(raster, px, py, alpha);
            }
        }
    }
}

/// Anti-aliased circle splat for point features. alpha = clamp(r + 0.5 - dist, 0, 1).
pub fn draw_aa_circle(raster: &mut [f32], cx: f64, cy: f64, r: f64) {
    let r_ceil = (r + 1.0).ceil() as i32;
    let x0 = cx.floor() as i32 - r_ceil;
    let y0 = cy.floor() as i32 - r_ceil;
    let x1 = cx.ceil() as i32 + r_ceil;
    let y1 = cy.ceil() as i32 + r_ceil;
    for py in y0..=y1 {
        for px in x0..=x1 {
            let dist = (px as f64 - cx).hypot(py as f64 - cy);
            let alpha = (r + 0.5 - dist).clamp(0.0, 1.0) as f32;
            set_pixel(raster, px, py, alpha);
        }
    }
}

/// Wu's anti-aliased line drawing algorithm
pub fn draw_wu_line(raster: &mut [f32], x0: f64, y0: f64, x1: f64, y1: f64) {
    let mut x0 = x0;
    let mut y0 = y0;
    let mut x1 = x1;
    let mut y1 = y1;

    let steep = (y1 - y0).abs() > (x1 - x0).abs();
    if steep {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
    }
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }

    let dx = x1 - x0;
    let dy = y1 - y0;
    let gradient = if dx.abs() < 1e-10 { 1.0 } else { dy / dx };

    let set = |raster: &mut [f32], x: i32, y: i32, alpha: f32| {
        let (px, py) = if steep { (y, x) } else { (x, y) };
        if px >= 0 && px < RASTER_SIZE as i32 && py >= 0 && py < RASTER_SIZE as i32 {
            let idx = py as usize * RASTER_SIZE + px as usize;
            raster[idx] = f32::max(raster[idx], alpha);
        }
    };

    let xend = x0.round();
    let yend = y0 + gradient * (xend - x0);
    let xgap = 1.0 - (x0 + 0.5).fract();
    let xpxl1 = xend as i32;
    let ypxl1 = yend.floor() as i32;
    set(
        raster,
        xpxl1,
        ypxl1,
        (1.0 - yend.fract()) as f32 * xgap as f32,
    );
    set(raster, xpxl1, ypxl1 + 1, yend.fract() as f32 * xgap as f32);
    let mut intery = yend + gradient;

    let xend = x1.round();
    let yend = y1 + gradient * (xend - x1);
    let xgap = (x1 + 0.5).fract();
    let xpxl2 = xend as i32;
    let ypxl2 = yend.floor() as i32;
    set(
        raster,
        xpxl2,
        ypxl2,
        (1.0 - yend.fract()) as f32 * xgap as f32,
    );
    set(raster, xpxl2, ypxl2 + 1, yend.fract() as f32 * xgap as f32);

    for x in (xpxl1 + 1)..xpxl2 {
        let y = intery.floor() as i32;
        set(raster, x, y, (1.0 - intery.fract()) as f32);
        set(raster, x, y + 1, intery.fract() as f32);
        intery += gradient;
    }
}

/// Scanline fill using even-odd rule. rings[0] is exterior, rest are holes.
fn scanline_fill(raster: &mut [f32], rings: &[Vec<(f64, f64)>]) {
    if rings.is_empty() {
        return;
    }

    // Bounding box
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for ring in rings {
        for &(_, y) in ring {
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }

    let y_start = (min_y.floor() as i32).max(0);
    let y_end = (max_y.ceil() as i32).min(RASTER_SIZE as i32 - 1);

    for y in y_start..=y_end {
        let yf = y as f64;
        let mut xs: Vec<f64> = Vec::new();

        for ring in rings {
            let n = ring.len();
            for i in 0..n {
                let (x0, y0) = ring[i];
                let (x1, y1) = ring[(i + 1) % n];
                if (y0 <= yf && y1 > yf) || (y1 <= yf && y0 > yf) {
                    let t = (yf - y0) / (y1 - y0);
                    xs.push(x0 + t * (x1 - x0));
                }
            }
        }

        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Fill between pairs (even-odd)
        for chunk in xs.chunks(2) {
            if chunk.len() == 2 {
                let x_start = (chunk[0].floor() as i32).max(0);
                let x_end = (chunk[1].ceil() as i32).min(RASTER_SIZE as i32 - 1);
                for x in x_start..=x_end {
                    let idx = y as usize * RASTER_SIZE + x as usize;
                    raster[idx] = 1.0;
                }
            }
        }
    }
}

/// Writes a f32 raster to an 8-bit grayscale PNG file.
pub fn write_raster_png(raster: &[f32], path: &std::path::Path) -> Result<(), String> {
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    use image::ImageEncoder;
    let pixels: Vec<u8> = raster
        .iter()
        .map(|&v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create dirs {:?}: {}", parent, e))?;
    }
    let file =
        std::fs::File::create(path).map_err(|e| format!("Failed to create {:?}: {}", path, e))?;
    PngEncoder::new_with_quality(file, CompressionType::Best, FilterType::Up)
        .write_image(
            &pixels,
            RASTER_SIZE as u32,
            RASTER_SIZE as u32,
            image::ExtendedColorType::L8,
        )
        .map_err(|e| format!("Failed to write PNG {:?}: {}", path, e))
}

/// Returns all unique feature keys present in a tile (across all layers and geometry types).
pub fn make_feature_keys_in_tile(tile: &Tile) -> Vec<String> {
    let mut keys = Vec::new();
    for layer in &tile.layers {
        let tags_decoder = TagsDecoder::new(&layer.keys, &layer.values);
        for feature in &layer.features {
            let tags = match tags_decoder.decode(&feature.tags) {
                Ok(t) => t,
                Err(_) => continue,
            };
            let mut props = serde_json::Map::new();
            for (key, value) in tags {
                let json_value = tinymvt_value_to_json(&value);
                props.insert(key.to_string(), json_value);
            }
            let key = make_feature_key(&serde_json::Value::Object(props), None);
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
    }
    keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conv_png::compare_rasters;
    const EPSILON: f64 = 1e-6;

    // Verifies compare_rasters can detect a known difference.
    // A horizontal line from (100,100) to (200,100) at exact integer y:
    //   endpoints x=100, x=200 → value 0.5 each (xgap = 0.5)
    //   interior x=101..=199   → value 1.0 each (99 pixels)
    // All values ≥ 0.5, so sum = 0.5 + 99×1.0 + 0.5 = 100.0
    // score = sqrt(100.0 / RASTER_SIZE²)
    #[test]
    fn test_compare_rasters_known_line_score() {
        let mut r1 = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];
        let r2 = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];
        draw_wu_line(&mut r1, 100.0, 100.0, 200.0, 100.0);
        let score = compare_rasters(&r1, &r2);
        let expected = (100.0_f64 / (RASTER_SIZE * RASTER_SIZE) as f64).sqrt();
        assert!(
            (score - expected).abs() < EPSILON,
            "Expected score ~{}, got {}",
            expected,
            score
        );
    }
}
