use std::collections::HashMap;
use std::path::Path;

use crate::zip_eq_logged::ZipEqLoggedExt;
use ahash::RandomState;
use earcut::{utils3d::project3d_to_2d, Earcut};
use flatgeom::MultiPolygon;
use image::ImageFormat;
use indexmap::IndexSet;
use reearth_flow_atlas::{build_atlas, TextureInput};
use reearth_flow_gltf::{calculate_normal, Primitives};
use reearth_flow_types::{
    material::{self, Material},
    AttributeValue,
};
use serde::{Deserialize, Serialize};
use url::Url;

const DEFAULT_MAX_ATLAS_SIZE: u32 = 8192;

type PolygonUVs = reearth_flow_atlas::PolygonUVs;
type PolyAtlasIndex = Vec<Vec<Option<(usize, usize)>>>;
type PendingPolyAtlasIndex = Vec<Vec<Option<(String, usize)>>>;

struct PendingTextureInput {
    path: std::path::PathBuf,
    uvs: Vec<PolygonUVs>,
    wrapping: bool,
}

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

pub fn build_atlas_geometry(
    features: &[&GltfFeature],
    atlas_dir: &Path,
    image_format: ImageFormat,
    ext: &str,
    primitives: &mut Primitives,
    vertices: &mut IndexSet<[u32; 9], RandomState>,
) -> crate::errors::Result<()> {
    let (texture_materials, poly_index) = collect_atlas_inputs(features);

    if texture_materials.is_empty() {
        emit_atlas_geometry(features, &poly_index, None, None, primitives, vertices);
        return Ok(());
    }

    let (atlas, atlas_uri) =
        build_atlas_artifacts(&texture_materials, atlas_dir, image_format, ext)?;

    emit_atlas_geometry(
        features,
        &poly_index,
        Some(&atlas),
        Some(&atlas_uri),
        primitives,
        vertices,
    );

    Ok(())
}

fn collect_atlas_inputs(features: &[&GltfFeature]) -> (Vec<TextureInput>, PolyAtlasIndex) {
    let mut pending_materials: HashMap<String, PendingTextureInput> = HashMap::new();
    let mut material_order: Vec<String> = Vec::new();
    let mut poly_index: PendingPolyAtlasIndex = Vec::new();

    for feature in features.iter() {
        poly_index.push(collect_feature_poly_index(
            feature,
            &mut pending_materials,
            &mut material_order,
        ));
    }

    let (texture_materials, path_to_mat_idx) =
        finalize_texture_materials(material_order, pending_materials);
    let poly_index = finalize_poly_index(poly_index, &path_to_mat_idx);

    (texture_materials, poly_index)
}

fn collect_feature_poly_index(
    feature: &GltfFeature,
    pending_materials: &mut HashMap<String, PendingTextureInput>,
    material_order: &mut Vec<String>,
) -> Vec<Option<(String, usize)>> {
    let mut polygon_mappings = Vec::new();

    for (poly, mat_id) in feature
        .polygons
        .iter()
        .zip_eq_logged(feature.polygon_material_ids.iter())
    {
        let mat = &feature.materials[*mat_id as usize];
        let entry = (|| {
            let texture = mat.base_texture.as_ref()?;
            let path = texture
                .uri
                .to_file_path()
                .map_err(
                    |_| tracing::error!(uri = %texture.uri, "base_texture URI is not a file path"),
                )
                .ok()?;
            let path_str = path.to_string_lossy().into_owned();
            let pending = pending_materials
                .entry(path_str.clone())
                .or_insert_with(|| {
                    material_order.push(path_str.clone());
                    PendingTextureInput {
                        path,
                        uvs: Vec::new(),
                        wrapping: false,
                    }
                });
            if pending.wrapping {
                return None;
            }
            let raw_uvs: PolygonUVs = poly
                .raw_coords()
                .iter()
                .map(|&[_, _, _, u, v]| [u, v])
                .collect();
            let Some(uvs) = normalize_uvs(raw_uvs) else {
                pending.wrapping = true;
                pending.uvs.clear();
                return None;
            };
            let poly_within = pending.uvs.len();
            pending.uvs.push(uvs);
            Some((path_str, poly_within))
        })();
        polygon_mappings.push(entry);
    }

    polygon_mappings
}

