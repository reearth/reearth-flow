use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use ahash::RandomState;
use atlas_packer::pack::AtlasPacker;
use atlas_packer::texture::cache::TextureSizeCache;
use atlas_packer::texture::{DownsampleFactor, PolygonMappedTexture};
use earcut::{utils3d::project3d_to_2d, Earcut};
use flatgeom::MultiPolygon;
use indexmap::IndexSet;
use itertools::Itertools;
use reearth_flow_gltf::{calculate_normal, Primitives};
use reearth_flow_types::{
    material::{self, Material},
    AttributeValue,
};
use serde::{Deserialize, Serialize};
use url::Url;

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

pub fn load_textures_into_packer<F>(
    features: &[&GltfFeature],
    packer: &Mutex<AtlasPacker>,
    texture_size_cache: &TextureSizeCache,
    texture_id_generator: &F,
    geom_error: f64,
    limit_texture_resolution: bool,
) -> crate::errors::Result<(u32, u32, HashSet<String>)>
where
    F: Fn(usize, usize) -> String,
{
    let mut wrapping_textures = HashSet::new();
    let mut max_width = 0;
    let mut max_height = 0;

    for (feature_id, feature) in features.iter().enumerate() {
        for (poly_count, (mat, poly)) in feature
            .polygons
            .iter()
            .zip_eq(feature.polygon_material_ids.iter())
            .map(move |(poly, orig_mat_id)| {
                (feature.materials[*orig_mat_id as usize].clone(), poly)
            })
            .enumerate()
        {
            if let Some(base_texture) = mat.base_texture {
                let original_vertices = poly
                    .raw_coords()
                    .iter()
                    .map(|[x, y, z, u, v]| (*x, *y, *z, *u, *v))
                    .collect::<Vec<(f64, f64, f64, f64, f64)>>();

                let uv_coords = original_vertices
                    .iter()
                    .map(|(_, _, _, u, v)| (*u, *v))
                    .collect::<Vec<(f64, f64)>>();

                let texture_id = texture_id_generator(feature_id, poly_count);

                // Check if this texture has wrapping UVs
                if has_wrapping_uvs(&uv_coords) {
                    wrapping_textures.insert(texture_id);
                    continue; // Skip atlas packing for wrapping textures
                }

                let texture_uri = base_texture.uri.to_file_path().map_err(|_| {
                    crate::errors::SinkError::GltfWriter(
                        "Failed to convert texture URI to file path".to_string(),
                    )
                })?;
                let texture_size = texture_size_cache.get_or_insert(&texture_uri);

                let downsample_scale = if limit_texture_resolution {
                    reearth_flow_common::texture::get_texture_downsample_scale_of_polygon(
                        &original_vertices,
                        texture_size,
                    ) as f32
                } else {
                    1.0
                };

                let factor = reearth_flow_common::texture::apply_downsample_factor(
                    geom_error,
                    downsample_scale,
                );
                let downsample_factor = DownsampleFactor::new(&factor);

                let texture = PolygonMappedTexture::new(
                    &texture_uri,
                    texture_size,
                    &uv_coords,
                    downsample_factor,
                );

                // Track the full texture size after downsampling (like main branch)
                let scaled_width = (texture_size.0 as f32 * factor) as u32;
                let scaled_height = (texture_size.1 as f32 * factor) as u32;

                max_width = max_width.max(scaled_width);
                max_height = max_height.max(scaled_height);

                packer
                    .lock()
                    .map_err(|_| {
                        crate::errors::SinkError::GltfWriter(
                            "Failed to lock the texture packer".to_string(),
                        )
                    })?
                    .add_texture(texture_id, texture);
            }
        }
    }

    let max_width = max_width.next_power_of_two();
    let max_height = max_height.next_power_of_two();

    Ok((max_width, max_height, wrapping_textures))
}

