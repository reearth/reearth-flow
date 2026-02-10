use crate::compare_attributes::{analyze_attributes, make_feature_key, structural_casts, CastConfig};
use reearth_flow_geometry::types::coordinate::Coordinate;
use reearth_flow_gltf::{
    extract_feature_properties, material_from_gltf, parse_gltf, read_indices, read_mesh_features,
    read_positions_with_transform, read_vertex_colors, traverse_scene, Transform,
};
use reearth_flow_types::material::Material;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct TilesetInfo {
    #[allow(dead_code)]
    pub path: PathBuf,
    pub content: Value,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DetailLevel {
    pub(crate) geometric_error: f64,
    pub(crate) source_idx: Option<u32>,
    pub(crate) texture_name: Option<String>,
    pub(crate) triangles: Vec<[usize; 3]>,
}

/// Per-feature data collected across primitives within a single GLB file
struct FeatureData {
    triangles: Vec<[usize; 3]>,
    has_texture: bool,
    texture_name: Option<String>,
}

/// Find top-level 3D Tiles directories (directories containing tileset.json)
pub fn find_cesium_tile_directories(base_path: &Path) -> Result<Vec<String>, String> {
    let mut dirs = HashSet::new();

    for entry in WalkDir::new(base_path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.file_name() == "tileset.json")
    {
        if let Ok(rel) = entry.path().parent().unwrap().strip_prefix(base_path) {
            if let Some(first_component) = rel.iter().next() {
                dirs.insert(first_component.to_string_lossy().to_string());
            }
        }
    }

    let mut result: Vec<_> = dirs.into_iter().collect();
    result.sort();
    Ok(result)
}

/// Load tileset.json file
pub fn load_tileset(dir: &Path) -> Result<TilesetInfo, String> {
    let tileset_path = dir.join("tileset.json");

    if !tileset_path.exists() {
        return Err(format!("tileset.json not found in {:?}", dir));
    }

    let content = fs::read_to_string(&tileset_path)
        .map_err(|e| format!("Failed to read tileset.json from {:?}: {}", tileset_path, e))?;

    let json: Value = serde_json::from_str(&content).map_err(|e| {
        format!(
            "Failed to parse tileset.json from {:?}: {}",
            tileset_path, e
        )
    })?;

    Ok(TilesetInfo {
        path: tileset_path,
        content: json,
    })
}

pub(crate) struct GeometryCollector {
    tileset_dir: PathBuf,
    casts: HashMap<String, CastConfig>,
    pub(crate) vertex_positions: Vec<Coordinate>,
    pub(crate) vertex_colors: Option<Vec<[f32; 4]>>,
    pub(crate) vertex_materials: Option<Vec<u32>>,
    pub(crate) materials: Vec<Material>,
    pub(crate) detail_levels: HashMap<String, Vec<DetailLevel>>,
    /// Feature attributes keyed by feature identifier (from make_feature_key)
    pub(crate) feature_attributes: HashMap<String, Value>,
}

impl GeometryCollector {
    fn new(tileset_dir: PathBuf, casts: &HashMap<String, CastConfig>) -> Self {
        Self {
            tileset_dir,
            casts: structural_casts(casts),
            vertex_positions: Vec::new(),
            vertex_colors: None,
            vertex_materials: None,
            materials: Vec::new(),
            detail_levels: HashMap::new(),
            feature_attributes: HashMap::new(),
        }
    }

    /// Helper to append vertex attribute data, handling backfill when attributes appear mid-collection
    fn append_vertex_attribute<T: Clone>(
        existing: &mut Option<Vec<T>>,
        new_data: Option<Vec<T>>,
        vertex_offset: usize,
        new_vertex_count: usize,
        default_value: T,
    ) {
        match (existing.as_mut(), new_data) {
            (Some(existing_vec), Some(new_vec)) => {
                // Both exist - just append
                existing_vec.extend(new_vec);
            }
            (None, Some(new_vec)) => {
                // First primitive with this attribute - backfill previous vertices with default
                let mut vec = vec![default_value; vertex_offset];
                vec.extend(new_vec);
                *existing = Some(vec);
            }
            (Some(existing_vec), None) => {
                // Current primitive lacks this attribute - extend with default
                existing_vec.extend(vec![default_value; new_vertex_count]);
            }
            (None, None) => {
                // No attribute in either - no action needed
            }
        }
    }

    fn process_tile(&mut self, tile: &Value) -> Result<(), String> {
        let geometric_error = tile
            .get("geometricError")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| "Missing or invalid geometricError in tile".to_string())?;

        let glb_paths = self.extract_glb_paths(tile)?;

        for glb_path in glb_paths {
            self.process_glb(&glb_path, geometric_error)?;
        }

        if let Some(children) = tile.get("children").and_then(|c| c.as_array()) {
            for child in children {
                self.process_tile(child)?;
            }
        }

        Ok(())
    }

    fn extract_glb_paths(&self, tile: &Value) -> Result<Vec<PathBuf>, String> {
        let mut glb_paths = Vec::new();

        if let Some(content) = tile.get("content") {
            if let Some(uri) = content.get("uri").and_then(|u| u.as_str()) {
                if uri.ends_with(".glb") {
                    let glb_path = self.tileset_dir.join(uri);
                    if !glb_path.exists() {
                        return Err(format!(
                            "GLB file referenced in tileset does not exist: {:?}",
                            glb_path
                        ));
                    }
                    glb_paths.push(glb_path);
                }
            }
        }

        if let Some(contents) = tile.get("contents").and_then(|c| c.as_array()) {
            for content_item in contents {
                if let Some(uri) = content_item.get("uri").and_then(|u| u.as_str()) {
                    if uri.ends_with(".glb") {
                        let glb_path = self.tileset_dir.join(uri);
                        if !glb_path.exists() {
                            return Err(format!(
                                "GLB file referenced in tileset does not exist: {:?}",
                                glb_path
                            ));
                        }
                        glb_paths.push(glb_path);
                    }
                }
            }
        }

        Ok(glb_paths)
    }

    fn process_glb(&mut self, glb_path: &Path, geometric_error: f64) -> Result<(), String> {
        let content = fs::read(glb_path)
            .map_err(|e| format!("Failed to read GLB file {:?}: {}", glb_path, e))?;
        let gltf = parse_gltf(&bytes::Bytes::from(content))
            .map_err(|e| format!("Failed to parse GLB {:?}: {}", glb_path, e))?;

        let features = extract_feature_properties(&gltf)
            .map_err(|e| format!("Failed to extract features from {:?}: {}", glb_path, e))?;

        // Extract directory name for risk type detection
        let dir_name = self.tileset_dir.file_name().and_then(|n| n.to_str());

        // Create feature list with keys generated by make_feature_key
        let mut feature_list: Vec<(String, serde_json::Map<String, Value>)> = Vec::new();
        for props in features {
            let props_value = Value::Object(props.clone());
            let key = make_feature_key(&props_value, dir_name);

            // Check for conflicting duplicate keys within the same dataset.
            // Only structural casts (Json, ListToDict) for data-equivalence; no primitive casts.
            if let Some(existing) = self.feature_attributes.get(&key) {
                analyze_attributes(
                    &key,
                    existing,
                    &props_value,
                    self.casts.clone(),
                    Default::default(),
                )
                .map_err(|e| format!("Conflicting feature_key in {:?}: {}", glb_path, e))?;
            } else {
                self.feature_attributes.insert(key.clone(), props_value);
            }

            feature_list.push((key, props));
        }

        let buffer_data = vec![gltf
            .blob
            .as_ref()
            .ok_or_else(|| format!("GLB file {:?} has no binary blob", glb_path))?
            .clone()];

        // Collect per-feature data across all primitives in this GLB file
        let mut file_feature_data: HashMap<u32, FeatureData> = HashMap::new();

        // Process scene graph to capture node transforms
        for scene in gltf.scenes() {
            traverse_scene(&scene, |node, world_transform| -> Result<(), String> {
                // Process mesh if attached to this node
                if let Some(mesh) = node.mesh() {
                    for primitive in mesh.primitives() {
                        self.process_primitive_collect(
                            &primitive,
                            &buffer_data,
                            glb_path,
                            world_transform,
                            &mut file_feature_data,
                        )?;
                    }
                }
                Ok(())
            })?;
        }

        // Now create one DetailLevel per feature for this GLB file
        for (feature_id, data) in file_feature_data {
            let ident = Self::lookup_ident(feature_id, &feature_list, glb_path)?;
            let (source_idx, texture_name) = if data.has_texture {
                (Some(0), data.texture_name) // source_idx value doesn't matter, just needs to be Some
            } else {
                (None, None)
            };

            let detail_level = DetailLevel {
                geometric_error,
                source_idx,
                texture_name,
                triangles: data.triangles,
            };
            self.detail_levels
                .entry(ident)
                .or_default()
                .push(detail_level);
        }

        Ok(())
    }

    /// Process a primitive and collect feature data without creating DetailLevels yet
    fn process_primitive_collect(
        &mut self,
        primitive: &::gltf::Primitive,
        buffer_data: &[Vec<u8>],
        glb_path: &Path,
        transform: &Transform,
        file_feature_data: &mut HashMap<u32, FeatureData>,
    ) -> Result<(), String> {
        let feature_ids = read_mesh_features(primitive, buffer_data)
            .map_err(|e| format!("Failed to read mesh features from {:?}: {}", glb_path, e))?
            .ok_or_else(|| format!("Primitive has no feature IDs: {:?}", glb_path))?;

        let position_accessor = primitive
            .get(&::gltf::Semantic::Positions)
            .ok_or_else(|| format!("Primitive has no positions: {:?}", glb_path))?;
        let positions =
            read_positions_with_transform(&position_accessor, buffer_data, Some(transform))
                .map_err(|e| format!("Failed to read positions: {}", e))?;

        let indices = primitive
            .indices()
            .ok_or_else(|| format!("Primitive has no indices: {:?}", glb_path))?;
        let indices = read_indices(&indices, buffer_data)
            .map_err(|e| format!("Failed to read indices: {}", e))?;

        let vertex_colors = primitive
            .get(&::gltf::Semantic::Colors(0))
            .map(|accessor| read_vertex_colors(&accessor, buffer_data))
            .transpose()
            .map_err(|e| format!("Failed to read vertex colors: {}", e))?;

        // Calculate vertex offset for appending to existing vertex arrays
        let vertex_offset = self.vertex_positions.len();
        let new_vertex_count = positions.len();

        // Calculate material offset - the index where this primitive's material will be stored
        let material_offset = self.materials.len() as u32;

        // Append new vertex data instead of replacing
        self.vertex_positions.extend(positions);

        // Use helper to append optional vertex attributes with proper backfilling
        Self::append_vertex_attribute(
            &mut self.vertex_colors,
            vertex_colors,
            vertex_offset,
            new_vertex_count,
            [1.0, 1.0, 1.0, 1.0], // Default white color
        );

        // In glTF, materials are per-primitive, not per-vertex.
        // All vertices in this primitive use the same material at material_offset.
        let vertex_materials_for_primitive = vec![material_offset; new_vertex_count];
        Self::append_vertex_attribute(
            &mut self.vertex_materials,
            Some(vertex_materials_for_primitive),
            vertex_offset,
            new_vertex_count,
            material_offset, // Default to this primitive's material
        );

        // Extract and store material information
        let gltf_material = primitive.material();
        let flow_material = material_from_gltf(&gltf_material)
            .map_err(|e| format!("Failed to extract material from {:?}: {}", glb_path, e))?;
        self.materials.push(flow_material);

        let texture_info = gltf_material
            .pbr_metallic_roughness()
            .base_color_texture()
            .map(|tex_info| {
                let texture = tex_info.texture();
                let texture_name = texture
                    .source()
                    .name()
                    .map(|s| s.to_string())
                    .or_else(|| Some(format!("texture_{}", texture.source().index())));
                texture_name
            });

        let has_texture = texture_info.is_some();
        let texture_name = texture_info.flatten();

        // Group triangles by feature ID, offsetting indices by vertex_offset
        if !indices.len().is_multiple_of(3) {
            return Err(format!(
                "Invalid index count {} (not divisible by 3) in {:?}",
                indices.len(),
                glb_path
            ));
        }

        for chunk in indices.chunks(3) {
            let (idx0, idx1, idx2) = (chunk[0], chunk[1], chunk[2]);
            let (fid0, fid1, fid2) = (feature_ids[idx0], feature_ids[idx1], feature_ids[idx2]);

            assert!(
                fid0 == fid1 && fid1 == fid2,
                "Triangle vertices have inconsistent feature IDs: {} {} {} in {:?}",
                fid0,
                fid1,
                fid2,
                glb_path
            );

            // Offset indices to account for previously appended vertices
            let triangle = [
                idx0 + vertex_offset,
                idx1 + vertex_offset,
                idx2 + vertex_offset,
            ];

            let entry = file_feature_data
                .entry(fid0)
                .or_insert_with(|| FeatureData {
                    triangles: Vec::new(),
                    has_texture: false,
                    texture_name: None,
                });
            entry.triangles.push(triangle);
            // If any primitive has texture, the feature has texture
            if has_texture {
                entry.has_texture = true;
                if entry.texture_name.is_none() {
                    entry.texture_name = texture_name.clone();
                }
            }
        }

        Ok(())
    }

    fn lookup_ident(
        feature_id: u32,
        feature_list: &[(String, serde_json::Map<String, Value>)],
        glb_path: &Path,
    ) -> Result<String, String> {
        feature_list
            .get(feature_id as usize)
            .map(|(ident, _)| ident.clone())
            .ok_or_else(|| {
                format!(
                    "Feature ID {} not found in property table for {:?}",
                    feature_id, glb_path
                )
            })
    }
}

pub fn collect_geometries_by_ident(
    tileset_dir: &Path,
    casts: &HashMap<String, CastConfig>,
) -> Result<GeometryCollector, String> {
    let tileset_info = load_tileset(tileset_dir)?;
    let mut collector = GeometryCollector::new(tileset_dir.to_path_buf(), casts);

    if let Some(root) = tileset_info.content.get("root") {
        collector.process_tile(root)?;
    }

    Ok(collector)
}
