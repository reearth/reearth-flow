//! Terrain rules: what a `dem:TINRelief` can hold and which classes are relief.
//!
//! `dem:tin` holds a `gml:TriangulatedSurface`, so only triangles can be terrain.
//! The rules are pure; the converter and the writer call them, and the sink
//! runs [`settle_terrain`] over the joined trees.

use reearth_flow_citygml::schema::{QName, SchemaSet};
use reearth_flow_diagnostics::ErrorCode;

use super::classes::{CityClass, Prefixes};
use super::hierarchy::TopLevel;
use super::model::{GeometryEntry, GmlElement, GmlSurface, ObjectNode};
use super::placement::Placed;
use super::properties::XmlProperty;

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
pub(super) fn is_triangle(surface: &GmlSurface) -> bool {
    let ring = &surface.exterior;
    let closed = ring.len() > 1 && ring.first() == ring.last();
    let corners = if closed { ring.len() - 1 } else { ring.len() };
    corners == 3 && surface.interiors.is_empty()
}

/// What stops `entries` being written as terrain, if anything. A solid or curve
/// wins over a non-triangle face when both are present.
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
pub(super) fn is_relief_component(schema: &SchemaSet, class: &CityClass) -> bool {
    schema.substitutes_for(&class.element, &QName::new(RELIEF_NS, "_ReliefComponent"))
}

/// Whether `class` is `dem:ReliefFeature`, which groups components and has no
/// geometry property of its own. Found through the schema's substitution
/// groups like the components, so an element that substitutes for it counts.
pub(super) fn is_relief_feature(schema: &SchemaSet, class: &CityClass) -> bool {
    schema.substitutes_for(&class.element, &QName::new(RELIEF_NS, "ReliefFeature"))
}

/// The terrain pass, run over the joined trees before the sink decides what is
/// written. Terrain that cannot be written is cleared and reported, and the
/// relief objects left with nothing to write are removed, top-level trees
/// included: the sink writes an orphan whatever it holds. A feature whose own
/// object was pruned leaves its tree's `roots`. Returns `(source, code)`, each
/// pair once, in the shape of `Joined::reports`.
///
/// A `dem:TINRelief` that arrives with no geometry, as when `lodFilter`
/// removed it, is removed without a report: nothing was wrong with it.
pub(super) fn settle_terrain(
    top: &mut Vec<TopLevel>,
    schema: &SchemaSet,
    // Filling `dem:lod` in what is left places it with these.
    prefixes: &Prefixes,
) -> Vec<(usize, ErrorCode)> {
    let mut reports = Vec::new();
    for tree in top.iter_mut() {
        reject_terrain(&mut tree.object, schema, &mut reports);
    }
    top.retain(|tree| !nothing_to_write(&tree.object, schema));
    // A feature whose own object was pruned is no longer held by its tree, so
    // the sink reports it empty. Every object a feature builds, synthetic
    // owners included, carries the feature's `source`, and the join only moves
    // whole feature roots, so a feature is held while any object with its
    // `source` is left.
    for TopLevel { object, roots, .. } in top.iter_mut() {
        roots.retain(|&source| holds_source(object, source));
    }
    // Last, so only an object that will be written gets a `dem:lod`.
    for tree in top.iter_mut() {
        fill_lod(&mut tree.object, schema, prefixes);
    }
    reports
}

/// Gives every relief object under `object`, itself included, the `dem:lod` it
/// must have (`minOccurs` 1) and has not got, children first so a
/// `dem:ReliefFeature` can read its components'. A relief component takes the
/// highest LOD of its geometry; a feature, the highest `dem:lod` among its
/// components. An object with no LOD to take, such as a `dem:BreaklineRelief`
/// the reader gave no geometry, is left as it was, as are the objects that
/// already hold a `dem:lod`.
fn fill_lod(object: &mut ObjectNode, schema: &SchemaSet, prefixes: &Prefixes) {
    for child in &mut object.children {
        fill_lod(&mut child.object, schema, prefixes);
    }
    let relief_feature = is_relief_feature(schema, &object.class);
    if !relief_feature && !is_relief_component(schema, &object.class) {
        return;
    }
    let lod = QName::new(RELIEF_NS, "lod");
    if find_lod(object, &lod, prefixes).is_some() {
        return;
    }
    let value = if relief_feature {
        object
            .children
            .iter()
            .filter(|child| is_relief_component(schema, &child.object.class))
            .filter_map(|child| find_lod(&child.object, &lod, prefixes)?.parse::<u8>().ok())
            .max()
    } else {
        object.geometries.iter().map(|entry| entry.lod).max()
    };
    let (Some(value), Some(prefix)) = (value, prefixes.prefix_of(RELIEF_NS)) else {
        return;
    };
    let Some(slot) = schema
        .class_for_element(&object.class.element)
        .and_then(|class| class.slot_index(&lod))
    else {
        return;
    };
    let mut property = XmlProperty::new(prefix, "lod");
    property.text = Some(value.to_string());
    object.properties.push(Placed { slot, property });
}

