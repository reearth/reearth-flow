use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
    sync::Arc,
};

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::{
    coordinate::Coordinate, geometry::Geometry3D as FlowGeometry3D, multi_polygon::MultiPolygon3D,
    polygon::Polygon3D,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::{
    errors::SourceError,
    file::reader::runner::{get_content, FileReaderCommonParam},
};

#[derive(Debug, Clone, Default)]
pub(crate) struct ObjReaderFactory;

impl SourceFactory for ObjReaderFactory {
    fn name(&self) -> &str {
        "ObjReader"
    }

    fn description(&self) -> &str {
        "Reads 3D models from Wavefront OBJ files, supporting vertices, faces, normals, texture coordinates, and materials"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ObjReaderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File", "3D"]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
        _state: Option<Vec<u8>>,
    ) -> Result<Box<dyn Source>, BoxedError> {
        let params = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::ObjReaderFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::ObjReaderFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::ObjReaderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let reader = ObjReader { params };
        Ok(Box::new(reader))
    }
}

#[derive(Debug, Clone)]
pub(super) struct ObjReader {
    params: ObjReaderParam,
}

/// # ObjReader Parameters
///
/// Configuration for reading Wavefront OBJ 3D model files with support for
/// vertices, faces, normals, texture coordinates, and material definitions.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct ObjReaderParam {
    #[serde(flatten)]
    pub(super) common: FileReaderCommonParam,

    /// # Parse Materials
    /// Enable parsing of material definitions from MTL files referenced in the OBJ file
    #[serde(default = "default_true")]
    pub(super) parse_materials: bool,

    /// # Material File
    /// Expression that returns the path to an external MTL file to use instead of mtllib directives in the OBJ file. When specified, this overrides any material library references in the OBJ file.
    #[serde(default)]
    pub(super) material_file: Option<Expr>,

    /// # Triangulate
    /// Convert polygons with more than 3 vertices into triangles using fan triangulation
    #[serde(default)]
    pub(super) triangulate: bool,

    /// # Merge Groups
    /// Merge all groups and objects into a single feature instead of creating separate features per group/object
    #[serde(default)]
    pub(super) merge_groups: bool,

    /// # Include Normals
    /// Include vertex normal data in the output geometry
    #[serde(default = "default_true")]
    pub(super) include_normals: bool,

    /// # Include Texture Coordinates
    /// Include texture coordinate (UV) data in the output geometry
    #[serde(default = "default_true")]
    pub(super) include_texcoords: bool,
}

fn default_true() -> bool {
    true
}

#[async_trait::async_trait]
impl Source for ObjReader {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "ObjReader"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let content = get_content(&ctx, &self.params.common, storage_resolver.clone()).await?;

        read_obj(&ctx, storage_resolver, &content, &self.params, sender)
            .await
            .map_err(Into::<BoxedError>::into)
    }
}

#[derive(Debug, Clone, Default)]
struct ObjData {
    vertices: Vec<[f64; 3]>,
    normals: Vec<[f64; 3]>,
    texcoords: Vec<[f64; 3]>,
    faces: Vec<Face>,
    groups: Vec<String>,
    objects: Vec<String>,
    material_libs: Vec<String>,
    comments: Vec<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Face {
    vertices: Vec<FaceVertex>,
    group: Option<String>,
    object: Option<String>,
    material: Option<String>,
    smoothing_group: Option<String>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct FaceVertex {
    vertex_index: i32,
    texture_index: Option<i32>,
    normal_index: Option<i32>,
}

#[derive(Debug, Clone, Default)]
struct Material {
    name: String,
    ambient: Option<[f32; 3]>,
    diffuse: Option<[f32; 3]>,
    specular: Option<[f32; 3]>,
    shininess: Option<f32>,
    transparency: Option<f32>,
    illumination: Option<i32>,
    texture_map: Option<String>,
}

fn safe_f64_to_number(value: f64) -> serde_json::Number {
    if value.is_finite() {
        serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0))
    } else {
        serde_json::Number::from(0)
    }
}

