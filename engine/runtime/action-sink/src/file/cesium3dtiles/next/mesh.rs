//! Extracts a single renderable mesh from the new `Geometry` type and
//! reprojects it to WGS84 (for tileset placement) and ECEF (for glTF
//! vertices).
//!
//! Pass-1 scope: only a bare `PolygonMesh` leaf is supported — the dominant
//! shape PLATEAU CityGML produces (`Surface`/`CompositeSurface`/`Shell` map to
//! `PolygonMesh`). Every other leaf (`Point`, `LineString`, `Polygon`, `Solid`,
//! `Csg`, `PointCloud`, any collection) is rejected for now; broader leaf
//! coverage is a later pass.
//!
//! CRS: reprojection requires the leaf to already carry a real `Coordinate::Crs`
//! (hand-built fixtures set this directly at construction). There is no
//! fallback for `Coordinate::Euclidean` input yet — that only matters once a
//! reader that produces untagged geometry is in the loop, which is out of
//! scope for this pass.

use reearth_flow_geometry::coordinate::EpsgCode;
use reearth_flow_geometry::ops::{
    triangulation::Cache as TriangulationCache, Reproject, ReprojectionCache, Triangulate,
};
use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};

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
}

/// Extract and reproject the one supported leaf out of `geometry`.
///
/// Returns `None` (after a `tracing::warn!`) for any unsupported shape or any
/// reprojection failure (most commonly: the leaf's `Coordinate` isn't a `Crs`).
pub(super) fn extract(geometry: &Geometry) -> Option<ExtractedMesh> {
    if !matches!(
        geometry,
        Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(_))
    ) {
        tracing::warn!(
            "Cesium3DTilesWriter (new-geometry, pass 1): only a bare PolygonMesh is supported \
             today; skipping feature with geometry {geometry:?}"
        );
        return None;
    }

    let mut triangulation_cache = TriangulationCache::new();
    let triangulated = match geometry.clone().triangulate(&mut triangulation_cache) {
        Ok(g) => g,
        Err(e) => {
            tracing::warn!("Cesium3DTilesWriter: failed to triangulate PolygonMesh: {e:?}");
            return None;
        }
    };
    let Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(mesh)) = triangulated else {
        tracing::warn!(
            "Cesium3DTilesWriter: triangulating a PolygonMesh did not yield a TriangularMesh \
             (got {triangulated:?}); skipping feature"
        );
        return None;
    };

    let mut reproject_cache = ReprojectionCache::new();

    let mut geographic = (*mesh).clone();
    if let Err(e) = geographic.reproject(WGS84_GEOGRAPHIC, &mut reproject_cache) {
        tracing::warn!("Cesium3DTilesWriter: failed to reproject to WGS84 geographic: {e:?}");
        return None;
    }

    let mut ecef = (*mesh).clone();
    if let Err(e) = ecef.reproject(WGS84_GEOCENTRIC, &mut reproject_cache) {
        tracing::warn!("Cesium3DTilesWriter: failed to reproject to ECEF: {e:?}");
        return None;
    }

    Some(ExtractedMesh {
        ecef_vertices: ecef.vertices().to_vec(),
        geographic_vertices: geographic.vertices().to_vec(),
        indices: mesh.triangles().collect(),
    })
}
