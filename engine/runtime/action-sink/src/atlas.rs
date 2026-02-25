// atlas builder shared between glTF and 3D Tiles generation

use std::collections::HashMap;
use std::sync::Mutex;

use ahash::RandomState;
use atlas_packer::export::AtlasExporter;
use atlas_packer::pack::AtlasPacker;
use atlas_packer::place::GuillotineTexturePlacer;
use atlas_packer::place::TexturePlacerConfig;
use atlas_packer::texture::cache::{TextureCache, TextureSizeCache};
use atlas_packer::texture::{DownsampleFactor, PolygonMappedTexture};
use earcut::{utils3d::project3d_to_2d, Earcut};
use flatgeom::MultiPolygon;
use indexmap::IndexSet;
use reearth_flow_gltf::{calculate_normal, Primitives};
use reearth_flow_types::{
    material::{self, Material},
    AttributeValue,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::zip_eq_logged::ZipEqLoggedExt;

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
) -> crate::errors::Result<(u32, u32)>
where
    F: Fn(usize, usize) -> String,
{
    let mut max_width = 0;
    let mut max_height = 0;

    for (feature_id, feature) in features.iter().enumerate() {
        for (poly_count, (mat, poly)) in feature
            .polygons
            .iter()
            .zip_eq_logged(feature.polygon_material_ids.iter())
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

                // Check if this texture has wrapping UVs
                if has_wrapping_uvs(&uv_coords) {
                    continue; // Skip atlas packing for wrapping textures
                }

                let texture_id = texture_id_generator(feature_id, poly_count);

                let texture_uri = base_texture.uri.to_file_path().map_err(|_| {
                    crate::errors::SinkError::atlas_builder(
                        "Failed to convert texture URI to file path",
                    )
                })?;
                let texture_size = texture_size_cache.get_or_insert(&texture_uri);

                // Skip atlas packing for large textures; the original file bytes
                // will be embedded directly (avoids WebP encoder dimension limits)
                if texture_size.0 + texture_size.1 >= 4096 {
                    continue;
                }

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

                let scaled_width = (texture_size.0 as f32 * factor) as u32;
                let scaled_height = (texture_size.1 as f32 * factor) as u32;

                max_width = max_width.max(scaled_width);
                max_height = max_height.max(scaled_height);

                packer
                    .lock()
                    .map_err(|_| {
                        crate::errors::SinkError::atlas_builder("Failed to lock the texture packer")
                    })?
                    .add_texture(texture_id, texture);
            }
        }
    }

    let max_width = max_width.next_power_of_two();
    let max_height = max_height.next_power_of_two();

    Ok((max_width, max_height))
}

