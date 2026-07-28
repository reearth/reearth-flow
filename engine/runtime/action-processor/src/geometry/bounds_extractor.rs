#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::{Aabb, BoundingBox};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::coordinate::Coordinate;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::coordnum::CoordNum;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::geometry::{Geometry, Geometry2D, Geometry3D};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::point::Point;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{CityGmlGeometry, GeometryValue};

#[cfg(not(feature = "new-geometry"))]
use num_traits::NumCast;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::collections::HashMap;
#[cfg(not(feature = "new-geometry"))]
use std::fmt::Debug;

use super::errors::GeometryProcessorError;
#[cfg(not(feature = "new-geometry"))]
use super::utils::finite_z;

/// The extent of a geometry along each axis. `min_z` / `max_z` are absent when
/// the extent is planar.
#[cfg(not(feature = "new-geometry"))]
#[derive(Debug, Clone)]
pub struct Bounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: Option<f64>,
    pub max_z: Option<f64>,
}

/// Whether every bound of the box is finite. A box with a non-finite bound
/// cannot be expressed as a JSON number.
#[cfg(feature = "new-geometry")]
fn bounds_are_finite(aabb: &Aabb) -> bool {
    let (min, max) = match aabb {
        Aabb::D2 { min, max } => (&min[..], &max[..]),
        Aabb::D3 { min, max } => (&min[..], &max[..]),
    };
    min.iter().chain(max).all(|v| v.is_finite())
}

#[derive(Debug, Clone, Default)]
pub(super) struct BoundsExtractorFactory;

impl ProcessorFactory for BoundsExtractorFactory {
    fn name(&self) -> &str {
        "Bounds Extractor"
    }

    fn description(&self) -> &str {
        "Extract Bounding Box Coordinates from Feature Geometry"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(BoundsExtractorParam))
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
        let params: BoundsExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::BoundsExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::BoundsExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::BoundsExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = BoundsExtractor { params };
        Ok(Box::new(process))
    }
}

/// # BoundsExtractor Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BoundsExtractorParam {
    /// # Minimum X Attribute
    /// Attribute name for storing the minimum X coordinate (defaults to "xmin")
    xmin: Option<Attribute>,
    /// # Maximum X Attribute
    /// Attribute name for storing the maximum X coordinate (defaults to "xmax")
    xmax: Option<Attribute>,
    /// # Minimum Y Attribute
    /// Attribute name for storing the minimum Y coordinate (defaults to "ymin")
    ymin: Option<Attribute>,
    /// # Maximum Y Attribute
    /// Attribute name for storing the maximum Y coordinate (defaults to "ymax")
    ymax: Option<Attribute>,
    /// # Minimum Z Attribute
    /// Attribute name for storing the minimum Z coordinate (defaults to "zmin")
    zmin: Option<Attribute>,
    /// # Maximum Z Attribute
    /// Attribute name for storing the maximum Z coordinate (defaults to "zmax")
    zmax: Option<Attribute>,
}

#[derive(Debug, Clone)]
pub struct BoundsExtractor {
    params: BoundsExtractorParam,
}

impl Processor for BoundsExtractor {
    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = feature.geometry.clone();
        let geometry_value = geometry.value.clone();
        let bounds = match geometry_value {
            GeometryValue::None => None,
            GeometryValue::CityGmlGeometry(city_gml) => Self::calc_city_gml(&city_gml),
            GeometryValue::FlowGeometry2D(flow_2d) => Self::calc_2d(&flow_2d),
            GeometryValue::FlowGeometry3D(flow_3d) => Self::calc_3d(&flow_3d),
        };
        if let Some(bounds) = bounds {
            let mut new_feature = feature.clone();
            self.insert_bounds(&mut new_feature, &bounds);
            fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
        } else {
            fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
        };
        Ok(())
    }

    /// Attach the geometry's axis-aligned bounding box to the feature as
    /// attributes. A feature whose geometry has no box (absent, empty, or a type
    /// that does not support the operation) or whose box has a non-finite bound
    /// is routed to `rejected`.
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let aabb = feature
            .geometry
            .bounding_box()
            .ok()
            .filter(bounds_are_finite);
        match aabb {
            Some(aabb) => {
                let mut new_feature = feature.clone();
                self.insert_bounds(&mut new_feature, aabb);
                fw.send(ctx.new_with_feature_and_port(new_feature, FEATURES_PORT.clone()));
            }
            None => fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone())),
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
        "Bounds Extractor"
    }
}

