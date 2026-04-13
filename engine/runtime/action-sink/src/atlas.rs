// atlas builder shared between glTF and 3D Tiles generation

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ahash::RandomState;
use earcut::{utils3d::project3d_to_2d, Earcut};
use flatgeom::MultiPolygon;
use image::{ImageFormat, RgbaImage};
use indexmap::IndexSet;
use reearth_flow_gltf::{calculate_normal, Primitives};
use reearth_flow_types::{
    material::{self, Material},
    AttributeValue,
};
use serde::{Deserialize, Serialize};
use texture_packer::exporter::ImageExporter;
use texture_packer::texture::Texture;
use texture_packer::{TexturePacker, TexturePackerConfig};
use url::Url;

use crate::zip_eq_logged::ZipEqLoggedExt;

const MAX_ATLAS_SIZE: u32 = 8192;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GltfFeature {
    // polygons [x, y, z, u, v]
    pub polygons: MultiPolygon<'static, [f64; 5]>,
    // material ids for each polygon
    pub polygon_material_ids: Vec<u32>,
    // materials
    pub materials: IndexSet<Material>,
    // attribute values
    pub attributes: HashMap<String, AttributeValue>,
}

/// Check if UV coordinates wrap (go outside [0,1] range)
pub fn has_wrapping_uvs(uv_coords: &[(f64, f64)]) -> bool {
    uv_coords
        .iter()
        .any(|(u, v)| *u < 0.0 || *u > 1.0 || *v < 0.0 || *v > 1.0)
}

struct AtlasInfo {
    /// Maps texture path string → placed frame rect in the atlas
    frame_map: HashMap<String, texture_packer::Rect>,
    width: u32,
    height: u32,
    uri: Url,
}

/// Collect unique packable textures from features, sorted by area descending.
/// Skips textures larger than MAX_ATLAS_SIZE or used only with wrapping UVs.
fn collect_unique_textures(
    features: &[&GltfFeature],
) -> crate::errors::Result<Vec<(PathBuf, (u32, u32))>> {
    let mut seen: HashMap<PathBuf, (u32, u32)> = HashMap::new();

    for feature in features {
        for (poly, mat_id) in feature
            .polygons
            .iter()
            .zip_eq_logged(feature.polygon_material_ids.iter())
        {
            let mat = &feature.materials[*mat_id as usize];
            let Some(base_texture) = &mat.base_texture else {
                continue;
            };

            let path = base_texture.uri.to_file_path().map_err(|_| {
                crate::errors::SinkError::atlas_builder(
                    "Failed to convert texture URI to file path",
                )
            })?;

            if seen.contains_key(&path) {
                continue;
            }

            let uv_coords: Vec<(f64, f64)> = poly
                .raw_coords()
                .iter()
                .map(|[_, _, _, u, v]| (*u, *v))
                .collect();
            if has_wrapping_uvs(&uv_coords) {
                continue;
            }

            let (w, h) = image::image_dimensions(&path).map_err(|e| {
                crate::errors::SinkError::atlas_builder(format!(
                    "Failed to read image dimensions for '{}': {e}",
                    path.display()
                ))
            })?;

            if w > MAX_ATLAS_SIZE || h > MAX_ATLAS_SIZE {
                continue;
            }

            seen.insert(path, (w, h));
        }
    }

    let mut textures: Vec<(PathBuf, (u32, u32))> = seen.into_iter().collect();
    // Sort by area descending for best skyline packing
    textures.sort_by(|a, b| {
        let area_a = a.1 .0 as u64 * a.1 .1 as u64;
        let area_b = b.1 .0 as u64 * b.1 .1 as u64;
        area_b.cmp(&area_a)
    });

    Ok(textures)
}

/// Per-texture overhead in each dimension from extrusion on both sides.
const TEXTURE_EXTRUSION: u32 = 1;
const TEXTURE_OVERHEAD: u32 = 2 * TEXTURE_EXTRUSION;

