//! The legacy world's half of the converter seam: `CityGmlGeometry` in, the
//! shared [`super::model`] out.
//!
//! Its behaviour is fixed by what the legacy build already emits, so nothing
//! here narrows or widens: interior shells were discarded at read time and can
//! never reach it, points are dropped, triangles fold into `MultiSurface`, and
//! the material/texture palettes are the feature's whole global arrays.

use reearth_flow_geometry::types::coordinate::Coordinate3D;
use reearth_flow_geometry::types::polygon::Polygon3D;
use reearth_flow_types::conversion::CrsCoverage;
use reearth_flow_types::geometry::{CityGmlGeometry, GeometryType, GeometryValue, GmlGeometry};
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::Feature;

use super::model::{
    AppearanceBundle, BoundingEnvelope, ConvertedCityObject, GeometryEntry, GmlElement, GmlSolid,
    GmlSurface, TextureRef, TextureSource,
};
use crate::errors::SinkError;

/// Convert one feature's geometry into the shared CityGML model.
///
/// The legacy world's CRS is a whole-feature EPSG rather than a per-leaf frame,
/// so no coverage is folded here: [`srs_name`] reads the feature field directly,
/// exactly as this writer always has.
pub fn convert_city_object(
    feature: &Feature,
    lod_mask: &LodMask,
) -> Result<ConvertedCityObject, SinkError> {
    let GeometryValue::CityGmlGeometry(ref geometry) = feature.geometry.value else {
        // A feature carrying some other geometry has never produced a city
        // object here; it is passed over, not reported.
        return Ok(ConvertedCityObject {
            geometries: Vec::new(),
            appearance: AppearanceBundle {
                materials: Vec::new(),
                textures: Vec::new(),
            },
            envelope: None,
            crs: CrsCoverage::NoCoordinates,
            textures: Vec::new(),
            omissions: Vec::new(),
        });
    };

    let (geometries, appearance) = convert_citygml_geometry(geometry, lod_mask);
    // Deliberately not filtered by LOD: this reproduces the envelope the legacy
    // build has always written, which is folded over every vertex of the
    // feature's geometry.
    let envelope = compute_envelope(geometry);
    let textures = geometry
        .textures
        .iter()
        .map(|texture| TextureRef {
            key: texture.uri.to_string(),
            source: TextureSource::Uri(texture.uri.clone()),
        })
        .collect();

    Ok(ConvertedCityObject {
        geometries,
        appearance,
        envelope,
        crs: CrsCoverage::NoCoordinates,
        textures,
        omissions: Vec::new(),
    })
}

/// The OGC CRS URI to declare, reproducing today's chain verbatim: the
/// `epsgCode` parameter, else the *first* feature's whole-geometry EPSG, else
/// EPSG:4326.
///
/// `coverage` is unused: it is folded over per-leaf frames, which the legacy
/// geometry model does not have. Changing this chain would change the legacy
/// build's output, which this port does not do.
pub fn srs_name(
    features: &[Feature],
    epsg_code: Option<u32>,
    _coverage: CrsCoverage,
) -> Result<String, SinkError> {
    Ok(epsg_code
        .or_else(|| {
            features
                .first()
                .and_then(|f| f.geometry.epsg)
                .map(|e| e as u32)
        })
        .map(|code| format!("http://www.opengis.net/def/crs/EPSG/0/{code}"))
        .unwrap_or_else(|| "http://www.opengis.net/def/crs/EPSG/0/4326".to_string()))
}

/// The shared model's coordinates are bare ordinate triples, so every legacy
/// ring crosses the seam through here.
fn ring_coords(coords: &[Coordinate3D<f64>]) -> Vec<[f64; 3]> {
    coords.iter().map(|c| [c.x, c.y, c.z]).collect()
}

impl From<&Polygon3D<f64>> for GmlSurface {
    fn from(polygon: &Polygon3D<f64>) -> Self {
        Self {
            id: None,
            exterior: ring_coords(&polygon.exterior().0),
            interiors: polygon
                .interiors()
                .iter()
                .map(|ring| ring_coords(&ring.0))
                .collect(),
            material_idx: None,
            texture_idx: None,
            uv_exterior: Vec::new(),
            uv_interiors: Vec::new(),
        }
    }
}

pub fn convert_citygml_geometry(
    geometry: &CityGmlGeometry,
    lod_filter: &LodMask,
) -> (Vec<GeometryEntry>, AppearanceBundle) {
    let need_appearance = !geometry.materials.is_empty() || !geometry.textures.is_empty();

    let entries = geometry
        .gml_geometries
        .iter()
        .filter_map(|gml_geom| {
            let lod = gml_geom.lod.unwrap_or(0);
            if !lod_filter.has_lod(lod) {
                return None;
            }
            // The model carries the wrapper's local name as a plain string, so
            // `PropertyType`'s `Display` — which is what the writer formatted
            // anyway — is applied here rather than leaking the type across.
            let property = gml_geom.gml_trait.as_ref().map(|t| t.property.to_string());
            convert_gml_geometry(gml_geom, geometry, need_appearance).map(|elem| GeometryEntry {
                lod,
                property,
                element: elem,
            })
        })
        .collect();

    let appearance = AppearanceBundle {
        materials: geometry.materials.clone(),
        textures: geometry.textures.clone(),
    };

    (entries, appearance)
}

