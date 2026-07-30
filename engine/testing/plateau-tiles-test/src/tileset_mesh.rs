use crate::align_cesium::load_tileset;
use crate::tileset::collect_tile_contents;
use glam::DVec3;
use reearth_flow_gltf::{parse_gltf, read_indices, read_positions_with_transform, traverse_scene};
use std::path::Path;

/// Loads every triangle referenced by a tileset's `tileset.json`, in world
/// space (glTF node transforms already applied) and `f64`, matching how
/// `align_cesium.rs`'s `GeometryCollector` reads the same files. This is the
/// only place that knows about glTF/3D Tiles — `render3d` takes plain
/// `[Vec3; 3]` triangles and has no format awareness at all. Kept at `f64`
/// here (not cast down to `f32`) because this data can be ECEF-scale and the
/// precision-safe recentering (`render3d::input::recenter_and_cast`) happens
/// per-render, so one load can serve multiple cameras.
pub fn load_triangles(tileset_dir: &Path) -> Result<Vec<[DVec3; 3]>, String> {
    let tileset_info = load_tileset(tileset_dir)?;
    let mut triangles = Vec::new();

    let Some(root) = tileset_info.content.get("root") else {
        return Ok(triangles);
    };

    for content in collect_tile_contents(tileset_dir, root)? {
        let bytes = std::fs::read(&content.path)
            .map_err(|e| format!("Failed to read GLB {:?}: {}", content.path, e))?;
        let gltf = parse_gltf(&bytes::Bytes::from(bytes))
            .map_err(|e| format!("Failed to parse GLB {:?}: {}", content.path, e))?;
        let buffer_data = vec![gltf
            .blob
            .as_ref()
            .ok_or_else(|| format!("GLB {:?} has no binary blob", content.path))?
            .clone()];

        for scene in gltf.scenes() {
            traverse_scene(&scene, |node, world_transform| -> Result<(), String> {
                let Some(mesh) = node.mesh() else {
                    return Ok(());
                };
                for primitive in mesh.primitives() {
                    let Some(pos_accessor) = primitive.get(&::gltf::Semantic::Positions) else {
                        continue;
                    };
                    let positions = read_positions_with_transform(
                        &pos_accessor,
                        &buffer_data,
                        Some(world_transform),
                    )
                    .map_err(|e| format!("Failed to read positions: {}", e))?;

                    let Some(idx_accessor) = primitive.indices() else {
                        continue;
                    };
                    let indices = read_indices(&idx_accessor, &buffer_data)
                        .map_err(|e| format!("Failed to read indices: {}", e))?;
                    if !indices.len().is_multiple_of(3) {
                        return Err(format!(
                            "Invalid index count {} (not divisible by 3) in {:?}",
                            indices.len(),
                            content.path
                        ));
                    }

                    // Cesium stores an ECEF (x, y, z) vertex as (x, z, -y); flip it back to true ECEF.
                    for chunk in indices.chunks(3) {
                        let v = |i: usize| {
                            let c = positions[i];
                            DVec3::new(c.x, -c.z, c.y)
                        };
                        triangles.push([v(chunk[0]), v(chunk[1]), v(chunk[2])]);
                    }
                }
                Ok(())
            })?;
        }
    }

    Ok(triangles)
}
