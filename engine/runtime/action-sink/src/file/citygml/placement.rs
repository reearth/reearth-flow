//! Where each property goes in its class's content model.
//!
//! The top level is strict: a property no slot accepts, or one past its
//! slot's `maxOccurs`, is left out and reported, so the document stays valid.
//! Below it, content is only reordered. It almost always came from a source
//! document, so a gap in the model there must not lose valid data; anything
//! out of place is still visible to the schema gate.

use reearth_flow_citygml::schema::{MaxOccurs, QName, SchemaSet};

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
pub fn place(
    schema: &SchemaSet,
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
        if holds_city_object(schema, &property) {
            skipped.push(Skip::NestedObject);
            continue;
        }
        let name = property.qname();
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
                reorder_nested(schema, &mut property);
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
fn holds_city_object(schema: &SchemaSet, property: &XmlProperty) -> bool {
    property
        .children
        .iter()
        .any(|child| is_city_object(schema, &child.qname()))
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
fn reorder_nested(schema: &SchemaSet, property: &mut XmlProperty) {
    for child in &mut property.children {
        reorder_nested(schema, child);
    }
    if let Some(class) = schema.class_for_element(&property.qname()) {
        property
            .children
            .sort_by_cached_key(|child| class.slot_index(&child.qname()).unwrap_or(usize::MAX));
    }
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
        let (placed, skipped) = place(schema, &building(), vec![bounded_by, address]);
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
        let (placed, _) = place(schema, &building(), vec![wrapper]);
        let children: Vec<&str> = placed[0].property.children[0]
            .children
            .iter()
            .map(|c| c.local.as_str())
            .collect();
        assert_eq!(children, ["xalAddress", "multiPoint", "class"]);
    }
}
