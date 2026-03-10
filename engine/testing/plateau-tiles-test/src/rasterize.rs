use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::GrayImage;
use image::ImageEncoder;

pub const RASTER_SIZE: usize = 1024;
pub const STROKE_PIXELS: f64 = (RASTER_SIZE / 256) as f64;

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

/// Wu's anti-aliased line drawing algorithm.
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
pub fn scanline_fill(raster: &mut [f32], rings: &[Vec<(f64, f64)>]) {
    if rings.is_empty() {
        return;
    }

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

/// Reads an 8-bit grayscale PNG file into a f32 raster.
pub fn read_raster_png(path: &std::path::Path) -> Result<Vec<f32>, String> {
    let img = image::open(path)
        .map_err(|e| format!("Failed to read PNG {:?}: {}", path, e))?
        .into_luma8();
    let GrayImage { .. } = img;
    Ok(img.pixels().map(|p| p.0[0] as f32 / 255.0).collect())
}

/// Returns `RASTER_SIZE * RASTER_SIZE` zero raster.
pub fn empty_raster() -> Vec<f32> {
    vec![0.0f32; RASTER_SIZE * RASTER_SIZE]
}

/// Pixel-wise RMS comparison between two f32 rasters.
pub fn compare_rasters(r1: &[f32], r2: &[f32]) -> f64 {
    let sum: f64 = r1
        .iter()
        .zip(r2.iter())
        .map(|(a, b)| {
            let diff = ((*a as f64) - (*b as f64)).abs();
            if diff >= 0.5 {
                diff
            } else {
                0.0
            }
        })
        .sum();
    (sum / r1.len() as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
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

    // draw_aa_circle is analytical: alpha = clamp(r + 0.5 - dist, 0, 1).
    // Every pixel in the bounding box must match the formula exactly (within f32 precision).
    #[test]
    fn test_draw_aa_circle_point_geometry() {
        let (cx, cy, r) = (512.0_f64, 512.0_f64, 4.0_f64);
        let mut raster = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];
        draw_aa_circle(&mut raster, cx, cy, r);

        let r_ceil = (r + 1.0).ceil() as i32;
        for py in (cy as i32 - r_ceil)..=(cy as i32 + r_ceil) {
            for px in (cx as i32 - r_ceil)..=(cx as i32 + r_ceil) {
                let dist = (px as f64 - cx).hypot(py as f64 - cy);
                let expected = (r + 0.5 - dist).clamp(0.0, 1.0) as f32;
                let actual = raster[py as usize * RASTER_SIZE + px as usize];
                assert!(
                    (actual - expected).abs() < 1e-6,
                    "pixel ({},{}) expected {}, got {}",
                    px,
                    py,
                    expected,
                    actual
                );
            }
        }
    }

    // scanline_fill is exact: interior pixels are 1.0, exterior are 0.0.
    // Square ring (100,100)→(200,100)→(200,200)→(100,200):
    //   rows y=100..=199, cols x=100..=200 → 100×101 = 10100 pixels filled.
    #[test]
    fn test_scanline_fill_polygon_geometry() {
        let mut raster = vec![0.0f32; RASTER_SIZE * RASTER_SIZE];
        let ring = vec![
            (100.0, 100.0),
            (200.0, 100.0),
            (200.0, 200.0),
            (100.0, 200.0),
        ];
        scanline_fill(&mut raster, &[ring]);
        let area: f32 = raster.iter().sum();
        assert!(
            (area - 10100.0).abs() < EPSILON as f32,
            "filled area should be 10100 pixels, got {}",
            area
        );
    }
}
