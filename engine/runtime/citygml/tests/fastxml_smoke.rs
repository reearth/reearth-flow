//! fastxml compiles the schema sets Flow ships and the i-UR sets PLATEAU
//! datasets carry, offline. The schema model is built on fastxml's compiled
//! output, so a set it rejects here is a set the model cannot load.

use std::fs;
use std::path::{Path, PathBuf};

use fastxml::schema::xsd::{compile_schemas, parse_xsd_ast, register_builtin_types, XsdSchema};

fn bundle_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas")
}

/// Every document in the embedded bundle, by its `MANIFEST.tsv` key.
fn bundle_files() -> Vec<PathBuf> {
    let manifest = fs::read_to_string(bundle_dir().join("MANIFEST.tsv")).unwrap();
    let files: Vec<PathBuf> = manifest
        .lines()
        .filter(|line| !line.starts_with('#') && !line.starts_with("path\t"))
        .filter_map(|line| line.split('\t').next())
        .filter(|key| !key.is_empty())
        .map(|key| bundle_dir().join(key))
        .collect();
    assert!(!files.is_empty(), "MANIFEST.tsv lists no documents");
    files
}

/// Every `.xsd` under a dataset's `schemas/iur/` tree, so `urf` and any
/// sibling ADE come in with `uro`.
fn iur_files(dataset: &str) -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testing/data/fixtures/plateau-citymodel")
        .join(dataset)
        .join("schemas/iur");
    let mut files = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "xsd") {
                files.push(path);
            }
        }
    }
    files.sort();
    assert!(!files.is_empty(), "no .xsd under {}", root.display());
    files
}

fn parse(path: &Path) -> XsdSchema {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    parse_xsd_ast(&bytes).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn compile(files: &[PathBuf]) {
    let schemas: Vec<XsdSchema> = files.iter().map(|path| parse(path)).collect();
    let mut compiled = compile_schemas(schemas).unwrap_or_else(|e| panic!("{e}"));
    register_builtin_types(&mut compiled);
}

#[test]
fn the_bundle_compiles_on_fastxml() {
    compile(&bundle_files());
}

#[test]
fn i_ur_3_1_compiles_on_fastxml() {
    let mut files = bundle_files();
    files.extend(iur_files("27100_osaka-shi_city_2024_citygml_1_op"));
    compile(&files);
}

#[test]
fn i_ur_3_2_compiles_on_fastxml() {
    let mut files = bundle_files();
    files.extend(iur_files("12347_tako-machi_city_2025_citygml_1_op"));
    compile(&files);
}
