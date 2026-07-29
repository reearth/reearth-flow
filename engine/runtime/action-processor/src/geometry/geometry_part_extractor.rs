use std::{collections::HashMap, sync::Arc, vec};

use once_cell::sync::Lazy;
use reearth_flow_geometry::types::{
    coordinate::Coordinate2D,
    face::Face,
    geometry::{Geometry2D, Geometry3D},
    line_string::{LineString, LineString2D},
    multi_polygon::MultiPolygon2D,
    polygon::{Polygon, Polygon2D},
    solid::{Solid2D, Solid3D},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Feature, Geometry, GeometryValue};
use serde_json::Value;

/// Each surface pulled out of a geometry, as its own feature.
pub static EXTRACTED_PORT: Lazy<Port> = Lazy::new(|| Port::new("extracted"));
/// The original feature with its extracted surfaces removed. Emitted only when
/// extraction produced something.
pub static REMAINING_PORT: Lazy<Port> = Lazy::new(|| Port::new("remaining"));
/// Features left as they arrived, because there was nothing to extract from them.
pub static UNTOUCHED_PORT: Lazy<Port> = Lazy::new(|| Port::new("untouched"));

#[derive(Debug, Clone, Default)]
pub struct GeometryPartExtractorFactory;

impl ProcessorFactory for GeometryPartExtractorFactory {
    fn name(&self) -> &str {
        "Geometry Part Extractor"
    }

    fn description(&self) -> &str {
        "Extracts the individual surfaces of a geometry, emitting each as a separate feature."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
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
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(GeometryPartExtractor))
    }
}

#[derive(Debug, Clone)]
pub struct GeometryPartExtractor;

impl Processor for GeometryPartExtractor {
    #[cfg(not(feature = "new-geometry"))]
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

        let extracted = extract_surfaces(feature, &ctx, fw)?;
        if !extracted {
            // No surfaces were extracted, send to untouched port
            fw.send(ctx.new_with_feature_and_port(feature.clone(), UNTOUCHED_PORT.clone()));
        }

        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Geometry Part Extractor"
    }
}

#[cfg(not(feature = "new-geometry"))]
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

#[cfg(not(feature = "new-geometry"))]
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

#[cfg(not(feature = "new-geometry"))]
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

#[cfg(not(feature = "new-geometry"))]
fn send_remaining_feature_with_empty_geometry(
    original_feature: &Feature,
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
) {
    let mut remaining_feature = original_feature.clone();
    // Create empty geometry but keep the same type structure
    remaining_feature.geometry = Arc::new(Geometry::default());

    fw.send(ctx.new_with_feature_and_port(remaining_feature, REMAINING_PORT.clone()));
}

#[cfg(not(feature = "new-geometry"))]
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

    let mut surface_geometry = (*original_feature.geometry).clone();
    surface_geometry.value = GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon));
    surface_feature.geometry = Arc::new(surface_geometry);

    fw.send(ctx.new_with_feature_and_port(surface_feature, EXTRACTED_PORT.clone()));
}

#[cfg(not(feature = "new-geometry"))]
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

    let mut surface_geometry = (*original_feature.geometry).clone();
    surface_geometry.value = GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon));
    surface_feature.geometry = Arc::new(surface_geometry);

    fw.send(ctx.new_with_feature_and_port(surface_feature, EXTRACTED_PORT.clone()));
}

#[cfg(not(feature = "new-geometry"))]
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
        len: 1,               // Single surface has length 1
        line_strings: vec![], // Single surface doesn't need line strings
        points: vec![],       // Single surface doesn't need points
        ty: original_gml_geo.ty,
        gml_trait: None, // Extracted geometry loses original trait semantics
        lod: original_gml_geo.lod,
        pos: 0, // Extracted single-polygon feature starts at index 0 in the flat arrays
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
        polygon_uvs: if global_poly_idx < original_citygml.polygon_uvs.0.len() {
            MultiPolygon2D::new(vec![original_citygml.polygon_uvs.0[global_poly_idx].clone()])
        } else {
            MultiPolygon2D::new(vec![create_placeholder_uv_polygon(polygon)])
        },
    };

    let mut surface_geometry = (*original_feature.geometry).clone();
    surface_geometry.value = GeometryValue::CityGmlGeometry(new_citygml);
    surface_feature.geometry = Arc::new(surface_geometry);

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

/// Create a placeholder UV polygon matching the ring structure of a 3D polygon.
/// Each vertex gets (0.0, 0.0) UV coordinates.
fn create_placeholder_uv_polygon(
    poly3d: &reearth_flow_geometry::types::polygon::Polygon3D<f64>,
) -> Polygon2D<f64> {
    let exterior_uvs: Vec<Coordinate2D<f64>> = poly3d
        .exterior()
        .0
        .iter()
        .map(|_| Coordinate2D::new_(0.0, 0.0))
        .collect();

    let interior_uvs: Vec<LineString2D<f64>> = poly3d
        .interiors()
        .iter()
        .map(|ring| {
            let coords: Vec<Coordinate2D<f64>> = ring
                .0
                .iter()
                .map(|_| Coordinate2D::new_(0.0, 0.0))
                .collect();
            LineString2D::new(coords)
        })
        .collect();

    Polygon2D::new(LineString2D::new(exterior_uvs), interior_uvs)
}
