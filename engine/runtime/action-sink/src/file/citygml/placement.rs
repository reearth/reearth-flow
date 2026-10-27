//! Where each property goes in its class's content model.
//!
//! The top level is strict: a property no slot accepts, or one past its
//! slot's `maxOccurs`, is left out and reported, so the document stays valid.
//! Below it, content is only reordered. It almost always came from a source
//! document, so a gap in the model there must not lose valid data; anything
//! out of place is still visible to the schema gate. The one exception is an
//! element whose required child was left out for having no value: it is
//! dropped and reported, rather than written invalid.

use reearth_flow_citygml::schema::{MaxOccurs, QName, SchemaSet};

use super::classes::Prefixes;
use super::model::GeometryEntry;
use super::properties::{has_geometry_name, Skip, XmlProperty};

const CORE: &str = "http://www.opengis.net/citygml/2.0";
const DEM: &str = "http://www.opengis.net/citygml/relief/2.0";
const GML: &str = "http://www.opengis.net/gml";

/// The GML 3.1.1 property types that hold a geometry.
const GML_GEOMETRY_PROPERTY_TYPES: &[&str] = &[
    "GeometryPropertyType",
    "GeometricPrimitivePropertyType",
    "GeometricComplexPropertyType",
    "PointPropertyType",
    "CurvePropertyType",
    "LineStringPropertyType",
    "SurfacePropertyType",
    "PolygonPropertyType",
    "TriangulatedSurfacePropertyType",
    "SolidPropertyType",
    "CompositeCurvePropertyType",
    "CompositeSurfacePropertyType",
    "CompositeSolidPropertyType",
    "MultiGeometryPropertyType",
    "MultiPointPropertyType",
    "MultiCurvePropertyType",
    "MultiSurfacePropertyType",
    "MultiSolidPropertyType",
];

/// A property and the slot it occupies.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub slot: usize,
    pub property: XmlProperty,
}

/// Assign each property the first slot that accepts it and still has room,
/// in arrival order. The result keeps arrival order; the writer sorts by slot.
///
/// `schema` is the set the writer writes against, ADEs included, and
/// `prefixes` the table the properties were built with.
pub fn place(
    schema: &SchemaSet,
    prefixes: &Prefixes,
    class_element: &QName,
    properties: Vec<XmlProperty>,
) -> (Vec<Placed>, Vec<Skip>) {
    let mut placed = Vec::new();
    let mut skipped = Vec::new();
    let Some(class) = schema.class_for_element(class_element) else {
        skipped.extend(properties.iter().map(|_| Skip::NotPlaced));
        return (placed, skipped);
    };
    let slots = class.slots();
    let mut used = vec![0u32; slots.len()];
    for mut property in properties {
        if !prune_emptied(schema, prefixes, &mut property, &mut skipped) {
            continue;
        }
        if holds_city_object(schema, prefixes, &property) {
            skipped.push(Skip::NestedObject);
            continue;
        }
        let name = property.qname(prefixes);
        let mut accepted = false;
        let free = slots.iter().enumerate().find(|(index, slot)| {
            let accepts = slot.accepts(&name);
            accepted |= accepts;
            accepts
                && match slot.max {
                    MaxOccurs::Unbounded => true,
                    MaxOccurs::Bounded(max) => used[*index] < max,
                }
        });
        match free {
            Some((index, _)) => {
                used[index] += 1;
                reorder_nested(schema, prefixes, &mut property);
                placed.push(Placed {
                    slot: index,
                    property,
                });
            }
            None if accepted => skipped.push(Skip::OverLimit),
            None => skipped.push(Skip::NotPlaced),
        }
    }
    (placed, skipped)
}

