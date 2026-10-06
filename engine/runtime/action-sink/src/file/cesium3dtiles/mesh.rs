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
    /// Per-vertex base-map UV (top-left origin), parallel to `positions`; a
    /// tiling polygon's UVs are shifted by its [`repeat_offset`]. Empty when
    /// the feature has no textured polygon or UVs were not requested.
    pub(super) uvs: Vec<[f32; 2]>,
    /// Whether each source polygon's UVs tile (see [`reearth_flow_atlas::tiles`]),
    /// parallel to `polygon_tris`; empty exactly when `uvs` is.
    pub(super) polygon_tiles: Vec<bool>,
}

/// Which optional per-vertex/per-polygon channels [`extract`] keeps.
#[derive(Clone, Copy)]
pub(super) struct ExtractOptions {
    pub(super) normals: bool,
    pub(super) uvs: bool,
    /// How far outside `[0, 1]` a UV may drift before its polygon tiles.
    pub(super) wrap_tolerance: f64,
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
                options.wrap_tolerance,
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
    let (uvs, polygon_tiles) = if options.uvs && textured {
        let uvs = acc.uvs.iter().map(|uv| uv.map(|v| v as f32)).collect();
        (uvs, acc.polygon_tiles)
    } else {
        (Vec::new(), Vec::new())
    };

    let mesh = ExtractedMesh {
        positions: acc.positions,
        indices: acc.indices,
        polygon_tris: acc.polygon_tris,
        polygon_normals,
        polygon_material: acc.polygon_material,
        materials,
        uvs,
        polygon_tiles,
    };
    Some((mesh, bounds))
}

/// The whole-repeat offset of a polygon's UVs: the floor of their minimum per
/// axis.
pub(super) fn repeat_offset(uvs: &[[f64; 2]]) -> [f64; 2] {
    let min = uvs
        .iter()
        .fold([f64::INFINITY; 2], |m, [u, v]| [m[0].min(*u), m[1].min(*v)]);
    min.map(|m| if m.is_finite() { m.floor() } else { 0.0 })
}

/// [`extract`]'s full-precision working buffers.
#[derive(Default)]
struct Accumulator {
    positions: Vec<[f64; 3]>,
    uvs: Vec<[f64; 2]>,
    polygon_tiles: Vec<bool>,
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
    wrap_tolerance: f64,
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
    let first_vertex = acc.positions.len();
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
    // A tiling polygon samples a repeating page, where a whole-repeat shift
    // is invisible; shifting here keeps its UVs small enough for f32. The
    // subtraction is exact: the offset is an integer no greater than any of
    // the UVs it is taken from.
    let polygon_uvs = &mut acc.uvs[first_vertex..];
    let tiles = reearth_flow_atlas::tiles(polygon_uvs.iter(), wrap_tolerance);
    if tiles {
        let offset = repeat_offset(polygon_uvs);
        for uv in polygon_uvs {
            *uv = [uv[0] - offset[0], uv[1] - offset[1]];
        }
    }
    acc.polygon_tiles.push(tiles);
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
        wrap_tolerance: 0.1,
    };

    /// `quad_feature(true)` with its UV ring replaced by `source`.
    fn quad_with_uvs(source: &[[f64; 2]; 4]) -> CityGmlGeometry {
        let mut city_gml = quad_feature(true);
        let uv_exterior = LineString2D::new(
            source
                .iter()
                .map(|&[u, v]| Coordinate2D::new_(u, v))
                .collect(),
        );
        city_gml.polygon_uvs = reearth_flow_geometry::types::multi_polygon::MultiPolygon2D::new(
            vec![Polygon2D::new(uv_exterior, vec![])],
        );
        city_gml
    }

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
    fn repeat_offset_is_the_floor_of_the_minimum_per_axis() {
        assert_eq!(
            repeat_offset(&[[133_226.7, 23.3], [133_228.6, 27.7]]),
            [133_226.0, 23.0]
        );
        assert_eq!(
            repeat_offset(&[[-266_459.1, 74.5], [-266_455.5, 80.6]]),
            [-266_460.0, 74.0]
        );
        assert_eq!(repeat_offset(&[]), [0.0, 0.0]);
    }

    // A tiling polygon far from the origin is shifted by its whole-repeat
    // offset, so its f32 UVs keep the sub-texel fraction.
    #[test]
    fn distant_tiling_uvs_keep_their_fraction() {
        let source = [
            [133_226.7, 23.3],
            [133_228.6, 23.3],
            [133_228.6, 27.7],
            [133_226.7, 27.7],
        ];
        let (mesh, _) = extract(&quad_with_uvs(&source), OPTIONS).expect("mesh extracted");

        assert_eq!(mesh.polygon_tiles, vec![true]);
        // CityGML's `v` is bottom-up; the mesh stores `1 - v`, so the offset
        // is `[floor(133_226.7), floor(1 - 27.7)]`.
        let offset = [133_226.0, -27.0];
        for &[u, v] in &mesh.uvs {
            assert!(
                source.iter().any(|s| {
                    (s[0] - offset[0] - u as f64).abs() < 1e-6
                        && (1.0 - s[1] - offset[1] - v as f64).abs() < 1e-6
                }),
                "uv ({u}, {v}) drifted from every shifted source corner"
            );
        }
    }

    // A polygon drifting within `wrap_tolerance` of `[0, 1]` does not tile and
    // keeps its UVs as they are, so its packed region does not move.
    #[test]
    fn uvs_within_tolerance_are_not_shifted() {
        let source = [[-0.05, 0.0], [0.5, 0.0], [0.5, 1.05], [-0.05, 1.05]];
        let (mesh, _) = extract(&quad_with_uvs(&source), OPTIONS).expect("mesh extracted");

        assert_eq!(mesh.polygon_tiles, vec![false]);
        let min_u = mesh
            .uvs
            .iter()
            .map(|uv| uv[0])
            .fold(f32::INFINITY, f32::min);
        assert_eq!(min_u, -0.05);
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
            ..OPTIONS
        };
        let (mesh, _) = extract(&city_gml, options).expect("mesh extracted");

        assert!(mesh.polygon_normals.is_empty());
        assert!(mesh.uvs.is_empty());
        assert_eq!(mesh.polygon_tris, vec![2]);
    }
}
