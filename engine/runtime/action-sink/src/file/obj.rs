use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::geometry::Geometry3D as FlowGeometry3D;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Attribute, Expr, Feature, GeometryValue};
use rhai::AST;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;

#[derive(Debug, Clone, Default)]
pub struct ObjWriterFactory;

impl SinkFactory for ObjWriterFactory {
    fn name(&self) -> &str {
        "ObjWriter"
    }

    fn description(&self) -> &str {
        "Writes 3D features to Wavefront OBJ format with optional material (MTL) files"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ObjWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File", "3D"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: ObjWriterParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::BuildFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::BuildFactory(format!("Failed to deserialize `with` parameter: {e}"))
            })?
        } else {
            return Err(
                SinkError::BuildFactory("Missing required parameter `with`".to_string()).into(),
            );
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let output = expr_engine
            .compile(params.output.as_ref())
            .map_err(|e| SinkError::BuildFactory(e.to_string()))?;

        let sink = ObjWriter {
            output,
            global_params: with,
            buffer: Vec::new(),
            write_materials: params.write_materials.unwrap_or(true),
            write_normals: params.write_normals.unwrap_or(true),
            write_texcoords: params.write_texcoords.unwrap_or(true),
        };
        Ok(Box::new(sink))
    }
}

/// # OBJ Writer Parameters
/// Configure output settings for writing 3D features to Wavefront OBJ format
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ObjWriterParam {
    /// # Output Path
    /// Expression for the output file path where the OBJ file will be written
    output: Expr,
    /// # Write Materials
    /// Enable writing of material (MTL) file alongside the OBJ file
    #[serde(default)]
    write_materials: Option<bool>,
    /// # Write Normals
    /// Include vertex normal vectors in the output
    #[serde(default)]
    write_normals: Option<bool>,
    /// # Write Texture Coordinates
    /// Include texture coordinate (UV) data in the output
    #[serde(default)]
    write_texcoords: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ObjWriter {
    output: AST,
    global_params: Option<HashMap<String, Value>>,
    buffer: Vec<Feature>,
    write_materials: bool,
    write_normals: bool,
    write_texcoords: bool,
}

impl Sink for ObjWriter {
    fn name(&self) -> &str {
        "ObjWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        self.buffer.push(ctx.feature);
        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let storage_resolver = Arc::clone(&ctx.storage_resolver);

        let scope = expr_engine.new_scope();
        if let Some(ref params) = self.global_params {
            for (k, v) in params {
                scope.set(k.as_str(), v.clone());
            }
        }

        let path = scope
            .eval_ast::<String>(&self.output)
            .map_err(|e| SinkError::ObjWriter(e.to_string()))?;
        let output = Uri::from_str(path.as_str())?;

        let (obj_content, mtl_content) = features_to_obj(&self.buffer, self)?;

        let storage = storage_resolver
            .resolve(&output)
            .map_err(|e| SinkError::ObjWriter(format!("Failed to resolve storage: {e}")))?;
        storage
            .put_sync(output.path().as_path(), Bytes::from(obj_content))
            .map_err(|e| SinkError::ObjWriter(format!("Failed to write OBJ file: {e}")))?;

        if self.write_materials && !mtl_content.is_empty() {
            let mtl_path = output.to_string().replace(".obj", ".mtl");
            let mtl_uri = Uri::from_str(&mtl_path)?;
            let mtl_storage = storage_resolver
                .resolve(&mtl_uri)
                .map_err(|e| SinkError::ObjWriter(format!("Failed to resolve MTL storage: {e}")))?;
            mtl_storage
                .put_sync(mtl_uri.path().as_path(), Bytes::from(mtl_content))
                .map_err(|e| SinkError::ObjWriter(format!("Failed to write MTL file: {e}")))?;
        }

        Ok(())
    }
}

fn features_to_obj(
    features: &[Feature],
    writer: &ObjWriter,
) -> Result<(String, String), BoxedError> {
    let mut obj_output = String::new();
    let mut mtl_output = String::new();
    let mut materials: IndexMap<String, Material> = IndexMap::new();

    obj_output.push_str("# Generated by Re:Earth Flow ObjWriter\n");

    if writer.write_materials {
        let obj_path = "output.obj";
        let mtl_filename = obj_path.replace(".obj", ".mtl");
        obj_output.push_str(&format!("mtllib {mtl_filename}\n"));
    }
    obj_output.push('\n');

    let mut vertex_offset = 0;
    let mut normal_offset = 0;
    let mut texcoord_offset = 0;

    for (feature_idx, feature) in features.iter().enumerate() {
        let geometry = match &feature.geometry.value {
            GeometryValue::FlowGeometry3D(geom) => geom,
            _ => continue, // Skip non-3D geometries
        };

        let object_name = feature
            .attributes
            .get(&Attribute::new("object"))
            .and_then(|v| v.as_string())
            .or_else(|| {
                feature
                    .attributes
                    .get(&Attribute::new("group"))
                    .and_then(|v| v.as_string())
            })
            .unwrap_or_else(|| format!("feature_{feature_idx}"));

        obj_output.push_str(&format!("o {object_name}\n"));

        let (vertices, normals, texcoords) = extract_geometry_data(geometry);

        for [x, y, z] in &vertices {
            obj_output.push_str(&format!("v {x} {y} {z}\n"));
        }

        if writer.write_texcoords {
            for [u, v] in &texcoords {
                obj_output.push_str(&format!("vt {u} {v}\n"));
            }
        }

        if writer.write_normals {
            for [nx, ny, nz] in &normals {
                obj_output.push_str(&format!("vn {nx} {ny} {nz}\n"));
            }
        }

        let material_name = if writer.write_materials {
            extract_material_name(feature, &mut materials, &object_name)
        } else {
            None
        };

        if let Some(ref mat_name) = material_name {
            obj_output.push_str(&format!("usemtl {mat_name}\n"));
        }

        let face_count = vertices.len() / 3; // Assuming triangulated
        for i in 0..face_count {
            let v1 = vertex_offset + i * 3 + 1;
            let v2 = vertex_offset + i * 3 + 2;
            let v3 = vertex_offset + i * 3 + 3;

            if writer.write_texcoords
                && writer.write_normals
                && !texcoords.is_empty()
                && !normals.is_empty()
            {
                let vt1 = texcoord_offset + i * 3 + 1;
                let vt2 = texcoord_offset + i * 3 + 2;
                let vt3 = texcoord_offset + i * 3 + 3;
                let vn1 = normal_offset + i * 3 + 1;
                let vn2 = normal_offset + i * 3 + 2;
                let vn3 = normal_offset + i * 3 + 3;
                obj_output.push_str(&format!(
                    "f {v1}/{vt1}/{vn1} {v2}/{vt2}/{vn2} {v3}/{vt3}/{vn3}\n"
                ));
            } else if writer.write_texcoords && !texcoords.is_empty() {
                let vt1 = texcoord_offset + i * 3 + 1;
                let vt2 = texcoord_offset + i * 3 + 2;
                let vt3 = texcoord_offset + i * 3 + 3;
                obj_output.push_str(&format!("f {v1}/{vt1} {v2}/{vt2} {v3}/{vt3}\n"));
            } else if writer.write_normals && !normals.is_empty() {
                let vn1 = normal_offset + i * 3 + 1;
                let vn2 = normal_offset + i * 3 + 2;
                let vn3 = normal_offset + i * 3 + 3;
                obj_output.push_str(&format!("f {v1}//{vn1} {v2}//{vn2} {v3}//{vn3}\n"));
            } else {
                obj_output.push_str(&format!("f {v1} {v2} {v3}\n"));
            }
        }

        vertex_offset += vertices.len();
        texcoord_offset += texcoords.len();
        normal_offset += normals.len();

        obj_output.push('\n');
    }

    // Generate MTL content
    if writer.write_materials && !materials.is_empty() {
        mtl_output = generate_mtl_content(&materials);
    }

    Ok((obj_output, mtl_output))
}

#[derive(Debug, Clone)]
struct Material {
    ambient: Option<[f32; 3]>,
    diffuse: Option<[f32; 3]>,
    specular: Option<[f32; 3]>,
    shininess: Option<f32>,
    transparency: Option<f32>,
}

type VertexData = (Vec<[f64; 3]>, Vec<[f64; 3]>, Vec<[f64; 2]>);

fn extract_geometry_data(geometry: &FlowGeometry3D) -> VertexData {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let texcoords = Vec::new();

    match geometry {
        FlowGeometry3D::Point(point) => {
            vertices.push([point.x(), point.y(), point.z()]);
        }
        FlowGeometry3D::MultiPoint(multi_point) => {
            for point in multi_point.iter() {
                vertices.push([point.x(), point.y(), point.z()]);
            }
        }
        FlowGeometry3D::LineString(line) => {
            for coord in line.coords() {
                vertices.push([coord.x, coord.y, coord.z]);
            }
        }
        FlowGeometry3D::MultiLineString(multi_line) => {
            for line in multi_line.iter() {
                for coord in line.coords() {
                    vertices.push([coord.x, coord.y, coord.z]);
                }
            }
        }
        FlowGeometry3D::Polygon(polygon) => {
            for coord in polygon.exterior().coords() {
                vertices.push([coord.x, coord.y, coord.z]);
            }
            if vertices.len() >= 3 {
                if let Some(normal) = calculate_polygon_normal(&vertices) {
                    for _ in &vertices {
                        normals.push(normal);
                    }
                }
            }
        }
        FlowGeometry3D::MultiPolygon(multi_polygon) => {
            for polygon in multi_polygon.iter() {
                let start_idx = vertices.len();
                for coord in polygon.exterior().coords() {
                    vertices.push([coord.x, coord.y, coord.z]);
                }
                // Calculate normals for each polygon
                let poly_verts = &vertices[start_idx..];
                if poly_verts.len() >= 3 {
                    if let Some(normal) = calculate_polygon_normal(poly_verts) {
                        for _ in poly_verts {
                            normals.push(normal);
                        }
                    }
                }
            }
        }
        _ => {}
    }

    (vertices, normals, texcoords)
}

fn calculate_polygon_normal(vertices: &[[f64; 3]]) -> Option<[f64; 3]> {
    if vertices.len() < 3 {
        return None;
    }

    let v0 = vertices[0];
    let v1 = vertices[1];
    let v2 = vertices[2];

    let u = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let v = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    let nx = u[1] * v[2] - u[2] * v[1];
    let ny = u[2] * v[0] - u[0] * v[2];
    let nz = u[0] * v[1] - u[1] * v[0];

    let length = (nx * nx + ny * ny + nz * nz).sqrt();
    if length > 0.0 {
        Some([nx / length, ny / length, nz / length])
    } else {
        None
    }
}

fn extract_material_name(
    feature: &Feature,
    materials: &mut IndexMap<String, Material>,
    default_name: &str,
) -> Option<String> {
    if let Some(material_props) = feature
        .attributes
        .get(&Attribute::new("materialProperties"))
    {
        if let Some(props_map) = material_props.as_map() {
            if let Some((mat_name, mat_value)) = props_map.into_iter().next() {
                if !materials.contains_key(&mat_name) {
                    let mut material = Material {
                        ambient: None,
                        diffuse: None,
                        specular: None,
                        shininess: None,
                        transparency: None,
                    };

                    if let Some(mat_map) = mat_value.as_map() {
                        if let Some(reearth_flow_types::AttributeValue::Array(arr)) =
                            mat_map.get("diffuse")
                        {
                            if arr.len() >= 3 {
                                material.diffuse = Some([
                                    arr[0].as_f64().unwrap_or(0.0) as f32,
                                    arr[1].as_f64().unwrap_or(0.0) as f32,
                                    arr[2].as_f64().unwrap_or(0.0) as f32,
                                ]);
                            }
                        }
                    }

                    materials.insert(mat_name.clone(), material);
                }
                return Some(mat_name.clone());
            }
        }
    }

    let mat_name = format!("material_{default_name}");
    if !materials.contains_key(&mat_name) {
        materials.insert(
            mat_name.clone(),
            Material {
                ambient: Some([0.2, 0.2, 0.2]),
                diffuse: Some([0.8, 0.8, 0.8]),
                specular: Some([1.0, 1.0, 1.0]),
                shininess: Some(32.0),
                transparency: Some(1.0),
            },
        );
    }
    Some(mat_name)
}

fn generate_mtl_content(materials: &IndexMap<String, Material>) -> String {
    let mut output = String::new();
    output.push_str("# Generated by Re:Earth Flow ObjWriter\n\n");

    for (name, material) in materials {
        output.push_str(&format!("newmtl {name}\n"));

        if let Some([r, g, b]) = material.ambient {
            output.push_str(&format!("Ka {r} {g} {b}\n"));
        }

        if let Some([r, g, b]) = material.diffuse {
            output.push_str(&format!("Kd {r} {g} {b}\n"));
        }

        if let Some([r, g, b]) = material.specular {
            output.push_str(&format!("Ks {r} {g} {b}\n"));
        }

        if let Some(ns) = material.shininess {
            output.push_str(&format!("Ns {ns}\n"));
        }

        if let Some(d) = material.transparency {
            output.push_str(&format!("d {d}\n"));
        }

        output.push_str("illum 2\n\n");
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use reearth_flow_geometry::types::{
        coordinate::Coordinate, geometry::Geometry3D, polygon::Polygon3D,
    };
    use reearth_flow_types::{Attribute, AttributeValue, Feature, Geometry, GeometryValue};

    #[test]
    fn test_generate_simple_obj() {
        let mut features = Vec::new();

        let coords = vec![
            Coordinate {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Coordinate {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Coordinate {
                x: 0.5,
                y: 1.0,
                z: 0.0,
            },
            Coordinate {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ];

        let polygon = Polygon3D::new(coords.into(), vec![]);
        let geometry =
            Geometry::with_value(GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon)));

        let mut attributes = IndexMap::new();
        attributes.insert(
            Attribute::new("object"),
            AttributeValue::String("pyramid".to_string()),
        );

        let feature = Feature {
            geometry,
            attributes,
            ..Default::default()
        };

        features.push(feature);

        let expr_engine = reearth_flow_eval_expr::engine::Engine::new();
        let output_ast = expr_engine.compile("\"/tmp/test.obj\"").unwrap();

        let writer = ObjWriter {
            output: output_ast,
            global_params: None,
            buffer: Vec::new(),
            write_materials: true,
            write_normals: true,
            write_texcoords: false,
        };

        let (obj_content, mtl_content) = features_to_obj(&features, &writer).unwrap();

        assert!(obj_content.contains("# Generated by Re:Earth Flow ObjWriter"));
        assert!(obj_content.contains("o pyramid"));
        assert!(obj_content.contains("v 0 0 0"));
        assert!(obj_content.contains("v 1 0 0"));
        assert!(obj_content.contains("v 0.5 1 0"));
        assert!(obj_content.contains("vn "));
        assert!(obj_content.contains("f "));

        assert!(mtl_content.contains("# Generated by Re:Earth Flow ObjWriter"));
        assert!(mtl_content.contains("newmtl material_pyramid"));

        println!("Generated OBJ:\n{}", obj_content);
        println!("\nGenerated MTL:\n{}", mtl_content);
    }

    #[test]
    fn test_extract_geometry_data() {
        let coords = vec![
            Coordinate {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Coordinate {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Coordinate {
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
            Coordinate {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ];

        let polygon = Polygon3D::new(coords.into(), vec![]);
        let geometry = Geometry3D::Polygon(polygon);

        let (vertices, normals, _texcoords) = extract_geometry_data(&geometry);

        assert_eq!(vertices.len(), 4);
        assert_eq!(vertices[0], [0.0, 0.0, 0.0]);
        assert_eq!(vertices[1], [1.0, 0.0, 0.0]);
        assert_eq!(vertices[2], [1.0, 1.0, 0.0]);

        assert_eq!(normals.len(), 4);
        assert!(normals[0][2].abs() > 0.9);
    }

    #[test]
    fn test_calculate_polygon_normal() {
        let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];

        let normal = calculate_polygon_normal(&vertices).unwrap();

        assert!(normal[2].abs() > 0.99);
        assert!(normal[0].abs() < 0.01);
        assert!(normal[1].abs() < 0.01);
    }

    #[test]
    fn test_generate_mtl_content() {
        let mut materials = IndexMap::new();
        materials.insert(
            "test_material".to_string(),
            Material {
                ambient: Some([0.2, 0.2, 0.2]),
                diffuse: Some([0.8, 0.5, 0.3]),
                specular: Some([1.0, 1.0, 1.0]),
                shininess: Some(32.0),
                transparency: Some(1.0),
            },
        );

        let mtl_content = generate_mtl_content(&materials);

        assert!(mtl_content.contains("newmtl test_material"));
        assert!(mtl_content.contains("Ka 0.2 0.2 0.2"));
        assert!(mtl_content.contains("Kd 0.8 0.5 0.3"));
        assert!(mtl_content.contains("Ks 1 1 1"));
        assert!(mtl_content.contains("Ns 32"));
        assert!(mtl_content.contains("d 1"));
        assert!(mtl_content.contains("illum 2"));
    }
}
