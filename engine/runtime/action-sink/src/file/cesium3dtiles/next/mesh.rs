use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::ops::reproject::transform_coords_3d;
use reearth_flow_geometry::ops::{
    triangulation::Cache as TriangulationCache, Reproject, ReprojectionCache,
};
use reearth_flow_geometry::polygon_mesh::PolygonMesh3D;
use reearth_flow_geometry::solid::Shell;
use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};

use super::appearance::{self, ResolvedMaterial};

/// WGS84, 3D geographic (lat, lon, height) — used for the tileset's bounding region.
const WGS84_GEOGRAPHIC: EpsgCode = EpsgCode::new(4979);
/// WGS84, geocentric (ECEF) — used for glTF vertex positions.
const WGS84_GEOCENTRIC: EpsgCode = EpsgCode::new(4978);

/// Per-file scratch shared across every mesh, so repeated extraction amortizes
/// its PROJ setup and earcut allocations.
///
/// One `ReprojectionCache` per target CRS: each only ever sees a single
/// (from, to) pair, so its cached PROJ transform is never evicted (a cache
/// handed both target CRSes would thrash between them every call). The
/// triangulation cache reuses earcut's arenas and index/vertex scratch across
/// meshes.
#[derive(Default)]
pub(super) struct ExtractCaches {
    geographic: ReprojectionCache,
    geocentric: ReprojectionCache,
    triangulation: TriangulationCache,
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
    /// Material palette resolved under the default theme, front side (empty when
    /// the mesh carries no appearance); `triangle_material` indexes it.
    pub(super) materials: Vec<ResolvedMaterial>,
    /// Per output-triangle palette index (parallel to `indices`); `None` =
    /// unbound face, which renders with the writer's default material.
    pub(super) triangle_material: Vec<Option<u32>>,
    /// Per output-corner base-map UV, length `3 * indices.len()`; `[0.0, 0.0]`
    /// where the triangle is untextured.
    pub(super) corner_uv: Vec<[f64; 2]>,
}

/// Extract and reproject every `PolygonMesh` reachable from `geometry`, merged
/// into one combined mesh (index-offset concatenation). `caches` should be
/// shared across every feature in the file.
///
/// Returns `None` when nothing was found, or everything found failed to
/// triangulate/reproject (each failure is `tracing::warn!`ed individually).
pub(super) fn extract(geometry: &Geometry, caches: &mut ExtractCaches) -> Option<ExtractedMesh> {
    let mut meshes = Vec::new();
    collect_geometry(geometry, &mut meshes);

    let mut combined = ExtractedMesh {
        ecef_vertices: Vec::new(),
        geographic_vertices: Vec::new(),
        indices: Vec::new(),
        polygon_normals: Vec::new(),
        polygon_tris: Vec::new(),
        materials: Vec::new(),
        triangle_material: Vec::new(),
        corner_uv: Vec::new(),
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
        // Offset this mesh's binding indices into the running merged palette.
        let material_base = combined.materials.len() as u32;
        combined.triangle_material.extend(
            extracted
                .triangle_material
                .into_iter()
                .map(|opt| opt.map(|m| m + material_base)),
        );
        combined.materials.extend(extracted.materials);
        combined.corner_uv.extend(extracted.corner_uv);
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
fn extract_one(mut mesh: PolygonMesh3D, caches: &mut ExtractCaches) -> Option<ExtractedMesh> {
    let source_crs = source_crs(mesh.frame())?;

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

    if let Err(e) = mesh.reproject(WGS84_GEOCENTRIC, &mut caches.geocentric) {
        tracing::warn!("Cesium3DTilesWriter: failed to reproject to ECEF: {e:?}");
        return None;
    }

    let result = match mesh.triangulate_with_normals(&mut caches.triangulation) {
        Ok(result) => result,
        Err(e) => {
            tracing::warn!("Cesium3DTilesWriter: failed to triangulate mesh: {e:?}");
            return None;
        }
    };

    let indices: Vec<[u32; 3]> = result.mesh.triangles().collect();
    // Resolve appearance onto the triangulated mesh, or fall back to a fully
    // unbound / untextured mesh so the merge in `extract` stays uniform.
    let (materials, triangle_material, corner_uv) = match result.mesh.appearance() {
        Some(app) => {
            let resolved = appearance::resolve(app, indices.len());
            (
                resolved.materials,
                resolved.triangle_material,
                resolved.corner_uv,
            )
        }
        None => (
            Vec::new(),
            vec![None; indices.len()],
            vec![[0.0, 0.0]; indices.len() * 3],
        ),
    };

    Some(ExtractedMesh {
        ecef_vertices: result.mesh.vertices().to_vec(),
        geographic_vertices,
        indices,
        polygon_normals: result.polygon_normals,
        polygon_tris: result.polygon_tris,
        materials,
        triangle_material,
        corner_uv,
    })
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::polygon_mesh::{PolygonMesh3D, PolygonMesh3DData};

    use super::*;

    fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    // A face whose canonical orientation is outward, stored in a lat-first frame
    // (EPSG:4979, orientation sign -1), must emit an ECEF normal that points away
    // from the earth's centre. Triangulating in ECEF gives this for free: the
    // reflected source winding is cancelled by the reprojection, so the
    // right-hand-rule normal comes out outward.
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

        let mut caches = ExtractCaches::default();
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