fn convert_gml_geometry(
    gml_geom: &GmlGeometry,
    parent: &CityGmlGeometry,
    need_appearance: bool,
) -> Option<GmlElement> {
    match gml_geom.ty {
        GeometryType::Solid => {
            if gml_geom.polygons.is_empty() {
                return None;
            }
            let surfaces = gml_geom
                .polygons
                .iter()
                .enumerate()
                .map(|(i, poly)| {
                    make_gml_surface(poly, gml_geom.pos as usize + i, parent, need_appearance)
                })
                .collect();
            Some(GmlElement::Solid(GmlSolid {
                id: gml_geom.id.clone(),
                exterior: surfaces,
                // The legacy reader logs "interior of Solid is not supported,
                // skipped", so a void never reaches this converter.
                interiors: Vec::new(),
            }))
        }
        GeometryType::Surface | GeometryType::Triangle => {
            if gml_geom.polygons.is_empty() {
                return None;
            }
            let surfaces = gml_geom
                .polygons
                .iter()
                .enumerate()
                .map(|(i, poly)| {
                    make_gml_surface(poly, gml_geom.pos as usize + i, parent, need_appearance)
                })
                .collect();
            Some(GmlElement::MultiSurface {
                id: gml_geom.id.clone(),
                surfaces,
            })
        }
        GeometryType::Curve => {
            if gml_geom.line_strings.is_empty() {
                return None;
            }
            Some(GmlElement::MultiCurve {
                id: gml_geom.id.clone(),
                curves: gml_geom
                    .line_strings
                    .iter()
                    .map(|ls| ring_coords(&ls.0))
                    .collect(),
            })
        }
        GeometryType::Point => None,
    }
}

fn make_gml_surface(
    poly: &Polygon3D<f64>,
    poly_global_idx: usize,
    parent: &CityGmlGeometry,
    need_appearance: bool,
) -> GmlSurface {
    if !need_appearance {
        return GmlSurface::from(poly);
    }

    let material_idx = parent
        .polygon_materials
        .get(poly_global_idx)
        .copied()
        .flatten();
    let texture_idx = parent
        .polygon_textures
        .get(poly_global_idx)
        .copied()
        .flatten();

    let (uv_exterior, uv_interiors) = if texture_idx.is_some() {
        if let Some(uv_poly) = parent.polygon_uvs.0.get(poly_global_idx) {
            let uv_ext = uv_poly.exterior().0.iter().map(|c| [c.x, c.y]).collect();
            let uv_int = uv_poly
                .interiors()
                .iter()
                .map(|ring| ring.0.iter().map(|c| [c.x, c.y]).collect())
                .collect();
            (uv_ext, uv_int)
        } else {
            (Vec::new(), Vec::new())
        }
    } else {
        (Vec::new(), Vec::new())
    };

    GmlSurface {
        id: None, // assigned by the writer using its id_counter
        exterior: ring_coords(&poly.exterior().0),
        interiors: poly
            .interiors()
            .iter()
            .map(|ring| ring_coords(&ring.0))
            .collect(),
        material_idx,
        texture_idx,
        uv_exterior,
        uv_interiors,
    }
}

/// Serialize `coords` as the body of a `gml:posList` — or of a `gml:lowerCorner`
/// / `gml:upperCorner`, which the writer formats the same way so a document's
/// envelope always reads in the same axis order as its geometry.
///
/// Legacy leaves store `x` as longitude/easting, while the CRSs this writer
/// declares put latitude/northing first, so the ordinates are transposed to
/// `y x z` here. The unified world stores ordinates in the CRS's own declared
/// order and so has its own, identity, formatter.
pub fn format_pos_list(coords: &[[f64; 3]]) -> String {
    coords
        .iter()
        .map(|c| format!("{} {} {}", c[1], c[0], c[2]))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn compute_envelope(geometry: &CityGmlGeometry) -> Option<BoundingEnvelope> {
    let vertices = geometry.get_vertices();
    if vertices.is_empty() {
        return None;
    }

    let (mut min_x, mut max_x) = (f64::MAX, f64::MIN);
    let (mut min_y, mut max_y) = (f64::MAX, f64::MIN);
    let (mut min_z, mut max_z) = (f64::MAX, f64::MIN);

    for v in &vertices {
        min_x = min_x.min(v.x);
        max_x = max_x.max(v.x);
        min_y = min_y.min(v.y);
        max_y = max_y.max(v.y);
        min_z = min_z.min(v.z);
        max_z = max_z.max(v.z);
    }

    Some(BoundingEnvelope {
        lower: [min_x, min_y, min_z],
        upper: [max_x, max_y, max_z],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::line_string::LineString3D;

    #[test]
    fn test_format_pos_list() {
        let coords = vec![[135.0, 35.0, 10.0], [135.1, 35.1, 11.0]];
        let result = format_pos_list(&coords);
        assert_eq!(result, "35 135 10 35.1 135.1 11");
    }

    #[test]
    fn test_gml_surface_from_polygon() {
        let exterior = LineString3D::new(vec![
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, 0.0, 0.0),
        ]);
        let polygon = Polygon3D::new(exterior, vec![]);
        let surface = GmlSurface::from(&polygon);
        assert_eq!(surface.exterior.len(), 4);
        assert!(surface.interiors.is_empty());
    }
}
