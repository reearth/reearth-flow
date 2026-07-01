//! Dumps CityGML 2.0 files as JSON features to stdout.
//!
//!   cargo run -p reearth-flow-action-processor --bin citygml2_dump -- a.gml b.gml
//!
//! Multiple files are parsed into one document so cross-file `xlink:href` references resolve.

use std::collections::HashSet;
use std::process::ExitCode;

use reearth_flow_action_processor::citygml_parser::parser::Parser;
use reearth_flow_action_processor::citygml_parser::pipeline::build_features;
use url::Url;

fn main() -> ExitCode {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: citygml2_dump <file.gml>...");
        return ExitCode::FAILURE;
    }

    // Parse every file into one parser so cross-file xlink:href references resolve.
    let mut parser = Parser::new();
    for path in &paths {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("read {path}: {e}");
                return ExitCode::FAILURE;
            }
        };
        let url = match std::fs::canonicalize(path)
            .ok()
            .and_then(|abs| Url::from_file_path(abs).ok())
        {
            Some(url) => url,
            None => {
                eprintln!("cannot resolve {path} to a file URL");
                return ExitCode::FAILURE;
            }
        };
        if let Err(e) = parser.parse(&bytes, &url) {
            eprintln!("parse {path}: {e}");
            return ExitCode::FAILURE;
        }
    }

    let features = build_features(parser, &HashSet::new());
    match serde_json::to_string_pretty(&features) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("serialize: {e}");
            ExitCode::FAILURE
        }
    }
}
