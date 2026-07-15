use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::ops::reproject::transform_coords_3d;
use reearth_flow_geometry::ops::{triangulation::Cache as TriangulationCache, ReprojectionCache};
use reearth_flow_geometry::polygon_mesh::PolygonMesh3D;
use reearth_flow_geometry::solid::Shell;
use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};

/// WGS84, 3D geographic (lat, lon, height) — used for the tileset's bounding region.
const WGS84_GEOGRAPHIC: EpsgCode = EpsgCode::new(4979);
/// WGS84, geocentric (ECEF) — used for glTF vertex positions.
const WGS84_GEOCENTRIC: EpsgCode = EpsgCode::new(4978);

/// One `ReprojectionCache` per target CRS: each cache only ever sees a
/// single (from, to) pair, so its cached PROJ transform is never evicted
/// (a cache handed both target CRSes would thrash between them every call).
#[derive(Default)]
pub(super) struct ReprojectCaches {
    geographic: ReprojectionCache,
    geocentric: ReprojectionCache,
}

pub(super) struct ExtractedMesh {
    /// Vertex positions in ECEF (WGS84 geocentric), metres.
    pub(super) ecef_vertices: Vec<[f64; 3]>,
    /// The same vertices in WGS84 geographic (lat, lon, height); degrees/metres.
    pub(super) geographic_vertices: Vec<[f64; 3]>,
    /// Triangle index triples, parallel to both vertex arrays above.
    pub(super) indices: Vec<[u32; 3]>,
    /// Each source polygon's flat normal (one entry per polygon, not per
    /// triangle); `polygon_tris[i]` is how many triangles polygon `i` expanded into.
    pub(super) polygon_normals: Vec<[f64; 3]>,
    /// Output triangle count per source polygon, parallel to `polygon_normals`.
    pub(super) polygon_tris: Vec<u32>,
}

/// Extract and reproject every `PolygonMesh` reachable from `geometry`, merged
/// into one combined mesh (index-offset concatenation). `caches` should be
/// shared across every feature in the file.
///
/// Returns `None` when nothing was found, or everything found failed to
/// triangulate/reproject (each failure is `tracing::warn!`ed individually).
pub(super) fn extract(geometry: &Geometry, caches: &mut ReprojectCaches) -> Option<ExtractedMesh> {
    let mut meshes = Vec::new();
    collect_geometry(geometry, &mut meshes);

    let mut combined = ExtractedMesh {
        ecef_vertices: Vec::new(),
        geographic_vertices: Vec::new(),
        indices: Vec::new(),
        polygon_normals: Vec::new(),
        polygon_tris: Vec::new(),
    };

    for mesh in meshes {
        let Some(extracted) = extract_one(mesh, caches) else {
            continue;
        };
        let base = combined.ecef_vertices.len() as u32;
        combined.indices.extend(
            extracted
                .indices
                .into_iter()
                .map(|[a, b, c]| [a + base, b + base, c + base]),
        );
        combined.ecef_vertices.extend(extracted.ecef_vertices);
        combined
            .geographic_vertices
            .extend(extracted.geographic_vertices);
        combined.polygon_normals.extend(extracted.polygon_normals);
        combined.polygon_tris.extend(extracted.polygon_tris);
    }

    if combined.ecef_vertices.is_empty() {
        None
    } else {
        Some(combined)
    }
}

/// Recurse through `GeometryCollection` — the reader's mandatory per-LOD
/// wrapper — into the `Euclidean3D` members it holds.
fn collect_geometry(geometry: &Geometry, out: &mut Vec<PolygonMesh3D>) {
    match geometry {
        Geometry::GeometryCollection(gc) => {
            for member in gc.members() {
                collect_geometry(member, out);
            }
        }
        Geometry::Euclidean3D(e) => collect_euclidean3d(e, out),
        other => tracing::warn!("Cesium3DTilesWriter: skipping unsupported geometry {other:?}"),
    }
}

