use std::{collections::HashMap, vec};

use once_cell::sync::Lazy;
use reearth_flow_geometry::types::{
    face::Face,
    geometry::{Geometry2D, Geometry3D},
    line_string::LineString,
    polygon::Polygon,
    solid::{Solid2D, Solid3D},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Feature, Geometry, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static EXTRACTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("extracted"));
pub static REMAINING_PORT: Lazy<Port> = Lazy::new(|| Port::new("remaining"));
pub static UNTOUCHED_PORT: Lazy<Port> = Lazy::new(|| Port::new("untouched"));

#[derive(Debug, Clone, Default)]
pub struct GeometryPartExtractorFactory;

impl ProcessorFactory for GeometryPartExtractorFactory {
    fn name(&self) -> &str {
        "GeometryPartExtractor"
    }

    fn description(&self) -> &str {
        "Extract geometry parts (surfaces) from 3D geometries as separate features"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryPartExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            EXTRACTED_PORT.clone(),
            REMAINING_PORT.clone(),
            UNTOUCHED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: GeometryPartExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryPartExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryPartExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            GeometryPartExtractorParam::default()
        };
        Ok(Box::new(GeometryPartExtractor::new(param)))
    }
}

/// # Geometry Part Extractor Parameters
/// Configure which geometry parts to extract from 3D geometries
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryPartExtractorParam {
    /// # Part Type
    /// Type of geometry part to extract
    #[serde(default, rename = "geometryPartType")]
    part_type: GeometryPartType,
}

impl Default for GeometryPartExtractorParam {
    fn default() -> Self {
        Self {
            part_type: GeometryPartType::Surface,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub enum GeometryPartType {
    /// Extract surfaces as separate features
    #[default]
    Surface,
}

#[derive(Debug, Clone)]
pub struct GeometryPartExtractor {
    param: GeometryPartExtractorParam,
}

impl GeometryPartExtractor {
    pub fn new(param: GeometryPartExtractorParam) -> Self {
        Self { param }
    }
}

impl Processor for GeometryPartExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            // Send feature to untouched port if geometry is empty
            fw.send(ctx.new_with_feature_and_port(feature.clone(), UNTOUCHED_PORT.clone()));
            return Ok(());
        }

        match &self.param.part_type {
            GeometryPartType::Surface => {
                let extracted = extract_surfaces(feature, &ctx, fw)?;
                if !extracted {
                    // No surfaces were extracted, send to untouched port
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), UNTOUCHED_PORT.clone()));
                }
            }
        }

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "GeometryPartExtractor"
    }
}

fn extract_surfaces(
    feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) -> Result<bool, BoxedError> {
    match &feature.geometry.value {
        GeometryValue::FlowGeometry2D(geometry) => match geometry {
            Geometry2D::Solid(solid) => {
                let extracted = extract_surfaces_from_solid_2d(solid, feature, ctx, fw);
                if extracted {
                    // Send remaining feature with empty geometry (surfaces removed)
                    send_remaining_feature_with_empty_geometry(feature, ctx, fw);
                }
                Ok(extracted)
            }
            _ => Ok(false),
        },
        GeometryValue::FlowGeometry3D(geometry) => match geometry {
            Geometry3D::Solid(solid) => {
                let extracted = extract_surfaces_from_solid_3d(solid, feature, ctx, fw);
                if extracted {
                    // Send remaining feature with empty geometry (surfaces removed)
                    send_remaining_feature_with_empty_geometry(feature, ctx, fw);
                }
                Ok(extracted)
            }
            _ => Ok(false),
        },
        GeometryValue::CityGmlGeometry(geometry) => {
            // CityGML geometries already contain surfaces as polygons
            let mut surface_count = 0;
            for (gml_geo_idx, geo_feature) in geometry.gml_geometries.iter().enumerate() {
                for (poly_idx, polygon) in geo_feature.polygons.iter().enumerate() {
                    create_surface_feature_from_citygml_polygon(
                        polygon,
                        geometry,
                        gml_geo_idx,
                        poly_idx,
                        feature,
                        ctx,
                        fw,
                    );
                    surface_count += 1;
                }
            }
            let extracted = surface_count > 0;
            if extracted {
                // Send remaining feature with empty geometry (surfaces removed)
                send_remaining_feature_with_empty_geometry(feature, ctx, fw);
            }
            Ok(extracted)
        }
        GeometryValue::None => {
            // No geometry to process
            Ok(false)
        }
    }
}

fn extract_surfaces_from_solid_2d(
    solid: &Solid2D<f64>,
    feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) -> bool {
    let faces = solid.all_faces();
    if faces.is_empty() {
        return false;
    }

    // Extract all faces as surfaces
    for face in &faces {
        create_surface_feature_from_face_2d(face, feature, ctx, fw);
    }
    true
}

