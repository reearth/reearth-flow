//! Packing-efficiency benchmark for the multi-page packer. Efficiency uses a
//! full-page budget (`atlas_size^2 * page_count`), matching the planned no-crop
//! model; the packer itself still crops pages today.
//!
//! With an output path as the first argument, writes a self-contained HTML page
//! showing the packed pages for the first `RESULTS_PER_TEST` trials per test.

use std::io::Cursor;

use image::{Rgba, RgbaImage};
use reearth_flow_atlas::{build_atlas_multipage, TextureCache, TextureInput};

const TOTAL: usize = 10;
const MAX_ATLAS_SIZE: u32 = 1024;
const EXTRUSION: u32 = 0;
const RESULTS_PER_TEST: usize = 3;

fn next_u32(state: &mut u64) -> u32 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state >> 32) as u32
}

fn random_color(state: &mut u64) -> Rgba<u8> {
    Rgba([
        (next_u32(state) & 0xff) as u8,
        (next_u32(state) & 0xff) as u8,
        (next_u32(state) & 0xff) as u8,
        255,
    ])
}

struct Report {
    avg_ratio: f64,
    std_dev: f64,
    avg_pages: f64,
    failures: usize,
}

struct Trial {
    pages: Vec<String>,
}

fn run(
    name: &str,
    sample_fn: impl Fn(&mut u64) -> Vec<(u32, u32)>,
    collect_trials: bool,
) -> (Report, Vec<Trial>) {
    let mut state: u64 = 0x9E3779B97F4A7C15;
    let mut ratios: Vec<f64> = Vec::with_capacity(TOTAL);
    let mut page_counts: Vec<f64> = Vec::with_capacity(TOTAL);
    let mut failures: usize = 0;
    let mut trials: Vec<Trial> = Vec::new();

    let tmp = tempfile::TempDir::new().expect("create temp dir for generated textures");

    for trial in 0..TOTAL {
        let dims = sample_fn(&mut state);
        let mut cache = TextureCache::default();

        let mut materials: Vec<TextureInput> = Vec::with_capacity(dims.len());
        let mut packed_pixels: u64 = 0;
        for (i, &(w, h)) in dims.iter().enumerate() {
            let path = tmp.path().join(format!("{name}_{trial:04}_{i}.png"));
            RgbaImage::from_pixel(w, h, random_color(&mut state))
                .save(&path)
                .expect("write generated texture");
            packed_pixels += w as u64 * h as u64;
            materials.push(TextureInput {
                path,
                uvs: vec![vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]],
                scale: 1.0,
            });
        }

        match build_atlas_multipage(&materials, MAX_ATLAS_SIZE, EXTRUSION, &mut cache) {
            Ok(Some(built)) => {
                let pages = built.pages.len();
                let budget = (MAX_ATLAS_SIZE as f64).powi(2) * pages as f64;
                ratios.push(packed_pixels as f64 / budget);
                page_counts.push(pages as f64);

                if collect_trials && trials.len() < RESULTS_PER_TEST {
                    let pages = built.pages.iter().map(png_data_uri).collect();
                    trials.push(Trial { pages });
                }
            }
            Ok(None) | Err(_) => failures += 1,
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
    let avg_pages = if page_counts.is_empty() {
        0.0
    } else {
        page_counts.iter().sum::<f64>() / n
    };
    (
        Report {
            avg_ratio,
            std_dev,
            avg_pages,
            failures,
        },
        trials,
    )
}

fn uniform_rect(state: &mut u64) -> Vec<(u32, u32)> {
    const MIN: u32 = 16;
    const MAX: u32 = 256;
    let count = 96 + next_u32(state) % 97;
    (0..count)
        .map(|_| {
            let w = MIN + next_u32(state) % (MAX - MIN + 1);
            let h = MIN + next_u32(state) % (MAX - MIN + 1);
            (w, h)
        })
        .collect()
}

fn power_of_two(state: &mut u64) -> Vec<(u32, u32)> {
    const MIN_EXP: u32 = 3; // 8
    const MAX_EXP: u32 = 9; // 512
    let count = 96 + next_u32(state) % 97;
    (0..count)
        .map(|_| {
            let w_exp = MIN_EXP + next_u32(state) % (MAX_EXP - MIN_EXP + 1);
            let h_exp = MIN_EXP + next_u32(state) % (MAX_EXP - MIN_EXP + 1);
            (1 << w_exp, 1 << h_exp)
        })
        .collect()
}

// Items a large fraction of the page can't tile densely, so packing is
// dominated by wasted space on partially-filled pages — the regime unique to
// the multi-page builder.
fn large_rect(state: &mut u64) -> Vec<(u32, u32)> {
    const MIN: u32 = 320;
    const MAX: u32 = 704;
    let count = 12 + next_u32(state) % 13;
    (0..count)
        .map(|_| {
            let w = MIN + next_u32(state) % (MAX - MIN + 1);
            let h = MIN + next_u32(state) % (MAX - MIN + 1);
            (w, h)
        })
        .collect()
}

fn png_data_uri(img: &RgbaImage) -> String {
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(img.clone())
        .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
        .expect("encode page png");
    format!("data:image/png;base64,{}", base64(&png))
}

fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[(n >> 18 & 63) as usize] as char);
        out.push(T[(n >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            T[(n >> 6 & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn print_report(report: &Report, name: &str) {
    println!("{}", name);
    println!("Failures (empty/error): {}", report.failures);
    println!("Average packing efficiency: {:.4}", report.avg_ratio);
    println!("Standard deviation: {:.4}", report.std_dev);
    println!("Average pages: {:.2}", report.avg_pages);
    println!();
}

fn build_html(sections: &[(&str, &Report, &[Trial])]) -> String {
    let mut s = String::from(
        "<!doctype html><meta charset=utf-8><title>atlas packing</title>\
         <style>body{font-family:sans-serif;margin:1rem}\
         img{border:1px solid #ccc;image-rendering:pixelated;max-width:256px;margin:2px;vertical-align:top}\
         section{margin-bottom:2rem}</style>",
    );
    for (name, r, trials) in sections {
        s.push_str(&format!("<section><h2>{name}</h2>"));
        s.push_str(&format!(
            "<p>efficiency {:.4} ± {:.4}, avg pages {:.2}, failures {}</p>",
            r.avg_ratio, r.std_dev, r.avg_pages, r.failures
        ));
        for (i, t) in trials.iter().enumerate() {
            s.push_str(&format!("<div>trial {i} ({} pages)<br>", t.pages.len()));
            for uri in &t.pages {
                s.push_str(&format!("<img src=\"{uri}\">"));
            }
            s.push_str("</div>");
        }
        s.push_str("</section>");
    }
    s
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html_path = std::env::args().nth(1);
    let collect = html_path.is_some();

    let (r_u, t_u) = run("uniform_rect", uniform_rect, collect);
    print_report(&r_u, "Uniform Rectangles");
    let (r_p, t_p) = run("power_of_two", power_of_two, collect);
    print_report(&r_p, "Power of Two");
    let (r_l, t_l) = run("large_rect", large_rect, collect);
    print_report(&r_l, "Large Rectangles");

    if let Some(path) = html_path {
        let html = build_html(&[
            ("Uniform Rectangles", &r_u, &t_u),
            ("Power of Two", &r_p, &t_p),
            ("Large Rectangles", &r_l, &t_l),
        ]);
        std::fs::write(&path, html)?;
        println!("wrote {path}");
    }
    Ok(())
}
