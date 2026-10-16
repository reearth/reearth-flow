//! The Japanese standard regional mesh cells (JIS X 0410) features touch.

use std::collections::HashMap;
#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

#[cfg(feature = "new-geometry")]
use std::collections::{BTreeMap, BTreeSet};

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::algorithm::bool_ops::BooleanOps;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::algorithm::bounding_rect::BoundingRect;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::algorithm::contains::Contains;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::coordinate::Coordinate2D;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::geometry::{Geometry, Geometry2D};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::line_string::LineString2D;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::polygon::Polygon2D;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::jpmesh::JPMeshCode;
use reearth_flow_types::jpmesh::JPMeshType;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::GeometryValue;
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "new-geometry")]
use reearth_flow_diagnostics::DiagnosticDraft;
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{
    collection::Collection2D,
    coordinate::{CoordinateFrame, EpsgCode},
    ops::common_frame,
    point::Point2D,
    Euclidean2DGeometry, Geometry,
};
#[cfg(feature = "new-geometry")]
use reearth_flow_types::jpmesh::{
    overlap::{cell_centre, cell_face, touched_cells},
    JPMeshCode,
};
#[cfg(feature = "new-geometry")]
use reearth_flow_types::{Attributes, Feature};

use super::errors::GeometryProcessorError;
use super::jp_mesh_code_extractor::MeshLevel;
#[cfg(feature = "new-geometry")]
use super::jp_mesh_code_extractor::{reaches_outside_mesh_range, Skip};

#[derive(Debug, Clone, Default)]
pub struct JPStandardGridAccumulatorFactory;

impl ProcessorFactory for JPStandardGridAccumulatorFactory {
    fn name(&self) -> &str {
        "JP Standard Grid Accumulator"
    }

    fn description(&self) -> &str {
        "Outputs the Japanese standard regional mesh cells (JIS X 0410) that each group of features touches, one feature per cell, in place of the features themselves. A large feature at a fine mesh level touches many cells, so it can produce very many features."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(JPStandardGridAccumulatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: JPStandardGridAccumulatorParam = match with {
            Some(with) => serde_json::from_value(Value::Object(with.into_iter().collect()))
                .map_err(|e| {
                    GeometryProcessorError::JPStandardGridAccumulatorFactory(format!(
                        "Failed to deserialize parameters: {e}"
                    ))
                })?,
            None => JPStandardGridAccumulatorParam::default(),
        };
        Ok(Box::new(JPStandardGridAccumulator {
            params,
            #[cfg(feature = "new-geometry")]
            groups: BTreeMap::new(),
        }))
    }
}

/// # JP Standard Grid Accumulator Parameters
///
/// Which mesh cells are collected, how each is output, and how features are grouped.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct JPStandardGridAccumulatorParam {
    /// # Mesh Level
    /// The size of the mesh cells collected.
    #[serde(default)]
    mesh_level: MeshLevel,
    /// # Output Geometry
    /// The geometry each output cell carries, in the coordinate system of its group's features.
    #[serde(default)]
    output: CellOutput,
    /// # Output Attribute
    /// Attribute that receives each cell's mesh code, written as a string of digits.
    #[serde(default = "default_output_attribute")]
    output_attribute: Attribute,
    /// # Group By Attributes
    /// Attributes whose values group features together. Each group collects its own cells,
    /// and its output carries these attributes; a missing attribute counts as null.
    group_by: Option<Vec<Attribute>>,
}

impl Default for JPStandardGridAccumulatorParam {
    fn default() -> Self {
        Self {
            mesh_level: MeshLevel::default(),
            output: CellOutput::default(),
            output_attribute: default_output_attribute(),
            group_by: None,
        }
    }
}