/// The text of the `dem:lod` an object holds. One that is not an integer still
/// counts as held: the object is written with it, as it came.
fn find_lod<'a>(object: &'a ObjectNode, lod: &QName, prefixes: &Prefixes) -> Option<&'a str> {
    object
        .properties
        .iter()
        .find(|placed| placed.property.qname(prefixes) == *lod)
        .map(|placed| placed.property.text.as_deref().unwrap_or("").trim())
}

/// Whether `object` or any object nested in it came from feature `source`.
fn holds_source(object: &ObjectNode, source: usize) -> bool {
    object.source == source
        || object
            .children
            .iter()
            .any(|child| holds_source(&child.object, source))
}

/// Clears what `object` and its children cannot write as terrain, children
/// first, and prunes the children left with nothing to write.
fn reject_terrain(
    object: &mut ObjectNode,
    schema: &SchemaSet,
    reports: &mut Vec<(usize, ErrorCode)>,
) {
    for child in &mut object.children {
        reject_terrain(&mut child.object, schema, reports);
    }
    object
        .children
        .retain(|child| !nothing_to_write(&child.object, schema));

    let code = if tin_property(schema, &object.class).is_some() {
        // All of an object's entries merge onto its one `tin`, so one bad
        // entry rejects them all: terrain is never written with holes.
        terrain_fault(&object.geometries).map(|fault| match fault {
            TerrainFault::NotTriangles => ErrorCode::CitygmlTinNotTriangles,
            TerrainFault::NotSurface => ErrorCode::CitygmlTinNotSurface,
        })
    } else if is_relief_feature(schema, &object.class) && !object.geometries.is_empty() {
        // It declares no geometry property, so settling would drop it anyway.
        Some(ErrorCode::CitygmlReliefFeatureGeometry)
    } else {
        None
    };
    if let Some(code) = code {
        object.geometries.clear();
        if !reports.contains(&(object.source, code)) {
            reports.push((object.source, code));
        }
    }
}