fn extract_surfaces_from_solid_3d(
    solid: &Solid3D<f64>,
    feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) -> bool {
    let faces = solid.all_faces();
    if faces.is_empty() {
        return false;
    }

    // Extract all faces as surfaces
    for face in &faces {
        create_surface_feature_from_face_3d(face, feature, ctx, fw);
    }
    true
}

fn send_remaining_feature_with_empty_geometry(
    original_feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) {
    let mut remaining_feature = original_feature.clone();
    // Create empty geometry but keep the same type structure
    remaining_feature.geometry = Geometry::new();

    fw.send(ctx.new_with_feature_and_port(remaining_feature, REMAINING_PORT.clone()));
}

fn create_surface_feature_from_face_2d(
    face: &Face<f64, reearth_flow_geometry::types::no_value::NoValue>,
    original_feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) {
    // Convert face to polygon
    if face.0.len() < 3 {
        return; // Not a valid polygon
    }

    let line_string = LineString::new(face.0.clone());
    let polygon = Polygon::new(line_string, vec![]);

    let mut surface_feature = original_feature.clone();
    surface_feature.refresh_id();

    let mut surface_geometry = original_feature.geometry.clone();
    surface_geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon));
    surface_feature.geometry = surface_geometry;

    fw.send(ctx.new_with_feature_and_port(surface_feature, EXTRACTED_PORT.clone()));
}

fn create_surface_feature_from_face_3d(
    face: &Face<f64, f64>,
    original_feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) {
    // Convert face to polygon
    if face.0.len() < 3 {
        return; // Not a valid polygon
    }

    let line_string = LineString::new(face.0.clone());
    let polygon = Polygon::new(line_string, vec![]);

    let mut surface_feature = original_feature.clone();
    surface_feature.refresh_id();

    let mut surface_geometry = original_feature.geometry.clone();
    surface_geometry.value = GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon));
    surface_feature.geometry = surface_geometry;

    fw.send(ctx.new_with_feature_and_port(surface_feature, EXTRACTED_PORT.clone()));
}

fn create_surface_feature_from_citygml_polygon(
    polygon: &reearth_flow_geometry::types::polygon::Polygon3D<f64>,
    original_citygml: &reearth_flow_types::CityGmlGeometry,
    gml_geo_idx: usize,
    poly_idx: usize,
    original_feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) {
    use reearth_flow_types::{CityGmlGeometry, GmlGeometry};

    let mut surface_feature = original_feature.clone();
    surface_feature.refresh_id();

    // Create a new CityGmlGeometry with only the single polygon
    let original_gml_geo = &original_citygml.gml_geometries[gml_geo_idx];
    let new_gml_geo = GmlGeometry {
        id: original_gml_geo.id.clone(),
        feature_id: original_gml_geo.feature_id.clone(),
        feature_type: original_gml_geo.feature_type.clone(),
        polygons: vec![polygon.clone()],
        len: 1,                     // Single surface has length 1
        composite_surfaces: vec![], // Single surface doesn't need composite surfaces
        line_strings: vec![],       // Single surface doesn't need line strings
        ty: original_gml_geo.ty,
        lod: original_gml_geo.lod,
        pos: original_gml_geo.pos,
    };

    // Calculate the material and texture indices for this specific polygon
    let global_poly_idx = calculate_global_polygon_index(original_citygml, gml_geo_idx, poly_idx);

    let new_citygml = CityGmlGeometry {
        gml_geometries: vec![new_gml_geo],
        materials: original_citygml.materials.clone(), // Keep all materials
        textures: original_citygml.textures.clone(),   // Keep all textures
        polygon_materials: if global_poly_idx < original_citygml.polygon_materials.len() {
            vec![original_citygml.polygon_materials[global_poly_idx]]
        } else {
            vec![None]
        },
        polygon_textures: if global_poly_idx < original_citygml.polygon_textures.len() {
            vec![original_citygml.polygon_textures[global_poly_idx]]
        } else {
            vec![None]
        },
        polygon_uvs: Default::default(),
    };

    let mut surface_geometry = original_feature.geometry.clone();
    surface_geometry.value = GeometryValue::CityGmlGeometry(new_citygml);
    surface_feature.geometry = surface_geometry;

    fw.send(ctx.new_with_feature_and_port(surface_feature, EXTRACTED_PORT.clone()));
}

fn calculate_global_polygon_index(
    citygml: &reearth_flow_types::CityGmlGeometry,
    gml_geo_idx: usize,
    poly_idx: usize,
) -> usize {
    let mut global_idx = 0;
    for (i, geo_feature) in citygml.gml_geometries.iter().enumerate() {
        if i == gml_geo_idx {
            return global_idx + poly_idx;
        }
        global_idx += geo_feature.polygons.len();
    }
    global_idx + poly_idx
}
