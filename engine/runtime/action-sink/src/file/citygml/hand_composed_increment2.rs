// Increment 2 of the hand-composed experiment: the same question, asked across
// city object classes. The reviewer asked for the test repeated for tran, fld
// and the rest.
//
// Included from `hand_composed_experiment.rs`; add a row to extend the matrix.

use reearth_flow_geometry::Geometry;
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::{AttributeValue, Feature};

use crate::file::citygml::model::CityObjectType;

// From the parent experiment module.
use super::{convert_city_object, wall};

/// What a user must put in `__citygml_feature_type` to reach each class, and
/// what happens when they supply something reasonable but unrecognised.
#[test]
fn the_city_object_class_is_chosen_by_lowercased_substring_match() {
    let cases: &[(&str, CityObjectType)] = &[
        // The qualified names a user would copy out of the CityGML standard.
        ("bldg:Building", CityObjectType::Building),
        ("tran:Road", CityObjectType::Road),
        ("veg:PlantCover", CityObjectType::PlantCover),
        ("dem:ReliefFeature", CityObjectType::ReliefFeature),
        // Unrecognised names degrade silently to a generic object.
        (
            "uro:FloodingRiskAttribute",
            CityObjectType::GenericCityObject,
        ),
        ("frn:Fence", CityObjectType::GenericCityObject),
        // Any string merely CONTAINING a keyword is captured by it.
        ("my_building_export_v2", CityObjectType::Building),
        ("road_centrelines_2024", CityObjectType::Road),
        ("BUILDING", CityObjectType::Building),
    ];

    println!("\n=== INCREMENT 2: __citygml_feature_type -> city object class");
    for (supplied, expected) in cases {
        let got = CityObjectType::from_feature_type(supplied);
        println!("  {supplied:<32} -> {got:?}");
        assert_eq!(&got, expected, "for input {supplied:?}");
    }
    println!("  match is substring, lowercased, first-wins. A dataset name that");
    println!("  happens to contain \"road\" becomes a tran:Road.");
}

/// `fld` is flood data. In PLATEAU it rides as `uro:` attributes on a building
/// rather than as its own city object class, so the attributes are the entire
/// payload. Since feature attributes never reach the converter, flood data has
/// no route to the output at all.
#[test]
fn flood_data_has_no_route_to_the_output() {
    let mut feature = Feature::from(Geometry::Euclidean3D(wall()));
    feature.insert(
        "__citygml_feature_type",
        AttributeValue::String("bldg:Building".to_string()),
    );
    // The shape PLATEAU fld data actually takes.
    feature.insert(
        "uro:bldgDisasterRiskAttribute",
        AttributeValue::String("RiverFloodingRiskAttribute".to_string()),
    );
    feature.insert(
        "uro:depth",
        AttributeValue::Number(serde_json::Number::from(2)),
    );
    feature.insert("uro:rank", AttributeValue::String("3".to_string()));

    let converted = convert_city_object(&feature, &LodMask::all()).expect("converts");

    println!("\n=== INCREMENT 2b: PLATEAU fld attributes on a Building");
    println!("  supplied : uro:bldgDisasterRiskAttribute, uro:depth, uro:rank");
    println!("  geometry entries out : {}", converted.geometries.len());
    println!("  the class is selectable; the flood attributes are not carried");

    assert_eq!(converted.geometries.len(), 1, "only the geometry survives");
}
