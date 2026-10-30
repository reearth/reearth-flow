//! The embedded CityGML 2.0 set, against the generated table it replaces and
//! against the i-UR schemas PLATEAU datasets ship.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use reearth_flow_citygml::schema::{MaxOccurs, QName, SchemaSet, Slot};

const HOOK: &str = "_GenericApplicationPropertyOf";

/// The writer's 17 classes and the element each writes.
const CLASSES: &[(&str, &str, &str)] = &[
    (
        "Building",
        "http://www.opengis.net/citygml/building/2.0",
        "Building",
    ),
    (
        "BuildingPart",
        "http://www.opengis.net/citygml/building/2.0",
        "BuildingPart",
    ),
    (
        "Road",
        "http://www.opengis.net/citygml/transportation/2.0",
        "Road",
    ),
    (
        "Railway",
        "http://www.opengis.net/citygml/transportation/2.0",
        "Railway",
    ),
    (
        "Track",
        "http://www.opengis.net/citygml/transportation/2.0",
        "Track",
    ),
    (
        "Square",
        "http://www.opengis.net/citygml/transportation/2.0",
        "Square",
    ),
    (
        "Bridge",
        "http://www.opengis.net/citygml/bridge/2.0",
        "Bridge",
    ),
    (
        "BridgePart",
        "http://www.opengis.net/citygml/bridge/2.0",
        "BridgePart",
    ),
    (
        "Tunnel",
        "http://www.opengis.net/citygml/tunnel/2.0",
        "Tunnel",
    ),
    (
        "TunnelPart",
        "http://www.opengis.net/citygml/tunnel/2.0",
        "TunnelPart",
    ),
    (
        "WaterBody",
        "http://www.opengis.net/citygml/waterbody/2.0",
        "WaterBody",
    ),
    (
        "LandUse",
        "http://www.opengis.net/citygml/landuse/2.0",
        "LandUse",
    ),
    (
        "SolitaryVegetationObject",
        "http://www.opengis.net/citygml/vegetation/2.0",
        "SolitaryVegetationObject",
    ),
    (
        "PlantCover",
        "http://www.opengis.net/citygml/vegetation/2.0",
        "PlantCover",
    ),
    (
        "CityFurniture",
        "http://www.opengis.net/citygml/cityfurniture/2.0",
        "CityFurniture",
    ),
    (
        "ReliefFeature",
        "http://www.opengis.net/citygml/relief/2.0",
        "ReliefFeature",
    ),
    (
        "GenericCityObject",
        "http://www.opengis.net/citygml/generics/2.0",
        "GenericCityObject",
    ),
];

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testing/data")
}

fn frozen() -> Vec<(String, Vec<String>)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/content_model_2_0.txt");
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let (class, props) = line.split_once('\t').unwrap();
            (
                class.to_owned(),
                props.split(' ').map(str::to_owned).collect(),
            )
        })
        .collect()
}

/// A class's declared slot names from `creationDate` onwards, which is where
/// the generated table started.
fn declared_from_core(set: &SchemaSet, namespace: &str, local: &str) -> Vec<String> {
    let class = set
        .class_for_element(&QName::new(namespace, local))
        .unwrap_or_else(|| panic!("{local} has no class model"));
    let names: Vec<String> = class
        .slots()
        .iter()
        .filter_map(|s| s.declared.as_ref())
        .map(|q| q.local.clone())
        .collect();
    let start = names
        .iter()
        .position(|n| n == "creationDate")
        .expect("every class inherits creationDate");
    names[start..].to_vec()
}

#[test]
fn the_core_set_compiles() {
    SchemaSet::core().unwrap();
}

#[test]
fn every_class_reproduces_the_generated_tables_order() {
    let set = SchemaSet::core().unwrap();
    let frozen = frozen();
    assert_eq!(frozen.len(), CLASSES.len());
    for (class, namespace, local) in CLASSES {
        let expected = &frozen.iter().find(|(c, _)| c == class).unwrap().1;
        let got: Vec<String> = declared_from_core(set, namespace, local)
            .into_iter()
            .filter(|n| !n.starts_with(HOOK))
            .collect();
        assert_eq!(&got, expected, "{class}");
    }
}

