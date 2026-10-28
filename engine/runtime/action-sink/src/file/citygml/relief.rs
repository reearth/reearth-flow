//! Terrain rules: what a `dem:TINRelief` can hold and which classes are relief.
//!
//! `dem:tin` holds a `gml:TriangulatedSurface`, so only triangles can be terrain.
//! These rules are pure; the converter and the sink call them.

use reearth_flow_citygml::schema::{QName, SchemaSet};

use super::classes::CityClass;
use super::model::{GeometryEntry, GmlElement, GmlSurface};

pub(super) const RELIEF_NS: &str = "http://www.opengis.net/citygml/relief/2.0";

/// Why a set of geometry entries cannot be written as terrain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TerrainFault {
    /// A surface that is not a triangle.
    NotTriangles,
    /// A solid or curve, which no triangulated surface can hold.
    NotSurface,
}

/// Whether a surface is one triangle: an exterior of exactly three corners and
/// no interior rings. A ring counts as closed when its last position equals its
/// first, and the repeat is not a corner.
// Tasks 3 and 5 call this; the allow goes then.
#[allow(dead_code)]
pub(super) fn is_triangle(surface: &GmlSurface) -> bool {
    let ring = &surface.exterior;
    let closed = ring.len() > 1 && ring.first() == ring.last();
    let corners = if closed { ring.len() - 1 } else { ring.len() };
    corners == 3 && surface.interiors.is_empty()
}

/// What stops `entries` being written as terrain, if anything. A solid or curve
/// wins over a non-triangle face when both are present.
// Tasks 3 and 5 call this; the allow goes then.
#[allow(dead_code)]
pub(super) fn terrain_fault(entries: &[GeometryEntry]) -> Option<TerrainFault> {
    let mut fault = None;
    for entry in entries {
        match &entry.element {
            GmlElement::Solid(_)
            | GmlElement::MultiSolid { .. }
            | GmlElement::MultiCurve { .. } => return Some(TerrainFault::NotSurface),
            GmlElement::MultiSurface { surfaces, .. } => {
                if !surfaces.iter().all(is_triangle) {
                    fault = Some(TerrainFault::NotTriangles);
                }
            }
            GmlElement::TriangulatedSurface { .. } => {}
        }
    }
    fault
}

/// The element `class` declares for a `dem:tinPropertyType` slot, which is how
/// a class says it holds terrain (`dem:tin`).
// Tasks 3 and 5 call this; the allow goes then.
#[allow(dead_code)]
pub(super) fn tin_property(schema: &SchemaSet, class: &CityClass) -> Option<QName> {
    let tin_type = QName::new(RELIEF_NS, "tinPropertyType");
    schema
        .class_for_element(&class.element)?
        .slots()
        .iter()
        .find(|slot| slot.property_type.as_ref() == Some(&tin_type))?
        .declared
        .clone()
}

/// Whether `class` is a `dem:_ReliefComponent`, such as `dem:TINRelief`.
// Tasks 3 and 5 call this; the allow goes then.
#[allow(dead_code)]
pub(super) fn is_relief_component(schema: &SchemaSet, class: &CityClass) -> bool {
    schema.substitutes_for(&class.element, &QName::new(RELIEF_NS, "_ReliefComponent"))
}

