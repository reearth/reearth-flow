//! Release benchmark for the new-geometry CityGML reader.
//!
//! Mirrors the reader's two hot phases over a directory of `.gml` files: the
//! per-file `process()` (read + `Parser::parse`) and the one-shot `finish()`
//! (pass-2 xlink resolution, appearance indexing, geometry build), timing each.
//!
//! Run:
//! ```text
//! cargo run --release -p reearth-flow-action-processor \
//!   --features new-geometry --example bench_citygml -- <dir>
//! ```

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use reearth_flow_action_processor::citygml_parser::parser::Parser;
use reearth_flow_action_processor::citygml_parser::pipeline::build_features;
use url::Url;

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("usage: bench_citygml <dir-of-gml>");

    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .expect("read dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "gml"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no .gml files in {dir}");

    let mut parser = Parser::new();
    let mut total_bytes = 0u64;
    let mut process_time = Duration::ZERO;
    let mut parse_only = Duration::ZERO;

    for path in &files {
        let t0 = Instant::now();
        let bytes = std::fs::read(path).expect("read file");
        total_bytes += bytes.len() as u64;
        let url = Url::from_file_path(std::fs::canonicalize(path).unwrap()).unwrap();
        let t1 = Instant::now();
        parser.parse(&bytes, &url).expect("parse");
        parse_only += t1.elapsed();
        process_time += t0.elapsed();
    }

    let t = Instant::now();
    let features = build_features(
        parser,
        &HashSet::new(),
        &HashMap::new(),
        None,
        false,
        false,
        false,
    );
    let finish_time = t.elapsed();

    let mib = total_bytes as f64 / 1024.0 / 1024.0;
    println!("files:        {}", files.len());
    println!("total input:  {mib:.2} MiB");
    println!("features:     {}", features.len());
    println!(
        "process():    {:.3} s  (read + parse; parse-only {:.3} s)",
        process_time.as_secs_f64(),
        parse_only.as_secs_f64(),
    );
    println!(
        "finish():     {:.3} s  (pass-2 build)",
        finish_time.as_secs_f64()
    );
    println!(
        "total:        {:.3} s  ({:.1} MiB/s)",
        (process_time + finish_time).as_secs_f64(),
        mib / (process_time + finish_time).as_secs_f64(),
    );
}