/// Whether `prefix:local` is one of `class_element`'s geometry properties,
/// which are written from geometry, never from text.
///
/// The reader's geometry properties are named `lod<N>…` and `tin`, but an ADE
/// may give one of its own attributes such a name. So a name counts only when
/// the schema does not type the property as something else: an ADE's
/// `lod1HeightType` declared as a `gml:CodeType` is an attribute. A property
/// the schema does not type keeps the name's verdict.
pub fn is_geometry_property(
    schema: &SchemaSet,
    prefixes: &Prefixes,
    class_element: &QName,
    prefix: &str,
    local: &str,
) -> bool {
    if !has_geometry_name(local) {
        return false;
    }
    let Some(namespace) = prefixes.namespace(prefix) else {
        return true;
    };
    property_type(schema, class_element, &QName::new(namespace, local)).is_none_or(is_geometry_type)
}

/// The type `class_element` gives its property `property`, if the schema
/// names one.
///
/// A local declaration is typed on the class's slot. A global element is
/// typed on its own declaration, which covers an ADE property substituted into
/// one of the class's `_GenericApplicationPropertyOf…` hooks, whose slot
/// carries the hook's type instead.
pub fn property_type<'a>(
    schema: &'a SchemaSet,
    class_element: &QName,
    property: &QName,
) -> Option<&'a QName> {
    schema
        .class_for_element(class_element)
        .and_then(|class| {
            class
                .slots()
                .iter()
                .find(|slot| slot.declared.as_ref() == Some(property))
        })
        .and_then(|slot| slot.property_type.as_ref())
        .or_else(|| schema.element_type(property))
}

/// Puts `entry` inside the element among `properties`, at any depth, whose
/// synthetic owner key is `key`, as the geometry property of the entry's name
/// that element's own content model declares, in schema order.
///
/// This is how geometry inside an ADE element is written where the source had
/// it: i-UR's `uro:DmGeometricAttribute` declares `uro:lod0Geometry`, which no
/// city object holding the attribute does. Nothing here knows i-UR: the
/// element, the property and its position all come from the ADE's XSD.
///
/// Hands `entry` back when there is no such element, when its class declares
/// no geometry property of that name, when that property's prefix is not
/// bound, or when the property is already full.
pub fn place_data_geometry<'a>(
    schema: &SchemaSet,
    prefixes: &Prefixes,
    properties: impl IntoIterator<Item = &'a mut XmlProperty>,
    key: &str,
    entry: Box<GeometryEntry>,
) -> Result<(), Box<GeometryEntry>> {
    let Some(element) = properties
        .into_iter()
        .find_map(|property| keyed_element(property, key))
    else {
        return Err(entry);
    };
    let Some(class) = schema.class_for_element(&element.qname(prefixes)) else {
        return Err(entry);
    };
    let Some(local) = entry.property.as_deref() else {
        return Err(entry);
    };
    let slot = class.slots().iter().find(|slot| {
        slot.declared
            .as_ref()
            .is_some_and(|declared| declared.local == local)
            && slot.property_type.as_ref().is_some_and(is_geometry_type)
    });
    let Some((slot, declared)) = slot.and_then(|slot| Some((slot, slot.declared.as_ref()?))) else {
        return Err(entry);
    };
    let Some(prefix) = prefixes.prefix_of(&declared.namespace) else {
        return Err(entry);
    };
    let held = element
        .children
        .iter()
        .filter(|child| slot.accepts(&child.qname(prefixes)))
        .count();
    if let MaxOccurs::Bounded(max) = slot.max {
        if held >= max as usize {
            return Err(entry);
        }
    }
    let mut property = XmlProperty::new(prefix, local);
    property.geometry = Some(entry);
    element.children.push(property);
    element.children.sort_by_cached_key(|child| {
        class
            .slot_index(&child.qname(prefixes))
            .unwrap_or(usize::MAX)
    });
    Ok(())
}

/// The element at or below `property` whose synthetic owner key is `key`.
fn keyed_element<'a>(property: &'a mut XmlProperty, key: &str) -> Option<&'a mut XmlProperty> {
    if property.synthetic_key.as_deref() == Some(key) {
        return Some(property);
    }
    property
        .children
        .iter_mut()
        .find_map(|child| keyed_element(child, key))
}

