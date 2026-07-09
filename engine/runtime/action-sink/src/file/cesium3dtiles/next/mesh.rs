//! Extracts every reachable `PolygonMesh` leaf (bare, or as a `Solid`'s
//! boundary shell) from `Geometry`, and reprojects it to WGS84 (tileset
//! placement) and ECEF (glTF vertices).
//!
//! Reprojection assumes JGD2011 (EPSG:6697) as the source CRS for every leaf,
//! since the CityGML reader tags every leaf `CoordinateFrame::Euclidean`
//! regardless of its real frame. Each mesh's vertex buffer is reprojected
//! directly via `transform_coords_3d`.

use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::ops::reproject::transform_coords_3d;
use reearth_flow_geometry::ops::{triangulation::Cache as TriangulationCache, ReprojectionCache};
use reearth_flow_geometry::polygon_mesh::PolygonMesh3D;
use reearth_flow_geometry::solid::Shell;
use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};

/// JGD2011.
const ASSUMED_SOURCE_CRS: EpsgCode = EpsgCode::new(6697);
/// WGS84, 3D geographic (lon, lat, height) — used for the tileset's bounding region.
const WGS84_GEOGRAPHIC: EpsgCode = EpsgCode::new(4979);
/// WGS84, geocentric (ECEF) — used for glTF vertex positions.
const WGS84_GEOCENTRIC: EpsgCode = EpsgCode::new(4978);

pub(super) struct ExtractedMesh {
    /// Vertex positions in ECEF (WGS84 geocentric), metres.
    pub(super) ecef_vertices: Vec<[f64; 3]>,
    /// The same vertices in WGS84 geographic (lon, lat, height); degrees/metres.
    pub(super) geographic_vertices: Vec<[f64; 3]>,
    /// Triangle index triples, parallel to both vertex arrays above.
    pub(super) indices: Vec<[u32; 3]>,
    /// Each source polygon's flat normal, in polygon order — see
    /// `PolygonMesh3D::triangulate_with_normals`. Kept compact (one entry per
    /// *polygon*, not per triangle); `polygon_tris[i]` says how many
    /// consecutive entries of `indices` polygon `i` was split into.
    pub(super) polygon_normals: Vec<[f64; 3]>,
    /// Output triangle count per source polygon, parallel to `polygon_normals`.
    pub(super) polygon_tris: Vec<u32>,
}

/// Extract and reproject every `PolygonMesh` reachable from `geometry`, merged
/// into one combined mesh (index-offset concatenation).
///
/// Returns `None` when nothing was found, or everything found failed to
/// triangulate/reproject (each failure is `tracing::warn!`ed individually).
pub(super) fn extract(geometry: &Geometry) -> Option<ExtractedMesh> {
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
        let Some(extracted) = extract_one(mesh) else {
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
/// shells, collecting every `PolygonMesh` found. A `Solid` shell has no
/// `Coordinate` of its own, so a placeholder frame is used — this writer
/// reprojects raw vertex buffers directly and ignores it anyway.
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
                        out.push(PolygonMesh3D::new(CoordinateFrame::Euclidean, data.clone()))
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

/// Triangulate and reproject one `PolygonMesh`.
fn extract_one(mut mesh: PolygonMesh3D) -> Option<ExtractedMesh> {
    let mut triangulation_cache = TriangulationCache::new();
    let result = mesh.triangulate_with_normals(&mut triangulation_cache);
    let (mesh, polygon_normals, polygon_tris) =
        (result.mesh, result.polygon_normals, result.polygon_tris);

    let mut reproject_cache = ReprojectionCache::new();

    let mut geographic_vertices = mesh.vertices().to_vec();
    if let Err(e) = transform_coords_3d(
        &mut reproject_cache,
        ASSUMED_SOURCE_CRS,
        WGS84_GEOGRAPHIC,
        &mut geographic_vertices,
    ) {
        tracing::warn!("Cesium3DTilesWriter: failed to reproject to WGS84 geographic: {e:?}");
        return None;
    }

    let mut ecef_vertices = mesh.vertices().to_vec();
    if let Err(e) = transform_coords_3d(
        &mut reproject_cache,
        ASSUMED_SOURCE_CRS,
        WGS84_GEOCENTRIC,
        &mut ecef_vertices,
    ) {
        tracing::warn!("Cesium3DTilesWriter: failed to reproject to ECEF: {e:?}");
        return None;
    }

    Some(ExtractedMesh {
        ecef_vertices,
        geographic_vertices,
        indices: mesh.triangles().collect(),
        polygon_normals,
        polygon_tris,
    })
}
