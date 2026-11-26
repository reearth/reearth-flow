//! Point clouds as tile content: extraction to ECEF and the per-cell points
//! glb. Points never share a glb with meshes, since Draco recompresses mesh
//! glbs whole and cannot carry a `POINTS` primitive's meshopt buffers.

use indexmap::IndexMap;
use reearth_flow_geometry::ops::reproject::transform_coords_3d;
use reearth_flow_geometry::point_cloud::PointCloud;
use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
use reearth_flow_gltf::next::glb;
use reearth_flow_gltf::next::metadata::{self, ColumnStats, MetadataOptions};
use reearth_flow_gltf::next::points::{PointEncoding, Points, push_points};
use reearth_flow_types::Feature;

use super::color_material;
use super::mesh::{ExtractCaches, WGS84_GEOCENTRIC, WGS84_GEOGRAPHIC, source_crs};
use super::primitive::{DEFAULT_MATERIAL, MaterialFactors};
use super::quadtree::GeoBox;

/// Every point cloud of one feature, merged.
pub(super) struct ExtractedPoints {
    /// Positions in ECEF (WGS84 geocentric), metres.
    pub(super) ecef: Vec<[f64; 3]>,
    /// Geographic extent of the points.
    pub(super) geo_box: GeoBox,
    /// Linear RGB per point, parallel to `ecef`; `None` if no cloud has colour.
    pub(super) colors: Option<Vec<[u16; 3]>>,
}

/// Linear RGB of points with no colour, matching the writer's default material.
fn default_color() -> [u16; 3] {
    let [r, g, b, _] = DEFAULT_MATERIAL.base_color_factor;
    [r, g, b].map(|c| (c * u16::MAX as f32).round() as u16)
}

/// Extract and reproject every `PointCloud` reachable from `geometry`, merged
/// into one. File-backed clouds are read here. `None` when none was found or
/// every one failed (each failure is `tracing::warn!`ed).
pub(super) fn extract(geometry: &Geometry, caches: &mut ExtractCaches) -> Option<ExtractedPoints> {
    let mut clouds = Vec::new();
    collect_geometry(geometry, &mut clouds);

    let mut ecef = Vec::new();
    let mut geo_box: Option<GeoBox> = None;
    let mut colors: Vec<Option<Vec<[u16; 3]>>> = Vec::new();
    let mut lengths = Vec::new();
    for cloud in clouds {
        let Some(one) = extract_one(cloud, caches) else {
            continue;
        };
        geo_box = Some(match geo_box {
            Some(b) => b.union(one.geo_box),
            None => one.geo_box,
        });
        lengths.push(one.ecef.len());
        ecef.extend(one.ecef);
        colors.push(one.colors);
    }
    let geo_box = geo_box?;
    let colors = colors.iter().any(Option::is_some).then(|| {
        colors
            .into_iter()
            .zip(lengths)
            .flat_map(|(c, n)| c.unwrap_or_else(|| vec![default_color(); n]))
            .collect()
    });
    Some(ExtractedPoints {
        ecef,
        geo_box,
        colors,
    })
}

fn collect_geometry<'a>(geometry: &'a Geometry, out: &mut Vec<&'a PointCloud>) {
    match geometry {
        Geometry::GeometryCollection(gc) => {
            for member in gc.members() {
                collect_geometry(member, out);
            }
        }
        Geometry::Euclidean3D(e) => collect_euclidean3d(e, out),
        _ => {}
    }
}

fn collect_euclidean3d<'a>(geometry: &'a Euclidean3DGeometry, out: &mut Vec<&'a PointCloud>) {
    match geometry {
        Euclidean3DGeometry::Collection(c) => {
            for member in c.members() {
                collect_euclidean3d(member, out);
            }
        }
        Euclidean3DGeometry::PointCloud(cloud) => out.push(&**cloud),
        _ => {}
    }
}

/// Read, reproject and colour one cloud.
fn extract_one(cloud: &PointCloud, caches: &mut ExtractCaches) -> Option<ExtractedPoints> {
    let source_crs = source_crs(cloud.frame())?;
    let loaded;
    let cloud = if cloud.is_loaded() {
        cloud
    } else {
        let mut owned = cloud.clone();
        if let Err(e) = owned.load() {
            tracing::warn!("Cesium3DTilesWriter: failed to read point cloud: {e}");
            return None;
        }
        loaded = owned;
        &loaded
    };
    let vertices: Vec<[f64; 3]> = cloud
        .vertices()
        .expect("a loaded point cloud has its points in memory")
        .collect();
    if vertices.is_empty() {
        return None;
    }

    let mut geographic = vertices.clone();
    if let Err(e) = transform_coords_3d(
        &mut caches.geographic,
        source_crs,
        WGS84_GEOGRAPHIC,
        &mut geographic,
    ) {
        tracing::warn!(
            "Cesium3DTilesWriter: failed to reproject points to WGS84 geographic: {e:?}"
        );
        return None;
    }
    let mut ecef = vertices;
    if let Err(e) = transform_coords_3d(
        &mut caches.geocentric,
        source_crs,
        WGS84_GEOCENTRIC,
        &mut ecef,
    ) {
        tracing::warn!("Cesium3DTilesWriter: failed to reproject points to ECEF: {e:?}");
        return None;
    }

    let colors = cloud
        .rgb()
        .expect("a loaded point cloud has its points in memory")
        .map(|rgb| linear_colors(rgb.collect()));
    Some(ExtractedPoints {
        ecef,
        geo_box: GeoBox::of(&geographic)?,
        colors,
    })
}