/// Whether a property type holds a geometry: a GML geometry property, an
/// implicit geometry, or a relief TIN.
fn is_geometry_type(property_type: &QName) -> bool {
    let local = property_type.local.as_str();
    match property_type.namespace.as_str() {
        GML => GML_GEOMETRY_PROPERTY_TYPES.contains(&local),
        CORE => local == "ImplicitRepresentationPropertyType",
        DEM => local == "tinPropertyType",
        _ => false,
    }
}

/// Whether a property wraps a city object, such as `bldg:boundedBy` around a
/// `bldg:WallSurface`. Those are nested objects, written from the ownership
/// keys rather than from attributes.
fn holds_city_object(schema: &SchemaSet, prefixes: &Prefixes, property: &XmlProperty) -> bool {
    property
        .children
        .iter()
        .any(|child| is_city_object(schema, &child.qname(prefixes)))
}

/// Whether `element` is `core:_CityObject` or substitutes for it.
pub(super) fn is_city_object(schema: &SchemaSet, element: &QName) -> bool {
    let city_object = QName::new(CORE, "_CityObject");
    let mut current = element.clone();
    // A substitution chain is short; the bound only guards a cyclic schema.
    for _ in 0..32 {
        if current == city_object {
            return true;
        }
        match schema.substitution_group(&current) {
            Some(head) => current = head.clone(),
            None => return false,
        }
    }
    false
}

/// Sort each element's children by its own content model where it has one,
/// depth first. Children no slot accepts keep their order after the rest.
fn reorder_nested(schema: &SchemaSet, prefixes: &Prefixes, property: &mut XmlProperty) {
    for child in &mut property.children {
        reorder_nested(schema, prefixes, child);
    }
    if let Some(class) = schema.class_for_element(&property.qname(prefixes)) {
        property.children.sort_by_cached_key(|child| {
            class
                .slot_index(&child.qname(prefixes))
                .unwrap_or(usize::MAX)
        });
    }
}

/// Drops, depth first, every element that an empty value left without a child
/// its schema requires, such as a `gen:stringAttribute` without `gen:value`
/// or a `core:Address` without `core:xalAddress`, and every element that is
/// left with nothing to write. Each drop is reported in `skipped`. Returns
/// whether `property` itself stays.
///
/// Only an element marked `emptied`, or one that lost a child here, is
/// judged: one the source wrote without a required child is kept as it was.
fn prune_emptied(
    schema: &SchemaSet,
    prefixes: &Prefixes,
    property: &mut XmlProperty,
    skipped: &mut Vec<Skip>,
) -> bool {
    let before = property.children.len();
    property
        .children
        .retain_mut(|child| prune_emptied(schema, prefixes, child, skipped));
    property.emptied |= property.children.len() < before;
    if !property.emptied {
        return true;
    }
    let complete = property.has_content()
        && schema
            .class_for_element(&property.qname(prefixes))
            .is_none_or(|class| {
                class.slots().iter().all(|slot| {
                    slot.min == 0
                        || property
                            .children
                            .iter()
                            .any(|child| slot.accepts(&child.qname(prefixes)))
                })
            });
    if !complete {
        skipped.push(Skip::EmptyValue);
    }
    complete
}

#[cfg(test)]
mod tests {
    use reearth_flow_citygml::schema::{QName, SchemaSet};

    use super::*;

    const BLDG: &str = "http://www.opengis.net/citygml/building/2.0";

    fn building() -> QName {
        QName::new(BLDG, "Building")
    }

    fn prop(prefix: &str, local: &str) -> XmlProperty {
        let mut p = XmlProperty::new(prefix, local);
        p.text = Some("x".to_owned());
        p
    }

    fn placed_names(placed: &[Placed]) -> Vec<String> {
        let mut sorted: Vec<&Placed> = placed.iter().collect();
        sorted.sort_by_key(|p| p.slot);
        sorted
            .iter()
            .map(|p| format!("{}:{}", p.property.prefix, p.property.local))
            .collect()
    }