fn finalize_texture_materials(
    material_order: Vec<String>,
    mut pending_materials: HashMap<String, PendingTextureInput>,
) -> (Vec<TextureInput>, HashMap<String, usize>) {
    let mut texture_materials = Vec::new();
    let mut path_to_mat_idx = HashMap::new();

    for path_str in material_order {
        let Some(pending) = pending_materials.remove(&path_str) else {
            continue;
        };
        if pending.wrapping || pending.uvs.is_empty() {
            continue;
        }
        let mat_idx = texture_materials.len();
        texture_materials.push(TextureInput {
            path: pending.path,
            uvs: pending.uvs,
        });
        path_to_mat_idx.insert(path_str, mat_idx);
    }

    (texture_materials, path_to_mat_idx)
}

fn finalize_poly_index(
    poly_index: PendingPolyAtlasIndex,
    path_to_mat_idx: &HashMap<String, usize>,
) -> PolyAtlasIndex {
    poly_index
        .into_iter()
        .map(|feature_row| {
            feature_row
                .into_iter()
                .map(|entry| {
                    let (path_str, poly_within) = entry?;
                    let mat_idx = *path_to_mat_idx.get(&path_str)?;
                    Some((mat_idx, poly_within))
                })
                .collect()
        })
        .collect()
}

fn build_atlas_artifacts(
    texture_materials: &[TextureInput],
    atlas_dir: &Path,
    image_format: ImageFormat,
    ext: &str,
) -> crate::errors::Result<(reearth_flow_atlas::BuiltAtlas, Url)> {
    let atlas = build_atlas(texture_materials, DEFAULT_MAX_ATLAS_SIZE)
        .map_err(crate::errors::SinkError::atlas_builder)?
        .ok_or_else(|| crate::errors::SinkError::atlas_builder("atlas produced no image"))?;

    let atlas_path = atlas_dir.join("0").with_extension(ext);
    atlas
        .image
        .save_with_format(&atlas_path, image_format)
        .map_err(crate::errors::SinkError::atlas_builder)?;

    let atlas_uri = Url::from_file_path(&atlas_path)
        .map_err(|_| crate::errors::SinkError::atlas_builder("failed to create atlas file URI"))?;

    Ok((atlas, atlas_uri))
}

fn emit_atlas_geometry(
    features: &[&GltfFeature],
    poly_index: &PolyAtlasIndex,
    atlas: Option<&reearth_flow_atlas::BuiltAtlas>,
    atlas_uri: Option<&Url>,
    primitives: &mut Primitives,
    vertices: &mut IndexSet<[u32; 9], RandomState>,
) {
    for (feature_id, feature) in features.iter().enumerate() {
        for (poly_idx, (mut mat, poly)) in feature
            .polygons
            .iter()
            .zip_eq_logged(feature.polygon_material_ids.iter())
            .map(|(poly, mat_id)| (feature.materials[*mat_id as usize].clone(), poly))
            .enumerate()
        {
            let remapped_uvs = poly_index[feature_id][poly_idx]
                .and_then(|(mi, pi)| atlas.as_ref()?.remapped_uvs.get(mi).map(|uvs| &uvs[pi]));
            if remapped_uvs.is_some() {
                if let Some(uri) = atlas_uri {
                    mat = material::Material {
                        base_color: mat.base_color,
                        base_texture: Some(material::Texture { uri: uri.clone() }),
                    };
                }
            }

            emit_polygon(feature_id, &poly, remapped_uvs, mat, primitives, vertices);
        }
    }
}

