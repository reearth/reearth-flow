//! Dumps CityGML 2.0 files as JSON features to stdout.
//!
//!   cargo run -p reearth-flow-action-processor --bin citygml2_dump -- a.gml b.gml
//!
//! Multiple files are parsed into one document so cross-file `xlink:href` references resolve.

use std::collections::HashSet;
use std::process::ExitCode;

use reearth_flow_action_processor::citygml_parser::pipeline::read_features;
use url::Url;

fn main() -> ExitCode {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: citygml2_dump <file.gml>...");
        return ExitCode::FAILURE;
    }

    let mut loaded: Vec<(Vec<u8>, Url)> = Vec::with_capacity(paths.len());
    for path in &paths {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("read {path}: {e}");
                return ExitCode::FAILURE;
            }
        };
        let url = match std::fs::canonicalize(path).ok().and_then(|abs| Url::from_file_path(abs).ok()) {
            Some(url) => url,
            None => {
                eprintln!("cannot resolve {path} to a file URL");
                return ExitCode::FAILURE;
            }
        };
        loaded.push((bytes, url));
    }

    let sources = loaded.iter().map(|(bytes, url)| (bytes.as_slice(), url));
    match read_features(sources, &HashSet::new()) {
        Ok(features) => match serde_json::to_string_pretty(&features) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("serialize: {e}");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("parse: {e}");
            ExitCode::FAILURE
        }
    }
}
