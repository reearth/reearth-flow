use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgb, RgbImage};
use reearth_flow_atlas::{build_atlas, TextureMaterial};

const MAX_ATLAS_SIZE: u32 = 1024;

fn next_u32(state: &mut u64) -> u32 {
    // Small xorshift PRNG so the example does not need an extra dependency.
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

fn parse_dims(line: &str, line_no: usize) -> Result<(u32, u32), String> {
    let mut parts = line.split(',').map(str::trim);
    let w = parts
        .next()
        .ok_or_else(|| format!("line {}: missing width", line_no))?
        .parse::<u32>()
        .map_err(|e| format!("line {}: invalid width: {}", line_no, e))?;
    let h = parts
        .next()
        .ok_or_else(|| format!("line {}: missing height", line_no))?
        .parse::<u32>()
        .map_err(|e| format!("line {}: invalid height: {}", line_no, e))?;
    if parts.next().is_some() {
        return Err(format!("line {}: expected exactly two columns", line_no));
    }
    if w == 0 || h == 0 {
        return Err(format!("line {}: width and height must be > 0", line_no));
    }
    Ok((w, h))
}

fn make_material(path: PathBuf) -> TextureMaterial {
    TextureMaterial {
        path,
        uvs: vec![vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]],
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err(format!("usage: {} <input.csv> <output.png>", args[0]).into());
    }

    let input_csv = Path::new(&args[1]);
    let output_path = Path::new(&args[2]);

    let csv = fs::read_to_string(input_csv)?;
    let dims: Vec<_> = csv
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(parse_dims(trimmed, idx + 1))
            }
        })
        .collect::<Result<_, _>>()?;

    if dims.is_empty() {
        return Err("input CSV must contain at least one non-empty `w,h` row".into());
    }

    let seed = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64 ^ 0x9E3779B97F4A7C15;
    let mut state = seed;

    let work_dir =
        env::temp_dir().join(format!("reearth-flow-atlas-example-{}", std::process::id()));
    if work_dir.exists() {
        fs::remove_dir_all(&work_dir)?;
    }
    fs::create_dir_all(&work_dir)?;

    let atlas_dir = work_dir.join("atlas");
    fs::create_dir_all(&atlas_dir)?;

    let materials: Vec<_> = dims
        .into_iter()
        .enumerate()
        .map(|(idx, (w, h))| -> Result<_, Box<dyn std::error::Error>> {
            let path = work_dir.join(format!("input_{idx:04}.png"));
            let image: RgbImage = ImageBuffer::from_pixel(w, h, random_color(&mut state));
            image.save(&path)?;
            Ok(make_material(path))
        })
        .collect::<Result<_, _>>()?;

    build_atlas(
        &materials,
        &atlas_dir,
        ImageFormat::Png,
        "png",
        MAX_ATLAS_SIZE,
    )?;

    let built_atlas = atlas_dir.join("0.png");
    if !built_atlas.exists() {
        return Err("atlas generation did not produce 0.png".into());
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&built_atlas, output_path)?;

    println!("wrote atlas to {}", output_path.display());
    Ok(())
}
