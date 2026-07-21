//! Ad-hoc smoke test for the new-geometry Cesium 3D Tiles writer: parses a
//! CityGML file and writes the resulting tileset, for visually inspecting the
//! output in a viewer.
//!
//! ```sh
//! cargo run -p reearth-flow-action-sink --features new-geometry \
//!     --example gml_to_3dtiles -- <input.gml> <output_dir> [draco] [compute_flat_normal] [lod]
//! ```
//!
//! `draco` and `compute_flat_normal` are optional `1`/`0` flags, both default `1`.
//! `lod` (optional) keeps only geometry from that LOD; without it every LOD is
//! written (LODs then overlap in the viewer).

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use reearth_flow_action_processor::citygml_parser::parser::Parser;
use reearth_flow_action_processor::citygml_parser::pipeline::{self, MEMBER_LOD_KEY};
use reearth_flow_action_sink::file::cesium3dtiles::next;
use reearth_flow_geometry::{Geometry, GeometryCollection};
use reearth_flow_types::{Attribute, Feature};
use url::Url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let (Some(input), Some(output)) = (args.next(), args.next()) else {
        eprintln!("usage: gml_to_3dtiles <input.gml> <output_dir> [draco] [compute_flat_normal]");
        std::process::exit(1);
    };
    let flag = |arg: Option<std::ffi::OsString>| arg.is_none_or(|a| a != "0");
    let draco = flag(args.next());
    let compute_flat_normal = flag(args.next());
    let lod: Option<u8> = args
        .next()
        .and_then(|a| a.to_str().and_then(|s| s.parse().ok()));

    let input_path = PathBuf::from(input);
    let output_dir = PathBuf::from(output);
    std::fs::create_dir_all(&output_dir)?;

    let bytes = std::fs::read(&input_path)?;
    let source_url = Url::from_file_path(std::fs::canonicalize(&input_path)?)
        .expect("input path should be a valid file path");

    let mut parser = Parser::new();
    parser.parse(&bytes, &source_url)?;

    let mut features = pipeline::build_features(
        parser,
        &HashSet::new(),
        &HashMap::new(),
        None,
        false,
        false,
        false,
    );
    println!("parsed {} feature(s)", features.len());

    if let Some(lod) = lod {
        for feature in &mut features {
            keep_lod(feature, lod);
        }
        println!("filtered to LOD {lod}");
    }

    let built = next::build(
        &features,
        next::MetadataOptions::default(),
        24,
        draco,
        compute_flat_normal,
    )?;

    std::fs::write(output_dir.join("tileset.json"), &built.tileset_json)?;
    for (relative_path, bytes) in built.tiles.iter().chain(&built.subtrees) {
        let path = output_dir.join(relative_path);
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, bytes)?;
    }
    println!(
        "wrote {}/tileset.json, {} content glb(s), {} subtree file(s)",
        output_dir.display(),
        built.tiles.len(),
        built.subtrees.len()
    );

    Ok(())
}

/// Drop every `GeometryCollection` member whose recorded LOD (see
/// [`MEMBER_LOD_KEY`]) isn't `lod`, so only that LOD is written and LODs no
/// longer overlap. Features whose geometry isn't a collection are left as-is.
fn keep_lod(feature: &mut Feature, lod: u8) {
    let Geometry::GeometryCollection(collection) = feature.geometry_mut() else {
        return;
    };
    let key = Attribute::new(MEMBER_LOD_KEY);
    let (members, attrs): (Vec<_>, Vec<_>) = collection
        .members()
        .iter()
        .zip(collection.member_attributes())
        .filter(|(_, attr)| attr.get(&key).and_then(|v| v.as_i64()) == Some(lod as i64))
        .map(|(member, attr)| (member.clone(), attr.clone()))
        .unzip();
    *feature.geometry_mut() = Geometry::GeometryCollection(
        GeometryCollection::with_attributes(members, attrs)
            .expect("attrs stay parallel to members"),
    );
}