/// LAS sRGB to linear, as glTF `COLOR_0` is linear. A cloud whose components
/// never exceed 255 is taken to store 8-bit colour, as many LAS writers do.
fn linear_colors(srgb: Vec<[u16; 3]>) -> Vec<[u16; 3]> {
    let eight_bit = srgb.iter().flatten().all(|&c| c <= 255);
    let full = if eight_bit { 255.0 } else { u16::MAX as f64 };
    srgb.into_iter()
        .map(|rgb| {
            rgb.map(|c| {
                let c = c as f64 / full;
                let linear = if c <= 0.04045 {
                    c / 12.92
                } else {
                    ((c + 0.055) / 1.055).powf(2.4)
                };
                (linear * u16::MAX as f64).round() as u16
            })
        })
        .collect()
}

/// Render one cell's point clouds to a glb: a single `POINTS` primitive with
/// each point's feature ID pointing at its feature's property-table row.
pub(super) fn build_cell_glb(
    members: &[&(&Feature, ExtractedPoints)],
    options: MetadataOptions,
    encoding: PointEncoding,
) -> (Vec<u8>, IndexMap<String, ColumnStats>) {
    let features: Vec<&Feature> = members.iter().map(|(f, _)| *f).collect();
    let table = metadata::build_table(&features, options);

    let count: usize = members.iter().map(|(_, p)| p.ecef.len()).sum();
    let mut sum = [0.0f64; 3];
    for (_, p) in members {
        for v in &p.ecef {
            for axis in 0..3 {
                sum[axis] += v[axis];
            }
        }
    }
    let origin = sum.map(|s| s / count as f64);

    let mut positions = Vec::with_capacity(count);
    let mut feature_ids = Vec::with_capacity(count);
    let has_colors = members.iter().any(|(_, p)| p.colors.is_some());
    let mut colors = Vec::with_capacity(if has_colors { count } else { 0 });
    for (row, (_, p)) in members.iter().enumerate() {
        // Same Z-up -> Y-up pre-rotation as the mesh content.
        positions.extend(
            p.ecef
                .iter()
                .map(|v| [v[0] - origin[0], v[2] - origin[2], -(v[1] - origin[1])]),
        );
        feature_ids.extend(std::iter::repeat_n(row as u32, p.ecef.len()));
        if has_colors {
            match &p.colors {
                Some(c) => colors.extend_from_slice(c),
                None => colors.extend(std::iter::repeat_n(default_color(), p.ecef.len())),
            }
        }
    }

    // Vertex colours carry the colour, so the material must not tint them.
    let material = if has_colors {
        color_material(MaterialFactors {
            base_color_factor: [1.0; 4],
            ..DEFAULT_MATERIAL
        })
    } else {
        color_material(DEFAULT_MATERIAL)
    };
    let mut builder = glb::Builder::new();
    let pushed = push_points(
        &mut builder,
        &Points {
            positions: &positions,
            feature_ids: &feature_ids,
            feature_count: members.len(),
            colors: has_colors.then_some(colors.as_slice()),
        },
        material,
        encoding,
    );
    let stats = metadata::encode(&table, &mut builder, &[pushed.handle]);
    let gltf_origin = [origin[0], origin[2], -origin[1]];
    let translation = std::array::from_fn(|axis| gltf_origin[axis] + pushed.offset[axis]);
    (builder.build_scaled(translation, pushed.scale), stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eight_bit_colour_is_widened_before_linearizing() {
        let wide = linear_colors(vec![[255 * 257, 128 * 257, 0]]);
        let narrow = linear_colors(vec![[255, 128, 0]]);
        assert_eq!(wide, narrow);
        assert_eq!(wide[0][0], u16::MAX);
        assert_eq!(wide[0][2], 0);
        // sRGB 128/255 is about 0.2158 linear.
        assert!((wide[0][1] as f64 / u16::MAX as f64 - 0.2158).abs() < 1e-3);
    }
}
