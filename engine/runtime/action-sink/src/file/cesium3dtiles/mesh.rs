use earcut::{utils3d::project3d_to_2d, Earcut};
use flatgeom::MultiPolygon;
use indexmap::IndexSet;
use nusamai_projection::cartesian::geodetic_to_geocentric;
use nusamai_projection::ellipsoid::Ellipsoid;
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_gltf::calculate_normal;
use reearth_flow_types::geometry::{CityGmlGeometry, GeometryType};
use reearth_flow_types::material::{self, Material, X3DMaterial};

use super::appearance::{self, ResolvedMaterial};
use super::quadtree::GeoBox;

/// A feature's triangulated render geometry, ready to place into a cell glb.
#[derive(Default)]
pub(super) struct ExtractedMesh {
    /// Vertex positions in ECEF (WGS84 geocentric), metres. Vertices are
    /// welded within a polygon, never across polygons.
    pub(super) positions: Vec<[f64; 3]>,
    /// Triangle index triples into `positions`.
    pub(super) indices: Vec<[u32; 3]>,
    /// Output triangle count per source polygon.
    pub(super) polygon_tris: Vec<u32>,
    /// Each source polygon's flat ECEF normal, parallel to `polygon_tris`;
    /// empty when normals were not requested.
    pub(super) polygon_normals: Vec<[f32; 3]>,
    /// Each source polygon's index into `materials`, parallel to `polygon_tris`.
    pub(super) polygon_material: Vec<u32>,
    /// Material palette `polygon_material` indexes.
    pub(super) materials: Vec<ResolvedMaterial>,
    /// Per-vertex base-map UV (top-left origin), parallel to `positions`;
    /// empty when the feature has no textured polygon or UVs were not requested.
    pub(super) uvs: Vec<[f32; 2]>,
}

/// Which optional per-vertex/per-polygon channels [`extract`] keeps.
#[derive(Clone, Copy)]
pub(super) struct ExtractOptions {
    pub(super) normals: bool,
    pub(super) uvs: bool,
}

/// Extract and triangulate every polygon (Solid/Surface/Triangle) in
/// `city_gml`, reprojecting to ECEF. Returns the mesh with the geographic
/// extent of its emitted vertices, or `None` when nothing was found.
pub(super) fn extract(
    city_gml: &CityGmlGeometry,
    options: ExtractOptions,
) -> Option<(ExtractedMesh, GeoBox)> {
    let ellipsoid = nusamai_projection::ellipsoid::wgs84();
    let default_material = X3DMaterial::default();
    let mut material_set: IndexSet<Material> = IndexSet::new();
    let mut acc = Accumulator::default();
    let mut earcutter = Earcut::new();

    for entry in &city_gml.gml_geometries {
        if !matches!(
            entry.ty,
            GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle
        ) {
            tracing::warn!(
                "Cesium3DTilesWriter: {:?} geometry is not supported; skipping",
                entry.ty
            );
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
                &mut acc,
            );
        }
    }

    let bounds = acc.bounds?;
    let materials = appearance::resolve(&material_set);
    let textured = acc
        .polygon_material
        .iter()
        .any(|&m| materials[m as usize].base_texture.is_some());

    let polygon_normals = if options.normals {
        acc.polygon_normals
            .iter()
            .map(|n| n.map(|v| v as f32))
            .collect()
    } else {
        Vec::new()
    };
    let uvs = if options.uvs && textured {
        acc.uvs.iter().map(|uv| uv.map(|v| v as f32)).collect()
    } else {
        Vec::new()
    };

    let mesh = ExtractedMesh {
        positions: acc.positions,
        indices: acc.indices,
        polygon_tris: acc.polygon_tris,
        polygon_normals,
        polygon_material: acc.polygon_material,
        materials,
        uvs,
    };
    Some((mesh, bounds))
}

/// [`extract`]'s full-precision working buffers.
#[derive(Default)]
struct Accumulator {
    positions: Vec<[f64; 3]>,
    uvs: Vec<[f64; 2]>,
    indices: Vec<[u32; 3]>,
    polygon_tris: Vec<u32>,
    polygon_normals: Vec<[f64; 3]>,
    polygon_material: Vec<u32>,
    bounds: Option<GeoBox>,
}

