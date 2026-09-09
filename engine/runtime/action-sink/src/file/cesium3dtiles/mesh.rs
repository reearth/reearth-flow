use earcut::{utils3d::project3d_to_2d, Earcut};
use flatgeom::MultiPolygon;
use indexmap::IndexSet;
use nusamai_projection::cartesian::geodetic_to_geocentric;
use nusamai_projection::ellipsoid::Ellipsoid;
use reearth_flow_gltf::calculate_normal;
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_types::geometry::{CityGmlGeometry, GeometryType};
use reearth_flow_types::material::{self, Material, X3DMaterial};

use super::appearance::{self, ResolvedMaterial};

/// A `CityGmlGeometry`'s polygons, triangulated and reprojected, merged into
/// one combined mesh (index-offset concatenation, no cross-polygon vertex
/// welding — later stages weld as needed).
#[derive(Default)]
pub(super) struct ExtractedMesh {
    /// Vertex positions in ECEF (WGS84 geocentric), metres.
    pub(super) ecef_vertices: Vec<[f64; 3]>,
    /// The same vertices in WGS84 geographic (lng, lat, height); degrees/metres.
    pub(super) geographic_vertices: Vec<[f64; 3]>,
    /// Triangle index triples, parallel to both vertex arrays above.
    pub(super) indices: Vec<[u32; 3]>,
    /// Each source polygon's flat normal (one entry per polygon, not per
    /// triangle); `polygon_tris[i]` is how many triangles polygon `i` expanded into.
    pub(super) polygon_normals: Vec<[f64; 3]>,
    /// Output triangle count per source polygon, parallel to `polygon_normals`.
    pub(super) polygon_tris: Vec<u32>,
    /// Material palette (empty when the geometry carries no appearance);
    /// `triangle_material` indexes it.
    pub(super) materials: Vec<ResolvedMaterial>,
    /// Per output-triangle palette index (parallel to `indices`); `None` =
    /// unbound face, which renders with the writer's default material.
    pub(super) triangle_material: Vec<Option<u32>>,
    /// Per output-corner base-map UV, length `3 * indices.len()`; `[0.0, 0.0]`
    /// where the triangle is untextured.
    pub(super) corner_uv: Vec<[f64; 2]>,
}

/// Extract and triangulate every polygon (Solid/Surface/Triangle) in
/// `city_gml`, reprojecting to ECEF. Returns `None` when nothing was found.
pub(super) fn extract(city_gml: &CityGmlGeometry) -> Option<ExtractedMesh> {
    let ellipsoid = nusamai_projection::ellipsoid::wgs84();
    let default_material = X3DMaterial::default();
    let mut material_set: IndexSet<Material> = IndexSet::new();
    let mut mesh = ExtractedMesh::default();
    let mut earcutter = Earcut::new();

    for entry in &city_gml.gml_geometries {
        if !matches!(
            entry.ty,
            GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle
        ) {
            continue;
        }
        let base = entry.pos as usize;
        for (i, poly) in entry.polygons.iter().enumerate() {
            let global_idx = base + i;
            let Some(uv_poly) = city_gml.polygon_uvs.0.get(global_idx) else {
                tracing::warn!(
                    "Cesium3DTilesWriter: polygon {global_idx} has no UV entry; skipping"
                );
                continue;
            };

            let mat_idx_src = city_gml
                .polygon_materials
                .get(global_idx)
                .copied()
                .flatten();
            let tex_idx_src = city_gml.polygon_textures.get(global_idx).copied().flatten();
            let material = Material {
                base_color: mat_idx_src
                    .and_then(|idx| city_gml.materials.get(idx as usize))
                    .unwrap_or(&default_material)
                    .diffuse_color
                    .into(),
                base_texture: tex_idx_src
                    .and_then(|idx| city_gml.textures.get(idx as usize))
                    .map(|tex| material::Texture {
                        uri: tex.uri.clone(),
                    }),
            };
            let (mat_idx, _) = material_set.insert_full(material);

            extract_polygon(
                poly,
                uv_poly,
                mat_idx as u32,
                &ellipsoid,
                &mut earcutter,
                &mut mesh,
            );
        }
    }

    mesh.materials = appearance::resolve(&material_set);
    if mesh.ecef_vertices.is_empty() {
        None
    } else {
        Some(mesh)
    }
}

