use reearth_flow_geometry::types::{coordinate::Coordinate, multi_polygon::MultiPolygon3D, polygon::Polygon3D};
use reearth_flow_gltf::{
    extract_feature_properties, parse_gltf, read_indices, read_mesh_features, read_positions_with_transform,
    Transform,
};
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
pub struct DetailLevel {
    pub(crate) multipolygon: MultiPolygon3D<f64>,
    pub(crate) geometric_error: f64,
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

    let json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse tileset.json from {:?}: {}", tileset_path, e))?;

    Ok(TilesetInfo {
        path: tileset_path,
        content: json,
    })
}

/// Collect all GLB file paths referenced in a tileset by traversing the tile hierarchy
pub fn collect_glb_paths_from_tileset(tileset_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let tileset_info = load_tileset(tileset_dir)?;
    let mut glb_paths = Vec::new();

    fn traverse_tile(
        tile: &Value,
        tileset_dir: &Path,
        glb_paths: &mut Vec<PathBuf>,
    ) -> Result<(), String> {
        // Check for content.uri (singular) pointing to a GLB file
        if let Some(content) = tile.get("content") {
            if let Some(uri) = content.get("uri").and_then(|u| u.as_str()) {
                if uri.ends_with(".glb") {
                    let glb_path = tileset_dir.join(uri);
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

        // Check for contents[] (plural array) with URIs
        if let Some(contents) = tile.get("contents").and_then(|c| c.as_array()) {
            for content_item in contents {
                if let Some(uri) = content_item.get("uri").and_then(|u| u.as_str()) {
                    if uri.ends_with(".glb") {
                        let glb_path = tileset_dir.join(uri);
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

        // Recursively traverse children
        if let Some(children) = tile.get("children").and_then(|c| c.as_array()) {
            for child in children {
                traverse_tile(child, tileset_dir, glb_paths)?;
            }
        }

        Ok(())
    }

    // Start traversal from the root tile
    if let Some(root) = tileset_info.content.get("root") {
        traverse_tile(root, tileset_dir, &mut glb_paths)?;
    }

    Ok(glb_paths)
}

struct GeometryCollector {
    tileset_dir: PathBuf,
    result: HashMap<String, Vec<DetailLevel>>,
}

impl GeometryCollector {
    fn new(tileset_dir: PathBuf) -> Self {
        Self {
            tileset_dir,
            result: HashMap::new(),
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
        let feature_list: Vec<(String, _)> = features.into_iter().collect();

        let buffer_data = vec![gltf
            .blob
            .as_ref()
            .ok_or_else(|| format!("GLB file {:?} has no binary blob", glb_path))?
            .clone()];

        // Process scene graph to capture node transforms
        for scene in gltf.scenes() {
            for node in scene.nodes() {
                let transform = Transform::from_node(&node);
                self.process_node(&node, &transform, &buffer_data, &feature_list, glb_path, geometric_error)?;
            }
        }

        Ok(())
    }

    fn process_node(
        &mut self,
        node: &::gltf::Node,
        parent_transform: &Transform,
        buffer_data: &[Vec<u8>],
        feature_list: &[(String, serde_json::Map<String, Value>)],
        glb_path: &Path,
        geometric_error: f64,
    ) -> Result<(), String> {
        // Process mesh if attached to this node
        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                self.process_primitive(&primitive, buffer_data, feature_list, glb_path, geometric_error, parent_transform)?;
            }
        }

        // Recursively process children with accumulated transforms
        for child in node.children() {
            let child_local = Transform::from_node(&child);
            let child_world = child_local.compose(parent_transform);
            self.process_node(&child, &child_world, buffer_data, feature_list, glb_path, geometric_error)?;
        }

        Ok(())
    }

    fn process_primitive(
        &mut self,
        primitive: &::gltf::Primitive,
        buffer_data: &[Vec<u8>],
        feature_list: &[(String, serde_json::Map<String, Value>)],
        glb_path: &Path,
        geometric_error: f64,
        transform: &Transform,
    ) -> Result<(), String> {
        let feature_ids = read_mesh_features(primitive, buffer_data)
            .map_err(|e| format!("Failed to read mesh features from {:?}: {}", glb_path, e))?
            .ok_or_else(|| format!("Primitive has no feature IDs: {:?}", glb_path))?;

        let position_accessor = primitive
            .get(&::gltf::Semantic::Positions)
            .ok_or_else(|| format!("Primitive has no positions: {:?}", glb_path))?;
        let positions = read_positions_with_transform(&position_accessor, buffer_data, Some(transform))
            .map_err(|e| format!("Failed to read positions: {}", e))?;

        let indices = primitive
            .indices()
            .ok_or_else(|| format!("Primitive has no indices: {:?}", glb_path))?;
        let indices = read_indices(&indices, buffer_data)
            .map_err(|e| format!("Failed to read indices: {}", e))?;

        self.split_by_feature(feature_ids, positions, indices, feature_list, glb_path, geometric_error)?;
        Ok(())
        }

    fn split_by_feature(
        &mut self,
        feature_ids: Vec<u32>,
        positions: Vec<Coordinate>,
        indices: Vec<usize>,
        feature_list: &[(String, serde_json::Map<String, Value>)],
        glb_path: &Path,
        geometric_error: f64,
    ) -> Result<(), String> {
        let mut feature_polygons: HashMap<u32, Vec<Polygon3D<f64>>> = HashMap::new();

        if indices.len() % 3 != 0 {
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

            let triangle = vec![positions[idx0], positions[idx1], positions[idx2], positions[idx0]];
            feature_polygons
                .entry(fid0)
                .or_insert_with(Vec::new)
                .push(Polygon3D::new(triangle.into(), vec![]));
        }

        for (feature_id, polygons) in feature_polygons {
            let gml_id = Self::lookup_gml_id(feature_id, feature_list, glb_path)?;
            self.store_geometry(gml_id, MultiPolygon3D::new(polygons), geometric_error);
        }

        Ok(())
    }

    fn lookup_gml_id(
        feature_id: u32,
        feature_list: &[(String, serde_json::Map<String, Value>)],
        glb_path: &Path,
    ) -> Result<String, String> {
        feature_list
            .get(feature_id as usize)
            .map(|(gml_id, _)| gml_id.clone())
            .ok_or_else(|| {
                format!(
                    "Feature ID {} not found in property table for {:?}",
                    feature_id, glb_path
                )
            })
    }

    fn store_geometry(&mut self, gml_id: String, multipolygon: MultiPolygon3D<f64>, geometric_error: f64) {
        let entry = self.result.entry(gml_id).or_insert_with(Vec::new);

        // Just append - features are added in depth order as we traverse the tree
        entry.push(DetailLevel {
            multipolygon,
            geometric_error,
        });
    }
}

pub fn collect_geometries_by_gmlid(
    tileset_dir: &Path,
) -> Result<HashMap<String, Vec<DetailLevel>>, String> {
    let tileset_info = load_tileset(tileset_dir)?;
    let mut collector = GeometryCollector::new(tileset_dir.to_path_buf());

    if let Some(root) = tileset_info.content.get("root") {
        collector.process_tile(root)?;
    }

    Ok(collector.result)
}