fn extract_material_properties(
    materials: &HashMap<String, Material>,
    used_materials: &[String],
) -> (AttributeValue, AttributeValue) {
    let materials_array = AttributeValue::Array(
        used_materials
            .iter()
            .map(|m| AttributeValue::String(m.clone()))
            .collect(),
    );

    let mut material_details = HashMap::new();
    for mat_name in used_materials {
        if let Some(mat) = materials.get(mat_name) {
            let mut mat_props = HashMap::new();

            if let Some(ambient) = mat.ambient {
                mat_props.insert(
                    "ambient".to_string(),
                    AttributeValue::Array(vec![
                        AttributeValue::Number(safe_f64_to_number(ambient[0] as f64)),
                        AttributeValue::Number(safe_f64_to_number(ambient[1] as f64)),
                        AttributeValue::Number(safe_f64_to_number(ambient[2] as f64)),
                    ]),
                );
            }

            if let Some(diffuse) = mat.diffuse {
                mat_props.insert(
                    "diffuse".to_string(),
                    AttributeValue::Array(vec![
                        AttributeValue::Number(safe_f64_to_number(diffuse[0] as f64)),
                        AttributeValue::Number(safe_f64_to_number(diffuse[1] as f64)),
                        AttributeValue::Number(safe_f64_to_number(diffuse[2] as f64)),
                    ]),
                );
            }

            if let Some(specular) = mat.specular {
                mat_props.insert(
                    "specular".to_string(),
                    AttributeValue::Array(vec![
                        AttributeValue::Number(safe_f64_to_number(specular[0] as f64)),
                        AttributeValue::Number(safe_f64_to_number(specular[1] as f64)),
                        AttributeValue::Number(safe_f64_to_number(specular[2] as f64)),
                    ]),
                );
            }

            if let Some(shininess) = mat.shininess {
                mat_props.insert(
                    "shininess".to_string(),
                    AttributeValue::Number(safe_f64_to_number(shininess as f64)),
                );
            }

            if let Some(transparency) = mat.transparency {
                mat_props.insert(
                    "transparency".to_string(),
                    AttributeValue::Number(safe_f64_to_number(transparency as f64)),
                );
            }

            if let Some(illumination) = mat.illumination {
                mat_props.insert(
                    "illumination".to_string(),
                    AttributeValue::Number(serde_json::Number::from(illumination)),
                );
            }

            if let Some(texture_map) = &mat.texture_map {
                mat_props.insert(
                    "textureMap".to_string(),
                    AttributeValue::String(texture_map.clone()),
                );
            }

            material_details.insert(mat_name.clone(), AttributeValue::Map(mat_props));
        }
    }

    let material_properties = if !material_details.is_empty() {
        AttributeValue::Map(material_details)
    } else {
        AttributeValue::Map(HashMap::new())
    };

    (materials_array, material_properties)
}

