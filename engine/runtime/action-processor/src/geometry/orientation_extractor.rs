use std::collections::HashMap;

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::algorithm::winding_order::Winding;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::geometry::Geometry3D;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::{algorithm::winding_order::WindingOrder, types::geometry::Geometry2D};
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{
    ops::{ring_winding_2d, RingWinding},
    predicates::view::{flatten_2d, Leaf2D},
    Geometry,
};
#[cfg(feature = "new-geometry")]
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::GeometryValue;
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

type WindingOrderResult = &'static str;

const NO_ORIENTATION: WindingOrderResult = "no_orientation";
#[cfg(not(feature = "new-geometry"))]
const INVALID_ORIENTATION: WindingOrderResult = "invalid_orientation";
const CLOCKWISE_ORIENTATION: WindingOrderResult = "clockwise";
const COUNTER_CLOCKWISE_ORIENTATION: WindingOrderResult = "counter_clockwise";

#[derive(Debug, Clone, Default)]
pub struct OrientationExtractorFactory;

impl ProcessorFactory for OrientationExtractorFactory {
    fn name(&self) -> &str {
        "Orientation Extractor"
    }

    #[cfg(not(feature = "new-geometry"))]
    fn description(&self) -> &str {
        "Extract Polygon Orientation to Attribute"
    }

    #[cfg(feature = "new-geometry")]
    fn description(&self) -> &str {
        "Writes which way a 2D face's rings wind into an attribute: `clockwise`, \
         `counter_clockwise`, or `no_orientation` when some ring encloses nothing. Winding is \
         read in canonical orientation, so the answer describes the ring on the ground rather \
         than the axis order its coordinates are stored in. A 3D face has no absolute winding \
         and must be flattened first."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(OrientationExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    #[cfg(not(feature = "new-geometry"))]
    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    #[cfg(feature = "new-geometry")]
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
        let params: OrientationExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::OrientationExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::OrientationExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::OrientationExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(OrientationExtractor {
            output_attribute: params.output_attribute,
        }))
    }
}