fn estimate_atlas_size(textures: &[(PathBuf, (u32, u32))]) -> (u32, u32) {
    let total_area: u64 = textures
        .iter()
        .map(|(_, (w, h))| (*w + TEXTURE_OVERHEAD) as u64 * (*h + TEXTURE_OVERHEAD) as u64)
        .sum();
    let max_eff_dim = textures
        .iter()
        .map(|(_, (w, h))| (*w + TEXTURE_OVERHEAD).max(*h + TEXTURE_OVERHEAD))
        .max()
        .unwrap_or(0);

    let n = textures.len() as f64;
    let grid_side = (n.sqrt().ceil() as u32).saturating_mul(max_eff_dim);
    let area_side = (total_area as f64).sqrt().ceil() as u32;

    let side = grid_side
        .max(area_side)
        .next_power_of_two()
        .min(MAX_ATLAS_SIZE);

    (side, side)
}

/// Pack textures into a single atlas, export to disk, and return placement info.
/// Returns None if there are no textures to pack.
fn pack_textures(
    textures: &[(PathBuf, (u32, u32))],
    atlas_dir: &Path,
    image_format: ImageFormat,
    ext: &str,
) -> crate::errors::Result<Option<AtlasInfo>> {
    if textures.is_empty() {
        return Ok(None);
    }

    let (atlas_w, atlas_h) = estimate_atlas_size(textures);

    let config = TexturePackerConfig {
        max_width: atlas_w,
        max_height: atlas_h,
        allow_rotation: false,
        trim: false,
        texture_extrusion: TEXTURE_EXTRUSION,
        force_max_dimensions: false,
        ..Default::default()
    };

    let mut packer: TexturePacker<RgbaImage, String> = TexturePacker::new_skyline(config);

    for (path, _) in textures {
        let image = image::open(path)
            .map_err(|e| {
                crate::errors::SinkError::atlas_builder(format!(
                    "Failed to open texture '{}': {e}",
                    path.display()
                ))
            })?
            .to_rgba8();

        let key = path.to_string_lossy().into_owned();
        packer.pack_own(key, image).map_err(|_| {
            crate::errors::SinkError::atlas_builder(format!(
                "Texture '{}' does not fit in atlas ({}x{})",
                path.display(),
                atlas_w,
                atlas_h,
            ))
        })?;
    }

    let atlas_actual_w = packer.width();
    let atlas_actual_h = packer.height();

    let atlas_image = ImageExporter::export(&packer, None)
        .map_err(|e| crate::errors::SinkError::atlas_builder(e))?;

    let atlas_path = atlas_dir.join("0").with_extension(ext);
    atlas_image
        .save_with_format(&atlas_path, image_format)
        .map_err(|e| {
            crate::errors::SinkError::atlas_builder(format!("Failed to save atlas: {e}"))
        })?;

    let uri = Url::from_file_path(&atlas_path).map_err(|_| {
        crate::errors::SinkError::atlas_builder("Failed to convert atlas path to URL")
    })?;

    let frame_map = packer
        .get_frames()
        .iter()
        .map(|(key, frame)| (key.clone(), frame.frame))
        .collect();

    Ok(Some(AtlasInfo {
        frame_map,
        width: atlas_actual_w,
        height: atlas_actual_h,
        uri,
    }))
}

/// Remap a UV coordinate from source texture space into atlas space.
///
/// `frame` is the placed rect in the atlas (image-space, y-down).
/// Input/output `v` uses GL convention (0 = bottom).
fn remap_uv(
    u: f64,
    v: f64,
    frame: &texture_packer::Rect,
    atlas_w: f64,
    atlas_h: f64,
) -> (f64, f64) {
    let px = u * frame.w as f64;
    let py = (1.0 - v) * frame.h as f64; // flip to image-space (y-down)
    let new_u = (px + frame.x as f64) / atlas_w;
    let new_v = 1.0 - (py + frame.y as f64) / atlas_h; // flip back to GL-space
    (new_u, new_v)
}

