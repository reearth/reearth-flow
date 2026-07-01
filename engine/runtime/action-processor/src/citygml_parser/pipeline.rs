use std::collections::HashSet;
use std::sync::Arc;

use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_types::{
    AttributeValue, CityGmlGeometry, Feature, Geometry, GeometryType, GeometryValue, GmlGeometry,
    CITYGML_PARENT_GML_ID_KEY, CITYGML_ROOT_GML_ID_KEY,
};
use url::Url;

use super::{
    codespace, flatten, geometry,
    parser::{self, Parser},
    utils::{gml_id_attr, XmlNode},
    xlink,
};

/// Resolves the parsed document (xlink + codespace) and returns one feature per top-level city
/// object, or — when `extract_tags` is non-empty — one feature per matching flattened node.
#[cfg(not(feature = "new-geometry"))]
pub fn build_features(parser: Parser, extract_tags: &HashSet<String>) -> Vec<Feature> {
    let (pending, raw_registry, ns_registry) = parser.finish();
    let mut codelist_resolver = codespace::CodelistResolver::new();
    let mut out = Vec::new();
    for feature_root in codespace::resolve(
        xlink::resolve(pending, &raw_registry),
        &mut codelist_resolver,
    ) {
        if extract_tags.is_empty() {
            out.push(build_feature(&feature_root));
        } else {
            let root_gml_id = gml_id_attr(&feature_root.attrs);

            for (node, parent_id) in flatten::extract(&feature_root, extract_tags, &ns_registry) {
                let mut feature = build_feature(&node);
                if let Some(id) = parent_id {
                    feature.insert(CITYGML_PARENT_GML_ID_KEY, AttributeValue::String(id));
                }
                if let Some(ref id) = root_gml_id {
                    feature.insert(CITYGML_ROOT_GML_ID_KEY, AttributeValue::String(id.clone()));
                }
                out.push(feature);
            }
        }
    }
    out
}

/// Parses one or more CityGML sources into a single document — so cross-file `xlink:href`
/// references resolve — and returns the extracted features.
#[cfg(not(feature = "new-geometry"))]
pub fn read_features<'a>(
    sources: impl IntoIterator<Item = (&'a [u8], &'a Url)>,
    extract_tags: &HashSet<String>,
) -> Result<Vec<Feature>, String> {
    let mut parser = Parser::new();
    for (bytes, source_url) in sources {
        parser.parse(bytes, source_url).map_err(|e| format!("{e}"))?;
    }
    Ok(build_features(parser, extract_tags))
}

#[cfg(not(feature = "new-geometry"))]
fn build_feature(node: &Arc<XmlNode>) -> Feature {
    let (stripped, raw_geoms) = geometry::extract_geometries(node);
    let mut feature = parser::to_feature(&stripped);
    if !raw_geoms.is_empty() {
        *feature.geometry_mut() = Geometry::with_value(GeometryValue::CityGmlGeometry(
            build_citygml_geometry(raw_geoms),
        ));
    }
    feature
}

// pos is assigned here; neutral appearance arrays prevent out-of-bounds access in downstream consumers.
fn build_citygml_geometry(raw: Vec<GmlGeometry>) -> CityGmlGeometry {
    let mut polygon_materials: Vec<Option<u32>> = Vec::new();
    let mut polygon_textures: Vec<Option<u32>> = Vec::new();
    let mut polygon_uvs: Vec<Polygon2D<f64>> = Vec::new();
    let mut current_pos: u32 = 0;
    let mut gml_geometries: Vec<GmlGeometry> = Vec::with_capacity(raw.len());

    for mut g in raw {
        if matches!(
            g.ty,
            GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle
        ) {
            g.pos = current_pos;
            current_pos += g.len;
            for poly in &g.polygons {
                polygon_materials.push(None);
                polygon_textures.push(None);
                polygon_uvs.push(neutral_uv_polygon(poly));
            }
        }
        gml_geometries.push(g);
    }

    CityGmlGeometry {
        gml_geometries,
        materials: Vec::new(),
        textures: Vec::new(),
        polygon_materials,
        polygon_textures,
        polygon_uvs: MultiPolygon2D::new(polygon_uvs),
    }
}

fn neutral_uv_polygon(poly: &Polygon3D<f64>) -> Polygon2D<f64> {
    let ext = LineString2D::new(vec![[0.0f64, 0.0f64].into(); poly.exterior().0.len()]);
    let ints = poly
        .interiors()
        .iter()
        .map(|ring| LineString2D::new(vec![[0.0f64, 0.0f64].into(); ring.0.len()]))
        .collect();
    Polygon2D::new(ext, ints)
}