pub fn process_geometry_with_atlas<F, P>(
    features: &[&GltfFeature],
    packed: &atlas_packer::pack::PackedAtlasProvider,
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
            .zip_eq_logged(feature.polygon_material_ids.iter())
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

            // Transform UVs if texture was packed into atlas
            if let Some(info) = packed.get_texture_info(&texture_id) {
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
                            crate::errors::SinkError::atlas_builder(
                                "Failed to convert atlas URI to URL",
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

/// Process geometry with atlas packing and export
/// Combines packing, geometry processing, and atlas export into one function
pub fn process_geometry_with_atlas_export<F, P, E>(
    features: &[&GltfFeature],
    packer: Mutex<AtlasPacker>,
    dimensions: (u32, u32),
    exporter: E,
    atlas_path: P,
    texture_cache: &TextureCache,
    texture_id_generator: F,
) -> Result<(Primitives, IndexSet<[u32; 9], RandomState>), crate::errors::SinkError>
where
    F: Fn(usize, usize) -> String,
    P: AsRef<std::path::Path>,
    E: AtlasExporter + Clone,
{
    let mut primitives: Primitives = Default::default();
    let mut vertices: IndexSet<[u32; 9], RandomState> = IndexSet::default();

    let (max_width, max_height) = dimensions;

    // Initialize texture packer config
    let config = TexturePlacerConfig::new_padded(max_width, max_height, 0, 2);

    let placer = GuillotineTexturePlacer::new(config.clone());
    let packer = packer
        .into_inner()
        .map_err(|_| crate::errors::SinkError::atlas_builder("Failed to unwrap texture packer"))?;

    // Pack textures into atlas
    let packed = packer.pack(placer);

    let ext = exporter.clone().get_extension().to_string();

    // Process geometry with atlas
    process_geometry_with_atlas(
        features,
        &packed,
        &ext,
        &texture_id_generator,
        |atlas_id| atlas_path.as_ref().join(atlas_id.to_string()),
        &mut primitives,
        &mut vertices,
    )?;

    // Export atlas textures
    packed.export(
        exporter,
        atlas_path.as_ref(),
        texture_cache,
        config.width(),
        config.height(),
    );

    Ok((primitives, vertices))
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

    fn create_test_feature(
        texture_path: &std::path::Path,
        uvs: Vec<(f64, f64)>,
        offset_x: f64,
    ) -> GltfFeature {
        // Create coplanar square polygon at z=0 with x offset
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
    fn test_load_textures_wrapping_detection() {
        let temp_dir = TempDir::new().unwrap();
        let texture_path = create_test_texture(temp_dir.path(), "test.png", 64, 64);

        // Feature with wrapping UVs
        let feature = create_test_feature(
            &texture_path,
            vec![(0.0, 0.0), (1.5, 0.0), (1.5, 1.0), (0.0, 1.0)],
            0.0,
        );
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
        // Wrapping textures are not added to the packer
        let packer = packer.into_inner().unwrap();
        let placer = GuillotineTexturePlacer::new(Default::default());
        // Pack textures into atlas
        let packed = packer.pack(placer);
        assert!(packed.get_texture_info(&texture_id_gen(0, 0)).is_none());
    }

    #[test]
    fn test_load_textures_max_size_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let texture1 = create_test_texture(temp_dir.path(), "test1.png", 64, 64);
        let texture2 = create_test_texture(temp_dir.path(), "test2.png", 100, 80);
        let texture3 = create_test_texture(temp_dir.path(), "test3.png", 50, 120);

        let feature1 = create_test_feature(
            &texture1,
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            0.0,
        );
        let feature2 = create_test_feature(
            &texture2,
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            0.0,
        );
        let feature3 = create_test_feature(
            &texture3,
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            0.0,
        );
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
        let (width, height) = result.unwrap();

        // Max width is 100 -> next power of two = 128
        // Max height is 120 -> next power of two = 128
        assert_eq!(width, 128);
        assert_eq!(height, 128);
    }

    #[test]
    fn test_load_textures_downsampling() {
        let temp_dir = TempDir::new().unwrap();
        let texture_path = create_test_texture(temp_dir.path(), "test.png", 256, 256);

        let feature = create_test_feature(
            &texture_path,
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            0.0,
        );
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
        let (width, height) = result.unwrap();

        // extremely large geom_error should lead to maximum downsampling
        assert_eq!(width, 1);
        assert_eq!(height, 1);
    }

    #[test]
    // test 16384x1 texture which crashes webP if included in atlas
    fn test_large_texture_skipped_from_atlas() {
        use atlas_packer::export::WebpAtlasExporter;
        use atlas_packer::texture::cache::TextureCache;

        let temp_dir = TempDir::new().unwrap();
        // 16384x1: width+height >= 2048, exceeds WebP's 16383px limit
        let texture_path = create_test_texture(temp_dir.path(), "large.jpg", 16384, 1);
        let feature = create_test_feature(
            &texture_path,
            vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            0.0,
        );
        let features = vec![&feature];
        let packer = Mutex::new(AtlasPacker::default());
        let texture_size_cache = TextureSizeCache::new();
        let texture_cache = TextureCache::new(200_000_000);
        let texture_id_gen = |fid: usize, pid: usize| format!("tex_{}_{}", fid, pid);

        load_textures_into_packer(
            &features,
            &packer,
            &texture_size_cache,
            &texture_id_gen,
            0.0,
            false,
        )
        .expect("load textures");

        let atlas_dir = temp_dir.path().join("atlas");
        std::fs::create_dir(&atlas_dir).unwrap();

        let (primitives, _) = process_geometry_with_atlas_export(
            &features,
            packer,
            (16384, 1),
            WebpAtlasExporter::default(),
            &atlas_dir,
            &texture_cache,
            texture_id_gen,
        )
        .expect("process geometry");

        // Without the fix: panics in WebpAtlasExporter, or mat points to .webp atlas
        // With the fix: texture skipped, mat retains original .jpg
        let (mat, _) = primitives.iter().next().unwrap();
        assert!(mat
            .base_texture
            .as_ref()
            .unwrap()
            .uri
            .to_string()
            .ends_with(".jpg"));
    }

    #[test]
    fn test_process_geometry_with_atlas_export_uv_mapping() {
        use atlas_packer::export::PngAtlasExporter;
        use image::{ImageBuffer, Rgb, RgbImage};

        let temp_path = tempfile::tempdir().expect("create temp dir").keep();

        // Create two 256x256 textures with distinct colors based on position
        let img1: RgbImage = ImageBuffer::from_fn(256, 256, |x, y| Rgb([x as u8, y as u8, 10]));
        let path1 = temp_path.join("texture1.png");
        img1.save(&path1).expect("save texture1");

        let img2: RgbImage = ImageBuffer::from_fn(256, 256, |x, y| Rgb([x as u8, y as u8, 20]));
        let path2 = temp_path.join("texture2.png");
        img2.save(&path2).expect("save texture2");

        // Create test features with non-overlapping polygons
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

        // Setup packer and load textures
        let packer = Mutex::new(AtlasPacker::default());
        let texture_size_cache = TextureSizeCache::new();
        let texture_id_gen = |fid: usize, pid: usize| format!("tex_{}_{}", fid, pid);

        load_textures_into_packer(
            &features,
            &packer,
            &texture_size_cache,
            &texture_id_gen,
            0.0,
            false,
        )
        .expect("load textures");

        // Create atlas output directory
        let atlas_dir = temp_path.join("atlas");
        std::fs::create_dir(&atlas_dir).expect("create atlas dir");

        // Setup texture cache
        let texture_cache = TextureCache::new(200_000_000);

        // Process geometry and export atlas with large dimensions to pack both textures together
        let (primitives, vertices) = process_geometry_with_atlas_export(
            &features,
            packer,
            (1000, 1000),
            PngAtlasExporter {
                ext: "png".to_string(),
            },
            &atlas_dir,
            &texture_cache,
            texture_id_gen,
        )
        .expect("process geometry");

        assert!(!primitives.is_empty(), "primitives should not be empty");
        assert!(!vertices.is_empty(), "vertices should not be empty");

        // Verify primitives structure
        assert_eq!(
            primitives.len(),
            1,
            "should have 1 material group for 1 atlas texture"
        );
        let (material, primitive) = primitives.iter().next().unwrap();
        assert!(
            material.base_texture.is_some(),
            "material should have a base texture"
        );
        let texture_uri = material.base_texture.as_ref().unwrap().uri.to_string();
        assert!(
            texture_uri.contains("atlas"),
            "texture should be from atlas directory"
        );
        assert!(
            texture_uri.ends_with(".png"),
            "texture should be PNG format"
        );
        assert!(
            !primitive.indices.is_empty(),
            "primitive should have indices"
        );
        assert!(
            !primitive.feature_ids.is_empty(),
            "primitive should have feature IDs"
        );
        assert_eq!(
            primitive.indices.len(),
            12,
            "should have 12 indices for 2 quads"
        );

        // Read back the exported atlas
        let atlas_path = atlas_dir.join("0.png");
        assert!(atlas_path.exists(), "atlas file should exist");

        let atlas_img = image::open(&atlas_path).expect("open atlas");
        let atlas_rgb = atlas_img.to_rgb8();

        // Build HashMap: world position (x,y,z) -> expected color from original texture
        let mut expected_colors: HashMap<(i32, i32, i32), [u8; 3]> = HashMap::new();

        for (feature, orig_img) in [(&feature1, &img1), (&feature2, &img2)] {
            let poly = feature.polygons.iter().next().expect("get polygon");
            for coord in poly.raw_coords() {
                let [x, y, z, u, v] = coord;
                let tex_x = (u * 255.0) as u32;
                let tex_y = ((1.0 - v) * 255.0) as u32; // V is flipped in vertex buffer (line 242)
                let pixel = orig_img.get_pixel(tex_x, tex_y);

                let key = (
                    (*x * 1000.0) as i32,
                    (*y * 1000.0) as i32,
                    (*z * 1000.0) as i32,
                );
                expected_colors.insert(key, pixel.0);
            }
        }

        // Verify: for each vertex, sample atlas at packed UV and compare to expected color
        for vertex_bits in vertices.iter() {
            let v_x = f32::from_bits(vertex_bits[0]);
            let v_y = f32::from_bits(vertex_bits[1]);
            let v_z = f32::from_bits(vertex_bits[2]);
            let u_packed = f32::from_bits(vertex_bits[6]) as f64;
            let v_packed = f32::from_bits(vertex_bits[7]) as f64;

            let key = (
                (v_x * 1000.0) as i32,
                (v_y * 1000.0) as i32,
                (v_z * 1000.0) as i32,
            );
            let expected = expected_colors
                .get(&key)
                .expect("vertex should have expected color");

            // Sample atlas at packed UV
            let atlas_x = (u_packed * atlas_rgb.width() as f64) as u32;
            let atlas_y = (v_packed * atlas_rgb.height() as f64) as u32;
            let atlas_pixel = atlas_rgb.get_pixel(atlas_x, atlas_y);

            assert_eq!(
                atlas_pixel.0, *expected,
                "Color mismatch at pos=({:.1},{:.1},{:.1}) uv=({:.4},{:.4})",
                v_x, v_y, v_z, u_packed, v_packed
            );
        }
    }
}
