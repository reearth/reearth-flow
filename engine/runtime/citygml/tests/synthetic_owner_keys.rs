#![cfg(feature = "new-geometry")]

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use reearth_flow_citygml::parser::{CityGmlVersion, Parser};
use reearth_flow_citygml::pipeline::{
    build_features, MEMBER_GEOMETRY_FEATURE_TYPE_KEY, MEMBER_GEOMETRY_GML_ID_KEY,
    MEMBER_GEOMETRY_PARENT_GML_ID_KEY, MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY,
};
use reearth_flow_geometry::Geometry;
use reearth_flow_types::{AttributeValue, Attributes};
use url::Url;

/// The PLATEAU 4 L-BLDG-06 fixture: buildings whose 6 ground, 25 roof and 77
/// wall surfaces carry no `gml:id`.
fn l_bldg_06() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../testing/data/testcases/quality-check/plateau4/02-bldg/L-BLDG-06/udx/bldg/input.gml",
    )
}

fn string_of<'a>(attrs: &'a Attributes, key: &str) -> Option<&'a str> {
    match attrs.get(key) {
        Some(AttributeValue::String(s)) => Some(s),
        Some(other) => panic!("{key} is not a string: {other:?}"),
        None => None,
    }
}

#[test]
fn l_bldg_06_id_less_surfaces_get_distinct_synthetic_keys_and_nothing_else() {
    let path = l_bldg_06();
    let source = std::fs::read(&path).unwrap();
    let url = Url::from_file_path(path.canonicalize().unwrap()).unwrap();
    let mut parser = Parser::with_extract_tags(CityGmlVersion::V2, HashSet::new());
    parser.parse(&source, &url).unwrap();
    let features = build_features(
        parser,
        &HashSet::new(),
        &HashMap::new(),
        None,
        true,
        false,
        &[],
        false,
    );

    let mut surface_keys = Vec::new();
    let mut building_members = 0;
    for feature in &features {
        let Geometry::GeometryCollection(gc) = &*feature.geometry else {
            continue;
        };
        for member in gc.member_attributes() {
            let owner_type = string_of(member, MEMBER_GEOMETRY_FEATURE_TYPE_KEY).unwrap();
            if owner_type.ends_with("Surface") {
                let key = string_of(member, MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY)
                    .unwrap_or_else(|| panic!("a {owner_type} member has no synthetic key"));
                assert!(key.starts_with("input.gml#_synid"), "unexpected key {key}");
                assert_eq!(string_of(member, MEMBER_GEOMETRY_GML_ID_KEY), None);
                surface_keys.push(key.to_string());
            } else {
                assert_eq!(owner_type, "bldg:Building");
                building_members += 1;
                assert_eq!(string_of(member, MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY), None);
                assert_eq!(string_of(member, MEMBER_GEOMETRY_PARENT_GML_ID_KEY), None);
            }
        }
    }

    assert!(building_members > 0, "the buildings own geometry too");
    let distinct: HashSet<&String> = surface_keys.iter().collect();
    assert_eq!(distinct.len(), surface_keys.len(), "keys are distinct");
    assert_eq!(distinct.len(), 108);
}
