// Manual dev smoke test: renders a real tileset from a top-down and an
// oblique camera, writing depth PNGs to an output dir for visual inspection.
// Not wired into the test suite yet.
//
// usage: cargo run --bin render3d-smoke -- <tileset_dir> <output_dir>

use glam::DVec3;
use plateau_tiles_test::render3d::camera::Camera3d;
use plateau_tiles_test::render3d::{input, rasterizer};
use plateau_tiles_test::tileset_mesh::load_triangles;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: render3d-smoke <tileset_dir> <output_dir>");
        std::process::exit(1);
    }
    let tileset_dir = PathBuf::from(&args[1]);
    let output_dir = PathBuf::from(&args[2]);
    std::fs::create_dir_all(&output_dir).unwrap();

    let load_start = Instant::now();
    let triangles = load_triangles(&tileset_dir).expect("failed to load triangles");
    println!(
        "loaded {} triangles in {:.2?}",
        triangles.len(),
        load_start.elapsed()
    );

    let mut min = DVec3::splat(f64::INFINITY);
    let mut max = DVec3::splat(f64::NEG_INFINITY);
    for tri in &triangles {
        for v in tri {
            min = min.min(*v);
            max = max.max(*v);
        }
    }
    println!("bbox min={:?} max={:?}", min, max);
    let center = (min + max) * 0.5;
    let size = max - min;
    let radius = size.length() * 0.5;

    // Input is ECEF, so "up" (local zenith) is the radial direction from
    // Earth's center, not any fixed global axis. Build a local ENU-ish frame
    // from that so the cameras actually look straight down / obliquely at the
    // ground instead of at some arbitrary angle relative to the ECEF axes.
    let up_local = center.normalize();
    let east = up_local.cross(DVec3::Z).normalize();
    let north = east.cross(up_local).normalize();

    let width = 1024;
    let height = 1024;

    let cameras = [
        (
            "top_down",
            Camera3d {
                position: center + up_local * radius * 2.0,
                look_at: center,
                up: north,
                fov_y_deg: 45.0,
                near: radius * 0.1,
                far: radius * 5.0,
            },
        ),
        (
            "oblique",
            Camera3d {
                position: center + (east - north + up_local) * radius * 1.5,
                look_at: center,
                up: up_local,
                fov_y_deg: 45.0,
                near: radius * 0.1,
                far: radius * 5.0,
            },
        ),
    ];

    for (name, camera) in cameras {
        let reference = camera.position;
        let render_start = Instant::now();
        let local_triangles = input::recenter_and_cast(&triangles, reference);
        let view_proj = camera.view_proj_f32(width, height, reference);
        let canvas = rasterizer::render_depth(&local_triangles, view_proj, width, height);
        println!("rendered {} in {:.2?}", name, render_start.elapsed());

        let exact_path = output_dir.join(format!("{}.exact.png", name));
        canvas.write_png_f32(&exact_path).unwrap();
        println!("wrote {}", exact_path.display());

        let preview_path = output_dir.join(format!("{}.preview.png", name));
        write_preview_png(&canvas, &preview_path);
        println!("wrote {}", preview_path.display());
    }
}

/// Min-max-stretches finite depth values to `[0, 1]` for human eyeballing only
/// (not used for comparison) — raw distances aren't meaningfully in `[0, 1]`.
fn write_preview_png(canvas: &plateau_tiles_test::rasterize::Canvas, path: &std::path::Path) {
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for &d in &canvas.data {
        if d.is_finite() {
            min = min.min(d);
            max = max.max(d);
        }
    }
    let range = (max - min).max(1e-9);

    let mut preview = plateau_tiles_test::rasterize::Canvas::new(canvas.width, canvas.height);
    for (i, &d) in canvas.data.iter().enumerate() {
        preview.data[i] = if d.is_finite() {
            1.0 - (d - min) / range
        } else {
            0.0
        };
    }
    preview.write_png(path).unwrap();
}
