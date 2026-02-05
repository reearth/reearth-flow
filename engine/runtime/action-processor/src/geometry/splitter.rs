use std::collections::HashMap;

use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::geometry::{Geometry2D, Geometry3D};
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{
    Attribute, AttributeValue, CityGmlGeometry, Feature, Geometry, GeometryValue, GmlGeometry,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

/// Split level for CityGML geometry
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SplitLevel {
    /// Split by GmlGeometry elements (e.g., RoofSurface, WallSurface)
    #[default]
    Element,
    /// Split down to individual polygons within each element
    Polygon,
}

/// Parameters for GeometrySplitter
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometrySplitterParam {
    /// Split level for CityGML geometry.
    /// - "element": Split by surface elements (RoofSurface, WallSurface, etc.) - default
    /// - "polygon": Split down to individual polygons within each element
    #[serde(default)]
    pub split_level: SplitLevel,
}

#[derive(Debug, Clone, Default)]
pub struct GeometrySplitterFactory;

impl ProcessorFactory for GeometrySplitterFactory {
    fn name(&self) -> &str {
        "GeometrySplitter"
    }

    fn description(&self) -> &str {
        "Split Multi-Geometries into Individual Features"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometrySplitterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
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
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: GeometrySplitterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometrySplitterFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometrySplitterFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            GeometrySplitterParam::default()
        };

        let process = GeometrySplitter {
            split_level: param.split_level,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct GeometrySplitter {
    split_level: SplitLevel,
}

impl Processor for GeometrySplitter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            return Ok(());
        }
        match &geometry.value {
            GeometryValue::CityGmlGeometry(city_gml_geometry) => {
                self.process_citygml_geometry(city_gml_geometry, feature, geometry, &ctx, fw)?;
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                self.process_flow_geometry_2d(geometry, &ctx, fw)?;
            }
            GeometryValue::FlowGeometry3D(geometry) => {
                self.process_flow_geometry_3d(geometry, &ctx, fw)?;
            }
            GeometryValue::None => {
                // Pass through empty geometry
                fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "GeometrySplitter"
    }
}

impl GeometrySplitter {
    fn process_flow_geometry_2d(
        &self,
        geometry: &Geometry2D,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match geometry {
            Geometry2D::MultiPolygon(multi_polygon) => {
                // Split MultiPolygon into individual Polygon features
                let polygons: Vec<_> = multi_polygon.iter().cloned().collect();
                // Multiple polygons - split into separate features
                for (index, polygon) in polygons.into_iter().enumerate() {
                    let mut new_feature = ctx.feature.clone();
                    new_feature.insert(
                        Attribute::new("_split_index"),
                        AttributeValue::Number((index + 1).into()),
                    );
                    new_feature.geometry_mut().value =
                        GeometryValue::FlowGeometry2D(Geometry2D::Polygon(polygon));
                    fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
                }
            }
            Geometry2D::MultiLineString(multi_line_string) => {
                // Split MultiLineString into individual LineString features
                let line_strings: Vec<_> = multi_line_string.iter().cloned().collect();
                // Multiple line strings - split into separate features
                for (index, line_string) in line_strings.into_iter().enumerate() {
                    let mut new_feature = ctx.feature.clone();
                    new_feature.insert(
                        Attribute::new("_split_index"),
                        AttributeValue::Number((index + 1).into()),
                    );
                    new_feature.geometry_mut().value =
                        GeometryValue::FlowGeometry2D(Geometry2D::LineString(line_string));
                    fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
                }
            }
            _ => {
                // For non-multi geometries, pass through unchanged
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), DEFAULT_PORT.clone()));
            }
        }
        Ok(())
    }

    fn process_flow_geometry_3d(
        &self,
        geometry: &Geometry3D,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match geometry {
            Geometry3D::MultiPolygon(multi_polygon) => {
                // Split MultiPolygon into individual Polygon features
                let polygons: Vec<_> = multi_polygon.iter().cloned().collect();
                // Multiple polygons - split into separate features
                for (index, polygon) in polygons.into_iter().enumerate() {
                    let mut new_feature = ctx.feature.clone();
                    new_feature.insert(
                        Attribute::new("_split_index"),
                        AttributeValue::Number((index + 1).into()),
                    );
                    new_feature.geometry_mut().value =
                        GeometryValue::FlowGeometry3D(Geometry3D::Polygon(polygon));
                    fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
                }
            }
            Geometry3D::MultiLineString(multi_line_string) => {
                // Split MultiLineString into individual LineString features
                let line_strings: Vec<_> = multi_line_string.iter().cloned().collect();
                // Multiple line strings - split into separate features
                for (index, line_string) in line_strings.into_iter().enumerate() {
                    let mut new_feature = ctx.feature.clone();
                    new_feature.insert(
                        Attribute::new("_split_index"),
                        AttributeValue::Number((index + 1).into()),
                    );
                    new_feature.geometry_mut().value =
                        GeometryValue::FlowGeometry3D(Geometry3D::LineString(line_string));
                    fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
                }
            }
            _ => {
                // For non-multi geometries, pass through unchanged
                fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), DEFAULT_PORT.clone()));
            }
        }
        Ok(())
    }

    fn process_citygml_geometry(
        &self,
        city_gml_geometry: &CityGmlGeometry,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // First, split by gml_geometries (element level)
        let element_features: Vec<CityGmlGeometry> = if city_gml_geometry.gml_geometries.len() < 2 {
            vec![city_gml_geometry.clone()]
        } else {
            city_gml_geometry.split_feature()
        };

        for split_feature in element_features {
            let Some(geometry_feature) = split_feature.gml_geometries.first().cloned() else {
                continue;
            };

            // If polygon-level splitting is requested, split further
            if self.split_level == SplitLevel::Polygon {
                self.emit_polygon_level_features(
                    &split_feature,
                    &geometry_feature,
                    feature,
                    geometry,
                    ctx,
                    fw,
                )?;
            } else {
                // Element-level splitting (default)
                self.emit_element_level_feature(
                    split_feature,
                    &geometry_feature,
                    feature,
                    geometry,
                    ctx,
                    fw,
                )?;
            }
        }

        Ok(())
    }

    fn emit_element_level_feature(
        &self,
        split_feature: CityGmlGeometry,
        geometry_feature: &GmlGeometry,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut new_geometry = geometry.clone();
        let mut attributes = (*feature.attributes).clone();

        Self::set_gml_attributes(&mut attributes, geometry_feature, feature);

        new_geometry.value = GeometryValue::CityGmlGeometry(split_feature);
        fw.send(ctx.new_with_feature_and_port(
            Feature::new_with_attributes_and_geometry(
                attributes,
                new_geometry,
                feature.metadata.clone(),
            ),
            DEFAULT_PORT.clone(),
        ));

        Ok(())
    }

    fn emit_polygon_level_features(
        &self,
        split_feature: &CityGmlGeometry,
        geometry_feature: &GmlGeometry,
        feature: &Feature,
        geometry: &Geometry,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Collect all polygons from gml_geometries and composite_surfaces
        let polygons = Self::collect_all_polygons(geometry_feature);

        if polygons.is_empty() {
            // No polygons, emit the element as-is
            return self.emit_element_level_feature(
                split_feature.clone(),
                geometry_feature,
                feature,
                geometry,
                ctx,
                fw,
            );
        }

        // Emit one feature per polygon
        for (index, polygon) in polygons.into_iter().enumerate() {
            let mut new_geometry = geometry.clone();
            let mut attributes = (*feature.attributes).clone();

            Self::set_gml_attributes(&mut attributes, geometry_feature, feature);

            // Add polygon index
            attributes.insert(
                Attribute::new("_polygon_index"),
                AttributeValue::Number((index + 1).into()),
            );

            // Create placeholder UV polygon matching the structure of the polygon
            let uv_polygon = Self::create_placeholder_uv_polygon(&polygon);

            // Debug: verify vertex counts match at creation time
            let poly_exterior_len = polygon.exterior().0.len();
            let uv_exterior_len = uv_polygon.exterior().0.len();
            if poly_exterior_len != uv_exterior_len {
                panic!(
                    "Splitter: Vertex count mismatch at creation!\n\
                    polygon exterior: {} vertices\n\
                    uv_polygon exterior: {} vertices",
                    poly_exterior_len, uv_exterior_len
                );
            }

            // Create a CityGmlGeometry with a single polygon
            let single_gml = GmlGeometry {
                id: geometry_feature.id.clone(),
                ty: geometry_feature.ty,
                gml_trait: geometry_feature.gml_trait.clone(),
                lod: geometry_feature.lod,
                pos: 0,
                len: 1,
                points: vec![],
                polygons: vec![polygon],
                line_strings: vec![],
                feature_id: geometry_feature.feature_id.clone(),
                feature_type: geometry_feature.feature_type.clone(),
                composite_surfaces: vec![],
            };

            let single_citygml = CityGmlGeometry {
                gml_geometries: vec![single_gml],
                materials: split_feature.materials.clone(),
                textures: split_feature.textures.clone(),
                polygon_materials: vec![None],
                polygon_textures: vec![None],
                polygon_uvs: MultiPolygon2D::new(vec![uv_polygon]),
            };

            // Debug assertion: verify the invariant that polygon count matches array lengths
            debug_assert_eq!(
                single_citygml.gml_geometries[0].polygons.len(),
                single_citygml.gml_geometries[0].len as usize,
                "Splitter: polygon count mismatch with len"
            );
            debug_assert_eq!(
                single_citygml.polygon_materials.len(),
                1,
                "Splitter: polygon_materials should have 1 element"
            );
            debug_assert_eq!(
                single_citygml.polygon_uvs.iter().count(),
                1,
                "Splitter: polygon_uvs should have 1 element"
            );

            new_geometry.value = GeometryValue::CityGmlGeometry(single_citygml);
            fw.send(ctx.new_with_feature_and_port(
                Feature::new_with_attributes_and_geometry(
                    attributes,
                    new_geometry,
                    feature.metadata.clone(),
                ),
                DEFAULT_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn collect_all_polygons(
        gml: &GmlGeometry,
    ) -> Vec<reearth_flow_geometry::types::polygon::Polygon3D<f64>> {
        let mut polygons = gml.polygons.clone();

        // Recursively collect from composite_surfaces
        for cs in &gml.composite_surfaces {
            polygons.extend(Self::collect_all_polygons(cs));
        }

        polygons
    }

    /// Create a placeholder UV polygon that matches the ring structure of a 3D polygon.
    /// Each vertex gets placeholder UV coordinates (0.0, 0.0).
    fn create_placeholder_uv_polygon(poly3d: &Polygon3D<f64>) -> Polygon2D<f64> {
        // Create exterior ring with placeholder UVs
        let exterior_uv_coords: Vec<Coordinate2D<f64>> = poly3d
            .exterior()
            .0
            .iter()
            .map(|_| Coordinate2D::new_(0.0, 0.0))
            .collect();
        let exterior_uv = LineString2D::new(exterior_uv_coords);

        // Create interior rings with placeholder UVs
        let interior_uvs: Vec<LineString2D<f64>> = poly3d
            .interiors()
            .iter()
            .map(|interior| {
                let coords: Vec<Coordinate2D<f64>> = interior
                    .0
                    .iter()
                    .map(|_| Coordinate2D::new_(0.0, 0.0))
                    .collect();
                LineString2D::new(coords)
            })
            .collect();

        Polygon2D::new(exterior_uv, interior_uvs)
    }

    fn set_gml_attributes(
        attributes: &mut indexmap::IndexMap<Attribute, AttributeValue>,
        geometry_feature: &GmlGeometry,
        feature: &Feature,
    ) {
        attributes.insert(
            Attribute::new("geometryName"),
            AttributeValue::String(
                geometry_feature
                    .gml_trait
                    .as_ref()
                    .map(|trait_| trait_.gml_geometry_type.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
            ),
        );
        attributes.insert(
            Attribute::new("gmlPropertyName"),
            AttributeValue::String(
                geometry_feature
                    .gml_trait
                    .as_ref()
                    .map(|trait_| trait_.property.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
            ),
        );

        if let Some(lod) = geometry_feature.lod {
            attributes.insert(
                Attribute::new("lod"),
                AttributeValue::String(lod.to_string()),
            );
        }

        attributes.insert(
            Attribute::new("featureId"),
            geometry_feature
                .feature_id
                .as_ref()
                .map(|id| AttributeValue::String(id.to_string()))
                .unwrap_or(AttributeValue::Null),
        );

        if !attributes.contains_key(&Attribute::new("featureType")) {
            attributes.insert(
                Attribute::new("featureType"),
                geometry_feature
                    .feature_type
                    .as_ref()
                    .map(|ft| AttributeValue::String(ft.to_string()))
                    .unwrap_or(AttributeValue::Null),
            );
        }

        // Set parent ID
        let parent_id = if let Some(feature_id) = &geometry_feature.feature_id {
            if let Some(AttributeValue::String(gml_id)) = feature.get("gmlId") {
                if gml_id == feature_id {
                    feature.get("gmlRootId").and_then(|v| {
                        if let AttributeValue::String(s) = v {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    })
                } else {
                    Some(gml_id.to_string())
                }
            } else {
                feature.get("gmlRootId").and_then(|v| {
                    if let AttributeValue::String(s) = v {
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
            }
        } else {
            None
        };

        attributes.insert(
            Attribute::new("featureParentId"),
            parent_id
                .map(AttributeValue::String)
                .unwrap_or(AttributeValue::Null),
        );
    }
}