impl BoundsExtractor {
    /// Write one bound under its configured attribute name, falling back to
    /// `default`. `bound` must be finite; a bound that is not representable as a
    /// JSON number is written as zero.
    fn insert_bound(
        &self,
        feature: &mut Feature,
        name: &Option<Attribute>,
        default: &str,
        bound: f64,
    ) {
        let attribute = name.clone().unwrap_or_else(|| Attribute::new(default));
        let number = Number::from_f64(bound).unwrap_or_else(|| Number::from(0));
        feature.insert(attribute, AttributeValue::Number(number));
    }

    /// Write the box onto the feature as one attribute per bound. A 2D box
    /// writes no z attributes: the optional per-vertex elevation of a
    /// 2D-embedded geometry is not part of its box.
    #[cfg(feature = "new-geometry")]
    fn insert_bounds(&self, feature: &mut Feature, aabb: Aabb) {
        let (min, max, z) = match aabb {
            Aabb::D2 { min, max } => (min, max, None),
            Aabb::D3 { min, max } => ([min[0], min[1]], [max[0], max[1]], Some((min[2], max[2]))),
        };
        self.insert_bound(feature, &self.params.xmin, "xmin", min[0]);
        self.insert_bound(feature, &self.params.xmax, "xmax", max[0]);
        self.insert_bound(feature, &self.params.ymin, "ymin", min[1]);
        self.insert_bound(feature, &self.params.ymax, "ymax", max[1]);
        if let Some((min_z, max_z)) = z {
            self.insert_bound(feature, &self.params.zmin, "zmin", min_z);
            self.insert_bound(feature, &self.params.zmax, "zmax", max_z);
        }
    }

    /// Write the bounds onto the feature as one attribute per bound. The z
    /// attributes are written only when the bounds carry z.
    #[cfg(not(feature = "new-geometry"))]
    fn insert_bounds(&self, feature: &mut Feature, bounds: &Bounds) {
        self.insert_bound(feature, &self.params.xmin, "xmin", bounds.min_x);
        self.insert_bound(feature, &self.params.xmax, "xmax", bounds.max_x);
        self.insert_bound(feature, &self.params.ymin, "ymin", bounds.min_y);
        self.insert_bound(feature, &self.params.ymax, "ymax", bounds.max_y);
        if let Some(min_z) = bounds.min_z {
            self.insert_bound(feature, &self.params.zmin, "zmin", min_z);
        }
        if let Some(max_z) = bounds.max_z {
            self.insert_bound(feature, &self.params.zmax, "zmax", max_z);
        }
    }
}

