//! Hand-composed feature experiment, per review feedback on #2355.
//!
//! The reviewer asked: for a real CityGML building, what set of `Feature`s
//! would a user have to compose upstream to get equivalent output, and is that
//! reasonable to ask of them?
//!
//! Deliberately NOT a round trip. A round trip passes because our own reader
//! populates the writer's undocumented keys with the right values, so both
//! sides collude and the contract stays invisible. These tests compose
//! features the way a user would have to, from scratch.
//!
//! Run with:
//!   cargo test -p reearth-flow-action-sink --features new-geometry \
//!     hand_composed_experiment -- --nocapture

#![cfg(all(test, feature = "new-geometry"))]

use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::polygon::Polygon3D;
use reearth_flow_geometry::{Euclidean3DGeometry, Geometry, GeometryCollection};
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature};

use super::converter::convert_city_object;

fn crs() -> CoordinateFrame {
    CoordinateFrame::Crs(EpsgCode::new(6697))
}

/// One square wall face, the simplest thing a user could plausibly supply.
fn wall() -> Euclidean3DGeometry {
    Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
        crs(),
        vec![
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 0.0, 3.0],
            [0.0, 0.0, 3.0],
            [0.0, 0.0, 0.0],
        ],
        Vec::<Vec<[f64; 3]>>::new(),
    )))
}

/// Attempt 1: the naive composition. A user builds a polygon and writes it.
/// No magic keys, because none are documented.
#[test]
fn a_naive_feature_produces_lod0_and_no_class() {
    let feature = Feature::from(Geometry::Euclidean3D(wall()));
    let converted = convert_city_object(&feature, &LodMask::all()).expect("converts");

    println!("\n=== ATTEMPT 1: naive feature, no attributes set");
    println!("geometry entries : {}", converted.geometries.len());
    for entry in &converted.geometries {
        println!("  lod={} property={:?}", entry.lod, entry.property);
    }
    println!("omissions        : {}", converted.omissions.len());

    assert_eq!(converted.geometries.len(), 1, "the polygon is written");
    assert_eq!(
        converted.geometries[0].lod, 0,
        "LoD silently defaults to 0: a user who wanted LoD2 gets LoD0 with no warning"
    );
    assert!(
        converted.geometries[0].property.is_none(),
        "no CityGML property name, so the writer will synthesise one"
    );
}

/// Attempt 2: the user wants LoD 2. There is no documented way to say so, and
/// the only mechanism is an attribute key that appears nowhere user-facing.
#[test]
fn setting_lod_requires_an_undocumented_member_key() {
    let mut attributes = Attributes::new();
    attributes.insert(Attribute::new("lod"), AttributeValue::Number(2.into()));

    let feature = Feature::from(Geometry::GeometryCollection(
        GeometryCollection::with_attributes(vec![Geometry::Euclidean3D(wall())], vec![attributes])
            .unwrap(),
    ));

    let converted = convert_city_object(&feature, &LodMask::all()).expect("converts");
    println!("\n=== ATTEMPT 2: member attribute `lod` = 2");
    for entry in &converted.geometries {
        println!("  lod={} property={:?}", entry.lod, entry.property);
    }

    assert_eq!(
        converted.geometries[0].lod, 2,
        "the key works, but only if you already knew it was called `lod`"
    );
}

/// Attempt 3: a typo, or the user picking any other reasonable name. There is
/// no error and no warning; the geometry is silently written at LoD 0.
#[test]
fn a_differently_named_lod_key_is_silently_ignored() {
    for key in ["LOD", "levelOfDetail", "citygmlLod", "lod_level"] {
        let mut attributes = Attributes::new();
        attributes.insert(Attribute::new(key), AttributeValue::Number(2.into()));

        let feature = Feature::from(Geometry::GeometryCollection(
            GeometryCollection::with_attributes(
                vec![Geometry::Euclidean3D(wall())],
                vec![attributes],
            )
            .unwrap(),
        ));

        let converted = convert_city_object(&feature, &LodMask::all()).expect("converts");
        println!(
            "\n=== ATTEMPT 3: member attribute `{key}` = 2  ->  lod={}",
            converted.geometries[0].lod
        );
        assert_eq!(
            converted.geometries[0].lod, 0,
            "`{key}` is ignored silently; only the exact string `lod` is read"
        );
    }
}

/// Attempt 4: the user wants a wall to be a WallSurface, which is the whole
/// point of LoD2 and above. There is no input that produces one.
#[test]
fn no_composition_produces_a_boundary_surface() {
    let mut attributes = Attributes::new();
    attributes.insert(Attribute::new("lod"), AttributeValue::Number(2.into()));
    // Every name a user might reasonably try.
    for key in ["boundedBy", "surfaceType", "citygmlSurface", "role"] {
        attributes.insert(
            Attribute::new(key),
            AttributeValue::String("WallSurface".to_string()),
        );
    }

    let feature = Feature::from(Geometry::GeometryCollection(
        GeometryCollection::with_attributes(vec![Geometry::Euclidean3D(wall())], vec![attributes])
            .unwrap(),
    ));

    let converted = convert_city_object(&feature, &LodMask::all()).expect("converts");
    println!("\n=== ATTEMPT 4: every plausible boundary-surface key set");
    println!("  entries: {}", converted.geometries.len());
    for entry in &converted.geometries {
        println!("  lod={} property={:?}", entry.lod, entry.property);
    }
    println!("  -> the converted model has no boundary-surface concept at all");

    // The model this converter produces cannot represent bldg:boundedBy.
    // Nothing the user supplies changes that.
    assert_eq!(converted.geometries.len(), 1);
}

/// Attempt 5: the user's own attributes. A feature carrying real data, of the
/// kind PLATEAU is full of, and of the kind a Flow processor would compute.
#[test]
fn feature_attributes_never_reach_the_converter() {
    let mut feature = Feature::from(Geometry::Euclidean3D(wall()));
    feature.insert(
        "bldg:measuredHeight",
        AttributeValue::Number(serde_json::Number::from_f64(12.5).unwrap()),
    );
    feature.insert(
        "uro:buildingID",
        AttributeValue::String("11222-bldg-0001".to_string()),
    );
    feature.insert(
        "solarRadiation",
        AttributeValue::Number(serde_json::Number::from(842)),
    );

    let converted = convert_city_object(&feature, &LodMask::all()).expect("converts");

    println!("\n=== ATTEMPT 5: three real attributes on the feature");
    println!("  supplied : bldg:measuredHeight, uro:buildingID, solarRadiation");
    println!("  converted model carries geometry only; attributes are not read");
    println!("  entries  : {}", converted.geometries.len());

    assert_eq!(converted.geometries.len(), 1, "geometry survives");
    // `convert_city_object` reads `feature.geometry` and the feature id. It
    // never looks at `feature.attributes`, so there is no path by which any of
    // the three values above can reach the output document.
}

// Increment 2: the same question across city object classes.
#[path = "hand_composed_increment2.rs"]
mod increment2;