    #[test]
    fn properties_take_schema_order_with_generic_attributes_before_the_class() {
        let schema = SchemaSet::core().unwrap();
        let (placed, skipped) = place(
            schema,
            &Prefixes::citygml2(),
            &building(),
            vec![
                prop("bldg", "measuredHeight"),
                prop("gen", "stringAttribute"),
                prop("bldg", "class"),
                prop("core", "creationDate"),
            ],
        );
        assert!(skipped.is_empty());
        assert_eq!(
            placed_names(&placed),
            [
                "core:creationDate",
                "gen:stringAttribute",
                "bldg:class",
                "bldg:measuredHeight"
            ]
        );
    }

    #[test]
    fn a_property_of_another_class_is_not_placed() {
        let schema = SchemaSet::core().unwrap();
        let (placed, skipped) = place(
            schema,
            &Prefixes::citygml2(),
            &building(),
            vec![prop("tran", "trafficArea"), prop("xlink", "href")],
        );
        assert!(placed.is_empty());
        assert_eq!(skipped, vec![Skip::NotPlaced, Skip::NotPlaced]);
    }

    #[test]
    fn a_second_value_for_a_single_property_is_over_the_limit_but_generics_repeat() {
        let schema = SchemaSet::core().unwrap();
        let (placed, skipped) = place(
            schema,
            &Prefixes::citygml2(),
            &building(),
            vec![
                prop("bldg", "measuredHeight"),
                prop("bldg", "measuredHeight"),
                prop("gen", "stringAttribute"),
                prop("gen", "stringAttribute"),
            ],
        );
        assert_eq!(placed.len(), 3);
        assert_eq!(skipped, vec![Skip::OverLimit]);
    }

    #[test]
    fn a_nested_city_object_is_reported_not_written() {
        let schema = SchemaSet::core().unwrap();
        let mut bounded_by = XmlProperty::new("bldg", "boundedBy");
        bounded_by
            .children
            .push(XmlProperty::new("bldg", "WallSurface"));
        let mut address = XmlProperty::new("bldg", "address");
        address.children.push(XmlProperty::new("core", "Address"));
        let (placed, skipped) = place(
            schema,
            &Prefixes::citygml2(),
            &building(),
            vec![bounded_by, address],
        );
        assert_eq!(placed_names(&placed), ["bldg:address"]);
        assert_eq!(skipped, vec![Skip::NestedObject]);
    }

    #[test]
    fn nested_children_are_reordered_but_never_dropped() {
        let schema = SchemaSet::core().unwrap();
        let mut address = XmlProperty::new("core", "Address");
        address.children.push(prop("core", "multiPoint"));
        address.children.push(prop("bldg", "class"));
        address.children.push(prop("core", "xalAddress"));
        let mut wrapper = XmlProperty::new("bldg", "address");
        wrapper.children.push(address);
        let (placed, _) = place(schema, &Prefixes::citygml2(), &building(), vec![wrapper]);
        let children: Vec<&str> = placed[0].property.children[0]
            .children
            .iter()
            .map(|c| c.local.as_str())
            .collect();
        assert_eq!(children, ["xalAddress", "multiPoint", "class"]);
    }

    /// A generic attribute whose `gen:value` was left out: the name, no
    /// value, marked emptied.
    fn emptied_generic(name: &str) -> XmlProperty {
        let mut generic = XmlProperty::new("gen", "stringAttribute");
        generic.attrs.push(("name".to_owned(), name.to_owned()));
        generic.emptied = true;
        generic
    }

    fn generic(name: &str) -> XmlProperty {
        let mut generic = XmlProperty::new("gen", "stringAttribute");
        generic.attrs.push(("name".to_owned(), name.to_owned()));
        generic.children.push(prop("gen", "value"));
        generic
    }

    /// A `core:Address` with a `gml:id` and a `core:multiPoint` reference but
    /// no `core:xalAddress`, inside its `bldg:address`.
    fn address_without_xal(emptied: bool) -> XmlProperty {
        let mut multi_point = XmlProperty::new("core", "multiPoint");
        multi_point
            .attrs
            .push(("xlink:href".to_owned(), "#p1".to_owned()));
        let mut address = XmlProperty::new("core", "Address");
        address.attrs.push(("gml:id".to_owned(), "a1".to_owned()));
        address.children.push(multi_point);
        address.emptied = emptied;
        let mut wrapper = XmlProperty::new("bldg", "address");
        wrapper.children.push(address);
        wrapper
    }

