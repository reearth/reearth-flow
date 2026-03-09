use plateau_tiles_test::conv_mvt;
use plateau_tiles_test::conv_png;
use plateau_tiles_test::profile_config::Convs;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct Profile {
    #[serde(default)]
    convs: Convs,
}

fn run(profile_path: &Path, fme_root: &Path) -> Result<(), String> {
    let content =
        fs::read_to_string(profile_path).map_err(|e| format!("Failed to read profile: {}", e))?;
    let profile: Profile =
        toml::from_str(&content).map_err(|e| format!("Failed to parse profile: {}", e))?;

    let testcase_dir = profile_path.parent().unwrap();

    for (id, entry) in &profile.convs.mvt {
        let fme_path = entry
            .fme_path
            .as_deref()
            .expect("convs.mvt entry requires fme_path");
        let truth_path = entry
            .truth_path
            .as_deref()
            .expect("convs.mvt entry requires truth_path");
        let mvt_dir = fme_root.join(fme_path);
        let output_path = testcase_dir.join(truth_path);
        conv_mvt::write_mvt_json(&mvt_dir, &output_path, entry.casts.as_ref())?;
        println!("wrote mvt/{} -> {}", id, output_path.display());
    }

    for (id, entry) in &profile.convs.mvt_png {
        let fme_path = entry
            .fme_path
            .as_deref()
            .expect("convs.mvt_png entry requires fme_path");
        let mvt_dir = fme_root.join(fme_path);
        let truth_dir = testcase_dir.join(&entry.truth_path);
        conv_png::write_png_truth(&mvt_dir, &truth_dir, entry.tiles.as_deref())?;
        println!("wrote mvt_png/{} -> {}", id, truth_dir.display());
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