/// Whether `class` is `dem:ReliefFeature`, which groups components and has no
/// geometry property of its own.
// Tasks 3 and 5 call this; the allow goes then.
#[allow(dead_code)]
pub(super) fn is_relief_feature(class: &CityClass) -> bool {
    class.element == QName::new(RELIEF_NS, "ReliefFeature")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::citygml::classes::{self, Prefixes};
    use crate::file::citygml::model::{GmlSolid, GmlSurface};
    use reearth_flow_citygml::schema::SchemaSet;

    fn class(value: &str) -> CityClass {
        classes::resolve(value, &Prefixes::citygml2(), SchemaSet::core().unwrap()).unwrap()
    }

    fn surface(exterior: Vec<[f64; 3]>, interiors: Vec<Vec<[f64; 3]>>) -> GmlSurface {
        GmlSurface {
            id: None,
            exterior,
            interiors,
            material_idx: None,
            texture_idx: None,
            uv_exterior: Vec::new(),
            uv_interiors: Vec::new(),
        }
    }

    const A: [f64; 3] = [0.0, 0.0, 0.0];
    const B: [f64; 3] = [1.0, 0.0, 0.0];
    const C: [f64; 3] = [0.0, 1.0, 0.0];
    const D: [f64; 3] = [1.0, 1.0, 0.0];

    fn entry(element: GmlElement) -> GeometryEntry {
        GeometryEntry {
            lod: 0,
            property: None,
            element,
            owner: Default::default(),
        }
    }

    fn mesh(surfaces: Vec<GmlSurface>) -> GeometryEntry {
        entry(GmlElement::MultiSurface { id: None, surfaces })
    }

    #[test]
    fn three_corners_closed_or_open_is_a_triangle() {
        assert!(is_triangle(&surface(vec![A, B, C], vec![])));
        assert!(is_triangle(&surface(vec![A, B, C, A], vec![])));
    }

    #[test]
    fn a_repeated_corner_still_counts_as_three() {
        assert!(is_triangle(&surface(vec![A, B, B, A], vec![])));
    }

    #[test]
    fn four_corners_or_an_interior_ring_is_not_a_triangle() {
        assert!(!is_triangle(&surface(vec![A, B, D, C, A], vec![])));
        assert!(!is_triangle(&surface(vec![A, B, D, C], vec![])));
        assert!(!is_triangle(&surface(
            vec![A, B, C, A],
            vec![vec![A, B, C, A]]
        )));
    }

    #[test]
    fn a_solid_or_curve_entry_is_not_surface() {
        let triangles = mesh(vec![surface(vec![A, B, C, A], vec![])]);
        let solid = entry(GmlElement::Solid(GmlSolid {
            id: None,
            exterior: vec![surface(vec![A, B, C, A], vec![])],
            interiors: vec![],
        }));
        let multi_solid = entry(GmlElement::MultiSolid {
            id: None,
            solids: vec![],
        });
        let curve = entry(GmlElement::MultiCurve {
            id: None,
            curves: vec![vec![A, B]],
        });
        for faulty in [solid, multi_solid, curve] {
            assert_eq!(
                terrain_fault(&[triangles.clone(), faulty]),
                Some(TerrainFault::NotSurface)
            );
        }
    }

    #[test]
    fn one_quad_among_triangles_faults_the_whole_set() {
        let entries = [mesh(vec![
            surface(vec![A, B, C, A], vec![]),
            surface(vec![A, B, D, C, A], vec![]),
        ])];
        assert_eq!(terrain_fault(&entries), Some(TerrainFault::NotTriangles));
        let clean = [mesh(vec![surface(vec![A, B, C, A], vec![])])];
        assert_eq!(terrain_fault(&clean), None);
    }

    #[test]
    fn tin_relief_declares_tin_and_a_building_declares_none() {
        let schema = SchemaSet::core().unwrap();
        assert_eq!(
            tin_property(schema, &class("dem:TINRelief")),
            Some(QName::new(RELIEF_NS, "tin"))
        );
        assert_eq!(tin_property(schema, &class("bldg:Building")), None);
    }

    #[test]
    fn relief_classes_are_recognised() {
        let schema = SchemaSet::core().unwrap();
        let tin = class("dem:TINRelief");
        let feature = class("dem:ReliefFeature");
        let building = class("bldg:Building");
        assert!(is_relief_component(schema, &tin));
        assert!(!is_relief_component(schema, &feature));
        assert!(!is_relief_component(schema, &building));
        assert!(is_relief_feature(&feature));
        assert!(!is_relief_feature(&tin));
        assert!(!is_relief_feature(&building));
    }
}