/// The generated table skipped every `ref=` element, so it had no ADE hooks.
#[test]
fn the_model_has_the_56_hook_positions_the_table_lacked() {
    let set = SchemaSet::core().unwrap();
    let hooks: usize = CLASSES
        .iter()
        .map(|(_, namespace, local)| {
            declared_from_core(set, namespace, local)
                .iter()
                .filter(|n| n.starts_with(HOOK))
                .count()
        })
        .sum();
    assert_eq!(hooks, 56);
}

#[test]
fn a_generic_attribute_is_placed_at_the_city_object_hook() {
    let set = SchemaSet::core().unwrap();
    let building = set
        .class_for_element(&QName::new(
            "http://www.opengis.net/citygml/building/2.0",
            "Building",
        ))
        .unwrap();
    let gen = QName::new(
        "http://www.opengis.net/citygml/generics/2.0",
        "stringAttribute",
    );
    let hook = QName::new(
        "http://www.opengis.net/citygml/2.0",
        "_GenericApplicationPropertyOfCityObject",
    );
    let hook_slot = building
        .slots()
        .iter()
        .position(|s| s.declared.as_ref() == Some(&hook))
        .unwrap();
    assert_eq!(building.slot_index(&gen), Some(hook_slot));
}

fn uro(dataset: &str, version: &str) -> (SchemaSet, String) {
    let path = data().join(format!(
        "fixtures/plateau-citymodel/{dataset}/schemas/iur/uro/{version}/urbanObject.xsd"
    ));
    (
        SchemaSet::with_ade(&[path]).unwrap(),
        format!("https://www.geospatial.jp/iur/uro/{version}"),
    )
}

fn hook_counts(set: &SchemaSet, namespace: &str) -> (usize, usize, usize) {
    let elements: Vec<&QName> = set
        .global_elements()
        .filter(|q| q.namespace == namespace)
        .collect();
    let hooks: Vec<&QName> = elements
        .iter()
        .filter_map(|q| set.substitution_group(q))
        .filter(|h| h.local.starts_with(HOOK))
        .collect();
    let distinct: BTreeSet<&QName> = hooks.iter().copied().collect();
    (elements.len(), hooks.len(), distinct.len())
}

#[test]
fn uro_3_2_matches_the_specs_hand_count() {
    let (set, namespace) = uro("12347_tako-machi_city_2025_citygml_1_op", "3.2");
    assert_eq!(hook_counts(&set, &namespace), (273, 99, 23));
    let tran = set
        .substitution_group(&QName::new(namespace.as_str(), "tranDmAttribute"))
        .unwrap();
    assert_eq!(
        tran.local,
        "_GenericApplicationPropertyOfTransportationComplex"
    );
}

#[test]
fn uro_3_1_counts() {
    let (set, namespace) = uro("27100_osaka-shi_city_2024_citygml_1_op", "3.1");
    assert_eq!(hook_counts(&set, &namespace), (268, 98, 22));
}

/// A global element whose type is anonymous has no class model. None of the
/// CityGML 2.0 elements Flow writes may be one: only the untyped, abstract
/// `_GenericApplicationPropertyOf*` hooks lack a model.
#[test]
fn every_citygml_element_but_the_hooks_has_a_class_model() {
    let set = SchemaSet::core().unwrap();
    let without: Vec<String> = set
        .global_elements()
        .filter(|q| q.namespace.starts_with("http://www.opengis.net/citygml/"))
        .filter(|q| !q.local.starts_with(HOOK))
        .filter(|q| set.class_for_element(q).is_none())
        .map(|q| q.to_string())
        .collect();
    assert!(without.is_empty(), "{without:?}");
    for (class, namespace, local) in CLASSES {
        assert!(
            set.class_for_element(&QName::new(*namespace, *local))
                .is_some(),
            "{class}"
        );
    }
}

