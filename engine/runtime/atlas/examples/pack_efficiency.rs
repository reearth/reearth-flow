use std::path::Path;

use image::{Rgb, RgbImage};
use reearth_flow_atlas::plan_layout;

const TOTAL: usize = 10;
const TEXTURES_PER_TRIAL: usize = 64;
const MAX_ATLAS_SIZE: u32 = 4096;

fn next_u32(state: &mut u64) -> u32 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state >> 32) as u32
}

fn random_color(state: &mut u64) -> Rgb<u8> {
    Rgb([
        (next_u32(state) & 0xff) as u8,
        (next_u32(state) & 0xff) as u8,
        (next_u32(state) & 0xff) as u8,
    ])
}

struct Report {
    avg_ratio: f64,
    std_dev: f64,
    avg_k: f64,
    failures: usize,
}

fn run(name: &str, dim_fn: impl Fn(&mut u64) -> (u32, u32), dump_dir: Option<&Path>) -> Report {
    let mut state: u64 = 0x9E3779B97F4A7C15;
    let mut ratios: Vec<f64> = Vec::with_capacity(TOTAL);
    let mut k_values: Vec<f64> = Vec::with_capacity(TOTAL);
    let mut failures: usize = 0;

    for trial in 0..TOTAL {
        let dims: Vec<_> = (0..TEXTURES_PER_TRIAL)
            .map(|_| dim_fn(&mut state))
            .collect();
        match plan_layout(&dims, MAX_ATLAS_SIZE) {
            Ok(plan) => {
                let atlas_area = plan.atlas_width as f64 * plan.atlas_height as f64;
                let ds = plan.downsample;
                let packed_pixels: u64 = dims
                    .iter()
                    .map(|&(w, h)| w.div_ceil(ds).max(1) as u64 * h.div_ceil(ds).max(1) as u64)
                    .sum();
                ratios.push(packed_pixels as f64 / atlas_area);
                k_values.push(plan.downsample.trailing_zeros() as f64);

                if let Some(dir) = dump_dir {
                    let mut img = RgbImage::new(plan.atlas_width, plan.atlas_height);
                    for (&(w, h), &(x, y)) in dims.iter().zip(plan.placements.iter()) {
                        let pw = w.div_ceil(ds).max(1);
                        let ph = h.div_ceil(ds).max(1);
                        let color = random_color(&mut state);
                        for py in y..y + ph {
                            for px in x..x + pw {
                                if px < plan.atlas_width && py < plan.atlas_height {
                                    img.put_pixel(px, py, color);
                                }
                            }
                        }
                    }
                    let path = dir.join(format!("{name}_{trial:04}.png"));
                    img.save(&path).unwrap();
                }
            }
            Err(_) => failures += 1,
        }
    }

    let n = ratios.len() as f64;
    let avg_ratio = if ratios.is_empty() {
        0.0
    } else {
        ratios.iter().sum::<f64>() / n
    };
    let std_dev = if ratios.len() < 2 {
        0.0
    } else {
        (ratios.iter().map(|r| (r - avg_ratio).powi(2)).sum::<f64>() / n).sqrt()
    };
    let avg_k = if k_values.is_empty() {
        0.0
    } else {
        k_values.iter().sum::<f64>() / n
    };
    Report {
        avg_ratio,
        std_dev,
        avg_k,
        failures,
    }
}

fn uniform_rect(state: &mut u64) -> (u32, u32) {
    const MIN: u32 = 16;
    const MAX: u32 = 256;
    let w = MIN + next_u32(state) % (MAX - MIN + 1);
    let h = MIN + next_u32(state) % (MAX - MIN + 1);
    (w, h)
}

fn power_of_two(state: &mut u64) -> (u32, u32) {
    const MIN_EXP: u32 = 3; // 8
    const MAX_EXP: u32 = 12; // 4096
    let w_exp = MIN_EXP + next_u32(state) % (MAX_EXP - MIN_EXP + 1);
    let h_exp = MIN_EXP + next_u32(state) % (MAX_EXP - MIN_EXP + 1);
    (1 << w_exp, 1 << h_exp)
}

fn print_report(report: &Report, name: &str) {
    println!("{}", name);
    println!("Failures (could not fit): {}", report.failures);
    println!("Average packing efficiency: {:.4}", report.avg_ratio);
    println!("Standard deviation: {:.4}", report.std_dev);
    println!("Average k: {:.2}", report.avg_k);
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dump = std::env::args().any(|a| a == "--dump");
    let dump_dir = if dump {
        let dir = std::env::temp_dir().join("reearth-flow-atlas-dump");
        std::fs::create_dir_all(&dir)?;
        println!("dumping layouts to {}\n", dir.display());
        Some(dir)
    } else {
        None
    };

    let r = run("uniform_rect", uniform_rect, dump_dir.as_deref());
    print_report(&r, "Uniform Rectangles");

    let r = run("power_of_two", power_of_two, dump_dir.as_deref());
    print_report(&r, "Power of Two");

    Ok(())
}