/// # Orientation Extractor Parameters
/// Configure where to store the extracted polygon orientation information
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OrientationExtractorParam {
    /// # Output Attribute
    /// Name of the attribute where the orientation (clockwise/counter_clockwise) will be stored
    output_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct OrientationExtractor {
    output_attribute: Attribute,
}

impl Processor for OrientationExtractor {
    fn num_threads(&self) -> usize {
        2
    }

    /// Reads the winding of every ring of every face, in canonical orientation,
    /// and reduces them to one value the way the legacy path does: the first
    /// ring's winding, unless some ring encloses nothing. A 3D face has no
    /// absolute winding and non-areal geometry has no rings, so both leave via
    /// `rejected` rather than being labelled `no_orientation`, which would be
    /// indistinguishable from a genuinely flat face.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let Geometry::Euclidean2D(geometry) = &*ctx.feature.geometry else {
            ctx.event_hub.debug_log(
                Some(ctx.error_span()),
                "orientation rejected: winding is a 2D property, flatten the geometry first"
                    .to_string(),
            );
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        };

        let mut leaves = Vec::new();
        flatten_2d(geometry, &mut leaves);
        let mut windings = Vec::new();
        let mut non_areal = false;
        for leaf in &leaves {
            let Some(view) = leaf.area_view() else {
                non_areal = true;
                continue;
            };
            for face in view.faces() {
                for ring in face.rings() {
                    let coords: Vec<_> = ring.coords().collect();
                    windings.push(ring_winding_2d(leaf.frame(), &coords));
                }
            }
        }

        // A point or a curve among the members has no ring to report, so the
        // feature cannot be given one answer.
        if non_areal || windings.is_empty() {
            ctx.event_hub.debug_log(
                Some(ctx.error_span()),
                "orientation rejected: the geometry bounds no area".to_string(),
            );
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }

        let result = if windings.contains(&RingWinding::Degenerate) {
            NO_ORIENTATION
        } else if windings[0] == RingWinding::Clockwise {
            CLOCKWISE_ORIENTATION
        } else {
            COUNTER_CLOCKWISE_ORIENTATION
        };
        let mut feature = ctx.feature.clone();
        feature.attributes_mut().insert(
            self.output_attribute.clone(),
            AttributeValue::String(result.to_string()),
        );
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            let mut feature = feature.clone();
            feature.attributes_mut().insert(
                self.output_attribute.clone(),
                AttributeValue::String(NO_ORIENTATION.to_string()),
            );
            fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::None => {
                let mut feature = feature.clone();
                feature.attributes_mut().insert(
                    self.output_attribute.clone(),
                    AttributeValue::String(NO_ORIENTATION.to_string()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(geometry) => match geometry {
                Geometry2D::Polygon(polygon) => {
                    let mut feature = feature.clone();
                    let ring_winding_orders = polygon
                        .rings()
                        .iter()
                        .map(|ring| ring.winding_order())
                        .collect::<Vec<_>>();
                    let result = detect_orientation_by_ring_winding_orders(ring_winding_orders);
                    feature.attributes_mut().insert(
                        self.output_attribute.clone(),
                        AttributeValue::String(result.to_string()),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
                Geometry2D::MultiPolygon(polygons) => {
                    let mut feature = feature.clone();
                    let ring_winding_orders = polygons
                        .iter()
                        .flat_map(|polygon| {
                            polygon
                                .rings()
                                .iter()
                                .map(|ring| ring.winding_order())
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    let result = detect_orientation_by_ring_winding_orders(ring_winding_orders);
                    feature.attributes_mut().insert(
                        self.output_attribute.clone(),
                        AttributeValue::String(result.to_string()),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
                _ => unimplemented!(),
            },
            GeometryValue::FlowGeometry3D(geometry) => match geometry {
                Geometry3D::Polygon(polygon) => {
                    let mut feature = feature.clone();
                    let ring_winding_orders = polygon
                        .rings()
                        .iter()
                        .map(|ring| ring.winding_order())
                        .collect::<Vec<_>>();
                    let result = detect_orientation_by_ring_winding_orders(ring_winding_orders);
                    feature.attributes_mut().insert(
                        self.output_attribute.clone(),
                        AttributeValue::String(result.to_string()),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
                Geometry3D::MultiPolygon(polygons) => {
                    let mut feature = feature.clone();
                    let ring_winding_orders = polygons
                        .iter()
                        .flat_map(|polygon| {
                            polygon
                                .rings()
                                .iter()
                                .map(|ring| ring.winding_order())
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    let result = detect_orientation_by_ring_winding_orders(ring_winding_orders);
                    feature.attributes_mut().insert(
                        self.output_attribute.clone(),
                        AttributeValue::String(result.to_string()),
                    );
                    fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
                }
                _ => fw.send(ctx.new_with_feature_and_port(feature.clone(), FEATURES_PORT.clone())),
            },
            GeometryValue::CityGmlGeometry(city_gml) => {
                let mut feature = feature.clone();
                let mut ring_winding_orders = Vec::new();

                // Check all polygons in the CityGML geometry
                for gml_geo in &city_gml.gml_geometries {
                    for polygon in &gml_geo.polygons {
                        // Get the exterior ring (first ring)
                        if let Some(exterior) = polygon.rings().first() {
                            ring_winding_orders.push(exterior.winding_order());
                        }

                        // Also check interior rings (holes)
                        for interior in polygon.rings().iter().skip(1) {
                            ring_winding_orders.push(interior.winding_order());
                        }
                    }
                }

                let result = detect_orientation_by_ring_winding_orders(ring_winding_orders);
                feature.attributes_mut().insert(
                    self.output_attribute.clone(),
                    AttributeValue::String(result.to_string()),
                );
                fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
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
        "Orientation Extractor"
    }
}

#[cfg(not(feature = "new-geometry"))]
fn detect_orientation_by_ring_winding_orders(
    ring_winding_orders: Vec<Option<WindingOrder>>,
) -> WindingOrderResult {
    if ring_winding_orders.is_empty() {
        return NO_ORIENTATION;
    }
    if !ring_winding_orders
        .iter()
        .all(|winding_order| winding_order.is_some())
    {
        return INVALID_ORIENTATION;
    }
    let ring_winding_orders = ring_winding_orders.iter().flatten().collect::<Vec<_>>();
    for ring_winding_order in ring_winding_orders.iter() {
        let orientation = match ring_winding_order {
            WindingOrder::Clockwise => CLOCKWISE_ORIENTATION,
            WindingOrder::CounterClockwise => COUNTER_CLOCKWISE_ORIENTATION,
            WindingOrder::None => NO_ORIENTATION,
        };
        if orientation == NO_ORIENTATION {
            return orientation;
        }
    }
    match ring_winding_orders.first().unwrap() {
        WindingOrder::Clockwise => CLOCKWISE_ORIENTATION,
        WindingOrder::CounterClockwise => COUNTER_CLOCKWISE_ORIENTATION,
        WindingOrder::None => NO_ORIENTATION,
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod new_geometry_tests {
    use super::*;
    use crate::tests::utils::create_default_execute_context;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
    use reearth_flow_geometry::line_string::LineString2D;
    use reearth_flow_geometry::polygon::Polygon2D;
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry};
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_types::Feature;

    /// A triangle whose coordinates are written counter-clockwise when read as
    /// `(x, y)`.
    const CCW_AS_WRITTEN: [[f64; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]];

    fn face(frame: CoordinateFrame, ring: Vec<[f64; 2]>) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(frame, ring, Vec::<Vec<[f64; 2]>>::new()),
        )))
    }

    /// Run the processor over `geometry`, returning the port it used and the
    /// orientation it wrote (if any).
    fn extract(geometry: Geometry) -> (Port, Option<AttributeValue>) {
        let feature = Feature::from(geometry);
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let ctx = create_default_execute_context(&feature);
        OrientationExtractor {
            output_attribute: Attribute::new("orientation"),
        }
        .process(ctx, &fw)
        .unwrap();

        let ProcessorChannelForwarder::Noop(noop) = fw else {
            unreachable!("the forwarder is the one built above");
        };
        let ports = noop.send_ports.lock().unwrap();
        assert_eq!(ports.len(), 1);
        let features = noop.send_features.lock().unwrap();
        assert_eq!(features.len(), 1);
        (
            ports[0].clone(),
            features[0]
                .attributes
                .get(&Attribute::new("orientation"))
                .cloned(),
        )
    }

    #[test]
    fn a_euclidean_face_reports_the_winding_of_its_coordinates() {
        let (port, orientation) =
            extract(face(CoordinateFrame::Euclidean, CCW_AS_WRITTEN.to_vec()));
        assert_eq!(port, *FEATURES_PORT);
        assert_eq!(
            orientation,
            Some(AttributeValue::String("counter_clockwise".to_string()))
        );
    }

    /// The reason this port reads winding canonically rather than raw: in a
    /// Plane Rectangular frame, coordinates are stored `(northing, easting)`,
    /// so the very same numbers describe a clockwise ring on the ground. A
    /// workflow asking "is this face wound correctly" must get the ground
    /// answer, not the storage answer.
    #[test]
    fn the_same_coordinates_in_a_plane_rectangular_frame_wind_the_other_way() {
        let (port, orientation) = extract(face(
            CoordinateFrame::Crs(EpsgCode::new(6675)),
            CCW_AS_WRITTEN.to_vec(),
        ));
        assert_eq!(port, *FEATURES_PORT);
        assert_eq!(
            orientation,
            Some(AttributeValue::String("clockwise".to_string()))
        );
    }

    #[test]
    fn a_collinear_face_has_no_orientation() {
        let (port, orientation) = extract(face(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [0.0, 0.0]],
        ));
        assert_eq!(port, *FEATURES_PORT);
        assert_eq!(
            orientation,
            Some(AttributeValue::String("no_orientation".to_string()))
        );
    }

    /// A 3D face has no absolute winding, so it is rejected rather than
    /// silently measured on its `(x, y)` shadow.
    #[test]
    fn a_three_dimensional_face_is_rejected() {
        let ring = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            reearth_flow_geometry::polygon::Polygon3D::from_rings(
                CoordinateFrame::Euclidean,
                ring,
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )));
        let (port, orientation) = extract(geometry);
        assert_eq!(port, *REJECTED_PORT);
        assert_eq!(orientation, None);
    }

    #[test]
    fn a_curve_bounds_no_area_and_is_rejected() {
        let geometry = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(CoordinateFrame::Euclidean, vec![[0.0, 0.0], [1.0, 1.0]]),
        ));
        let (port, orientation) = extract(geometry);
        assert_eq!(port, *REJECTED_PORT);
        assert_eq!(orientation, None);
    }
}