/// `tranDmAttribute` is typed, so the model holds its content; the one i-UR 3.2
/// element without a class is a plain `xs:gYear`.
#[test]
fn the_uro_elements_the_tests_touch_have_class_models() {
    let (set, namespace) = uro("12347_tako-machi_city_2025_citygml_1_op", "3.2");
    assert!(set
        .class_for_element(&QName::new(namespace.as_str(), "tranDmAttribute"))
        .is_some());
    let without: Vec<&str> = set
        .global_elements()
        .filter(|q| q.namespace == namespace)
        .filter(|q| set.class_for_element(q).is_none())
        .map(|q| q.local.as_str())
        .collect();
    assert_eq!(without, ["fiscalYearOfPublication"]);
}

fn ns(prefix: &str) -> &'static str {
    match prefix {
        "core" => "http://www.opengis.net/citygml/2.0",
        "bldg" => "http://www.opengis.net/citygml/building/2.0",
        "brid" => "http://www.opengis.net/citygml/bridge/2.0",
        "tun" => "http://www.opengis.net/citygml/tunnel/2.0",
        "tran" => "http://www.opengis.net/citygml/transportation/2.0",
        "wtr" => "http://www.opengis.net/citygml/waterbody/2.0",
        "dem" => "http://www.opengis.net/citygml/relief/2.0",
        "grp" => "http://www.opengis.net/citygml/cityobjectgroup/2.0",
        "gml" => "http://www.opengis.net/gml",
        other => panic!("unknown prefix {other}"),
    }
}

fn q(name: &str) -> QName {
    let (prefix, local) = name.split_once(':').unwrap();
    QName::new(ns(prefix), local)
}

fn core() -> &'static SchemaSet {
    SchemaSet::core().unwrap()
}

fn prefixed(name: &QName) -> String {
    let prefix = [
        "core", "bldg", "brid", "tun", "tran", "wtr", "dem", "grp", "gml",
    ]
    .into_iter()
    .find(|p| ns(p) == name.namespace)
    .unwrap_or_else(|| panic!("no prefix for {name}"));
    format!("{prefix}:{}", name.local)
}

/// The declared names of the properties of `class` that can hold `child`.
fn declared(set: &SchemaSet, class: &str, child: &str) -> Vec<String> {
    let class = q(class);
    let slots = set.class_for_element(&class).unwrap().slots();
    set.slots_holding(&class, &q(child))
        .into_iter()
        .map(|i| prefixed(slots[i].declared.as_ref().unwrap()))
        .collect()
}

fn slot_of<'a>(set: &'a SchemaSet, class: &str, property: &str) -> &'a Slot {
    let property = q(property);
    set.class_for_element(&q(class))
        .unwrap()
        .slots()
        .iter()
        .find(|s| s.declared.as_ref() == Some(&property))
        .unwrap()
}

#[test]
fn a_slot_records_its_declared_elements_type() {
    let building = |p| slot_of(core(), "bldg:Building", p).property_type.clone();
    assert_eq!(
        building("bldg:boundedBy"),
        Some(q("bldg:BoundarySurfacePropertyType"))
    );
    assert_eq!(building("bldg:measuredHeight"), Some(q("gml:LengthType")));
}

#[test]
fn a_slot_that_is_not_one_declared_element_has_no_property_type() {
    let building = core().class_for_element(&q("bldg:Building")).unwrap();
    for slot in building.slots().iter().filter(|s| s.declared.is_none()) {
        assert_eq!(slot.property_type, None);
    }
}

#[test]
fn a_referenced_element_takes_the_type_of_the_global_element() {
    // core:generalizesTo is declared by a reference to a global element.
    assert_eq!(
        slot_of(core(), "bldg:Building", "core:generalizesTo").property_type,
        Some(q("core:GeneralizationRelationType"))
    );
}