/// Triangulate and reproject one polygon.
fn extract_polygon(
    poly: &Polygon3D<f64>,
    uv_poly: &Polygon2D<f64>,
    mat_idx: u32,
    ellipsoid: &Ellipsoid,
    earcutter: &mut Earcut<f64>,
    acc: &mut Accumulator,
) {
    let flat_poly: flatgeom::Polygon3 = poly.clone().into();
    let flat_uv: flatgeom::Polygon2 = uv_poly.clone().into();

    let mut local: MultiPolygon<'static, [f64; 5]> = MultiPolygon::new();
    let mut geo_points: Vec<[f64; 3]> = Vec::new();
    let mut ring_buf: Vec<[f64; 5]> = Vec::new();
    for (ri, (ring, uv_ring)) in flat_poly.rings().zip(flat_uv.rings()).enumerate() {
        ring.iter_closed()
            .zip(uv_ring.iter_closed())
            .for_each(|(c, uv)| {
                let (x, y, z) = geodetic_to_geocentric(ellipsoid, c[0], c[1], c[2]);
                // CityGML texture coordinates are bottom-left.
                ring_buf.push([x, y, z, uv[0], 1.0 - uv[1]]);
                // `c` is lon/lat/height, the order `geodetic_to_geocentric` takes;
                // `GeoBox::of` reads lat/lon/height (EPSG:4979's own axis order).
                geo_points.push([c[1], c[0], c[2]]);
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

    // Weld by ring vertex, in first-use order.
    let raw = local_poly.raw_coords();
    let mut welded = vec![u32::MAX; raw.len()];
    for tri in index_buf.chunks_exact(3) {
        let mut out = [0u32; 3];
        for (corner, &idx) in tri.iter().enumerate() {
            let idx = idx as usize;
            if welded[idx] == u32::MAX {
                welded[idx] = acc.positions.len() as u32;
                let [x, y, z, u, v] = raw[idx];
                acc.positions.push([x, y, z]);
                acc.uvs.push([u, v]);
                let point = GeoBox::of(&[geo_points[idx]]).expect("one point");
                acc.bounds = Some(acc.bounds.map_or(point, |b| b.union(point)));
            }
            out[corner] = welded[idx];
        }
        acc.indices.push(out);
    }
    let tri_count = index_buf.len() / 3;
    acc.polygon_normals.push([nx, ny, nz]);
    acc.polygon_tris.push(tri_count as u32);
    acc.polygon_material.push(mat_idx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use nusamai_citygml::Color;
    use reearth_flow_geometry::types::coordinate::{Coordinate2D, Coordinate3D};
    use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
    use reearth_flow_types::geometry::GmlGeometry;
    use reearth_flow_types::material::Texture;

    fn quad_feature(with_texture: bool) -> CityGmlGeometry {
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

    const OPTIONS: ExtractOptions = ExtractOptions {
        normals: true,
        uvs: true,
    };

    #[test]
    fn extracts_one_quad_into_two_triangles() {
        let city_gml = quad_feature(true);
        let (mesh, bounds) = extract(&city_gml, OPTIONS).expect("mesh extracted");

        assert_eq!(mesh.indices.len(), 2, "one quad earcuts into two triangles");
        assert_eq!(mesh.polygon_tris, vec![2]);
        assert_eq!(mesh.polygon_normals.len(), 1);
        assert_eq!(mesh.positions.len(), 4, "corners weld within the polygon");
        assert_eq!(mesh.uvs.len(), 4);
        assert_eq!(mesh.polygon_material, vec![0]);

        // lat/lon/height, the order `GeoBox::of` reads (EPSG:4979's own axis
        // order), not the lon-first order `geodetic_to_geocentric` takes.
        assert!((35.68..35.69).contains(&bounds.south));
        assert!((139.76..139.77).contains(&bounds.west));

        for v in &mesh.positions {
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
        let (mesh, _) = extract(&city_gml, OPTIONS).expect("mesh extracted");

        assert_eq!(mesh.indices.len(), 2);
        assert_eq!(mesh.materials.len(), 1, "one bound color-only material");
        assert!(mesh.materials[0].base_texture.is_none());
        assert!(mesh.uvs.is_empty(), "no textured polygon, no UVs kept");
    }

    #[test]
    fn optional_channels_are_dropped_when_not_requested() {
        let city_gml = quad_feature(true);
        let options = ExtractOptions {
            normals: false,
            uvs: false,
        };
        let (mesh, _) = extract(&city_gml, options).expect("mesh extracted");

        assert!(mesh.polygon_normals.is_empty());
        assert!(mesh.uvs.is_empty());
        assert_eq!(mesh.polygon_tris, vec![2]);
    }
}