/// Build atlas and process geometry with remapped UVs.
///
/// Packs unique source textures into a single atlas image, remaps polygon UV
/// coordinates to atlas space, triangulates via earcut, and accumulates results
/// into `primitives` and `vertices`.
pub fn build_atlas_geometry(
    features: &[&GltfFeature],
    atlas_dir: &Path,
    image_format: ImageFormat,
    ext: &str,
    primitives: &mut Primitives,
    vertices: &mut IndexSet<[u32; 9], RandomState>,
) -> crate::errors::Result<()> {
    let textures = collect_unique_textures(features)?;
    let atlas = pack_textures(&textures, atlas_dir, image_format, ext)?;

    for (feature_id, feature) in features.iter().enumerate() {
        for (mut mat, mut poly) in feature
            .polygons
            .iter()
            .zip_eq_logged(feature.polygon_material_ids.iter())
            .map(|(poly, mat_id)| (feature.materials[*mat_id as usize].clone(), poly))
        {
            // Remap UVs if this polygon's texture was packed into the atlas
            if let Some(ref info) = atlas {
                if let Some(base_texture) = &mat.base_texture {
                    if let Ok(path) = base_texture.uri.to_file_path() {
                        let key = path.to_string_lossy().into_owned();
                        if let Some(frame) = info.frame_map.get(&key) {
                            let aw = info.width as f64;
                            let ah = info.height as f64;

                            poly.transform_inplace(|&[x, y, z, u, v]| {
                                let (new_u, new_v) = remap_uv(u, v, frame, aw, ah);
                                [x, y, z, new_u, new_v]
                            });

                            mat = material::Material {
                                base_color: mat.base_color,
                                base_texture: Some(material::Texture {
                                    uri: info.uri.clone(),
                                }),
                            };
                        }
                    }
                }
            }

            // Triangulate and emit geometry
            let primitive = primitives.entry(mat).or_default();
            primitive.feature_ids.insert(feature_id as u32);

            if let Some((nx, ny, nz)) =
                calculate_normal(poly.exterior().iter().map(|v| [v[0], v[1], v[2]]))
            {
                let num_outer_points = match poly.hole_indices().first() {
                    Some(&v) => v as usize,
                    None => poly.raw_coords().len(),
                };
                let mut earcutter = Earcut::new();
                let mut buf3d: Vec<[f64; 3]> = Vec::new();
                let mut buf2d: Vec<[f64; 2]> = Vec::new();
                let mut index_buf: Vec<u32> = Vec::new();

                buf3d.extend(poly.raw_coords().iter().map(|c| [c[0], c[1], c[2]]));

                if project3d_to_2d(&buf3d, num_outer_points, &mut buf2d) {
                    earcutter.earcut(buf2d.iter().cloned(), poly.hole_indices(), &mut index_buf);

                    primitive.indices.extend(index_buf.iter().map(|&idx| {
                        let [x, y, z, u, v] = poly.raw_coords()[idx as usize];
                        let vbits = [
                            (x as f32).to_bits(),
                            (y as f32).to_bits(),
                            (z as f32).to_bits(),
                            (nx as f32).to_bits(),
                            (ny as f32).to_bits(),
                            (nz as f32).to_bits(),
                            (u as f32).to_bits(),
                            ((1.0 - v) as f32).to_bits(),
                            (feature_id as f32).to_bits(),
                        ];
                        let (index, _) = vertices.insert_full(vbits);
                        index as u32
                    }));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_texture(dir: &Path, name: &str, width: u32, height: u32) -> PathBuf {
        use image::{ImageBuffer, Rgb};
        let img = ImageBuffer::<Rgb<u8>, _>::new(width, height);
        let path = dir.join(name);
        img.save(&path).unwrap();
        path
    }

    fn create_test_feature(
        texture_path: &Path,
        uvs: Vec<(f64, f64)>,
        offset_x: f64,
    ) -> GltfFeature {
        let coords: Vec<[f64; 5]> = vec![
            [offset_x, 0.0, 0.0, uvs[0].0, uvs[0].1],
            [offset_x + 1.0, 0.0, 0.0, uvs[1].0, uvs[1].1],
            [offset_x + 1.0, 1.0, 0.0, uvs[2].0, uvs[2].1],
            [offset_x, 1.0, 0.0, uvs[3].0, uvs[3].1],
        ];

        let mut polygons = MultiPolygon::new();
        polygons.add_exterior(coords);

        GltfFeature {
            polygons,
            polygon_material_ids: vec![0],
            materials: indexmap::indexset! {
                Material {
                    base_color: [1.0, 1.0, 1.0, 1.0],
                    base_texture: Some(material::Texture {
                        uri: Url::from_file_path(texture_path).unwrap(),
                    }),
                }
            },
            attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_wrapping_uvs_skipped() {
        let temp_dir = TempDir::new().unwrap();
        let texture_path = create_test_texture(temp_dir.path(), "test.png", 64, 64);

        let feature = create_test_feature(
            &texture_path,
            vec![(0.0, 0.0), (1.5, 0.0), (1.5, 1.0), (0.0, 1.0)],
            0.0,
        );
        let features = vec![&feature];
        let textures = collect_unique_textures(&features).unwrap();
        assert!(textures.is_empty());
    }

    #[test]
    fn test_estimate_atlas_size() {
        let textures = vec![
            (PathBuf::from("a"), (100u32, 80u32)),
            (PathBuf::from("b"), (100u32, 80u32)),
            (PathBuf::from("c"), (50u32, 120u32)),
        ];
        let (w, h) = estimate_atlas_size(&textures);
        // effective: (102,82),(102,82),(52,122); total_area≈23704, max_eff=122, grid=ceil(sqrt(3))*122=244, area≈154 → max(244,154)→next_pow2=256
        assert_eq!(w, 256);
        assert_eq!(h, 256);
    }

    #[test]
    fn test_large_texture_skipped() {
        let temp_dir = TempDir::new().unwrap();
        let texture_path = create_test_texture(temp_dir.path(), "large.png", 16384, 1);
        let feature = create_test_feature(
            &texture_path,
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            0.0,
        );
        let features = vec![&feature];
        let textures = collect_unique_textures(&features).unwrap();
        assert!(textures.is_empty(), "oversized texture should be excluded");
    }

    #[test]
    fn test_build_atlas_geometry_uv_mapping() {
        use image::{ImageBuffer, Rgb, RgbImage};

        let temp_dir = TempDir::new().unwrap();

        let img1: RgbImage =
            ImageBuffer::from_fn(256, 256, |x, y| Rgb([x as u8, y as u8, 10u8]));
        let path1 = temp_dir.path().join("texture1.png");
        img1.save(&path1).unwrap();

        let img2: RgbImage =
            ImageBuffer::from_fn(256, 256, |x, y| Rgb([x as u8, y as u8, 20u8]));
        let path2 = temp_dir.path().join("texture2.png");
        img2.save(&path2).unwrap();

        let feature1 = create_test_feature(
            &path1,
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            0.0,
        );
        let feature2 = create_test_feature(
            &path2,
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            2.0,
        );
        let features = vec![&feature1, &feature2];

        let atlas_dir = temp_dir.path().join("atlas");
        std::fs::create_dir(&atlas_dir).unwrap();

        let mut primitives: Primitives = Default::default();
        let mut vertices: IndexSet<[u32; 9], RandomState> = IndexSet::default();

        build_atlas_geometry(
            &features,
            &atlas_dir,
            ImageFormat::Png,
            "png",
            &mut primitives,
            &mut vertices,
        )
        .unwrap();

        assert!(!primitives.is_empty());
        assert!(!vertices.is_empty());
        assert_eq!(primitives.len(), 1, "both features share one atlas material");

        let atlas_path = atlas_dir.join("0.png");
        assert!(atlas_path.exists());
    }
}