/// Reproject one polygon's rings to ECEF, triangulate via earcut, and append
/// the result (vertices, triangles, flat normal, per-corner UV) to `mesh`.
fn extract_polygon(
    poly: &Polygon3D<f64>,
    uv_poly: &Polygon2D<f64>,
    mat_idx: u32,
    ellipsoid: &Ellipsoid,
    earcutter: &mut Earcut<f64>,
    mesh: &mut ExtractedMesh,
) {
    let flat_poly: flatgeom::Polygon3 = poly.clone().into();
    let flat_uv: flatgeom::Polygon2 = uv_poly.clone().into();

    // Built in parallel, ring by ring (closed), so `geo_points[i]` is the
    // geographic counterpart of `local_poly.raw_coords()[i]` (ECEF+uv) below.
    let mut local: MultiPolygon<'static, [f64; 5]> = MultiPolygon::new();
    let mut geo_points: Vec<[f64; 3]> = Vec::new();
    let mut ring_buf: Vec<[f64; 5]> = Vec::new();
    for (ri, (ring, uv_ring)) in flat_poly.rings().zip(flat_uv.rings()).enumerate() {
        ring.iter_closed()
            .zip(uv_ring.iter_closed())
            .for_each(|(c, uv)| {
                let (x, y, z) = geodetic_to_geocentric(ellipsoid, c[0], c[1], c[2]);
                ring_buf.push([x, y, z, uv[0], uv[1]]);
                geo_points.push([c[0], c[1], c[2]]);
            });
        if ri == 0 {
            local.add_exterior(ring_buf.drain(..));
        } else {
            local.add_interior(ring_buf.drain(..));
        }
    }
    let Some(local_poly) = local.iter().next() else {
        return;
    };

    let Some((nx, ny, nz)) =
        calculate_normal(local_poly.exterior().iter().map(|v| [v[0], v[1], v[2]]))
    else {
        return;
    };

    let num_outer_points = local_poly
        .hole_indices()
        .first()
        .map_or(local_poly.raw_coords().len(), |&v| v as usize);
    let mut buf3d: Vec<[f64; 3]> = Vec::new();
    let mut buf2d: Vec<[f64; 2]> = Vec::new();
    let mut index_buf: Vec<u32> = Vec::new();
    buf3d.extend(local_poly.raw_coords().iter().map(|c| [c[0], c[1], c[2]]));

    if !project3d_to_2d(&buf3d, num_outer_points, &mut buf2d) {
        return;
    }
    earcutter.earcut(
        buf2d.iter().cloned(),
        local_poly.hole_indices(),
        &mut index_buf,
    );
    if index_buf.is_empty() {
        return;
    }

    let base = mesh.ecef_vertices.len() as u32;
    let raw = local_poly.raw_coords();
    for &idx in &index_buf {
        let [x, y, z, u, v] = raw[idx as usize];
        mesh.ecef_vertices.push([x, y, z]);
        mesh.geographic_vertices.push(geo_points[idx as usize]);
        mesh.corner_uv.push([u, v]);
    }
    let tri_count = index_buf.len() / 3;
    for t in 0..tri_count {
        let o = base + (t * 3) as u32;
        mesh.indices.push([o, o + 1, o + 2]);
        mesh.triangle_material.push(Some(mat_idx));
    }
    mesh.polygon_normals.push([nx, ny, nz]);
    mesh.polygon_tris.push(tri_count as u32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::coordinate::{Coordinate2D, Coordinate3D};
    use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
    use reearth_flow_types::geometry::GmlGeometry;
    use nusamai_citygml::Color;
    use reearth_flow_types::material::Texture;

    fn quad_feature(with_texture: bool) -> CityGmlGeometry {
        // A flat, roughly-1m-square quad near Tokyo (lng/lat degrees, height metres).
        let base_lng = 139.767;
        let base_lat = 35.681;
        let d = 0.00001; // ~1m at this latitude
        let exterior = LineString3D::new(vec![
            Coordinate3D::new__(base_lng, base_lat, 0.0),
            Coordinate3D::new__(base_lng + d, base_lat, 0.0),
            Coordinate3D::new__(base_lng + d, base_lat + d, 0.0),
            Coordinate3D::new__(base_lng, base_lat + d, 0.0),
        ]);
        let polygon = Polygon3D::new(exterior, vec![]);

        let uv_exterior = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(1.0, 1.0),
            Coordinate2D::new_(0.0, 1.0),
        ]);
        let uv_polygon = Polygon2D::new(uv_exterior, vec![]);

        let (materials, textures, polygon_materials, polygon_textures) = if with_texture {
            (
                vec![X3DMaterial {
                    diffuse_color: Color::new(0.5, 0.5, 0.5),
                    specular_color: Color::new(0.04, 0.04, 0.04),
                    ambient_intensity: 0.9,
                }],
                vec![Texture {
                    uri: url::Url::from_file_path("/tmp/texture.png").unwrap(),
                }],
                vec![Some(0)],
                vec![Some(0)],
            )
        } else {
            (vec![], vec![], vec![None], vec![None])
        };

        CityGmlGeometry {
            gml_geometries: vec![GmlGeometry {
                id: None,
                ty: GeometryType::Surface,
                gml_trait: None,
                lod: Some(2),
                pos: 0,
                len: 1,
                polygons: vec![polygon],
                line_strings: vec![],
                points: vec![],
                feature_id: None,
                feature_type: None,
            }],
            materials,
            textures,
            polygon_materials,
            polygon_textures,
            polygon_uvs: reearth_flow_geometry::types::multi_polygon::MultiPolygon2D::new(vec![
                uv_polygon,
            ]),
        }
    }

    #[test]
    fn extracts_one_quad_into_two_triangles() {
        let city_gml = quad_feature(true);
        let mesh = extract(&city_gml).expect("mesh extracted");

        assert_eq!(mesh.indices.len(), 2, "one quad earcuts into two triangles");
        assert_eq!(mesh.polygon_tris, vec![2]);
        assert_eq!(mesh.polygon_normals.len(), 1);
        assert_eq!(mesh.ecef_vertices.len(), 6, "no cross-triangle dedup");
        assert_eq!(mesh.geographic_vertices.len(), 6);
        assert_eq!(mesh.corner_uv.len(), 6);
        assert_eq!(mesh.triangle_material, vec![Some(0), Some(0)]);

        // ECEF magnitude should be close to Earth's radius.
        for v in &mesh.ecef_vertices {
            let mag = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            assert!(
                (6_300_000.0..6_400_000.0).contains(&mag),
                "unexpected ECEF magnitude {mag}"
            );
        }

        assert_eq!(mesh.materials.len(), 1);
        assert!(mesh.materials[0].base_texture.is_some());
    }

    #[test]
    fn untextured_quad_has_no_material_but_still_extracts() {
        let city_gml = quad_feature(false);
        let mesh = extract(&city_gml).expect("mesh extracted");

        assert_eq!(mesh.indices.len(), 2);
        assert_eq!(mesh.materials.len(), 1, "one bound color-only material");
        assert!(mesh.materials[0].base_texture.is_none());
    }
}