fn default_output_attribute() -> Attribute {
    Attribute::new("meshCode".to_string())
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
enum CellOutput {
    /// # Polygon
    /// The cell's outline, its edges following lines of latitude and longitude.
    #[default]
    Polygon,
    /// # Point
    /// The cell's center.
    Point,
}

/// The cells one group has touched, in the coordinate system of the group's
/// first accepted feature.
#[cfg(feature = "new-geometry")]
#[derive(Debug, Clone)]
struct Group {
    epsg: EpsgCode,
    codes: BTreeSet<u64>,
}

/// One entry per `groupBy` attribute, `None` where the feature lacks it or it
/// is null.
#[cfg(feature = "new-geometry")]
type GroupKey = Vec<Option<AttributeValue>>;

#[derive(Debug, Clone)]
pub struct JPStandardGridAccumulator {
    #[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
    params: JPStandardGridAccumulatorParam,
    #[cfg(feature = "new-geometry")]
    groups: BTreeMap<GroupKey, Group>,
}

impl Processor for JPStandardGridAccumulator {
    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geometry) => {
                let bounds = if let Some(bounds) = geometry.bounding_rect() {
                    bounds
                } else {
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                };

                let meshes_1km = JPMeshCode::from_inside_bounds(bounds, JPMeshType::Mesh1km);

                for meshcode in meshes_1km {
                    let binded_geometry = if let Some(binded_geometry) =
                        self.bind_geometry_into_mesh_2d(geometry, meshcode.clone())
                    {
                        binded_geometry
                    } else {
                        continue;
                    };

                    let mut attributes = (*feature.attributes).clone();
                    attributes.insert(
                        Attribute::new("meshcode"),
                        AttributeValue::String(meshcode.to_number().to_string()),
                    );

                    let mut new_feature = feature.clone();
                    new_feature.geometry_mut().value =
                        GeometryValue::FlowGeometry2D(binded_geometry);
                    new_feature.attributes = Arc::new(attributes);

                    fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
                }
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
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

    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let key: GroupKey = self
            .params
            .group_by
            .iter()
            .flatten()
            // A null value and a missing attribute are the same key.
            .map(|name| match feature.attributes.get(name) {
                None | Some(AttributeValue::Null) => None,
                Some(value) => Some(value.clone()),
            })
            .collect();
        let group_epsg = self.groups.get(&key).map(|g| g.epsg);
        match self.touched(&feature.geometry, group_epsg) {
            Ok((epsg, cells)) => {
                let group = self.groups.entry(key).or_insert_with(|| Group {
                    epsg,
                    codes: BTreeSet::new(),
                });
                group.codes.extend(cells.iter().map(JPMeshCode::to_number));
            }
            Err(skip) => {
                // No `with_message`: a real run's diagnostics aggregator keeps
                // only the code, so the registry text is what users see.
                ctx.warn(DiagnosticDraft::new(skip.code()));
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mesh_type: JPMeshType = self.params.mesh_level.into();
        let group_by = self.params.group_by.clone().unwrap_or_default();
        for (key, group) in std::mem::take(&mut self.groups) {
            let mut attributes = Attributes::new();
            for (name, value) in group_by.iter().zip(key) {
                if let Some(value) = value {
                    attributes.insert(name.clone(), value);
                }
            }
            for number in group.codes {
                let code = JPMeshCode::from_number(number, mesh_type);
                let geometry = match self.params.output {
                    CellOutput::Polygon => Euclidean2DGeometry::Polygon(Box::new(
                        cell_face(&code, group.epsg).map_err(|e| {
                            GeometryProcessorError::JPStandardGridAccumulator(e.to_string())
                        })?,
                    )),
                    CellOutput::Point => Euclidean2DGeometry::Point(Point2D::new(
                        CoordinateFrame::Crs(group.epsg),
                        cell_centre(&code, group.epsg).map_err(|e| {
                            GeometryProcessorError::JPStandardGridAccumulator(e.to_string())
                        })?,
                    )),
                };
                let mut attributes = attributes.clone();
                attributes.insert(
                    self.params.output_attribute.clone(),
                    AttributeValue::String(number.to_string()),
                );
                let cell = Feature::new_with_attributes_and_geometry(
                    attributes,
                    Geometry::Euclidean2D(geometry),
                );
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    cell,
                    FEATURES_PORT.clone(),
                ));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "JP Standard Grid Accumulator"
    }
}

#[cfg(feature = "new-geometry")]
impl JPStandardGridAccumulator {
    /// The coordinate system of the feature and the cells it touches, or why
    /// it has none. `group_epsg` is the coordinate system of the feature's
    /// group, when the group already has a feature.
    fn touched(
        &self,
        geometry: &Geometry,
        group_epsg: Option<EpsgCode>,
    ) -> Result<(EpsgCode, Vec<JPMeshCode>), Skip> {
        if matches!(geometry, Geometry::None) {
            return Err(Skip::Unsupported);
        }
        let frame = common_frame(geometry).map_err(|_| Skip::MixedFrames)?;
        let mut members = Vec::new();
        flat_members(geometry, &mut members)?;
        let g = match members.len() {
            1 => members.remove(0),
            _ => Euclidean2DGeometry::Collection(Collection2D::new(members)),
        };
        let epsg = match frame {
            Some(CoordinateFrame::Crs(epsg)) => epsg,
            Some(_) => return Err(Skip::NoCrs),
            None => return Err(Skip::Unsupported),
        };
        if group_epsg.is_some_and(|e| e != epsg) {
            return Err(Skip::MixedFrames);
        }
        let mesh_type: JPMeshType = self.params.mesh_level.into();
        let cells = touched_cells(&g, epsg, mesh_type).map_err(|_| Skip::Failed)?;
        if !cells.is_empty() {
            Ok((epsg, cells))
        } else if reaches_outside_mesh_range(&g, epsg, mesh_type)? {
            Err(Skip::OutOfRange)
        } else {
            Err(Skip::NothingToMeasure)
        }
    }
}

/// The 2D members of a geometry, nested collections included. Cells are
/// matched on the plane, so any 3D part makes the whole feature 3D; a member
/// with no geometry adds nothing.
#[cfg(feature = "new-geometry")]
fn flat_members(geometry: &Geometry, out: &mut Vec<Euclidean2DGeometry>) -> Result<(), Skip> {
    match geometry {
        Geometry::Euclidean2D(g) => out.push(g.clone()),
        Geometry::GeometryCollection(c) => {
            for m in c.members() {
                flat_members(m, out)?;
            }
        }
        Geometry::Euclidean3D(_) => return Err(Skip::ThreeDimensional),
        Geometry::None => {}
    }
    Ok(())
}

#[cfg(not(feature = "new-geometry"))]
impl JPStandardGridAccumulator {
    fn bind_geometry_into_mesh_2d(
        &self,
        geometry: &Geometry2D,
        mesh: JPMeshCode,
    ) -> Option<Geometry2D> {
        let bounds = mesh.bounds();

        let bounds_polygon = Polygon2D::new(
            LineString2D::new(vec![
                Coordinate2D::new_(bounds.min().x, bounds.min().y),
                Coordinate2D::new_(bounds.max().x, bounds.min().y),
                Coordinate2D::new_(bounds.max().x, bounds.max().y),
                Coordinate2D::new_(bounds.min().x, bounds.max().y),
                Coordinate2D::new_(bounds.min().x, bounds.min().y),
            ]),
            vec![],
        );

        let bind_geometry = match geometry {
            Geometry::Point(point) => {
                let coords = point.0;
                if bounds.contains(&coords) {
                    geometry.clone()
                } else {
                    return None;
                }
            }
            Geometry::LineString(line_string) => {
                let multi_line_string =
                    reearth_flow_geometry::types::multi_line_string::MultiLineString2D::new(vec![
                        line_string.clone(),
                    ]);

                let clipped = <Polygon2D<f64> as BooleanOps>::clip(
                    &bounds_polygon,
                    &multi_line_string,
                    false,
                );

                if clipped.0.is_empty() {
                    return None;
                } else if clipped.0.len() == 1 {
                    Geometry::LineString(clipped.0[0].clone())
                } else {
                    Geometry::MultiLineString(clipped)
                }
            }
            Geometry::MultiLineString(multi_line_string) => {
                let clipped =
                    <Polygon2D<f64> as BooleanOps>::clip(&bounds_polygon, multi_line_string, false);

                if clipped.0.is_empty() {
                    return None;
                } else if clipped.0.len() == 1 {
                    Geometry::LineString(clipped.0[0].clone())
                } else {
                    Geometry::MultiLineString(clipped)
                }
            }
            Geometry::Polygon(polygon) => {
                let intersection = polygon.intersection(&bounds_polygon);

                if intersection.0.is_empty() {
                    return None;
                } else if intersection.0.len() == 1 {
                    Geometry::Polygon(intersection.0[0].clone())
                } else {
                    Geometry::MultiPolygon(intersection)
                }
            }
            Geometry::MultiPolygon(multi_polygon) => {
                let intersection =
                    multi_polygon.intersection(&MultiPolygon2D::new(vec![bounds_polygon]));
                if intersection.0.is_empty() {
                    return None;
                } else if intersection.0.len() == 1 {
                    Geometry::Polygon(intersection.0[0].clone())
                } else {
                    Geometry::MultiPolygon(intersection)
                }
            }
            _ => {
                return None;
            }
        };

        Some(bind_geometry)
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use std::sync::Arc;

    use reearth_flow_common::uri::Uri;
    use reearth_flow_diagnostics::ErrorCode;
    use reearth_flow_geometry::ops::reproject::{transform_coords_3d, ReprojectionCache};
    use reearth_flow_geometry::polygon::{Polygon2D, Polygon3D};
    use reearth_flow_geometry::Euclidean3DGeometry;
    use reearth_flow_runtime::event::Event;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_runtime::kvs;
    use reearth_flow_storage::resolve::StorageResolver;
    use reearth_flow_types::jpmesh::overlap::containing;
    use serde_json::json;

    const GEO: EpsgCode = EpsgCode::new(6668);
    const JGD9: EpsgCode = EpsgCode::new(6677);
    const JGD7: EpsgCode = EpsgCode::new(6675);

    /// `(longitude, latitude)` pairs in `to`'s declared axis order.
    fn project(lng_lat: &[(f64, f64)], to: EpsgCode) -> Vec<[f64; 2]> {
        let mut c: Vec<[f64; 3]> = lng_lat.iter().map(|&(lng, lat)| [lat, lng, 0.0]).collect();
        if to != GEO {
            transform_coords_3d(&mut ReprojectionCache::new(), GEO, to, &mut c).unwrap();
        }
        c.iter().map(|p| [p[0], p[1]]).collect()
    }

    /// A square around `(fx, fy)`, in fractions of 1 km cell 53394611, with
    /// sides `size` cells long.
    fn square(epsg: EpsgCode, fx: f64, fy: f64, size: f64) -> Geometry {
        let sw = JPMeshCode::from_number(53394611, JPMeshType::Mesh1km).bounds();
        let (dx, dy) = (45.0 / 3600.0, 30.0 / 3600.0);
        let at = |x: f64, y: f64| (sw.min().x + x * dx, sw.min().y + y * dy);
        let h = size / 2.0;
        let ring = [
            at(fx - h, fy - h),
            at(fx + h, fy - h),
            at(fx + h, fy + h),
            at(fx - h, fy + h),
            at(fx - h, fy - h),
        ];
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Crs(epsg),
                project(&ring, epsg),
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )))
    }

    fn feature(geometry: Geometry, attrs: &[(&str, &str)]) -> Feature {
        let attributes: Attributes = attrs
            .iter()
            .map(|(k, v)| {
                (
                    Attribute::new(k.to_string()),
                    AttributeValue::String(v.to_string()),
                )
            })
            .collect();
        Feature::new_with_attributes_and_geometry(attributes, geometry)
    }

    fn attr(f: &Feature, name: &str) -> Option<String> {
        match f.attributes.get(&Attribute::new(name.to_string())) {
            Some(AttributeValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    fn codes(out: &[Feature]) -> Vec<String> {
        out.iter().map(|f| attr(f, "meshCode").unwrap()).collect()
    }

    /// Run every feature through one processor, then finish it, and return
    /// the features it sent, the port each went to, and the code of every
    /// diagnostic raised.
    fn accumulate(
        with: Value,
        features: Vec<Feature>,
    ) -> (Vec<Feature>, Vec<Port>, Vec<ErrorCode>) {
        let hub = EventHub::new(64);
        let mut rx = hub.receiver.resubscribe();
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let with: HashMap<String, Value> = serde_json::from_value(with).unwrap();
        let mut processor = JPStandardGridAccumulatorFactory
            .build(
                NodeContext::default(),
                EventHub::new(1),
                "JP Standard Grid Accumulator".to_string(),
                Some(with),
            )
            .unwrap();

        for f in features {
            let ctx = ExecutorContext::new(
                f,
                FEATURES_PORT.clone(),
                Arc::new(serde_json::Map::new()),
                Arc::new(StorageResolver::new()),
                Arc::new(kvs::create_kv_store()),
                hub.clone(),
                Uri::for_test("file:///"),
            );
            processor.process(ctx, &fw).unwrap();
        }
        processor.finish(NodeContext::default(), &fw).unwrap();

        let ProcessorChannelForwarder::Noop(noop) = &fw else {
            unreachable!("built as a noop forwarder");
        };
        let features = noop.send_features.lock().unwrap().clone();
        let ports = noop.send_ports.lock().unwrap().clone();
        let mut warnings = Vec::new();
        while let Ok(event) = rx.try_recv() {
            if let Event::Diagnostic(diagnostic) = event {
                warnings.push(diagnostic.code);
            }
        }
        (features, ports, warnings)
    }

    #[test]
    fn overlapping_features_share_their_cells() {
        let (out, ports, warnings) = accumulate(
            json!({}),
            vec![
                feature(square(JGD9, 0.5, 0.5, 0.2), &[]),
                feature(square(JGD9, 0.55, 0.5, 0.2), &[]),
            ],
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(ports, vec![FEATURES_PORT.clone()]);
        assert_eq!(codes(&out), vec!["53394611"]);
    }

    #[test]
    fn a_polygon_across_four_cells_outputs_four_polygons() {
        let (out, ports, warnings) =
            accumulate(json!({}), vec![feature(square(JGD9, 1.0, 1.0, 0.4), &[])]);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(ports, vec![FEATURES_PORT.clone(); 4]);
        assert_eq!(
            codes(&out),
            vec!["53394611", "53394612", "53394621", "53394622"]
        );
        for f in &out {
            let Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) = &*f.geometry else {
                panic!("expected a polygon, got {:?}", f.geometry);
            };
            assert_eq!(p.frame(), &CoordinateFrame::Crs(JGD9));
        }
    }

    #[test]
    fn point_output_puts_points_at_cell_centers() {
        let (out, _, warnings) = accumulate(
            json!({ "output": "point" }),
            vec![feature(square(JGD9, 1.0, 1.0, 0.4), &[])],
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(out.len(), 4);
        for f in &out {
            let Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) = &*f.geometry else {
                panic!("expected a point, got {:?}", f.geometry);
            };
            assert_eq!(p.frame(), &CoordinateFrame::Crs(JGD9));
            let [x, y] = p.position();
            let code = containing([x, y, 0.0], JGD9, JPMeshType::Mesh1km)
                .unwrap()
                .unwrap();
            assert_eq!(Some(code.to_number().to_string()), attr(f, "meshCode"));
        }
    }

    #[test]
    fn groups_are_kept_apart() {
        let (out, _, warnings) = accumulate(
            json!({ "groupBy": ["g"] }),
            vec![
                feature(square(JGD9, 0.5, 0.5, 0.2), &[("g", "a")]),
                feature(square(JGD9, 0.5, 0.5, 0.2), &[("g", "b")]),
            ],
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(codes(&out), vec!["53394611", "53394611"]);
        let groups: Vec<_> = out.iter().map(|f| attr(f, "g")).collect();
        assert_eq!(groups, vec![Some("a".into()), Some("b".into())]);
    }

    #[test]
    fn missing_group_attributes_do_not_collide() {
        let (out, _, warnings) = accumulate(
            json!({ "groupBy": ["a", "b"] }),
            vec![
                feature(square(JGD9, 0.5, 0.5, 0.2), &[("a", "x")]),
                feature(square(JGD9, 0.5, 0.5, 0.2), &[("b", "x")]),
            ],
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(out.len(), 2);
        let present: Vec<_> = out.iter().map(|f| (attr(f, "a"), attr(f, "b"))).collect();
        assert!(present.contains(&(Some("x".into()), None)), "{present:?}");
        assert!(present.contains(&(None, Some("x".into()))), "{present:?}");
    }

    #[test]
    fn a_feature_in_another_crs_is_rejected() {
        let (out, ports, warnings) = accumulate(
            json!({}),
            vec![
                feature(square(JGD9, 0.5, 0.5, 0.2), &[("n", "first")]),
                feature(square(JGD7, 0.5, 0.5, 0.2), &[("n", "second")]),
            ],
        );
        assert_eq!(warnings, vec![ErrorCode::MeshMixedFrames]);
        assert_eq!(ports, vec![REJECTED_PORT.clone(), FEATURES_PORT.clone()]);
        assert_eq!(attr(&out[0], "n").as_deref(), Some("second"));
        assert_eq!(attr(&out[1], "meshCode").as_deref(), Some("53394611"));
    }

    #[test]
    fn three_d_no_crs_and_empty_are_rejected() {
        let ring: Vec<[f64; 3]> = project(
            &[
                (139.0, 35.0),
                (139.001, 35.0),
                (139.0, 35.001),
                (139.0, 35.0),
            ],
            JGD9,
        )
        .into_iter()
        .map(|[a, b]| [a, b, 10.0])
        .collect();
        let solid = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Crs(JGD9),
                ring,
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )));
        let bare = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]],
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )));
        let (out, ports, warnings) = accumulate(
            json!({}),
            vec![
                feature(solid, &[]),
                feature(bare, &[]),
                feature(Geometry::None, &[]),
            ],
        );
        assert_eq!(
            warnings,
            vec![
                ErrorCode::MeshThreeDimensional,
                ErrorCode::MeshNoCrs,
                ErrorCode::MeshUnsupportedGeometry
            ]
        );
        assert_eq!(ports, vec![REJECTED_PORT.clone(); 3]);
        assert!(out.iter().all(|f| attr(f, "meshCode").is_none()));
    }

    #[test]
    fn a_feature_outside_the_mesh_range_is_out_of_range() {
        // A 10 m square near London in EPSG:3857 (x = easting, y = northing).
        let (x, y) = (-13_358.0, 6_710_000.0);
        let london = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Crs(EpsgCode::new(3857)),
                vec![
                    [x, y],
                    [x + 10.0, y],
                    [x + 10.0, y + 10.0],
                    [x, y + 10.0],
                    [x, y],
                ],
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )));
        let (_, ports, warnings) = accumulate(json!({}), vec![feature(london, &[])]);
        assert_eq!(warnings, vec![ErrorCode::MeshOutOfRange]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
    }

    #[test]
    fn a_feature_touching_no_cell_has_nothing_to_measure() {
        // Every vertex at one spot: no area, no length, no point.
        let at = project(&[(139.70, 35.68)], JGD9)[0];
        let collapsed = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Crs(JGD9),
                vec![at, at, at, at],
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )));
        let (_, ports, warnings) = accumulate(json!({}), vec![feature(collapsed, &[])]);
        assert_eq!(warnings, vec![ErrorCode::MeshNothingToMeasure]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
    }

    #[test]
    fn a_null_group_attribute_groups_with_a_missing_one() {
        let mut with_null = feature(square(JGD9, 0.5, 0.5, 0.2), &[]);
        let mut attributes = (*with_null.attributes).clone();
        attributes.insert(Attribute::new("g".to_string()), AttributeValue::Null);
        with_null.attributes = Arc::new(attributes);
        let (out, _, warnings) = accumulate(
            json!({ "groupBy": ["g"] }),
            vec![feature(square(JGD9, 0.5, 0.5, 0.2), &[]), with_null],
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(codes(&out), vec!["53394611"]);
    }

    fn solid_in(epsg: EpsgCode) -> Geometry {
        let ring: Vec<[f64; 3]> = project(
            &[
                (139.0, 35.0),
                (139.001, 35.0),
                (139.0, 35.001),
                (139.0, 35.0),
            ],
            epsg,
        )
        .into_iter()
        .map(|[a, b]| [a, b, 10.0])
        .collect();
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Crs(epsg),
                ring,
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )))
    }

    #[test]
    fn a_rejected_first_feature_does_not_set_the_group_crs() {
        let (out, ports, warnings) = accumulate(
            json!({}),
            vec![
                feature(solid_in(JGD7), &[]),
                feature(square(JGD9, 0.5, 0.5, 0.2), &[]),
            ],
        );
        assert_eq!(warnings, vec![ErrorCode::MeshThreeDimensional]);
        assert_eq!(ports, vec![REJECTED_PORT.clone(), FEATURES_PORT.clone()]);
        assert_eq!(attr(&out[1], "meshCode").as_deref(), Some("53394611"));
    }

    #[test]
    fn a_collection_with_a_3d_part_is_three_dimensional() {
        let mixed = Geometry::GeometryCollection(reearth_flow_geometry::GeometryCollection::new([
            square(JGD9, 0.5, 0.5, 0.2),
            solid_in(JGD9),
        ]));
        let (_, ports, warnings) = accumulate(json!({}), vec![feature(mixed, &[])]);
        assert_eq!(warnings, vec![ErrorCode::MeshThreeDimensional]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
    }

    #[test]
    fn mesh_level_is_honoured() {
        let (out, _, warnings) = accumulate(
            json!({ "meshLevel": "mesh10km", "outputAttribute": "m" }),
            vec![feature(square(JGD9, 0.5, 0.5, 0.2), &[])],
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(out.len(), 1);
        assert_eq!(attr(&out[0], "m").as_deref(), Some("533946"));
    }
}