#[cfg(not(feature = "new-geometry"))]
impl BoundsExtractor {
    fn update_bounds(current_bounds: Option<Bounds>, new_bounds: Option<Bounds>) -> Option<Bounds> {
        match (current_bounds, new_bounds) {
            (Some(mut cb), Some(nb)) => {
                cb.min_x = cb.min_x.min(nb.min_x);
                cb.max_x = cb.max_x.max(nb.max_x);
                cb.min_y = cb.min_y.min(nb.min_y);
                cb.max_y = cb.max_y.max(nb.max_y);
                cb.min_z = match (cb.min_z, nb.min_z) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (Some(a), None) | (None, Some(a)) => Some(a),
                    (None, None) => None,
                };
                cb.max_z = match (cb.max_z, nb.max_z) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (Some(a), None) | (None, Some(a)) => Some(a),
                    (None, None) => None,
                };
                Some(cb)
            }
            (None, Some(nb)) => Some(nb),
            (Some(cb), None) => Some(cb),
            (None, None) => None,
        }
    }

    fn update_bounds_for_coord<T, Z>(
        bounds: Option<Bounds>,
        coord: &Coordinate<T, Z>,
    ) -> Option<Bounds>
    where
        T: CoordNum + NumCast + PartialOrd + Debug + Copy,
        Z: CoordNum,
    {
        let z_val = finite_z(coord.z);
        Self::update_bounds(
            bounds,
            Some(Bounds {
                min_x: NumCast::from(coord.x).unwrap(),
                max_x: NumCast::from(coord.x).unwrap(),
                min_y: NumCast::from(coord.y).unwrap(),
                max_y: NumCast::from(coord.y).unwrap(),
                min_z: z_val,
                max_z: z_val,
            }),
        )
    }

    fn update_bounds_for_point<T, Z>(point: &Point<T, Z>) -> Option<Bounds>
    where
        T: CoordNum + NumCast + PartialOrd + Debug + Copy,
        Z: CoordNum,
    {
        let z_val = finite_z(point.z());
        Some(Bounds {
            min_x: NumCast::from(point.x()).unwrap(),
            max_x: NumCast::from(point.x()).unwrap(),
            min_y: NumCast::from(point.y()).unwrap(),
            max_y: NumCast::from(point.y()).unwrap(),
            min_z: z_val,
            max_z: z_val,
        })
    }

    fn calc_city_gml(geos: &CityGmlGeometry) -> Option<Bounds> {
        let mut out_bounds: Option<Bounds> = None;

        geos.gml_geometries.iter().for_each(|geo_feature| {
            let mut bounds: Option<Bounds> = None;
            for polygon in &geo_feature.polygons {
                let p = Geometry::Polygon(polygon.clone());
                match p {
                    Geometry::Point(point) => {
                        bounds = Self::update_bounds_for_point(&point);
                    }
                    Geometry::Line(line) => {
                        for coord in &[line.start, line.end] {
                            bounds = Self::update_bounds_for_coord(bounds, coord);
                        }
                    }
                    Geometry::LineString(line_string) => {
                        for coord in line_string.0.iter() {
                            bounds = Self::update_bounds_for_coord(bounds, coord);
                        }
                    }
                    Geometry::Polygon(polygon) => {
                        for coord in polygon.exterior().0.iter() {
                            bounds = Self::update_bounds_for_coord(bounds, coord);
                        }
                    }
                    _ => {}
                }
            }
            if let Some(ref mut out_bounds) = out_bounds {
                *out_bounds = Self::update_bounds(Some(out_bounds.clone()), bounds).unwrap();
            } else {
                out_bounds = bounds;
            }
        });
        out_bounds
    }

    fn calc_2d(geos: &Geometry2D) -> Option<Bounds> {
        let mut bounds: Option<Bounds> = None;
        match geos {
            Geometry2D::Point(point) => {
                bounds = Self::update_bounds_for_point(point);
            }
            Geometry2D::Line(line) => {
                for coord in &[line.start, line.end] {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            Geometry2D::LineString(line_string) => {
                for coord in line_string.0.iter() {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            Geometry2D::Polygon(polygon) => {
                for coord in polygon.exterior().0.iter() {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            _ => {}
        }
        bounds
    }

    fn calc_3d(geos: &Geometry3D) -> Option<Bounds> {
        let mut bounds: Option<Bounds> = None;
        match geos {
            Geometry3D::Point(point) => {
                bounds = Self::update_bounds_for_point(point);
            }
            Geometry3D::Line(line) => {
                for coord in &[line.start, line.end] {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            Geometry3D::LineString(line_string) => {
                for coord in line_string.0.iter() {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            Geometry3D::Polygon(polygon) => {
                for coord in polygon.exterior().0.iter() {
                    bounds = Self::update_bounds_for_coord(bounds, coord);
                }
            }
            _ => {}
        }
        bounds
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::line_string::{LineString2D, LineString3D};
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

    fn extractor() -> BoundsExtractor {
        BoundsExtractor {
            params: BoundsExtractorParam {
                xmin: None,
                xmax: None,
                ymin: None,
                ymax: None,
                zmin: None,
                zmax: None,
            },
        }
    }

    fn number(feature: &Feature, key: &str) -> Option<f64> {
        match feature.attributes.get(&Attribute::new(key))? {
            AttributeValue::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    fn extract(geometry: Geometry) -> Feature {
        let aabb = geometry.bounding_box().unwrap();
        let mut feature = Feature::from(geometry);
        extractor().insert_bounds(&mut feature, aabb);
        feature
    }

    #[test]
    fn three_dimensional_geometry_yields_every_axis() {
        let line = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[1.0, -2.0, 3.0], [-4.0, 5.0, -6.0]],
        );
        let feature = extract(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(line)));
        assert_eq!(number(&feature, "xmin"), Some(-4.0));
        assert_eq!(number(&feature, "xmax"), Some(1.0));
        assert_eq!(number(&feature, "ymin"), Some(-2.0));
        assert_eq!(number(&feature, "ymax"), Some(5.0));
        assert_eq!(number(&feature, "zmin"), Some(-6.0));
        assert_eq!(number(&feature, "zmax"), Some(3.0));
    }

    #[test]
    fn two_dimensional_geometry_yields_no_z() {
        let line = LineString2D::from_coords(CoordinateFrame::Euclidean, [[1.0, 2.0], [3.0, 4.0]]);
        let feature = extract(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line)));
        assert_eq!(number(&feature, "xmin"), Some(1.0));
        assert_eq!(number(&feature, "ymax"), Some(4.0));
        assert!(feature.attributes.get(&Attribute::new("zmin")).is_none());
        assert!(feature.attributes.get(&Attribute::new("zmax")).is_none());
    }

    #[test]
    fn elevation_of_a_two_dimensional_geometry_is_not_folded_in() {
        let line = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Euclidean,
            [[1.0, 2.0], [3.0, 4.0]],
            10.0,
        );
        let feature = extract(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(line)));
        assert!(feature.attributes.get(&Attribute::new("zmin")).is_none());
    }

    #[test]
    fn custom_attribute_names_are_honoured() {
        let line = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
        );
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::LineString(line));
        let aabb = geometry.bounding_box().unwrap();
        let extractor = BoundsExtractor {
            params: BoundsExtractorParam {
                xmin: Some(Attribute::new("west")),
                xmax: Some(Attribute::new("east")),
                ymin: Some(Attribute::new("south")),
                ymax: Some(Attribute::new("north")),
                zmin: Some(Attribute::new("bottom")),
                zmax: Some(Attribute::new("top")),
            },
        };
        let mut feature = Feature::from(geometry);
        extractor.insert_bounds(&mut feature, aabb);
        assert_eq!(number(&feature, "west"), Some(0.0));
        assert_eq!(number(&feature, "top"), Some(1.0));
        assert!(feature.attributes.get(&Attribute::new("xmin")).is_none());
    }

    #[test]
    fn geometry_without_an_extent_has_no_bounds() {
        assert!(Geometry::None.bounding_box().is_err());
        let empty = LineString3D::from_coords(CoordinateFrame::Euclidean, Vec::<[f64; 3]>::new());
        assert!(
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(empty))
                .bounding_box()
                .is_err()
        );
    }

    #[test]
    fn a_non_finite_bound_is_not_representable() {
        assert!(!bounds_are_finite(&Aabb::D2 {
            min: [f64::NAN, 0.0],
            max: [1.0, 1.0],
        }));
        assert!(!bounds_are_finite(&Aabb::D3 {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, f64::INFINITY],
        }));
        assert!(bounds_are_finite(&Aabb::D3 {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 1.0],
        }));
    }
}