fn emit_polygon(
    feature_id: usize,
    poly: &flatgeom::Polygon<'_, [f64; 5]>,
    remapped_uvs: Option<&PolygonUVs>,
    mat: Material,
    primitives: &mut Primitives,
    vertices: &mut IndexSet<[u32; 9], RandomState>,
) {
    let primitive = primitives.entry(mat).or_default();
    primitive.feature_ids.insert(feature_id as u32);

    let Some((nx, ny, nz)) = calculate_normal(poly.exterior().iter().map(|v| [v[0], v[1], v[2]]))
    else {
        return;
    };

    let num_outer_points = poly
        .hole_indices()
        .first()
        .map_or(poly.raw_coords().len(), |&v| v as usize);
    let mut earcutter = Earcut::new();
    let mut buf3d: Vec<[f64; 3]> = Vec::new();
    let mut buf2d: Vec<[f64; 2]> = Vec::new();
    let mut index_buf: Vec<u32> = Vec::new();

    buf3d.extend(poly.raw_coords().iter().map(|c| [c[0], c[1], c[2]]));

    if project3d_to_2d(&buf3d, num_outer_points, &mut buf2d) {
        earcutter.earcut(buf2d.iter().cloned(), poly.hole_indices(), &mut index_buf);

        primitive.indices.extend(index_buf.iter().map(|&idx| {
            let [x, y, z, orig_u, orig_v] = poly.raw_coords()[idx as usize];
            let [u, v] = remapped_uvs.map_or([orig_u, orig_v], |r| r[idx as usize]);
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

// Workaround for missing `wrapMode` in CityGML
// infer wrapping from UV range with 0.1 tolerance to handle dataset UV errors and clamp to [0,1]
fn normalize_uvs(uvs: PolygonUVs) -> Option<PolygonUVs> {
    const WRAP_THRESHOLD: f64 = 0.1;
    if uvs.iter().any(|[u, v]| {
        *u < -WRAP_THRESHOLD
            || *u > 1.0 + WRAP_THRESHOLD
            || *v < -WRAP_THRESHOLD
            || *v > 1.0 + WRAP_THRESHOLD
    }) {
        return None;
    }
    Some(
        uvs.into_iter()
            .map(|[u, v]| [u.clamp(0.0, 1.0), v.clamp(0.0, 1.0)])
            .collect(),
    )
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
        use reearth_flow_types::material;

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
    fn test_build_atlas_geometry_uv_mapping() {
        use image::{ImageBuffer, Rgb, RgbImage};

        let temp_dir = TempDir::new().unwrap();

        let img1: RgbImage = ImageBuffer::from_fn(256, 256, |x, y| Rgb([x as u8, y as u8, 10u8]));
        img1.save(temp_dir.path().join("texture1.png")).unwrap();

        let img2: RgbImage = ImageBuffer::from_fn(256, 256, |x, y| Rgb([x as u8, y as u8, 20u8]));
        img2.save(temp_dir.path().join("texture2.png")).unwrap();

        let path1 = temp_dir.path().join("texture1.png");
        let path2 = temp_dir.path().join("texture2.png");

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

        let atlas_dir = temp_dir.path().join("atlas");
        std::fs::create_dir(&atlas_dir).unwrap();

        let mut primitives: Primitives = Default::default();
        let mut vertices: IndexSet<[u32; 9], RandomState> = IndexSet::default();

        build_atlas_geometry(
            &[&feature1, &feature2],
            &atlas_dir,
            ImageFormat::Png,
            "png",
            &mut primitives,
            &mut vertices,
        )
        .unwrap();

        assert!(!primitives.is_empty());
        assert!(!vertices.is_empty());
        assert_eq!(
            primitives.len(),
            1,
            "both features share one atlas material"
        );
        assert!(atlas_dir.join("0.png").exists());
    }

    #[test]
    fn test_build_atlas_geometry_retries_with_downscaling() {
        let temp_dir = TempDir::new().unwrap();
        let path1 = create_test_texture(temp_dir.path(), "large1.png", 4096, 4096);
        let path2 = create_test_texture(temp_dir.path(), "large2.png", 4096, 4096);

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

        let atlas_dir = temp_dir.path().join("atlas");
        std::fs::create_dir(&atlas_dir).unwrap();

        let mut primitives: Primitives = Default::default();
        let mut vertices: IndexSet<[u32; 9], RandomState> = IndexSet::default();

        build_atlas_geometry(
            &[&feature1, &feature2],
            &atlas_dir,
            ImageFormat::Png,
            "png",
            &mut primitives,
            &mut vertices,
        )
        .unwrap();

        let atlas = image::open(atlas_dir.join("0.png")).unwrap();
        assert!(atlas.width() <= DEFAULT_MAX_ATLAS_SIZE);
        assert!(atlas.height() <= DEFAULT_MAX_ATLAS_SIZE);
        assert_eq!(
            primitives.len(),
            1,
            "retry path should still share one atlas"
        );
    }

    #[test]
    fn test_wrapping_texture_uses_original_material() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_texture(temp_dir.path(), "t.png", 64, 64);

        let feature = create_test_feature(
            &path,
            vec![(0.0, 0.0), (1.5, 0.0), (1.5, 1.0), (0.0, 1.0)],
            0.0,
        );

        let atlas_dir = temp_dir.path().join("atlas");
        std::fs::create_dir(&atlas_dir).unwrap();

        let mut primitives: Primitives = Default::default();
        let mut vertices: IndexSet<[u32; 9], RandomState> = IndexSet::default();

        build_atlas_geometry(
            &[&feature],
            &atlas_dir,
            ImageFormat::Png,
            "png",
            &mut primitives,
            &mut vertices,
        )
        .unwrap();

        // Wrapping texture: primitive still created but material keeps original URI
        assert!(!primitives.is_empty());
        let mat = primitives.keys().next().unwrap();
        assert_eq!(
            mat.base_texture.as_ref().unwrap().uri,
            Url::from_file_path(&path).unwrap(),
            "wrapping texture must retain original URI"
        );
    }
}