    /// Whether something was skipped, all of it for an empty value.
    fn only_empty(skipped: &[Skip]) -> bool {
        !skipped.is_empty() && skipped.iter().all(|skip| *skip == Skip::EmptyValue)
    }

    /// `gen:value` is required, so a generic attribute whose value was left
    /// out goes as a whole, and is reported, rather than being written
    /// invalid.
    #[test]
    fn a_generic_attribute_whose_value_was_left_out_is_dropped_and_reported() {
        let schema = SchemaSet::core().unwrap();
        let (placed, skipped) = place(
            schema,
            &Prefixes::citygml2(),
            &building(),
            vec![emptied_generic("note"), generic("kept")],
        );
        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0].property.attr("name"), Some("kept"));
        assert!(only_empty(&skipped), "{skipped:?}");
    }

    /// A set needs one member; it goes once its last one does, and stays while
    /// any is left.
    #[test]
    fn a_generic_attribute_set_goes_with_its_last_member() {
        let schema = SchemaSet::core().unwrap();
        let set = |name: &str, members: Vec<XmlProperty>| {
            let mut set = XmlProperty::new("gen", "genericAttributeSet");
            set.attrs.push(("name".to_owned(), name.to_owned()));
            set.children = members;
            set
        };
        let (placed, skipped) = place(
            schema,
            &Prefixes::citygml2(),
            &building(),
            vec![
                set("gone", vec![emptied_generic("a")]),
                set("kept", vec![emptied_generic("b"), generic("c")]),
            ],
        );
        assert_eq!(placed.len(), 1);
        let kept = &placed[0].property;
        assert_eq!(kept.attr("name"), Some("kept"));
        assert_eq!(kept.children.len(), 1);
        assert_eq!(kept.children[0].attr("name"), Some("c"));
        assert!(only_empty(&skipped), "{skipped:?}");
    }

    /// `core:Address` requires `core:xalAddress`: when that was left out the
    /// address goes despite its other content, and the property wrapping it,
    /// left with nothing, goes too. Both are reported.
    #[test]
    fn an_element_missing_a_required_child_after_an_empty_value_is_dropped_with_its_wrapper() {
        let schema = SchemaSet::core().unwrap();
        let (placed, skipped) = place(
            schema,
            &Prefixes::citygml2(),
            &building(),
            vec![address_without_xal(true), prop("bldg", "class")],
        );
        assert_eq!(placed_names(&placed), ["bldg:class"]);
        assert!(only_empty(&skipped), "{skipped:?}");
    }

    /// A `gml:id` keeps an element through conversion, but not past the
    /// required-child rule: an address an empty value left with only its id
    /// still goes, with its wrapper.
    #[test]
    fn an_address_left_with_only_its_gml_id_is_still_dropped() {
        let schema = SchemaSet::core().unwrap();
        let prefixes = Prefixes::citygml2();
        let tree = serde_json::json!({ "bldg:address": { "core:Address": {
            "@gml:id": "a1",
            "core:xalAddress": ""
        } } });
        let reearth_flow_types::AttributeValue::Map(tree) =
            reearth_flow_types::AttributeValue::from(tree)
        else {
            unreachable!()
        };
        let converted = super::super::properties::from_attributes(
            &tree,
            &std::collections::HashSet::new(),
            &prefixes,
            &|_, local| has_geometry_name(local),
        );
        assert_eq!(converted.properties.len(), 1);
        let (placed, skipped) = place(schema, &prefixes, &building(), converted.properties);
        assert!(placed.is_empty());
        assert!(only_empty(&skipped), "{skipped:?}");
    }

    /// Only an empty value drops an element: one the source already wrote
    /// without its required child is written as it was.
    #[test]
    fn an_element_the_source_wrote_without_a_required_child_is_kept() {
        let schema = SchemaSet::core().unwrap();
        let (placed, skipped) = place(
            schema,
            &Prefixes::citygml2(),
            &building(),
            vec![address_without_xal(false)],
        );
        assert_eq!(placed_names(&placed), ["bldg:address"]);
        assert!(skipped.is_empty());
    }

    const TEST_ADE: &str = "urn:reearth:test-ade";

    /// A small ADE hooking two `lod`-named properties into `bldg:Building`: an
    /// attribute typed `gml:CodeType` and a geometry.
    fn lod_named_ade() -> (tempfile::TempDir, SchemaSet, Prefixes) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ade.xsd");
        std::fs::write(
            &path,
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:gml="http://www.opengis.net/gml"
    xmlns:bldg="{BLDG}"
    targetNamespace="{TEST_ADE}" elementFormDefault="qualified">
  <xs:import namespace="http://www.opengis.net/gml"
      schemaLocation="http://schemas.opengis.net/gml/3.1.1/base/gml.xsd"/>
  <xs:import namespace="{BLDG}"
      schemaLocation="http://schemas.opengis.net/citygml/building/2.0/building.xsd"/>
  <xs:element name="lod1HeightType" type="gml:CodeType"
      substitutionGroup="bldg:_GenericApplicationPropertyOfAbstractBuilding"/>
  <xs:element name="lod0RoofEdge" type="gml:MultiSurfacePropertyType"
      substitutionGroup="bldg:_GenericApplicationPropertyOfAbstractBuilding"/>
</xs:schema>"#
            ),
        )
        .unwrap();
        let schema = SchemaSet::with_ade(&[path]).unwrap();
        let prefixes = Prefixes::with_ade(&[("ade".into(), TEST_ADE.into())]);
        (dir, schema, prefixes)
    }

    #[test]
    fn core_geometry_properties_are_geometry() {
        let schema = SchemaSet::core().unwrap();
        let prefixes = Prefixes::citygml2();
        let is_geometry = |class: &QName, prefix, local| {
            is_geometry_property(schema, &prefixes, class, prefix, local)
        };

        assert!(is_geometry(&building(), "bldg", "lod2Solid"));
        assert!(is_geometry(&building(), "bldg", "lod1TerrainIntersection"));
        assert!(is_geometry(&building(), "bldg", "lod0FootPrint"));
        let tin_relief = QName::new(DEM, "TINRelief");
        assert!(is_geometry(&tin_relief, "dem", "tin"));
        let furniture = QName::new(
            "http://www.opengis.net/citygml/cityfurniture/2.0",
            "CityFurniture",
        );
        assert!(is_geometry(&furniture, "frn", "lod2ImplicitRepresentation"));
        assert!(!is_geometry(&building(), "bldg", "measuredHeight"));
    }

    /// The name alone used to decide, so an ADE attribute named `lod…` was
    /// left out as geometry.
    #[test]
    fn an_ade_attribute_named_like_geometry_is_an_attribute() {
        let (_dir, schema, prefixes) = lod_named_ade();

        assert!(!is_geometry_property(
            &schema,
            &prefixes,
            &building(),
            "ade",
            "lod1HeightType"
        ));
        assert!(is_geometry_property(
            &schema,
            &prefixes,
            &building(),
            "ade",
            "lod0RoofEdge"
        ));

        let tree = serde_json::json!({ "ade:lod1HeightType": "点群から取得_中央値" });
        let reearth_flow_types::AttributeValue::Map(tree) =
            reearth_flow_types::AttributeValue::from(tree)
        else {
            unreachable!()
        };
        let converted = super::super::properties::from_attributes(
            &tree,
            &std::collections::HashSet::new(),
            &prefixes,
            &|prefix, local| is_geometry_property(&schema, &prefixes, &building(), prefix, local),
        );
        assert!(converted.skipped.is_empty(), "{:?}", converted.skipped);
        let (placed, skipped) = place(&schema, &prefixes, &building(), converted.properties);
        assert_eq!(placed_names(&placed), ["ade:lod1HeightType"]);
        assert!(skipped.is_empty());
    }

    /// A geometry name the schema does not type keeps the name's verdict.
    #[test]
    fn an_untyped_geometry_name_is_still_geometry() {
        let schema = SchemaSet::core().unwrap();
        assert!(is_geometry_property(
            schema,
            &Prefixes::citygml2(),
            &building(),
            "bldg",
            "lod9Unknown"
        ));
    }

    const URO: &str = "https://www.geospatial.jp/iur/uro/3.1";

    /// i-UR 3.1, read from a PLATEAU dataset's own copy.
    fn iur() -> (SchemaSet, Prefixes) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../testing/data/fixtures/plateau-citymodel/15202_nagaoka-shi_city_2024_citygml_1_op/schemas/iur/uro/3.1/urbanObject.xsd",
        );
        let schema = SchemaSet::with_ade(&[path]).unwrap();
        let prefixes = Prefixes::with_ade(&[("uro".to_owned(), URO.to_owned())]);
        (schema, prefixes)
    }

    fn curve(property: &str) -> Box<GeometryEntry> {
        Box::new(GeometryEntry {
            lod: 0,
            property: Some(property.to_owned()),
            element: super::super::model::GmlElement::MultiCurve {
                id: None,
                curves: vec![vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]],
            },
            owner: Default::default(),
        })
    }

    /// `uro:bldgDmAttribute` around a `uro:DmGeometricAttribute` keyed `k1`,
    /// holding what its geometry was read from between.
    fn dm_attribute() -> XmlProperty {
        let mut dm = XmlProperty::new("uro", "DmGeometricAttribute");
        dm.synthetic_key = Some("k1".to_owned());
        for local in [
            "dmCode",
            "geometryType",
            "mapLevel",
            "shapeType",
            "elevation",
        ] {
            dm.children.push(prop("uro", local));
        }
        let mut wrapper = XmlProperty::new("uro", "bldgDmAttribute");
        wrapper.children.push(dm);
        wrapper
    }

    fn locals(property: &XmlProperty) -> Vec<&str> {
        property.children.iter().map(|c| c.local.as_str()).collect()
    }

    #[test]
    fn geometry_of_an_ade_element_goes_inside_it_in_schema_order() {
        let (schema, prefixes) = iur();
        let mut properties = [prop("bldg", "class"), dm_attribute()];
        let placed = place_data_geometry(
            &schema,
            &prefixes,
            properties.iter_mut(),
            "k1",
            curve("lod0Geometry"),
        );
        assert!(placed.is_ok());
        let dm = &properties[1].children[0];
        assert_eq!(
            locals(dm),
            [
                "dmCode",
                "geometryType",
                "mapLevel",
                "shapeType",
                "elevation",
                "lod0Geometry"
            ]
        );
        let geometry = &dm.children[5];
        assert_eq!(geometry.prefix, "uro");
        assert_eq!(geometry.geometry, Some(curve("lod0Geometry")));
        assert!(geometry.has_content());
    }

    /// Each refusal hands the geometry back, so the caller still reports it.
    #[test]
    fn geometry_with_nowhere_to_go_inside_an_ade_element_is_handed_back() {
        let (schema, prefixes) = iur();
        let mut properties = [dm_attribute()];
        let mut place = |key: &str, entry: Box<GeometryEntry>| {
            place_data_geometry(&schema, &prefixes, properties.iter_mut(), key, entry)
        };
        // No element carries the key.
        assert!(place("k2", curve("lod0Geometry")).is_err());
        // The element declares no geometry property of that name.
        assert!(place("k1", curve("lod2Solid")).is_err());
        // `uro:dmCode` is not a geometry property, whatever its name.
        assert!(place("k1", curve("dmCode")).is_err());
        // `uro:lod0Geometry` holds one geometry.
        assert!(place("k1", curve("lod0Geometry")).is_ok());
        assert!(place("k1", curve("lod0Geometry")).is_err());
        assert_eq!(
            locals(&properties[0].children[0])
                .iter()
                .filter(|local| **local == "lod0Geometry")
                .count(),
            1
        );
    }
}
