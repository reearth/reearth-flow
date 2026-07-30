use crate::profile_config::CameraConfig;
use crate::render3d::camera::Camera3d;
use crate::render3d::{input, rasterizer};
use crate::tileset_mesh::load_triangles;
use glam::DVec3;
use std::collections::HashMap;
use std::path::Path;

fn to_camera3d(cfg: &CameraConfig) -> Camera3d {
    let position = DVec3::from_array(cfg.position);
    let up = match cfg.up {
        Some(u) => DVec3::from_array(u),
        None => position.normalize(),
    };
    Camera3d {
        position,
        look_at: DVec3::from_array(cfg.look_at),
        up,
        fov_y_deg: cfg.fov_y_deg,
        near: cfg.near,
        far: cfg.far,
    }
}

/// Renders every named camera's depth buffer against a 3D Tiles tileset,
/// writing one lossless-f32 PNG per camera into `out_dir` (see
/// `Canvas::write_png_f32`).
pub fn render_cameras_to_pngs(
    tileset_dir: &Path,
    out_dir: &Path,
    cameras: &HashMap<String, CameraConfig>,
    width: usize,
    height: usize,
) -> Result<(), String> {
    if !tileset_dir.exists() {
        return Err(format!("tileset_dir does not exist: {:?}", tileset_dir));
    }
    let triangles = load_triangles(tileset_dir)?;

    for (name, cfg) in cameras {
        let camera = to_camera3d(cfg);
        let reference = camera.position;
        let local_triangles = input::recenter_and_cast(&triangles, reference);
        let view_proj = camera.view_proj_f32(width, height, reference);
        let canvas = rasterizer::render_depth(&local_triangles, view_proj, width, height);
        canvas.write_png_f32(&out_dir.join(format!("{}.png", name)))?;
    }

    Ok(())
}
