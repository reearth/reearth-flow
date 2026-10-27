//! Japanese standard regional mesh codes (JIS X 0410) for features.

use std::collections::HashMap;

#[cfg(feature = "new-geometry")]
use std::collections::BTreeMap;

#[cfg(feature = "new-geometry")]
use reearth_flow_diagnostics::{DiagnosticDraft, ErrorCode};
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{
    collection::Collection2D,
    coordinate::{CoordinateFrame, EpsgCode},
    ops::{
        centroid, common_frame, crs_is_projected_metres, rings_as_faces_2d, vertex::first_vertex,
        Aabb, BoundingBox, CentroidError,
    },
    predicates::view::{flatten_2d, Leaf2D},
    Euclidean2DGeometry, Geometry,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT, REJECTED_PORT},
};
#[cfg(feature = "new-geometry")]
use reearth_flow_runtime::{
    executor_operation::ExecutorContext, forwarder::ProcessorChannelForwarder,
};
#[cfg(feature = "new-geometry")]
use reearth_flow_types::jpmesh::overlap::{
    containing, largest_area_overlap, largest_length_overlap,
};
#[cfg(feature = "new-geometry")]
use reearth_flow_types::jpmesh::JPMeshCode;
use reearth_flow_types::jpmesh::JPMeshType;
use reearth_flow_types::Attribute;
#[cfg(feature = "new-geometry")]
use reearth_flow_types::AttributeValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct JPMeshCodeExtractorFactory;

impl ProcessorFactory for JPMeshCodeExtractorFactory {
    fn name(&self) -> &str {
        "JP Mesh Code Extractor"
    }

    fn description(&self) -> &str {
        "Adds the Japanese standard regional mesh code (JIS X 0410) of the mesh each feature belongs to."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(JPMeshCodeExtractorParam))
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
        let params: JPMeshCodeExtractorParam = match with {
            Some(with) => serde_json::from_value(Value::Object(with.into_iter().collect()))
                .map_err(|e| {
                    GeometryProcessorError::JPMeshCodeExtractorFactory(format!(
                        "Failed to deserialize parameters: {e}"
                    ))
                })?,
            None => JPMeshCodeExtractorParam::default(),
        };
        Ok(Box::new(JPMeshCodeExtractor { params }))
    }
}

/// # JP Mesh Code Extractor Parameters
///
/// Which mesh each feature is assigned to, at which size, and where the code is stored.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct JPMeshCodeExtractorParam {
    /// # Location
    /// Which part of the feature decides its mesh. A point exactly on a mesh edge belongs
    /// to the mesh to its north or east.
    #[serde(default)]
    location: MeshLocation,
    /// # Mesh Level
    /// The size of the mesh whose code is assigned.
    #[serde(default)]
    mesh_level: MeshLevel,
    /// # Output Attribute
    /// Attribute that receives the mesh code, written as a string of digits.
    #[serde(default = "default_output_attribute")]
    output_attribute: Attribute,
}

impl Default for JPMeshCodeExtractorParam {
    fn default() -> Self {
        Self {
            location: MeshLocation::default(),
            mesh_level: MeshLevel::default(),
            output_attribute: default_output_attribute(),
        }
    }
}