/// Whether `object` is a relief object that would be written empty: a class
/// holding terrain with none left (`tin` has `minOccurs` 1), or a
/// `dem:ReliefFeature` with no relief component left (`reliefComponent` has
/// `minOccurs` 1). Other relief components, such as a `dem:BreaklineRelief`,
/// are written as before.
fn nothing_to_write(object: &ObjectNode, schema: &SchemaSet) -> bool {
    if tin_property(schema, &object.class).is_some() {
        !object.has_geometry()
    } else if is_relief_feature(schema, &object.class) {
        !object
            .children
            .iter()
            .any(|child| is_relief_component(schema, &child.object.class))
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::citygml::classes::{self, Prefixes};
    use crate::file::citygml::model::{GmlSolid, GmlSurface, NestedObject};
    use crate::file::citygml::placement::Placed;
    use crate::file::citygml::properties::XmlProperty;
    use crate::file::citygml::writer::CityGmlXmlWriter;
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

    fn triangle() -> GeometryEntry {
        mesh(vec![surface(vec![A, B, C, A], vec![])])
    }

    fn quad() -> GeometryEntry {
        mesh(vec![surface(vec![A, B, D, C, A], vec![])])
    }

    fn node(ty: &str, source: usize, geometries: Vec<GeometryEntry>) -> ObjectNode {
        ObjectNode {
            class: class(ty),
            key: None,
            synthetic: false,
            gml_id: None,
            geometries,
            properties: Vec::new(),
            children: Vec::new(),
            source,
        }
    }

    /// `object` with `children` nested in its `property`.
    fn holding(mut object: ObjectNode, property: &str, children: Vec<ObjectNode>) -> ObjectNode {
        let (prefix, local) = property.split_once(':').unwrap();
        let namespace = match prefix {
            "dem" => RELIEF_NS,
            "bldg" => "http://www.opengis.net/citygml/building/2.0",
            other => panic!("{other}"),
        };
        object.children = children
            .into_iter()
            .map(|object| NestedObject {
                slot: 0,
                property: QName::new(namespace, local),
                object,
            })
            .collect();
        object
    }

    fn tree(object: ObjectNode, orphan: bool) -> TopLevel {
        TopLevel {
            object,
            orphan,
            roots: Vec::new(),
        }
    }

    fn settle(top: &mut Vec<TopLevel>) -> Vec<(usize, ErrorCode)> {
        settle_terrain(top, SchemaSet::core().unwrap(), &Prefixes::citygml2())
    }

    #[test]
    fn a_quad_in_a_tin_relief_clears_all_its_terrain_and_reports_once() {
        let tin = node("dem:TINRelief", 0, vec![triangle(), quad(), quad()]);
        let feature = holding(
            node("dem:ReliefFeature", 1, vec![]),
            "dem:reliefComponent",
            vec![tin, node("dem:TINRelief", 1, vec![triangle()])],
        );
        let mut top = vec![tree(feature, false)];

        let reports = settle(&mut top);

        assert_eq!(reports, [(0, ErrorCode::CitygmlTinNotTriangles)]);
        let children = &top[0].object.children;
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].object.source, 1);
        assert_eq!(children[0].object.geometries, [triangle()]);
    }

    #[test]
    fn a_solid_on_a_tin_relief_reports_not_surface() {
        let solid = entry(GmlElement::Solid(GmlSolid {
            id: None,
            exterior: vec![surface(vec![A, B, C, A], vec![])],
            interiors: vec![],
        }));
        let mut top = vec![tree(
            node("dem:TINRelief", 3, vec![triangle(), solid]),
            false,
        )];

        let reports = settle(&mut top);

        assert_eq!(reports, [(3, ErrorCode::CitygmlTinNotSurface)]);
        assert!(top.is_empty());
    }

    #[test]
    fn a_rejected_tin_relief_is_pruned_from_its_relief_feature() {
        let feature = holding(
            node("dem:ReliefFeature", 0, vec![]),
            "dem:reliefComponent",
            vec![
                node("dem:TINRelief", 0, vec![triangle()]),
                node("dem:TINRelief", 0, vec![quad()]),
            ],
        );
        let mut top = vec![tree(feature, false)];

        let reports = settle(&mut top);

        assert_eq!(reports, [(0, ErrorCode::CitygmlTinNotTriangles)]);
        assert_eq!(top.len(), 1);
        let children = &top[0].object.children;
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].object.geometries, [triangle()]);
    }

    /// Review Focus 2: components joined from two other features, both
    /// rejected, leave their relief feature with nothing to write.
    #[test]
    fn a_relief_feature_left_without_components_is_removed_from_top() {
        let feature = holding(
            node("dem:ReliefFeature", 0, vec![]),
            "dem:reliefComponent",
            vec![
                node("dem:TINRelief", 1, vec![quad()]),
                node("dem:TINRelief", 2, vec![quad()]),
            ],
        );
        let mut top = vec![
            tree(feature, false),
            tree(node("bldg:Building", 3, vec![quad()]), false),
        ];

        let reports = settle(&mut top);

        assert_eq!(
            reports,
            [
                (1, ErrorCode::CitygmlTinNotTriangles),
                (2, ErrorCode::CitygmlTinNotTriangles),
            ]
        );
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].object.source, 3);
    }

    /// Review Focus 1: the sink writes an orphan whatever it holds, so the
    /// pass removes it.
    #[test]
    fn an_orphan_tin_relief_left_empty_is_removed_from_top() {
        let mut top = vec![
            tree(node("dem:TINRelief", 0, vec![quad()]), true),
            tree(node("dem:TINRelief", 1, vec![]), true),
        ];

        let reports = settle(&mut top);

        // The second had no terrain to reject, as when `lodFilter` removed it.
        assert_eq!(reports, [(0, ErrorCode::CitygmlTinNotTriangles)]);
        assert!(top.is_empty());
    }

    #[test]
    fn a_relief_feature_with_its_own_geometry_reports_and_is_removed() {
        let mut top = vec![tree(node("dem:ReliefFeature", 4, vec![triangle()]), false)];

        let reports = settle(&mut top);

        assert_eq!(reports, [(4, ErrorCode::CitygmlReliefFeatureGeometry)]);
        assert!(top.is_empty());
    }

    #[test]
    fn a_relief_feature_keeps_its_valid_component_when_another_is_rejected() {
        let feature = holding(
            node("dem:ReliefFeature", 0, vec![triangle()]),
            "dem:reliefComponent",
            vec![
                node("dem:TINRelief", 1, vec![quad()]),
                node("dem:TINRelief", 2, vec![triangle()]),
            ],
        );
        let mut top = vec![tree(feature, true)];

        let reports = settle(&mut top);

        assert_eq!(
            reports,
            [
                (1, ErrorCode::CitygmlTinNotTriangles),
                (0, ErrorCode::CitygmlReliefFeatureGeometry),
            ]
        );
        assert_eq!(top.len(), 1);
        let object = &top[0].object;
        assert!(object.geometries.is_empty());
        assert_eq!(object.children.len(), 1);
        assert_eq!(object.children[0].object.source, 2);
    }

    #[test]
    fn a_building_tree_is_untouched() {
        let building = holding(
            node("bldg:Building", 0, vec![quad()]),
            "bldg:boundedBy",
            vec![
                node("bldg:WallSurface", 0, vec![quad()]),
                node("bldg:RoofSurface", 0, vec![]),
            ],
        );
        let mut top = vec![
            tree(building, false),
            tree(node("bldg:Building", 1, vec![]), true),
        ];

        let reports = settle(&mut top);

        assert!(reports.is_empty());
        assert_eq!(top.len(), 2);
        let object = &top[0].object;
        assert_eq!(object.geometries, [quad()]);
        assert_eq!(object.children.len(), 2);
        assert_eq!(object.children[0].object.geometries, [quad()]);
    }

    /// A feature whose own component was pruned out of another feature's
    /// relief feature is no longer one its tree holds, so the sink reports it.
    #[test]
    fn a_feature_whose_component_was_pruned_leaves_the_roots() {
        let feature = holding(
            node("dem:ReliefFeature", 0, vec![]),
            "dem:reliefComponent",
            vec![
                node("dem:TINRelief", 1, vec![]),
                node("dem:TINRelief", 2, vec![triangle()]),
                node("dem:TINRelief", 3, vec![quad()]),
            ],
        );
        let mut top = vec![TopLevel {
            object: feature,
            orphan: false,
            roots: vec![0, 1, 2, 3],
        }];

        let reports = settle(&mut top);

        assert_eq!(reports, [(3, ErrorCode::CitygmlTinNotTriangles)]);
        assert_eq!(top[0].roots, [0, 2]);
    }

    /// Spec section 5: a relief component that holds no terrain, such as a
    /// `dem:BreaklineRelief`, is written as before, as is the relief feature
    /// holding it.
    #[test]
    fn an_empty_breakline_relief_is_untouched() {
        let feature = holding(
            node("dem:ReliefFeature", 0, vec![]),
            "dem:reliefComponent",
            vec![node("dem:BreaklineRelief", 0, vec![])],
        );
        let mut top = vec![
            TopLevel {
                object: feature,
                orphan: false,
                roots: vec![0],
            },
            tree(node("dem:BreaklineRelief", 1, vec![]), true),
        ];

        let reports = settle(&mut top);

        assert!(reports.is_empty());
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].object.children.len(), 1);
        assert_eq!(top[0].roots, [0]);
    }

    fn at_lod(mut entry: GeometryEntry, lod: u8) -> GeometryEntry {
        entry.lod = lod;
        entry
    }

    fn dem_lod(object: &ObjectNode) -> Vec<String> {
        let prefixes = Prefixes::citygml2();
        let lod = QName::new(RELIEF_NS, "lod");
        object
            .properties
            .iter()
            .filter(|placed| placed.property.qname(&prefixes) == lod)
            .map(|placed| placed.property.text.clone().unwrap())
            .collect()
    }

    fn with_dem_lod(mut object: ObjectNode, value: &str) -> ObjectNode {
        let schema = SchemaSet::core().unwrap();
        let mut property = XmlProperty::new("dem", "lod");
        property.text = Some(value.to_owned());
        let slot = schema
            .class_for_element(&object.class.element)
            .unwrap()
            .slot_index(&QName::new(RELIEF_NS, "lod"))
            .unwrap();
        object.properties.push(Placed { slot, property });
        object
    }

    #[test]
    fn a_tin_relief_without_dem_lod_gets_its_geometry_lod() {
        let mut top = vec![tree(
            node(
                "dem:TINRelief",
                0,
                vec![at_lod(triangle(), 1), at_lod(triangle(), 2)],
            ),
            true,
        )];

        settle(&mut top);

        assert_eq!(dem_lod(&top[0].object), ["2"]);
        let slot = top[0].object.properties[0].slot;
        let schema = SchemaSet::core().unwrap();
        let class = schema.class_for_element(&top[0].object.class.element);
        assert_eq!(
            class.unwrap().slot_index(&QName::new(RELIEF_NS, "lod")),
            Some(slot)
        );
    }

    #[test]
    fn an_existing_dem_lod_is_kept_as_is() {
        let object = with_dem_lod(node("dem:TINRelief", 0, vec![at_lod(triangle(), 2)]), "1");
        let mut top = vec![tree(object, true)];

        settle(&mut top);

        assert_eq!(dem_lod(&top[0].object), ["1"]);
        assert_eq!(top[0].object.properties.len(), 1);
    }

    #[test]
    fn a_relief_feature_takes_the_highest_component_lod() {
        let feature = holding(
            node("dem:ReliefFeature", 0, vec![]),
            "dem:reliefComponent",
            vec![
                node("dem:TINRelief", 0, vec![at_lod(triangle(), 1)]),
                // Its own `dem:lod`, set apart from its geometry.
                with_dem_lod(node("dem:TINRelief", 0, vec![at_lod(triangle(), 1)]), "3"),
                node("dem:TINRelief", 0, vec![at_lod(triangle(), 2)]),
            ],
        );
        let mut top = vec![tree(feature, false)];

        settle(&mut top);

        let object = &top[0].object;
        assert_eq!(dem_lod(object), ["3"]);
        let lods: Vec<_> = object
            .children
            .iter()
            .map(|child| dem_lod(&child.object))
            .collect();
        assert_eq!(lods, [["1"], ["3"], ["2"]]);
    }

    /// A component the pass removes does not count towards the feature's LOD,
    /// and a relief object with no LOD to take is left as it was.
    #[test]
    fn a_pruned_component_does_not_set_the_feature_lod() {
        let feature = holding(
            node("dem:ReliefFeature", 0, vec![]),
            "dem:reliefComponent",
            vec![
                node("dem:TINRelief", 1, vec![at_lod(quad(), 4)]),
                node("dem:TINRelief", 2, vec![at_lod(triangle(), 2)]),
                node("dem:BreaklineRelief", 3, vec![]),
            ],
        );
        let mut top = vec![tree(feature, false)];

        settle(&mut top);

        let object = &top[0].object;
        assert_eq!(dem_lod(object), ["2"]);
        assert_eq!(dem_lod(&object.children[1].object), Vec::<String>::new());
    }

    #[test]
    fn a_building_gets_no_dem_lod() {
        let mut top = vec![tree(
            node("bldg:Building", 0, vec![at_lod(triangle(), 2)]),
            false,
        )];

        settle(&mut top);

        assert!(top[0].object.properties.is_empty());
    }

    #[test]
    fn the_filled_dem_lod_sits_before_tin_in_the_written_object() {
        let mut top = vec![tree(
            node("dem:TINRelief", 0, vec![at_lod(triangle(), 1)]),
            true,
        )];
        settle(&mut top);
        let object = top.remove(0).object;

        let mut buf = Vec::new();
        let mut writer = CityGmlXmlWriter::new(
            &mut buf,
            false,
            "http://www.opengis.net/def/crs/EPSG/0/6697".to_string(),
        );
        writer.write_object_tree(object, &[None]).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        let lod = xml.find("<dem:lod>1</dem:lod>").expect(&xml);
        let tin = xml.find("<dem:tin>").expect(&xml);
        assert!(lod < tin, "{xml}");
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
        assert!(is_relief_feature(schema, &feature));
        assert!(!is_relief_feature(schema, &tin));
        assert!(!is_relief_feature(schema, &building));
    }
}