/// Recurse through `Collection` members and unpack a `Solid`'s boundary
/// shells, collecting every `PolygonMesh` found. A `Solid` shell is a
/// coordinate-free mesh; its frame lives on the enclosing `Solid`, so each
/// unpacked shell is re-paired with the solid's frame for reprojection.
fn collect_euclidean3d(geometry: &Euclidean3DGeometry, out: &mut Vec<PolygonMesh3D>) {
    match geometry {
        Euclidean3DGeometry::Collection(c) => {
            for member in c.members() {
                collect_euclidean3d(member, out);
            }
        }
        Euclidean3DGeometry::PolygonMesh(mesh) => out.push((**mesh).clone()),
        Euclidean3DGeometry::Solid(solid) => {
            for shell in std::iter::once(solid.exterior()).chain(solid.interiors()) {
                match shell {
                    Shell::PolygonMesh(data) => {
                        out.push(PolygonMesh3D::new(solid.frame().clone(), data.clone()))
                    }
                    Shell::TriangularMesh(_) => tracing::warn!(
                        "Cesium3DTilesWriter: a Solid shell is a TriangularMesh; \
                         TriangularMesh leaves aren't supported, skipping"
                    ),
                }
            }
        }
        other => tracing::warn!(
            "Cesium3DTilesWriter: only PolygonMesh/Solid are supported; skipping {other:?}"
        ),
    }
}

/// The EPSG source CRS a mesh's coordinates are reprojected from. A mesh
/// tagged with anything but a concrete `Crs` (e.g. `Euclidean`, meaning the
/// reader found no srsName) cannot be placed on the globe, so it is skipped.
fn source_crs(frame: &CoordinateFrame) -> Option<EpsgCode> {
    match frame {
        CoordinateFrame::Crs(epsg) => Some(*epsg),
        other => {
            tracing::warn!("Cesium3DTilesWriter: mesh has no geographic CRS ({other:?}); skipping");
            None
        }
    }
}

/// Triangulate and reproject one `PolygonMesh`.
///
/// Reprojection into ECEF (a right-handed frame) carries the source winding into
/// its canonical, CCW-outward orientation on its own: a reflected source frame
/// (lat/northing-first) reprojects with a reversed orientation that cancels its
/// reflected winding. So the reprojected triangles are already CCW-front for glTF,
/// and the flat normals are recomputed from that ECEF geometry rather than carried
/// from the source frame (where they point inward for a reflected frame and would
/// need a separate source-to-ECEF rotation).
fn extract_one(mut mesh: PolygonMesh3D, caches: &mut ReprojectCaches) -> Option<ExtractedMesh> {
    let source_crs = source_crs(mesh.frame())?;

    let mut triangulation_cache = TriangulationCache::new();
    let result = mesh.triangulate_with_normals(&mut triangulation_cache);
    let (mesh, polygon_tris) = (result.mesh, result.polygon_tris);

    let mut geographic_vertices = mesh.vertices().to_vec();
    if let Err(e) = transform_coords_3d(
        &mut caches.geographic,
        source_crs,
        WGS84_GEOGRAPHIC,
        &mut geographic_vertices,
    ) {
        tracing::warn!("Cesium3DTilesWriter: failed to reproject to WGS84 geographic: {e:?}");
        return None;
    }

    let mut ecef_vertices = mesh.vertices().to_vec();
    if let Err(e) = transform_coords_3d(
        &mut caches.geocentric,
        source_crs,
        WGS84_GEOCENTRIC,
        &mut ecef_vertices,
    ) {
        tracing::warn!("Cesium3DTilesWriter: failed to reproject to ECEF: {e:?}");
        return None;
    }

    let indices: Vec<[u32; 3]> = mesh.triangles().collect();
    let polygon_normals = ecef_polygon_normals(&ecef_vertices, &indices, &polygon_tris);

    Some(ExtractedMesh {
        ecef_vertices,
        geographic_vertices,
        indices,
        polygon_normals,
        polygon_tris,
    })
}

