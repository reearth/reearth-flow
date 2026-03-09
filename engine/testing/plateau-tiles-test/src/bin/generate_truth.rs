#![allow(dead_code, unused_imports)]

#[path = "../cast_config.rs"]
mod cast_config;
#[path = "../align_mvt.rs"]
mod align_mvt;
#[path = "../compare_attributes.rs"]
mod compare_attributes;
#[path = "../conv_mvt.rs"]
mod conv_mvt;
#[path = "../conv_png.rs"]
mod conv_png;
#[path = "../raster.rs"]
mod raster;

use cast_config::CastConfigValue;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct Profile {
    #[serde(default)]
    convs: Convs,
}

#[derive(Debug, Deserialize, Default)]
struct Convs {
    #[serde(default)]
    mvt: HashMap<String, ConvMvtEntry>,
    #[serde(default)]
    png: HashMap<String, ConvPngEntry>,
}

#[derive(Debug, Deserialize)]
struct ConvMvtEntry {
    fme_path: String,
    truth_path: String,
    #[serde(default)]
    casts: Option<HashMap<String, CastConfigValue>>,
}

#[derive(Debug, Deserialize)]
struct ConvPngEntry {
    fme_path: String,
    truth_path: String,
}

fn run(profile_path: &Path, fme_root: &Path) -> Result<(), String> {
    let content =
        fs::read_to_string(profile_path).map_err(|e| format!("Failed to read profile: {}", e))?;
    let profile: Profile =
        toml::from_str(&content).map_err(|e| format!("Failed to parse profile: {}", e))?;

    let testcase_dir = profile_path.parent().unwrap();

    for (id, entry) in &profile.convs.mvt {
        let mvt_dir = fme_root.join(&entry.fme_path);
        let output_path = testcase_dir.join(&entry.truth_path);
        conv_mvt::write_mvt_json(&mvt_dir, &output_path, entry.casts.as_ref())?;
        println!("wrote mvt/{} -> {}", id, output_path.display());
    }

    for (id, entry) in &profile.convs.png {
        let mvt_dir = fme_root.join(&entry.fme_path);
        let truth_dir = testcase_dir.join(&entry.truth_path);
        conv_png::write_png_truth(&mvt_dir, &truth_dir)?;
        println!("wrote png/{} -> {}", id, truth_dir.display());
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: generate-truth <profile.toml> <fme_root_dir>");
        std::process::exit(1);
    }

    let profile_path = PathBuf::from(&args[1]);
    let fme_root = PathBuf::from(&args[2]);

    if let Err(e) = run(&profile_path, &fme_root) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