async fn read_obj(
    ctx: &NodeContext,
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    content: &Bytes,
    params: &ObjReaderParam,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), SourceError> {
    let obj_data = parse_obj_content(content)?;

    let obj_uri = if let Some(dataset) = &params.common.dataset {
        Uri::from_str(dataset.to_string().trim_matches('"'))
            .unwrap_or_else(|_| Uri::from_str("file://./unknown.obj").unwrap())
    } else {
        Uri::from_str("file://./unknown.obj").unwrap()
    };

    let materials = if params.parse_materials {
        let mut all_materials = HashMap::new();

        if let Some(external_mtl_expr) = &params.material_file {
            let scope = ctx.expr_engine.new_scope();
            let external_mtl = ctx
                .expr_engine
                .eval_scope::<String>(external_mtl_expr.as_ref(), &scope)
                .unwrap_or_else(|_| external_mtl_expr.to_string());
            let mtl_uri =
                resolve_material_path(ctx, storage_resolver.clone(), &obj_uri, &external_mtl)
                    .await?;
            if let Some(mtl_uri) = mtl_uri {
                match parse_mtl(ctx, storage_resolver.clone(), &mtl_uri).await {
                    Ok(mats) => {
                        all_materials.extend(mats);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to parse external material file {}: {}",
                            external_mtl,
                            e
                        );
                    }
                }
            }
        } else if !obj_data.material_libs.is_empty() {
            for mtl_lib in &obj_data.material_libs {
                let mtl_uri =
                    resolve_material_path(ctx, storage_resolver.clone(), &obj_uri, mtl_lib).await?;
                if let Some(mtl_uri) = mtl_uri {
                    match parse_mtl(ctx, storage_resolver.clone(), &mtl_uri).await {
                        Ok(mats) => {
                            all_materials.extend(mats);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse material file {}: {}", mtl_lib, e);
                        }
                    }
                }
            }
        }
        all_materials
    } else {
        HashMap::new()
    };

    if params.merge_groups {
        let geometry = create_geometry_from_faces(&obj_data, &obj_data.faces, params)?;
        let mut attributes = IndexMap::new();

        attributes.insert(
            Attribute::new("source"),
            AttributeValue::String("OBJ".to_string()),
        );

        if !obj_data.objects.is_empty() {
            attributes.insert(
                Attribute::new("objects"),
                AttributeValue::Array(
                    obj_data
                        .objects
                        .iter()
                        .map(|o| AttributeValue::String(o.clone()))
                        .collect(),
                ),
            );
        }

        if !obj_data.groups.is_empty() {
            attributes.insert(
                Attribute::new("groups"),
                AttributeValue::Array(
                    obj_data
                        .groups
                        .iter()
                        .map(|g| AttributeValue::String(g.clone()))
                        .collect(),
                ),
            );
        }

        if !materials.is_empty() {
            let used_materials: Vec<_> = obj_data
                .faces
                .iter()
                .filter_map(|f| f.material.as_ref())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .cloned()
                .collect();

            if !used_materials.is_empty() {
                let (materials_array, material_properties) =
                    extract_material_properties(&materials, &used_materials);

                attributes.insert(Attribute::new("materials"), materials_array);

                if let AttributeValue::Map(ref props) = material_properties {
                    if !props.is_empty() {
                        attributes
                            .insert(Attribute::new("materialProperties"), material_properties);
                    }
                }
            }
        }

        attributes.insert(
            Attribute::new("faceCount"),
            AttributeValue::Number(serde_json::Number::from(obj_data.faces.len())),
        );

        let feature = Feature {
            geometry,
            attributes,
            ..Default::default()
        };

        sender
            .send((
                DEFAULT_PORT.clone(),
                IngestionMessage::OperationEvent { feature },
            ))
            .await
            .map_err(|e| SourceError::ObjReader(format!("Failed to send feature: {e}")))?;
    } else {
        let mut face_groups: HashMap<String, Vec<&Face>> = HashMap::new();

        for face in &obj_data.faces {
            let key = if let Some(obj) = &face.object {
                obj.clone()
            } else if let Some(grp) = &face.group {
                grp.clone()
            } else {
                "default".to_string()
            };

            face_groups.entry(key).or_default().push(face);
        }

        for (group_name, faces) in face_groups {
            let face_refs: Vec<Face> = faces.into_iter().cloned().collect();
            let geometry = create_geometry_from_faces(&obj_data, &face_refs, params)?;

            let mut attributes = IndexMap::new();
            attributes.insert(
                Attribute::new("source"),
                AttributeValue::String("OBJ".to_string()),
            );

            if obj_data.objects.contains(&group_name) {
                attributes.insert(
                    Attribute::new("object"),
                    AttributeValue::String(group_name.clone()),
                );
            } else {
                attributes.insert(
                    Attribute::new("group"),
                    AttributeValue::String(group_name.clone()),
                );
            }

            let group_materials: Vec<_> = face_refs
                .iter()
                .filter_map(|f| f.material.as_ref())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .cloned()
                .collect();

            if !group_materials.is_empty() {
                let (materials_array, material_properties) =
                    extract_material_properties(&materials, &group_materials);

                attributes.insert(Attribute::new("materials"), materials_array);

                if let AttributeValue::Map(ref props) = material_properties {
                    if !props.is_empty() {
                        attributes
                            .insert(Attribute::new("materialProperties"), material_properties);
                    }
                }
            }

            attributes.insert(
                Attribute::new("faceCount"),
                AttributeValue::Number(serde_json::Number::from(face_refs.len())),
            );

            let feature = Feature {
                geometry,
                attributes,
                ..Default::default()
            };

            sender
                .send((
                    DEFAULT_PORT.clone(),
                    IngestionMessage::OperationEvent { feature },
                ))
                .await
                .map_err(|e| SourceError::ObjReader(format!("Failed to send feature: {e}")))?;
        }
    }

    Ok(())
}