#[test]
fn every_property_that_can_hold_a_wall_is_returned_in_schema_order() {
    assert_eq!(
        declared(core(), "bldg:Building", "bldg:WallSurface"),
        ["core:generalizesTo", "bldg:boundedBy"]
    );
    assert_eq!(
        declared(core(), "grp:CityObjectGroup", "bldg:Building"),
        ["core:generalizesTo", "grp:groupMember", "grp:parent"]
    );
}

/// Each nested class, the property of its container that holds it.
const CONTAINMENT: &[(&str, &str, &str)] = &[
    ("bldg:Building", "bldg:WallSurface", "bldg:boundedBy"),
    ("bldg:WallSurface", "bldg:Window", "bldg:opening"),
    (
        "bldg:Building",
        "bldg:BuildingPart",
        "bldg:consistsOfBuildingPart",
    ),
    (
        "bldg:Building",
        "bldg:BuildingInstallation",
        "bldg:outerBuildingInstallation",
    ),
    (
        "bldg:Building",
        "bldg:IntBuildingInstallation",
        "bldg:interiorBuildingInstallation",
    ),
    ("bldg:Building", "bldg:Room", "bldg:interiorRoom"),
    (
        "bldg:Room",
        "bldg:BuildingFurniture",
        "bldg:interiorFurniture",
    ),
    (
        "bldg:Room",
        "bldg:IntBuildingInstallation",
        "bldg:roomInstallation",
    ),
    ("bldg:Room", "bldg:InteriorWallSurface", "bldg:boundedBy"),
    (
        "brid:Bridge",
        "brid:BridgePart",
        "brid:consistsOfBridgePart",
    ),
    ("brid:Bridge", "brid:WallSurface", "brid:boundedBy"),
    (
        "brid:Bridge",
        "brid:BridgeConstructionElement",
        "brid:outerBridgeConstruction",
    ),
    ("tun:Tunnel", "tun:TunnelPart", "tun:consistsOfTunnelPart"),
    ("tun:Tunnel", "tun:WallSurface", "tun:boundedBy"),
    ("tran:Road", "tran:TrafficArea", "tran:trafficArea"),
    (
        "tran:Road",
        "tran:AuxiliaryTrafficArea",
        "tran:auxiliaryTrafficArea",
    ),
    ("wtr:WaterBody", "wtr:WaterSurface", "wtr:boundedBy"),
    (
        "dem:ReliefFeature",
        "dem:BreaklineRelief",
        "dem:reliefComponent",
    ),
];

#[test]
fn each_nested_class_has_its_containment_property() {
    for (class, child, property) in CONTAINMENT {
        let mut got = declared(core(), class, child);
        got.retain(|p| p != "core:generalizesTo");
        assert_eq!(got, [*property], "{class} / {child}");
    }
}

#[test]
fn a_class_with_no_property_for_the_child_returns_only_generalizes_to() {
    assert_eq!(
        declared(core(), "bldg:Building", "bldg:Window"),
        ["core:generalizesTo"]
    );
    assert_eq!(
        declared(core(), "bldg:Building", "tran:TrafficArea"),
        ["core:generalizesTo"]
    );
}

#[test]
fn an_unknown_class_has_no_slot_holding_anything() {
    let unknown = QName::new("urn:none", "Nothing");
    assert!(core()
        .slots_holding(&unknown, &q("bldg:WallSurface"))
        .is_empty());
}

#[test]
fn every_containment_property_in_the_table_is_unbounded() {
    for (class, child, property) in CONTAINMENT {
        let class = q(class);
        let slots = core().class_for_element(&class).unwrap().slots();
        let property = q(property);
        let index = core()
            .slots_holding(&class, &q(child))
            .into_iter()
            .find(|&i| slots[i].declared.as_ref() == Some(&property))
            .unwrap();
        assert_eq!(slots[index].max, MaxOccurs::Unbounded, "{property}");
    }
}
