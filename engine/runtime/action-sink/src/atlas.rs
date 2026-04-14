use std::collections::HashMap;
use std::path::Path;

use crate::zip_eq_logged::ZipEqLoggedExt;
use ahash::RandomState;
use earcut::{utils3d::project3d_to_2d, Earcut};
use flatgeom::MultiPolygon;
use image::ImageFormat;
use indexmap::IndexSet;
use reearth_flow_atlas::{build_atlas, TextureMaterial};
use reearth_flow_gltf::{calculate_normal, Primitives};
use reearth_flow_types::{
    material::{self, Material},
    AttributeValue,
};
use serde::{Deserialize, Serialize};
use url::Url;

const DEFAULT_MAX_ATLAS_SIZE: u32 = 8192;

type PolygonUVs = reearth_flow_atlas::PolygonUVs;

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
    // Conversion pass: group polygon UVs by texture path, track mapping back.
    let mut texture_materials: Vec<TextureMaterial> = Vec::new();
    let mut path_to_mat_idx: HashMap<String, usize> = HashMap::new();
    // poly_index[feature_id][poly_idx] = Some((mat_idx, poly_within_mat))
    let mut poly_index: Vec<Vec<Option<(usize, usize)>>> = Vec::new();

    for feature in features.iter() {
        let mut feature_row = Vec::new();
        for (poly, mat_id) in feature
            .polygons
            .iter()
            .zip_eq_logged(feature.polygon_material_ids.iter())
        {
            let mat = &feature.materials[*mat_id as usize];
            let entry = (|| {
                let path = mat.base_texture.as_ref()?.uri.to_file_path().ok()?;
                let path_str = path.to_string_lossy().into_owned();
                let mat_idx = if let Some(&idx) = path_to_mat_idx.get(&path_str) {
                    idx
                } else {
                    let idx = texture_materials.len();
                    texture_materials.push(TextureMaterial {
                        path,
                        uvs: Vec::new(),
                    });
                    path_to_mat_idx.insert(path_str, idx);
                    idx
                };
                let poly_within = texture_materials[mat_idx].uvs.len();
                let uvs: PolygonUVs = poly
                    .raw_coords()
                    .iter()
                    .map(|&[_, _, _, u, v]| [u, v])
                    .collect();
                texture_materials[mat_idx].uvs.push(uvs);
                Some((mat_idx, poly_within))
            })();
            feature_row.push(entry);
        }
        poly_index.push(feature_row);
    }

    let atlas = if texture_materials.is_empty() {
        None
    } else {
        Some(
            build_atlas(&texture_materials, DEFAULT_MAX_ATLAS_SIZE)
                .map_err(crate::errors::SinkError::atlas_builder)?,
        )
    };
    if let Some(image) = atlas.as_ref().and_then(|atlas| atlas.image.as_ref()) {
        image
            .save_with_format(atlas_dir.join("0").with_extension(ext), image_format)
            .map_err(crate::errors::SinkError::atlas_builder)?;
    }
    let atlas_uri = atlas
        .as_ref()
        .and_then(|atlas| atlas.image.as_ref().map(|_| ()))
        .and_then(|_| Url::from_file_path(atlas_dir.join("0").with_extension(ext)).ok());

    // Triangulation pass: use remapped UVs where available.
    for (feature_id, feature) in features.iter().enumerate() {
        for (poly_idx, (mut mat, poly)) in feature
            .polygons
            .iter()
            .zip_eq_logged(feature.polygon_material_ids.iter())
            .map(|(poly, mat_id)| (feature.materials[*mat_id as usize].clone(), poly))
            .enumerate()
        {
            let remapped_uvs: Option<&PolygonUVs> =
                poly_index[feature_id][poly_idx].and_then(|(mi, pi)| {
                    atlas
                        .as_ref()?
                        .remapped_uvs
                        .get(mi)?
                        .as_ref()
                        .map(|uvs| &uvs[pi])
                });

            if remapped_uvs.is_some() {
                if let Some(ref uri) = atlas_uri {
                    mat = material::Material {
                        base_color: mat.base_color,
                        base_texture: Some(material::Texture { uri: uri.clone() }),
                    };
                }
            }

            let primitive = primitives.entry(mat).or_default();
            primitive.feature_ids.insert(feature_id as u32);

            let Some((nx, ny, nz)) =
                calculate_normal(poly.exterior().iter().map(|v| [v[0], v[1], v[2]]))
            else {
                continue;
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