fn default_output_attribute() -> Attribute {
    Attribute::new("meshCode".to_string())
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
enum MeshLocation {
    /// # Largest Overlap
    /// The mesh holding the largest share of the feature: the most area for surfaces, the
    /// most length for open lines, and the most points for points, ties going to the smaller
    /// mesh code. A closed line counts as the surface it encloses. Needs flat geometry;
    /// surfaces and lines also need a projected coordinate system in metres, while points
    /// work in any.
    /// Measures are rounded to 0.01 square metres or metres, so a feature that measures zero
    /// in every mesh is rejected.
    #[default]
    LargestOverlap,
    /// # Centroid
    /// The mesh containing the feature's centroid: area-weighted for surfaces,
    /// length-weighted for lines, the mean for points; a solid uses its boundary surface.
    /// It can lie outside the feature for concave or hollow shapes.
    Centroid,
    /// # Bounding Box Center
    /// The mesh containing the center of the feature's bounding box, taken in the
    /// feature's own coordinate system.
    BoundingBoxCenter,
    /// # First Vertex
    /// The mesh containing the feature's first vertex.
    FirstVertex,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
enum MeshLevel {
    /// # 80 km (1st Mesh)
    /// The primary mesh, 40 minutes of latitude by 1 degree of longitude, with a 4-digit code.
    Mesh80km,
    /// # 10 km (2nd Mesh)
    /// The 1st mesh cut 8 by 8, 5 minutes of latitude by 7.5 minutes of longitude, with a
    /// 6-digit code.
    Mesh10km,
    /// # 1 km (3rd Mesh)
    /// The 2nd mesh cut 10 by 10, 30 seconds of latitude by 45 seconds of longitude, with
    /// an 8-digit code.
    #[default]
    Mesh1km,
    /// # 500 m (1/2 Mesh)
    /// The 3rd mesh cut 2 by 2, with a 9-digit code.
    Mesh500m,
    /// # 250 m (1/4 Mesh)
    /// The 1/2 mesh cut 2 by 2, with a 10-digit code.
    Mesh250m,
    /// # 125 m (1/8 Mesh)
    /// The 1/4 mesh cut 2 by 2, with an 11-digit code.
    Mesh125m,
    /// # 100 m (1/10 Mesh)
    /// The 3rd mesh cut 10 by 10, 3 seconds of latitude by 4.5 seconds of longitude, with
    /// a 10-digit code.
    Mesh100m,
}

impl From<MeshLevel> for JPMeshType {
    fn from(l: MeshLevel) -> Self {
        match l {
            MeshLevel::Mesh80km => JPMeshType::Mesh80km,
            MeshLevel::Mesh10km => JPMeshType::Mesh10km,
            MeshLevel::Mesh1km => JPMeshType::Mesh1km,
            MeshLevel::Mesh500m => JPMeshType::Mesh500m,
            MeshLevel::Mesh250m => JPMeshType::Mesh250m,
            MeshLevel::Mesh125m => JPMeshType::Mesh125m,
            MeshLevel::Mesh100m => JPMeshType::Mesh100m,
        }
    }
}

#[derive(Debug, Clone)]
struct JPMeshCodeExtractor {
    #[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
    params: JPMeshCodeExtractorParam,
}

/// Why a feature got no code: each maps to one registry code.
#[cfg(feature = "new-geometry")]
#[derive(Debug, PartialEq)]
enum Skip {
    NoCrs,
    MixedFrames,
    NotProjected,
    ThreeDimensional,
    OutOfRange,
    Unsupported,
    NothingToMeasure,
    Failed,
}

#[cfg(feature = "new-geometry")]
impl Skip {
    fn code(&self) -> ErrorCode {
        match self {
            Skip::NoCrs => ErrorCode::MeshNoCrs,
            Skip::MixedFrames => ErrorCode::MeshMixedFrames,
            Skip::NotProjected => ErrorCode::MeshNotProjected,
            Skip::ThreeDimensional => ErrorCode::MeshThreeDimensional,
            Skip::OutOfRange => ErrorCode::MeshOutOfRange,
            Skip::Unsupported => ErrorCode::MeshUnsupportedGeometry,
            Skip::NothingToMeasure => ErrorCode::MeshNothingToMeasure,
            Skip::Failed => ErrorCode::MeshAssignmentFailed,
        }
    }
}

impl Processor for JPMeshCodeExtractor {
    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        match self.mesh_code(&feature.geometry) {
            Ok(code) => {
                let mut out = feature.clone();
                out.insert(
                    self.params.output_attribute.clone(),
                    AttributeValue::String(code.to_number().to_string()),
                );
                fw.send(ctx.new_with_feature_and_port(out, FEATURES_PORT.clone()));
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
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "JPMeshCodeExtractor"
    }
}

#[cfg(feature = "new-geometry")]
impl JPMeshCodeExtractor {
    fn mesh_code(&self, geometry: &Geometry) -> Result<JPMeshCode, Skip> {
        // One code per feature needs one coordinate system: a point read from
        // parts in different frames has no meaning. Every part is checked,
        // including the ones a mode cannot read a point from, before any mode
        // reads one.
        let frame = common_frame(geometry).map_err(|_| Skip::MixedFrames)?;
        let mesh_type: JPMeshType = self.params.mesh_level.into();
        let point = match self.params.location {
            MeshLocation::LargestOverlap => return largest_overlap(geometry, mesh_type),
            MeshLocation::Centroid => centroid(geometry)
                .map_err(|e| match e {
                    CentroidError::MixedFrames => Skip::MixedFrames,
                    CentroidError::Unsupported(_) => Skip::Unsupported,
                })?
                .map(|(p, _)| p),
            MeshLocation::BoundingBoxCenter => bbox_center(geometry),
            MeshLocation::FirstVertex => first_vertex(geometry).map(|(p, _)| p),
        };
        let p = point.ok_or(Skip::Unsupported)?;
        let Some(CoordinateFrame::Crs(epsg)) = frame else {
            return Err(Skip::NoCrs);
        };
        containing(p, epsg, mesh_type)
            .map_err(|_| Skip::Failed)?
            .ok_or(Skip::OutOfRange)
    }
}

/// The largest-overlap rule: area for areal geometry (a closed line counts as
/// the surface it encloses), length for open lines, the most frequent
/// containing mesh for points. Only the highest dimension present counts. The
/// caller has already checked that every part shares one frame.
#[cfg(feature = "new-geometry")]
fn largest_overlap(geometry: &Geometry, mesh_type: JPMeshType) -> Result<JPMeshCode, Skip> {
    let flattened;
    let g = match geometry {
        Geometry::Euclidean2D(g) => g,
        Geometry::Euclidean3D(_) => return Err(Skip::ThreeDimensional),
        Geometry::None => return Err(Skip::Unsupported),
        Geometry::GeometryCollection(_) => {
            let mut members = Vec::new();
            flat_members(geometry, &mut members)?;
            flattened = Euclidean2DGeometry::Collection(Collection2D::new(members));
            &flattened
        }
    };
    let mut leaves = Vec::new();
    flatten_2d(g, &mut leaves);
    let frame = leaves.first().ok_or(Skip::Unsupported)?.frame();
    let CoordinateFrame::Crs(epsg) = frame else {
        return Err(Skip::NoCrs);
    };

    let is_areal = |l: &Leaf2D| match l {
        Leaf2D::Polygon(_) | Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_) => true,
        Leaf2D::Line(line) => line.is_closed_ring(),
        Leaf2D::Point(_) => false,
    };
    let is_line = |l: &Leaf2D| matches!(l, Leaf2D::Line(_));
    // Counting points needs no linear units, so a points-only feature is
    // accepted in any coordinate system.
    if !leaves.iter().any(|l| is_areal(l) || is_line(l)) {
        return majority_cell(&leaves, *epsg, mesh_type);
    }
    // Areas and lengths are measured in the CRS's own units and reported in
    // square metres and metres, so only a projected metre CRS will do.
    match crs_is_projected_metres(*epsg) {
        Ok(true) => {}
        Ok(false) => return Err(Skip::NotProjected),
        // PROJ could not resolve the CRS at all, so "reproject to a projected
        // metre CRS" is the wrong advice: report it like the point paths do.
        Err(_) => return Err(Skip::Failed),
    }
    let overlap = if leaves.iter().any(is_areal) {
        let areal = collection_or_single(leaves.iter().filter(|l| is_areal(l)).map(owned));
        largest_area_overlap(&rings_as_faces_2d(&areal), *epsg, mesh_type)
    } else {
        let lines = collection_or_single(leaves.iter().filter(|l| is_line(l)).map(owned));
        largest_length_overlap(&lines, *epsg, mesh_type)
    };
    match overlap.map_err(|_| Skip::Failed)? {
        Some(o) => Ok(o.selected),
        // `None` both for a feature reaching outside the mesh range and for one
        // that measures zero in every mesh: tell them apart by its extent.
        None => match g.bounding_box() {
            Ok(Aabb::D2 { min, max }) => {
                for corner in [[min[0], min[1]], [min[0], max[1]], [max[0], min[1]], max] {
                    let at = [corner[0], corner[1], 0.0];
                    if containing(at, *epsg, mesh_type)
                        .map_err(|_| Skip::Failed)?
                        .is_none()
                    {
                        return Err(Skip::OutOfRange);
                    }
                }
                Err(Skip::NothingToMeasure)
            }
            _ => Err(Skip::Unsupported),
        },
    }
}

/// The mesh holding the most points, the smaller code winning a tie.
#[cfg(feature = "new-geometry")]
fn majority_cell(
    leaves: &[Leaf2D],
    epsg: EpsgCode,
    mesh_type: JPMeshType,
) -> Result<JPMeshCode, Skip> {
    let mut counts: BTreeMap<u64, (usize, JPMeshCode)> = BTreeMap::new();
    for leaf in leaves {
        let Leaf2D::Point(p) = leaf else { continue };
        let [x, y] = p.position();
        let code = containing([x, y, 0.0], epsg, mesh_type)
            .map_err(|_| Skip::Failed)?
            .ok_or(Skip::OutOfRange)?;
        counts.entry(code.to_number()).or_insert((0, code)).0 += 1;
    }
    let mut best: Option<(usize, JPMeshCode)> = None;
    // Ascending keys, so keeping only a strictly larger count keeps the
    // smaller code on a tie.
    for (count, code) in counts.into_values() {
        if best.as_ref().is_none_or(|(n, _)| count > *n) {
            best = Some((count, code));
        }
    }
    best.map(|(_, code)| code).ok_or(Skip::Unsupported)
}

/// The members of a geometry collection as one list of 2D geometry, nested
/// collections included. A 3D member, or a member with no geometry, cannot be
/// measured on the plane, so the whole collection is unsupported.
#[cfg(feature = "new-geometry")]
fn flat_members(geometry: &Geometry, out: &mut Vec<Euclidean2DGeometry>) -> Result<(), Skip> {
    match geometry {
        Geometry::Euclidean2D(g) => out.push(g.clone()),
        Geometry::GeometryCollection(c) => {
            for m in c.members() {
                flat_members(m, out)?;
            }
        }
        Geometry::Euclidean3D(_) | Geometry::None => return Err(Skip::Unsupported),
    }
    Ok(())
}

/// The center of the geometry's bounding box in its own frame.
#[cfg(feature = "new-geometry")]
fn bbox_center(geometry: &Geometry) -> Option<[f64; 3]> {
    match geometry.bounding_box() {
        Ok(Aabb::D2 { min, max }) => Some([(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0, 0.0]),
        Ok(Aabb::D3 { min, max }) => Some([
            (min[0] + max[0]) / 2.0,
            (min[1] + max[1]) / 2.0,
            (min[2] + max[2]) / 2.0,
        ]),
        Err(_) => None,
    }
}

#[cfg(feature = "new-geometry")]
fn collection_or_single(
    mut members: impl Iterator<Item = Euclidean2DGeometry>,
) -> Euclidean2DGeometry {
    let first = members.next();
    match (first, members.next()) {
        (Some(only), None) => only,
        (first, second) => Euclidean2DGeometry::Collection(Collection2D::new(
            first.into_iter().chain(second).chain(members),
        )),
    }
}

/// The leaf as an owned geometry.
#[cfg(feature = "new-geometry")]
fn owned(leaf: &Leaf2D) -> Euclidean2DGeometry {
    match *leaf {
        Leaf2D::Point(p) => Euclidean2DGeometry::Point(p.clone()),
        Leaf2D::Line(l) => Euclidean2DGeometry::LineString(l.clone()),
        Leaf2D::Polygon(p) => Euclidean2DGeometry::Polygon(Box::new(p.clone())),
        Leaf2D::PolygonMesh(m) => Euclidean2DGeometry::PolygonMesh(Box::new(m.clone())),
        Leaf2D::TriangularMesh(m) => Euclidean2DGeometry::TriangularMesh(Box::new(m.clone())),
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use std::sync::Arc;

    use reearth_flow_common::uri::Uri;
    use reearth_flow_geometry::collection::{Collection2D, Collection3D};
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
    use reearth_flow_geometry::line_string::LineString2D;
    use reearth_flow_geometry::ops::reproject::{transform_coords_3d, ReprojectionCache};
    use reearth_flow_geometry::point::{Point2D, Point3D};
    use reearth_flow_geometry::point_cloud::PointCloud;
    use reearth_flow_geometry::polygon::{Polygon2D, Polygon3D};
    use reearth_flow_geometry::{
        Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection,
    };
    use reearth_flow_runtime::event::Event;
    use reearth_flow_runtime::forwarder::NoopChannelForwarder;
    use reearth_flow_runtime::kvs;
    use reearth_flow_storage::resolve::StorageResolver;
    use reearth_flow_types::{Attribute, AttributeValue, Feature};
    use serde_json::json;

    /// The Z-bldg-03 fixture footprint as (longitude, latitude): 9.14 m² in
    /// 3rd mesh 54377085.
    const FOOTPRINT: [(f64, f64); 5] = [
        (137.07022204628032, 36.65423985231743),
        (137.07018801464667, 36.65426289610968),
        (137.0701714804155, 36.654247021162256),
        (137.07020551204704, 36.65422397737467),
        (137.07022204628032, 36.65423985231743),
    ];
    const PRCS: EpsgCode = EpsgCode::new(6675);
    const GEO: EpsgCode = EpsgCode::new(6668);

    /// Coordinates in `to`'s declared axis order ([lat, lng] for 6668,
    /// [northing, easting] for 6675).
    fn project(lng_lat: &[(f64, f64)], to: EpsgCode) -> Vec<[f64; 2]> {
        let mut c: Vec<[f64; 3]> = lng_lat.iter().map(|&(lng, lat)| [lat, lng, 0.0]).collect();
        if to != GEO {
            transform_coords_3d(&mut ReprojectionCache::new(), GEO, to, &mut c).unwrap();
        }
        c.iter().map(|p| [p[0], p[1]]).collect()
    }

    fn polygon_from(epsg: EpsgCode, ring: Vec<[f64; 2]>) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Crs(epsg),
                ring,
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )))
    }

    fn polygon_in(epsg: EpsgCode) -> Geometry {
        polygon_from(epsg, project(&FOOTPRINT, epsg))
    }

    fn first_point_in(epsg: EpsgCode) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            CoordinateFrame::Crs(epsg),
            project(&FOOTPRINT[..1], epsg)[0],
        )))
    }

    fn code_of(f: &Feature, attr: &str) -> Option<String> {
        match f.attributes.get(&Attribute::new(attr.to_string())) {
            Some(AttributeValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    fn build(with: Value) -> Box<dyn Processor> {
        let with: HashMap<String, Value> = serde_json::from_value(with).unwrap();
        JPMeshCodeExtractorFactory
            .build(
                NodeContext::default(),
                EventHub::new(1),
                "JP Mesh Code Extractor".to_string(),
                Some(with),
            )
            .unwrap()
    }

    /// Run every `geometry` through one processor and return the features it
    /// sent, the port each went to, and the code of every diagnostic raised.
    /// The contexts are built by hand so the test holds a receiver on the hub
    /// the diagnostics travel through (see `area_calculator.rs`).
    fn process_many(
        with: Value,
        geometries: Vec<Geometry>,
    ) -> (Vec<Feature>, Vec<Port>, Vec<ErrorCode>) {
        let hub = EventHub::new(64);
        let mut rx = hub.receiver.resubscribe();
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        let mut processor = build(with);

        for geometry in geometries {
            let ctx = ExecutorContext::new(
                Feature::from(geometry),
                FEATURES_PORT.clone(),
                Arc::new(serde_json::Map::new()),
                Arc::new(StorageResolver::new()),
                Arc::new(kvs::create_kv_store()),
                hub.clone(),
                Uri::for_test("file:///"),
            );
            processor.process(ctx, &fw).unwrap();
        }

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
    fn largest_overlap_matches_the_plateau_fixture() {
        let (out, ports, warnings) = process_many(json!({}), vec![polygon_in(PRCS)]);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(ports, vec![FEATURES_PORT.clone()]);
        assert_eq!(code_of(&out[0], "meshCode").as_deref(), Some("54377085"));
    }

    #[test]
    fn every_point_mode_agrees_on_a_small_footprint() {
        for location in ["centroid", "boundingBoxCenter", "firstVertex"] {
            let (out, _, warnings) =
                process_many(json!({ "location": location }), vec![polygon_in(PRCS)]);
            assert!(warnings.is_empty(), "{location}: {warnings:?}");
            assert_eq!(
                code_of(&out[0], "meshCode").as_deref(),
                Some("54377085"),
                "{location}"
            );
        }
    }

    #[test]
    fn mesh_level_and_output_attribute_are_honoured() {
        let (out, ports, warnings) = process_many(
            json!({ "meshLevel": "mesh10km", "outputAttribute": "m" }),
            vec![polygon_in(PRCS)],
        );
        assert!(warnings.is_empty(), "{warnings:?} {ports:?}");
        assert_eq!(code_of(&out[0], "m").as_deref(), Some("543770"));
    }

    #[test]
    fn every_mesh_level_measures_against_the_cells_of_that_level() {
        // The footprint is a few metres across and sits inside one cell at
        // every level, so the largest overlap must name the cell its first
        // vertex is in.
        for level in [
            "mesh80km", "mesh10km", "mesh1km", "mesh500m", "mesh250m", "mesh125m", "mesh100m",
        ] {
            let (overlap, _, warnings) =
                process_many(json!({ "meshLevel": level }), vec![polygon_in(PRCS)]);
            assert!(warnings.is_empty(), "{level}: {warnings:?}");
            let (vertex, _, _) = process_many(
                json!({ "meshLevel": level, "location": "firstVertex" }),
                vec![polygon_in(PRCS)],
            );
            let code = code_of(&overlap[0], "meshCode");
            assert_eq!(code, code_of(&vertex[0], "meshCode"), "{level}");
            assert!(code.unwrap().starts_with("5437"), "{level}");
        }
    }

    #[test]
    fn representative_point_modes_respect_northing_first_axis_order() {
        // EPSG:6675 stores [northing, easting]; a swapped axis lands ~80 km away.
        let (out, _, _) = process_many(
            json!({ "location": "firstVertex" }),
            vec![first_point_in(PRCS)],
        );
        assert_eq!(code_of(&out[0], "meshCode").as_deref(), Some("54377085"));
    }

    #[test]
    fn a_point_in_largest_overlap_mode_takes_its_containing_cell() {
        let (out, _, warnings) = process_many(json!({}), vec![first_point_in(PRCS)]);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(code_of(&out[0], "meshCode").as_deref(), Some("54377085"));
    }

    #[test]
    fn a_frame_without_a_crs_is_rejected() {
        let square = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]],
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )));
        let (out, ports, warnings) = process_many(json!({}), vec![square]);
        assert_eq!(warnings, vec![ErrorCode::MeshNoCrs]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
        assert!(code_of(&out[0], "meshCode").is_none());
    }

    #[test]
    fn largest_overlap_rejects_geographic_input_but_point_modes_accept_it() {
        let (_, _, warnings) = process_many(json!({}), vec![polygon_in(GEO)]);
        assert_eq!(warnings, vec![ErrorCode::MeshNotProjected]);
        let (out, _, warnings) =
            process_many(json!({ "location": "centroid" }), vec![polygon_in(GEO)]);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(code_of(&out[0], "meshCode").as_deref(), Some("54377085"));
    }

    #[test]
    fn largest_overlap_rejects_3d_input() {
        let ring: Vec<[f64; 3]> = project(&FOOTPRINT, PRCS)
            .into_iter()
            .map(|p| [p[0], p[1], 10.0])
            .collect();
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Crs(PRCS),
                ring,
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )));
        let (_, ports, warnings) = process_many(json!({}), vec![g]);
        assert_eq!(warnings, vec![ErrorCode::MeshThreeDimensional]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
    }

    #[test]
    fn out_of_range_is_rejected() {
        // London, EPSG:4326 in its declared [lat, lng] order.
        let london = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [51.5, -0.12],
        )));
        let (_, ports, warnings) = process_many(json!({ "location": "centroid" }), vec![london]);
        assert_eq!(warnings, vec![ErrorCode::MeshOutOfRange]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
    }

    #[test]
    fn a_projected_polygon_outside_the_mesh_range_is_out_of_range_in_largest_overlap() {
        // A 10 m square near London in EPSG:3857 (x = easting, y = northing).
        let (x, y) = (-13_358.0, 6_710_000.0);
        let square = polygon_from(
            EpsgCode::new(3857),
            vec![
                [x, y],
                [x + 10.0, y],
                [x + 10.0, y + 10.0],
                [x, y + 10.0],
                [x, y],
            ],
        );
        let (_, ports, warnings) = process_many(json!({}), vec![square]);
        assert_eq!(warnings, vec![ErrorCode::MeshOutOfRange]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
    }

    #[test]
    fn a_footprint_too_small_to_measure_is_rejected_as_such() {
        // A 5 cm square (0.0025 m²) inside the fixture's mesh rounds to zero.
        let [n, e] = project(&FOOTPRINT[..1], PRCS)[0];
        let d = 0.05;
        let square = polygon_from(
            PRCS,
            vec![[n, e], [n, e + d], [n + d, e + d], [n + d, e], [n, e]],
        );
        let (_, ports, warnings) = process_many(json!({}), vec![square]);
        assert_eq!(warnings, vec![ErrorCode::MeshNothingToMeasure]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
    }

    #[test]
    fn every_point_mode_rejects_parts_in_different_coordinate_systems() {
        let mixed = Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(
                CoordinateFrame::Crs(PRCS),
                project(&FOOTPRINT[..1], PRCS)[0],
            )),
            Euclidean2DGeometry::Point(Point2D::new(
                CoordinateFrame::Crs(GEO),
                project(&FOOTPRINT[1..2], GEO)[0],
            )),
        ]);
        let g = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(mixed));
        for location in ["centroid", "boundingBoxCenter", "firstVertex"] {
            let (_, ports, warnings) =
                process_many(json!({ "location": location }), vec![g.clone()]);
            assert_eq!(warnings, vec![ErrorCode::MeshMixedFrames], "{location}");
            assert_eq!(ports, vec![REJECTED_PORT.clone()], "{location}");
        }
    }

    #[test]
    fn a_line_goes_to_the_mesh_holding_most_of_its_length() {
        // East from the fixture across 3rd mesh 54377085's east edge
        // (137.075 E): about 0.0048 degrees inside it, 0.001 beyond.
        let line = project(&[(137.0702, 36.6542), (137.076, 36.6542)], PRCS);
        let g = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
            CoordinateFrame::Crs(PRCS),
            line,
        )));
        let (out, ports, warnings) = process_many(json!({}), vec![g]);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(ports, vec![FEATURES_PORT.clone()]);
        assert_eq!(code_of(&out[0], "meshCode").as_deref(), Some("54377085"));
    }

    #[test]
    fn a_multi_point_takes_the_cell_holding_most_points() {
        let near = project(&FOOTPRINT[..2], PRCS);
        let far = project(&[(139.7671, 35.6812)], PRCS)[0]; // Tokyo Station, another mesh
        let pts = Collection2D::new(
            [near[0], near[1], far]
                .into_iter()
                .map(|p| Euclidean2DGeometry::Point(Point2D::new(CoordinateFrame::Crs(PRCS), p))),
        );
        let (out, _, _) = process_many(
            json!({}),
            vec![Geometry::Euclidean2D(Euclidean2DGeometry::Collection(pts))],
        );
        assert_eq!(code_of(&out[0], "meshCode").as_deref(), Some("54377085"));
    }

    #[test]
    fn a_geographic_multi_point_takes_the_majority_cell_in_largest_overlap() {
        let pts = Collection2D::new(
            project(&FOOTPRINT[..3], GEO)
                .into_iter()
                .map(|p| Euclidean2DGeometry::Point(Point2D::new(CoordinateFrame::Crs(GEO), p))),
        );
        let (out, ports, warnings) = process_many(
            json!({}),
            vec![Geometry::Euclidean2D(Euclidean2DGeometry::Collection(pts))],
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(ports, vec![FEATURES_PORT.clone()]);
        assert_eq!(code_of(&out[0], "meshCode").as_deref(), Some("54377085"));
    }

    #[test]
    fn largest_overlap_rejects_parts_in_different_coordinate_systems() {
        let mixed = Collection2D::new([
            Euclidean2DGeometry::Point(Point2D::new(
                CoordinateFrame::Crs(PRCS),
                project(&FOOTPRINT[..1], PRCS)[0],
            )),
            Euclidean2DGeometry::Point(Point2D::new(
                CoordinateFrame::Crs(GEO),
                project(&FOOTPRINT[1..2], GEO)[0],
            )),
        ]);
        let g = Geometry::Euclidean2D(Euclidean2DGeometry::Collection(mixed));
        let (_, ports, warnings) = process_many(json!({}), vec![g]);
        assert_eq!(warnings, vec![ErrorCode::MeshMixedFrames]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
    }

    #[test]
    fn a_feature_without_geometry_is_rejected_in_every_mode() {
        for location in [
            "largestOverlap",
            "centroid",
            "boundingBoxCenter",
            "firstVertex",
        ] {
            let (out, ports, warnings) =
                process_many(json!({ "location": location }), vec![Geometry::None]);
            assert_eq!(
                warnings,
                vec![ErrorCode::MeshUnsupportedGeometry],
                "{location}"
            );
            assert_eq!(ports, vec![REJECTED_PORT.clone()], "{location}");
            assert!(code_of(&out[0], "meshCode").is_none(), "{location}");
        }
    }

    #[test]
    fn a_leading_unsupported_part_does_not_hide_mixed_frames() {
        // The point cloud has no centroid or vertex of its own, so a check that
        // stops at the first unsupported part would never see the GEO point.
        let at = |epsg: EpsgCode, i: usize| {
            let [a, b] = project(&FOOTPRINT[i..i + 1], epsg)[0];
            [a, b, 0.0]
        };
        let cloud = PointCloud::from_positions(CoordinateFrame::Crs(PRCS), [at(PRCS, 0)]);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(vec![
            Euclidean3DGeometry::PointCloud(Box::new(cloud)),
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Crs(PRCS), at(PRCS, 1))),
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Crs(GEO), at(GEO, 2))),
        ])));
        for location in [
            "largestOverlap",
            "centroid",
            "boundingBoxCenter",
            "firstVertex",
        ] {
            let (_, ports, warnings) =
                process_many(json!({ "location": location }), vec![g.clone()]);
            assert_eq!(warnings, vec![ErrorCode::MeshMixedFrames], "{location}");
            assert_eq!(ports, vec![REJECTED_PORT.clone()], "{location}");
        }
    }

    #[test]
    fn a_point_cloud_takes_its_box_centre_cell_but_has_no_centroid_or_first_vertex() {
        let positions: Vec<[f64; 3]> = project(&FOOTPRINT[..4], PRCS)
            .into_iter()
            .map(|[a, b]| [a, b, 0.0])
            .collect();
        let cloud = Geometry::Euclidean3D(Euclidean3DGeometry::PointCloud(Box::new(
            PointCloud::from_positions(CoordinateFrame::Crs(PRCS), positions),
        )));
        let (out, ports, warnings) = process_many(
            json!({ "location": "boundingBoxCenter" }),
            vec![cloud.clone()],
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(ports, vec![FEATURES_PORT.clone()]);
        assert_eq!(code_of(&out[0], "meshCode").as_deref(), Some("54377085"));
        for location in ["centroid", "firstVertex"] {
            let (_, ports, warnings) =
                process_many(json!({ "location": location }), vec![cloud.clone()]);
            assert_eq!(
                warnings,
                vec![ErrorCode::MeshUnsupportedGeometry],
                "{location}"
            );
            assert_eq!(ports, vec![REJECTED_PORT.clone()], "{location}");
        }
    }

    #[test]
    fn a_2d_geometry_collection_is_measured() {
        let g = Geometry::GeometryCollection(GeometryCollection::new([polygon_in(PRCS)]));
        let (out, ports, warnings) = process_many(json!({}), vec![g]);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(ports, vec![FEATURES_PORT.clone()]);
        assert_eq!(code_of(&out[0], "meshCode").as_deref(), Some("54377085"));
    }

    #[test]
    fn a_geometry_collection_with_a_3d_member_is_unsupported_in_largest_overlap() {
        let ring: Vec<[f64; 3]> = project(&FOOTPRINT, PRCS)
            .into_iter()
            .map(|p| [p[0], p[1], 10.0])
            .collect();
        let solid_3d = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(
                CoordinateFrame::Crs(PRCS),
                ring,
                Vec::<Vec<[f64; 3]>>::new(),
            ),
        )));
        let g = Geometry::GeometryCollection(GeometryCollection::new([polygon_in(PRCS), solid_3d]));
        let (_, ports, warnings) = process_many(json!({}), vec![g]);
        assert_eq!(warnings, vec![ErrorCode::MeshUnsupportedGeometry]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
    }

    #[test]
    fn largest_overlap_rejects_a_feet_based_crs() {
        // A 30 ft square in NAD83 / California zone 3 (US survey feet).
        let (x, y) = (6_000_000.0, 2_100_000.0);
        let square = polygon_from(
            EpsgCode::new(2227),
            vec![
                [x, y],
                [x + 30.0, y],
                [x + 30.0, y + 30.0],
                [x, y + 30.0],
                [x, y],
            ],
        );
        let (_, ports, warnings) = process_many(json!({}), vec![square]);
        assert_eq!(warnings, vec![ErrorCode::MeshNotProjected]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
    }

    #[test]
    fn largest_overlap_reports_an_unresolvable_crs_as_a_failure() {
        // PROJ cannot resolve this EPSG code, which is a different problem from
        // a CRS that resolves but is not projected in metres: telling the user
        // to reproject would be the wrong remediation.
        let square = polygon_from(
            EpsgCode::new(65_000),
            vec![
                [0.0, 0.0],
                [30.0, 0.0],
                [30.0, 30.0],
                [0.0, 30.0],
                [0.0, 0.0],
            ],
        );
        let (_, ports, warnings) = process_many(json!({}), vec![square]);
        assert_eq!(warnings, vec![ErrorCode::MeshAssignmentFailed]);
        assert_eq!(ports, vec![REJECTED_PORT.clone()]);
    }

    #[test]
    fn a_closed_line_is_measured_as_the_surface_it_encloses() {
        let ring = project(&FOOTPRINT, PRCS);
        assert_eq!(ring.first(), ring.last());
        let g = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
            CoordinateFrame::Crs(PRCS),
            ring,
        )));
        let (out, ports, warnings) = process_many(json!({}), vec![g]);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(ports, vec![FEATURES_PORT.clone()]);
        assert_eq!(code_of(&out[0], "meshCode").as_deref(), Some("54377085"));
    }
}