/// One outward unit normal per source polygon, taken from the first ECEF triangle
/// of each; a degenerate polygon (zero triangles) gets a placeholder that is never
/// repeated into the output. `polygon_tris` gives each polygon's triangle count in
/// polygon order, and `triangles` are grouped by polygon in that same order.
fn ecef_polygon_normals(
    vertices: &[[f64; 3]],
    triangles: &[[u32; 3]],
    polygon_tris: &[u32],
) -> Vec<[f64; 3]> {
    let mut normals = Vec::with_capacity(polygon_tris.len());
    let mut base = 0usize;
    for &count in polygon_tris {
        let normal = if count == 0 {
            [0.0, 0.0, 1.0]
        } else {
            let [a, b, c] = triangles[base];
            face_normal(
                vertices[a as usize],
                vertices[b as usize],
                vertices[c as usize],
            )
        };
        normals.push(normal);
        base += count as usize;
    }
    normals
}

/// Unit right-hand-rule normal of triangle `a, b, c`; `+Z` when the triangle is
/// degenerate.
fn face_normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f64; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 0.0 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::polygon_mesh::{PolygonMesh3D, PolygonMesh3DData};

    use super::*;

    fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    #[test]
    fn face_normal_of_ccw_xy_triangle_points_up() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(n, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn face_normal_of_degenerate_triangle_falls_back_to_up() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert_eq!(n, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn ecef_polygon_normals_groups_by_polygon_and_placeholders_degenerate() {
        // Polygon 0: one triangle in the z = 1 plane, CCW -> +Z normal.
        // Polygon 1: zero triangles (degenerate) -> placeholder, not read from `triangles`.
        let vertices = vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]];
        let triangles = vec![[0u32, 1, 2]];
        let polygon_tris = vec![1u32, 0];

        let normals = ecef_polygon_normals(&vertices, &triangles, &polygon_tris);

        assert_eq!(normals.len(), 2);
        assert_eq!(normals[0], [0.0, 0.0, 1.0]);
        assert_eq!(normals[1], [0.0, 0.0, 1.0]);
    }

    // A face whose canonical orientation is outward, stored in a lat-first frame
    // (EPSG:4979, orientation sign -1), must emit an ECEF normal that points away
    // from the earth's centre. This fails if the normal is carried from the source
    // frame (where it points inward for a reflected frame) instead of recomputed
    // from the reprojected ECEF geometry.
    #[test]
    fn outward_face_in_lat_first_frame_emits_outward_ecef_normal() {
        // Vertices stored (lat, lon, height). In real ENU at this location the ring
        // A -> B -> C turns counter-clockwise seen from above (an upward, outward
        // face); its right-hand-rule normal in stored order points the opposite way,
        // and reprojection into right-handed ECEF flips it back to outward.
        let a = [35.0, 139.0, 0.0];
        let b = [35.0, 139.001, 0.0];
        let c = [35.001, 139.0, 0.0];
        let data = PolygonMesh3DData::from_parts(vec![a, b, c], [[0u32, 1, 2]]).unwrap();
        let mesh = PolygonMesh3D::new(CoordinateFrame::Crs(EpsgCode::new(4979)), data);
        let geometry = Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(Box::new(mesh)));

        let mut caches = ReprojectCaches::default();
        let extracted = extract(&geometry, &mut caches).expect("mesh extracts");

        assert_eq!(extracted.polygon_normals.len(), 1);
        // The ECEF position of a point on the ellipsoid surface is itself an outward
        // radial direction, so an outward normal has a positive dot with it.
        let normal = extracted.polygon_normals[0];
        for &vertex in &extracted.ecef_vertices {
            assert!(
                dot(normal, vertex) > 0.0,
                "normal {normal:?} should point outward at {vertex:?}"
            );
        }
    }
}
