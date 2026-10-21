//! Where each property goes in its class's content model.
//!
//! The top level is strict: a property no slot accepts, or one past its
//! slot's `maxOccurs`, is left out and reported, so the document stays valid.
//! Below it, content is only reordered. It almost always came from a source
//! document, so a gap in the model there must not lose valid data; anything
//! out of place is still visible to the schema gate. The one exception is an
//! element whose required child was left out for an empty value: it is
//! dropped, silently, rather than written invalid.

use reearth_flow_citygml::schema::{MaxOccurs, QName, SchemaSet};

use super::classes::Prefixes;
use super::properties::{Skip, XmlProperty};

const CORE: &str = "http://www.opengis.net/citygml/2.0";

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
        if !prune_emptied(schema, prefixes, &mut property) {
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

/// Whether a property wraps a city object, such as `bldg:boundedBy` around a
/// `bldg:WallSurface`. Those are nested objects, written from the ownership
/// keys rather than from attributes.
fn holds_city_object(schema: &SchemaSet, prefixes: &Prefixes, property: &XmlProperty) -> bool {
    property
        .children
        .iter()
        .any(|child| is_city_object(schema, &child.qname(prefixes)))
}

fn is_city_object(schema: &SchemaSet, element: &QName) -> bool {
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
/// left with nothing at all. Returns whether `property` itself stays.
///
/// Only an element marked `emptied`, or one that lost a child here, is
/// judged: one the source wrote without a required child is kept as it was.
fn prune_emptied(schema: &SchemaSet, prefixes: &Prefixes, property: &mut XmlProperty) -> bool {
    let before = property.children.len();
    property
        .children
        .retain_mut(|child| prune_emptied(schema, prefixes, child));
    property.emptied |= property.children.len() < before;
    if !property.emptied {
        return true;
    }
    if property.is_empty() {
        return false;
    }
    let Some(class) = schema.class_for_element(&property.qname(prefixes)) else {
        return true;
    };
    class.slots().iter().all(|slot| {
        slot.min == 0
            || property
                .children
                .iter()
                .any(|child| slot.accepts(&child.qname(prefixes)))
    })
}

/// Whether `local` is the local name of a property `class_element`'s content
/// model names, in any namespace. Only named elements count, never a
/// wildcard, so pipeline data such as `path` is not mistaken for one.
pub fn names_property(schema: &SchemaSet, class_element: &QName, local: &str) -> bool {
    schema
        .class_for_element(class_element)
        .is_some_and(|class| {
            class.slots().iter().any(|slot| {
                slot.accepts
                    .elements
                    .iter()
                    .any(|element| element.local == local)
            })
        })
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
    fn a_property_is_named_by_its_local_name_alone() {
        let schema = SchemaSet::core().unwrap();
        assert!(names_property(schema, &building(), "measuredHeight"));
        assert!(names_property(schema, &building(), "creationDate"));
        assert!(!names_property(schema, &building(), "path"));
        assert!(!names_property(
            schema,
            &QName::new(BLDG, "NoSuchClass"),
            "measuredHeight"
        ));
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

    /// A generic attribute as conversion leaves it when its `gen:value` was
    /// empty: the name, no value, marked emptied.
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

    /// `gen:value` is required, so a generic attribute whose value was empty
    /// goes as a whole, silently, rather than being written invalid.
    #[test]
    fn a_generic_attribute_whose_value_was_empty_is_dropped_silently() {
        let schema = SchemaSet::core().unwrap();
        let (placed, skipped) = place(
            schema,
            &Prefixes::citygml2(),
            &building(),
            vec![emptied_generic("note"), generic("kept")],
        );
        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0].property.attr("name"), Some("kept"));
        assert!(skipped.is_empty());
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
        assert!(skipped.is_empty());
    }

    /// `core:Address` requires `core:xalAddress`: when that was empty the
    /// address goes despite its `gml:id`, and the property wrapping it, left
    /// with nothing, goes too.
    #[test]
    fn an_element_missing_a_required_child_after_an_empty_value_is_dropped_with_its_wrapper() {
        let schema = SchemaSet::core().unwrap();
        let mut address = XmlProperty::new("core", "Address");
        address.attrs.push(("gml:id".to_owned(), "a1".to_owned()));
        address.emptied = true;
        let mut wrapper = XmlProperty::new("bldg", "address");
        wrapper.children.push(address);
        let (placed, skipped) = place(
            schema,
            &Prefixes::citygml2(),
            &building(),
            vec![wrapper, prop("bldg", "class")],
        );
        assert_eq!(placed_names(&placed), ["bldg:class"]);
        assert!(skipped.is_empty());
    }

    /// Only an empty value drops an element: one the source already wrote
    /// without its required child is written as it was.
    #[test]
    fn an_element_the_source_wrote_without_a_required_child_is_kept() {
        let schema = SchemaSet::core().unwrap();
        let mut generic = emptied_generic("as-read");
        generic.emptied = false;
        let (placed, skipped) = place(schema, &Prefixes::citygml2(), &building(), vec![generic]);
        assert_eq!(placed_names(&placed), ["gen:stringAttribute"]);
        assert!(skipped.is_empty());
    }
}