fn parse_obj_content(content: &Bytes) -> Result<ObjData, SourceError> {
    let reader = BufReader::new(&content[..]);
    let mut obj_data = ObjData::default();

    let mut current_group: Option<String> = None;
    let mut current_object: Option<String> = None;
    let mut current_material: Option<String> = None;
    let mut current_smoothing_group: Option<String> = None;

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| {
            SourceError::ObjReader(format!(
                "Error reading OBJ file at line {}: {e}",
                line_num + 1
            ))
        })?;

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(comment) = line.strip_prefix('#') {
            obj_data.comments.push(comment.trim().to_string());
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" if parts.len() >= 4 => {
                let x = parts[1].parse::<f64>().map_err(|e| {
                    SourceError::ObjReader(format!(
                        "Invalid vertex X at line {}: {e}",
                        line_num + 1
                    ))
                })?;
                let y = parts[2].parse::<f64>().map_err(|e| {
                    SourceError::ObjReader(format!(
                        "Invalid vertex Y at line {}: {e}",
                        line_num + 1
                    ))
                })?;
                let z = parts[3].parse::<f64>().map_err(|e| {
                    SourceError::ObjReader(format!(
                        "Invalid vertex Z at line {}: {e}",
                        line_num + 1
                    ))
                })?;
                obj_data.vertices.push([x, y, z]);
            }
            "vn" if parts.len() >= 4 => {
                let nx = parts[1].parse::<f64>().map_err(|e| {
                    SourceError::ObjReader(format!(
                        "Invalid normal X at line {}: {e}",
                        line_num + 1
                    ))
                })?;
                let ny = parts[2].parse::<f64>().map_err(|e| {
                    SourceError::ObjReader(format!(
                        "Invalid normal Y at line {}: {e}",
                        line_num + 1
                    ))
                })?;
                let nz = parts[3].parse::<f64>().map_err(|e| {
                    SourceError::ObjReader(format!(
                        "Invalid normal Z at line {}: {e}",
                        line_num + 1
                    ))
                })?;
                obj_data.normals.push([nx, ny, nz]);
            }
            "vt" if parts.len() >= 2 => {
                let u = parts[1].parse::<f64>().map_err(|e| {
                    SourceError::ObjReader(format!(
                        "Invalid texture U at line {}: {e}",
                        line_num + 1
                    ))
                })?;
                let v = if parts.len() >= 3 {
                    parts[2].parse::<f64>().map_err(|e| {
                        SourceError::ObjReader(format!(
                            "Invalid texture V at line {}: {e}",
                            line_num + 1
                        ))
                    })?
                } else {
                    0.0
                };
                let w = if parts.len() >= 4 {
                    parts[3].parse::<f64>().map_err(|e| {
                        SourceError::ObjReader(format!(
                            "Invalid texture W at line {}: {e}",
                            line_num + 1
                        ))
                    })?
                } else {
                    0.0
                };
                obj_data.texcoords.push([u, v, w]);
            }
            "f" if parts.len() >= 4 => {
                let mut face_vertices = Vec::new();
                for part in &parts[1..] {
                    let vertex = parse_face_vertex(part).map_err(|e| {
                        SourceError::ObjReader(format!(
                            "Invalid face vertex at line {}: {}",
                            line_num + 1,
                            e
                        ))
                    })?;
                    face_vertices.push(vertex);
                }

                let face = Face {
                    vertices: face_vertices,
                    group: current_group.clone(),
                    object: current_object.clone(),
                    material: current_material.clone(),
                    smoothing_group: current_smoothing_group.clone(),
                };
                obj_data.faces.push(face);
            }
            "g" if parts.len() >= 2 => {
                let group_name = parts[1..].join(" ");
                if !obj_data.groups.contains(&group_name) {
                    obj_data.groups.push(group_name.clone());
                }
                current_group = Some(group_name);
            }
            "o" if parts.len() >= 2 => {
                let object_name = parts[1..].join(" ");
                if !obj_data.objects.contains(&object_name) {
                    obj_data.objects.push(object_name.clone());
                }
                current_object = Some(object_name);
            }
            "mtllib" if parts.len() >= 2 => {
                let mtl_lib = parts[1..].join(" ");
                obj_data.material_libs.push(mtl_lib);
            }
            "usemtl" if parts.len() >= 2 => {
                current_material = Some(parts[1..].join(" "));
            }
            "s" if parts.len() >= 2 => {
                for part in parts.iter().skip(1) {
                    match *part {
                        "on" => current_smoothing_group = Some("on".to_string()),
                        "off" => current_smoothing_group = None,
                        val => {
                            current_smoothing_group = Some(val.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(obj_data)
}

fn parse_face_vertex(vertex_str: &str) -> Result<FaceVertex, String> {
    let parts: Vec<&str> = vertex_str.split('/').collect();

    let vertex_index = parts[0]
        .parse::<i32>()
        .map_err(|e| format!("Invalid vertex index: {e}"))?;

    let texture_index = if parts.len() > 1 && !parts[1].is_empty() {
        Some(
            parts[1]
                .parse::<i32>()
                .map_err(|e| format!("Invalid texture index: {e}"))?,
        )
    } else {
        None
    };

    let normal_index = if parts.len() > 2 && !parts[2].is_empty() {
        Some(
            parts[2]
                .parse::<i32>()
                .map_err(|e| format!("Invalid normal index: {e}"))?,
        )
    } else {
        None
    };

    Ok(FaceVertex {
        vertex_index,
        texture_index,
        normal_index,
    })
}

async fn resolve_material_path(
    _ctx: &NodeContext,
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    obj_uri: &Uri,
    mtl_lib: &str,
) -> Result<Option<Uri>, SourceError> {
    if let Ok(uri) = Uri::from_str(mtl_lib) {
        let storage = storage_resolver
            .resolve(&uri)
            .map_err(|e| SourceError::ObjReader(format!("Failed to resolve MTL storage: {e}")))?;

        if storage.get(&uri.path()).await.is_ok() {
            return Ok(Some(uri));
        }
    }

    let obj_path = obj_uri.path();
    if let Some(parent_path) = Path::new(&obj_path).parent() {
        let mtl_path = parent_path.join(mtl_lib);
        let _mtl_path_str = mtl_path.to_string_lossy();

        let obj_uri_str = obj_uri.to_string();
        let base_uri = if let Some(slash_pos) = obj_uri_str.rfind('/') {
            &obj_uri_str[..slash_pos]
        } else {
            &obj_uri_str
        };
        let mtl_uri_str = format!("{base_uri}/{mtl_lib}");

        if let Ok(mtl_uri) = Uri::from_str(&mtl_uri_str) {
            return Ok(Some(mtl_uri));
        }
    }

    tracing::warn!("Could not resolve material file path: {}", mtl_lib);
    Ok(None)
}

async fn parse_mtl(
    _ctx: &NodeContext,
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    mtl_uri: &Uri,
) -> Result<HashMap<String, Material>, SourceError> {
    let storage = storage_resolver
        .resolve(mtl_uri)
        .map_err(|e| SourceError::ObjReader(format!("Failed to resolve MTL storage: {e}")))?;
    let result = storage
        .get(&mtl_uri.path())
        .await
        .map_err(|e| SourceError::ObjReader(format!("Failed to read MTL file: {e}")))?;
    let content = result
        .bytes()
        .await
        .map_err(|e| SourceError::ObjReader(format!("Failed to read MTL file content: {e}")))?;
    let content = content.to_vec();
    let reader = BufReader::new(&content[..]);
    let mut materials = HashMap::new();
    let mut current_material: Option<Material> = None;

    for line in reader.lines() {
        let line =
            line.map_err(|e| SourceError::ObjReader(format!("Error reading MTL file: {e}")))?;

        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "newmtl" => {
                if let Some(mat) = current_material.take() {
                    materials.insert(mat.name.clone(), mat);
                }
                if parts.len() > 1 {
                    let new_mat = Material {
                        name: parts[1..].join(" "),
                        ..Default::default()
                    };
                    current_material = Some(new_mat);
                }
            }
            "Ka" if parts.len() >= 4 => {
                if let Some(ref mut mat) = current_material {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                    ) {
                        mat.ambient = Some([r, g, b]);
                    }
                }
            }
            "Kd" if parts.len() >= 4 => {
                if let Some(ref mut mat) = current_material {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                    ) {
                        mat.diffuse = Some([r, g, b]);
                    }
                }
            }
            "Ks" if parts.len() >= 4 => {
                if let Some(ref mut mat) = current_material {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                    ) {
                        mat.specular = Some([r, g, b]);
                    }
                }
            }
            "Ns" if parts.len() >= 2 => {
                if let Some(ref mut mat) = current_material {
                    if let Ok(ns) = parts[1].parse::<f32>() {
                        mat.shininess = Some(ns);
                    }
                }
            }
            "d" | "Tr" if parts.len() >= 2 => {
                if let Some(ref mut mat) = current_material {
                    if let Ok(d) = parts[1].parse::<f32>() {
                        mat.transparency = Some(d);
                    }
                }
            }
            "illum" if parts.len() >= 2 => {
                if let Some(ref mut mat) = current_material {
                    if let Ok(illum) = parts[1].parse::<i32>() {
                        mat.illumination = Some(illum);
                    }
                }
            }
            "map_Kd" if parts.len() >= 2 => {
                if let Some(ref mut mat) = current_material {
                    mat.texture_map = Some(parts[1..].join(" "));
                }
            }
            _ => {}
        }
    }

    if let Some(mat) = current_material {
        materials.insert(mat.name.clone(), mat);
    }

    Ok(materials)
}