#[allow(clippy::too_many_arguments)]
pub fn process_geometry_with_atlas<F, P>(
    features: &[&GltfFeature],
    packed: &atlas_packer::pack::PackedAtlasProvider,
    wrapping_textures: &std::collections::HashSet<String>,
    ext: &str,
    texture_id_generator: F,
    atlas_path_builder: P,
    primitives: &mut Primitives,
    vertices: &mut IndexSet<[u32; 9], RandomState>,
) -> Result<(), crate::errors::SinkError>
where
    F: Fn(usize, usize) -> String,
    P: Fn(atlas_packer::AtlasID) -> std::path::PathBuf,
{
    for (feature_id, feature) in features.iter().enumerate() {
        for (poly_count, (mut mat, mut poly)) in feature
            .polygons
            .iter()
            .zip_eq(feature.polygon_material_ids.iter())
            .map(move |(poly, orig_mat_id)| {
                (feature.materials[*orig_mat_id as usize].clone(), poly)
            })
            .enumerate()
        {
            let original_vertices = poly
                .raw_coords()
                .iter()
                .map(|[x, y, z, u, v]| (*x, *y, *z, *u, *v))
                .collect::<Vec<(f64, f64, f64, f64, f64)>>();

            let texture_id = texture_id_generator(feature_id, poly_count);

            // Check if this is a wrapping texture
            if wrapping_textures.contains(&texture_id) {
                // For wrapping textures, keep the original UVs and use the original texture directly
                // No need to transform UVs or update material - use as-is
                // The material already has the correct texture URI from the feature
            } else if let Some(info) = packed.get_texture_info(&texture_id) {
                let atlas_placed_uv_coords = info
                    .placed_uv_coords
                    .iter()
                    .map(|(u, v)| (*u, *v))
                    .collect::<Vec<(f64, f64)>>();

                let updated_vertices = original_vertices
                    .iter()
                    .zip(atlas_placed_uv_coords.iter())
                    .map(|((x, y, z, _, _), (u, v))| (*x, *y, *z, *u, *v))
                    .collect::<Vec<(f64, f64, f64, f64, f64)>>();

                poly.transform_inplace(|&[x, y, z, _, _]| {
                    let (u, v) = updated_vertices
                        .iter()
                        .find(|(x_, y_, z_, _, _)| {
                            (*x_ - x).abs() < 1e-6
                                && (*y_ - y).abs() < 1e-6
                                && (*z_ - z).abs() < 1e-6
                        })
                        .map(|(_, _, _, u, v)| (*u, *v))
                        .unwrap();
                    [x, y, z, u, v]
                });

                // Build atlas file path using callback
                let atlas_uri = atlas_path_builder(info.atlas_id).with_extension(ext);

                mat = material::Material {
                    base_color: mat.base_color,
                    base_texture: Some(material::Texture {
                        uri: Url::from_file_path(atlas_uri).map_err(|_| {
                            crate::errors::SinkError::GltfWriter(
                                "Failed to convert atlas URI to URL".to_string(),
                            )
                        })?,
                    }),
                };
            }

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

                buf3d.clear();
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

pub fn encode_metadata<'a>(
    features: &'a [GltfFeature],
    typename: &str,
    metadata_encoder: &mut reearth_flow_gltf::MetadataEncoder,
) -> Vec<&'a GltfFeature> {
    features
        .iter()
        .filter(|feature| {
            let result = metadata_encoder.add_feature(typename, &feature.attributes);
            if let Err(e) = result {
                tracing::error!("Failed to add feature with error = {e:?}");
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_texture(dir: &std::path::Path, name: &str, width: u32, height: u32) -> PathBuf {
        use image::{ImageBuffer, Rgb};
        let img = ImageBuffer::<Rgb<u8>, _>::new(width, height);
        let path = dir.join(name);
        img.save(&path).unwrap();
        path
    }

    fn create_test_feature(texture_path: &std::path::Path, uvs: Vec<(f64, f64)>) -> GltfFeature {
        let coords: Vec<[f64; 5]> = uvs
            .iter()
            .enumerate()
            .map(|(i, (u, v))| [i as f64, i as f64, 0.0, *u, *v])
            .collect();

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
    fn test_load_textures_wrapping_detection() {
        let temp_dir = TempDir::new().unwrap();
        let texture_path = create_test_texture(temp_dir.path(), "test.png", 64, 64);

        // Feature with wrapping UVs
        let feature = create_test_feature(&texture_path, vec![(0.0, 0.0), (1.5, 1.0), (0.0, 1.0)]);
        let features = vec![&feature];

        let packer = Mutex::new(AtlasPacker::default());
        let texture_size_cache = TextureSizeCache::new();
        let texture_id_gen = |fid, pid| format!("tex_{}_{}", fid, pid);

        let result = load_textures_into_packer(
            &features,
            &packer,
            &texture_size_cache,
            &texture_id_gen,
            0.0,
            false,
        );

        assert!(result.is_ok());
        let (_, _, wrapping_textures) = result.unwrap();
        assert_eq!(wrapping_textures.len(), 1);
        assert!(wrapping_textures.contains("tex_0_0"));
    }

    #[test]
    fn test_load_textures_max_size_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let texture1 = create_test_texture(temp_dir.path(), "test1.png", 64, 64);
        let texture2 = create_test_texture(temp_dir.path(), "test2.png", 100, 80);
        let texture3 = create_test_texture(temp_dir.path(), "test3.png", 50, 120);

        let feature1 = create_test_feature(&texture1, vec![(0.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let feature2 = create_test_feature(&texture2, vec![(0.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let feature3 = create_test_feature(&texture3, vec![(0.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let features = vec![&feature1, &feature2, &feature3];

        let packer = Mutex::new(AtlasPacker::default());
        let texture_size_cache = TextureSizeCache::new();

        let result = load_textures_into_packer(
            &features,
            &packer,
            &texture_size_cache,
            &|f, p| format!("{}_{}", f, p),
            0.0,
            false,
        );

        assert!(result.is_ok());
        let (width, height, _) = result.unwrap();

        // Max width is 100 -> next power of two = 128
        // Max height is 120 -> next power of two = 128
        assert_eq!(width, 128);
        assert_eq!(height, 128);
    }

    #[test]
    fn test_load_textures_downsampling() {
        let temp_dir = TempDir::new().unwrap();
        let texture_path = create_test_texture(temp_dir.path(), "test.png", 256, 256);

        let feature = create_test_feature(&texture_path, vec![(0.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let features = vec![&feature];

        let packer = Mutex::new(AtlasPacker::default());
        let texture_size_cache = TextureSizeCache::new();

        // Test with downsample_factor = 0.5
        let result = load_textures_into_packer(
            &features,
            &packer,
            &texture_size_cache,
            &|f, p| format!("{}_{}", f, p),
            9e9,
            true,
        );

        assert!(result.is_ok());
        let (width, height, _) = result.unwrap();

        // extremely large geom_error should lead to maximum downsampling
        assert_eq!(width, 1);
        assert_eq!(height, 1);
    }
}
