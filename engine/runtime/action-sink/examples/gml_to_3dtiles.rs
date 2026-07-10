//! Ad-hoc smoke test for the new-geometry Cesium 3D Tiles writer: parses a
//! CityGML file and writes the resulting tileset, for visually inspecting the
//! output in a viewer.
//!
//! ```sh
//! cargo run -p reearth-flow-action-sink --features new-geometry \
//!     --example gml_to_3dtiles -- <input.gml> <output_dir>
//! ```

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use reearth_flow_action_processor::citygml_parser::parser::Parser;
use reearth_flow_action_processor::citygml_parser::pipeline;
use reearth_flow_action_sink::file::cesium3dtiles::next;
use url::Url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let (Some(input), Some(output)) = (args.next(), args.next()) else {
        eprintln!("usage: gml_to_3dtiles <input.gml> <output_dir>");
        std::process::exit(1);
    };

    let input_path = PathBuf::from(input);
    let output_dir = PathBuf::from(output);
    std::fs::create_dir_all(&output_dir)?;

    let bytes = std::fs::read(&input_path)?;
    let source_url = Url::from_file_path(std::fs::canonicalize(&input_path)?)
        .expect("input path should be a valid file path");

    let mut parser = Parser::new();
    parser.parse(&bytes, &source_url)?;

    let features = pipeline::build_features(
        parser,
        &HashSet::new(),
        &HashMap::new(),
        None,
        false,
        false,
        false,
    );
    println!("parsed {} feature(s)", features.len());

    let built = next::build(&features, next::MetadataOptions::default(), 24)?;

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