fn create_geometry_from_faces(
    obj_data: &ObjData,
    faces: &[Face],
    params: &ObjReaderParam,
) -> Result<Geometry, SourceError> {
    let mut polygons = Vec::new();

    for face in faces {
        let mut polygon_vertices = Vec::new();

        for vertex in &face.vertices {
            let v_idx = if vertex.vertex_index > 0 {
                (vertex.vertex_index - 1) as usize
            } else {
                (obj_data.vertices.len() as i32 + vertex.vertex_index) as usize
            };

            if v_idx >= obj_data.vertices.len() {
                return Err(SourceError::ObjReader(format!(
                    "Vertex index {} out of bounds",
                    vertex.vertex_index
                )));
            }

            let v = obj_data.vertices[v_idx];
            polygon_vertices.push(Coordinate {
                x: v[0],
                y: v[1],
                z: v[2],
            });
        }

        if params.triangulate && polygon_vertices.len() > 3 {
            for i in 1..polygon_vertices.len() - 1 {
                let triangle = vec![
                    polygon_vertices[0],
                    polygon_vertices[i],
                    polygon_vertices[i + 1],
                    polygon_vertices[0],
                ];
                let polygon = Polygon3D::new(triangle.into(), vec![]);
                polygons.push(polygon);
            }
        } else {
            if !polygon_vertices.is_empty() {
                let first = polygon_vertices[0];
                polygon_vertices.push(first);
            }

            let polygon = Polygon3D::new(polygon_vertices.into(), vec![]);
            polygons.push(polygon);
        }
    }

    let flow_geometry = if polygons.len() == 1 {
        FlowGeometry3D::Polygon(polygons.into_iter().next().unwrap())
    } else {
        FlowGeometry3D::MultiPolygon(MultiPolygon3D::new(polygons))
    };

    let geometry = Geometry::with_value(GeometryValue::FlowGeometry3D(flow_geometry));

    Ok(geometry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::Expr;

    #[test]
    fn test_parse_simple_obj() {
        let obj_content = b"# Simple pyramid
v 0.0 1.0 0.0
v -1.0 0.0 -1.0
v 1.0 0.0 -1.0
v 1.0 0.0 1.0
v -1.0 0.0 1.0

f 1 2 3
f 1 3 4
f 1 4 5
f 1 5 2
f 2 5 4 3
";

        let obj_data = parse_obj_content(&Bytes::from_static(obj_content)).unwrap();

        assert_eq!(obj_data.vertices.len(), 5);
        assert_eq!(obj_data.faces.len(), 5);
        assert_eq!(obj_data.vertices[0], [0.0, 1.0, 0.0]);
        assert_eq!(obj_data.faces[0].vertices.len(), 3);
        assert_eq!(obj_data.faces[4].vertices.len(), 4);
    }

    #[test]
    fn test_parse_obj_with_normals_and_textures() {
        let obj_content = b"v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0

vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0

vn 0.0 0.0 1.0

f 1/1/1 2/2/1 3/3/1 4/4/1
";

        let obj_data = parse_obj_content(&Bytes::from_static(obj_content)).unwrap();

        assert_eq!(obj_data.vertices.len(), 4);
        assert_eq!(obj_data.texcoords.len(), 4);
        assert_eq!(obj_data.normals.len(), 1);
        assert_eq!(obj_data.faces.len(), 1);

        let face = &obj_data.faces[0];
        assert_eq!(face.vertices.len(), 4);
        assert_eq!(face.vertices[0].texture_index, Some(1));
        assert_eq!(face.vertices[0].normal_index, Some(1));
    }

    #[test]
    fn test_triangulation() {
        let obj_content = b"v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0

f 1 2 3 4
";

        let obj_data = parse_obj_content(&Bytes::from_static(obj_content)).unwrap();

        let params = ObjReaderParam {
            common: FileReaderCommonParam {
                dataset: Some(Expr::new("\"test.obj\"")),
                inline: None,
            },
            parse_materials: false,
            material_file: None,
            triangulate: true,
            merge_groups: false,
            include_normals: true,
            include_texcoords: true,
        };

        let geometry = create_geometry_from_faces(&obj_data, &obj_data.faces, &params).unwrap();

        match &geometry.value {
            GeometryValue::FlowGeometry3D(FlowGeometry3D::MultiPolygon(mp)) => {
                assert_eq!(mp.0.len(), 2);
            }
            _ => panic!("Expected MultiPolygon3D"),
        }
    }

    #[test]
    fn test_negative_indices() {
        let obj_content = b"v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0

f -4 -3 -2 -1
";

        let obj_data = parse_obj_content(&Bytes::from_static(obj_content)).unwrap();

        assert_eq!(obj_data.faces.len(), 1);
        let face = &obj_data.faces[0];
        assert_eq!(face.vertices[0].vertex_index, -4);
        assert_eq!(face.vertices[1].vertex_index, -3);
        assert_eq!(face.vertices[2].vertex_index, -2);
        assert_eq!(face.vertices[3].vertex_index, -1);

        let params = ObjReaderParam {
            common: FileReaderCommonParam {
                dataset: Some(Expr::new("\"test.obj\"")),
                inline: None,
            },
            parse_materials: false,
            material_file: None,
            triangulate: false,
            merge_groups: false,
            include_normals: true,
            include_texcoords: true,
        };

        let geometry = create_geometry_from_faces(&obj_data, &obj_data.faces, &params).unwrap();
        assert!(matches!(
            &geometry.value,
            GeometryValue::FlowGeometry3D(FlowGeometry3D::Polygon(_))
        ));
    }

    #[test]
    fn test_groups_and_objects() {
        let obj_content = b"o Cube
g cube_group
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
f 1 2 3

o Pyramid
g pyramid_group
v 0.0 2.0 0.0
v 1.0 2.0 0.0
v 0.5 3.0 0.0
f 4 5 6
";

        let obj_data = parse_obj_content(&Bytes::from_static(obj_content)).unwrap();

        assert_eq!(obj_data.objects.len(), 2);
        assert_eq!(obj_data.groups.len(), 2);
        assert!(obj_data.objects.contains(&"Cube".to_string()));
        assert!(obj_data.objects.contains(&"Pyramid".to_string()));
        assert!(obj_data.groups.contains(&"cube_group".to_string()));
        assert!(obj_data.groups.contains(&"pyramid_group".to_string()));

        assert_eq!(obj_data.faces[0].object, Some("Cube".to_string()));
        assert_eq!(obj_data.faces[0].group, Some("cube_group".to_string()));
        assert_eq!(obj_data.faces[1].object, Some("Pyramid".to_string()));
        assert_eq!(obj_data.faces[1].group, Some("pyramid_group".to_string()));
    }

    #[test]
    fn test_safe_f64_to_number() {
        assert_eq!(
            safe_f64_to_number(1.5),
            serde_json::Number::from_f64(1.5).unwrap()
        );
        assert_eq!(
            safe_f64_to_number(0.0),
            serde_json::Number::from_f64(0.0).unwrap()
        );
        assert_eq!(
            safe_f64_to_number(-1.5),
            serde_json::Number::from_f64(-1.5).unwrap()
        );

        assert_eq!(safe_f64_to_number(f64::NAN), serde_json::Number::from(0));
        assert_eq!(
            safe_f64_to_number(f64::INFINITY),
            serde_json::Number::from(0)
        );
        assert_eq!(
            safe_f64_to_number(f64::NEG_INFINITY),
            serde_json::Number::from(0)
        );
    }

    #[test]
    fn test_material_properties_with_edge_values() {
        let mut materials = HashMap::new();
        let mat = Material {
            name: "test_mat".to_string(),
            diffuse: Some([1.0, f32::NAN, 0.5]),
            shininess: Some(f32::INFINITY),
            ..Default::default()
        };
        materials.insert("test_mat".to_string(), mat);

        let used_materials = vec!["test_mat".to_string()];
        let (_, material_properties) = extract_material_properties(&materials, &used_materials);

        let AttributeValue::Map(props) = material_properties else {
            panic!("Expected material_properties to be a Map");
        };

        let Some(AttributeValue::Map(mat_props)) = props.get("test_mat") else {
            panic!("Expected test_mat material to exist and be a Map");
        };

        let Some(AttributeValue::Array(diffuse)) = mat_props.get("diffuse") else {
            panic!("Expected diffuse property to exist and be an Array");
        };
        assert_eq!(diffuse.len(), 3);

        let AttributeValue::Number(n) = &diffuse[1] else {
            panic!("Expected diffuse[1] to be a Number");
        };
        assert_eq!(*n, serde_json::Number::from(0));

        let Some(AttributeValue::Number(shininess)) = mat_props.get("shininess") else {
            panic!("Expected shininess property to exist and be a Number");
        };
        assert_eq!(*shininess, serde_json::Number::from(0));
    }
}
