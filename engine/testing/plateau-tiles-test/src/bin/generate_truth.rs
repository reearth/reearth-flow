use plateau_tiles_test::conv::mvt;
use plateau_tiles_test::conv::png;
use plateau_tiles_test::profile_config::Convs;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct Profile {
    #[serde(default)]
    convs: Convs,
}

fn run(profile_path: &Path) -> Result<(), String> {
    let content =
        fs::read_to_string(profile_path).map_err(|e| format!("Failed to read profile: {}", e))?;
    let profile: Profile =
        toml::from_str(&content).map_err(|e| format!("Failed to parse profile: {}", e))?;

    let testcase_dir = profile_path.parent().unwrap();
    let fme_dir = testcase_dir.join("fme");

    for (id, entry) in &profile.convs.json {
        if !entry.generate_truth {
            continue;
        }
        let flow_file = fme_dir.join(&entry.flow_path);
        let output_path = fme_dir.join(&entry.output_path);
        plateau_tiles_test::conv::json::write_json(
            &flow_file,
            &output_path,
            entry.json_path.as_deref(),
            &entry.casts,
        )?;
        println!("wrote json/{} -> {}", id, output_path.display());
    }

    for (id, entry) in &profile.convs.mvt_attributes {
        if !entry.generate_truth {
            continue;
        }
        let stem = Path::new(&entry.path)
            .file_name()
            .expect("convs.mvt_attributes path must have a file name");
        let zip_path = fme_dir.join(stem).with_extension("zip");
        let tmp_dir = extract_zip_to_tmp(&zip_path)?;
        let output_path = fme_dir.join(&entry.truth_path);
        let result = mvt::write_mvt_json(&tmp_dir, &output_path, entry.casts.as_ref());
        fs::remove_dir_all(&tmp_dir).ok();
        result?;
        println!("wrote mvt_attributes/{} -> {}", id, output_path.display());
    }

    for (id, entry) in &profile.convs.mvt_png {
        if !entry.generate_truth {
            continue;
        }
        let stem = Path::new(&entry.path)
            .file_name()
            .expect("convs.mvt_png path must have a file name");
        let zip_path = fme_dir.join(stem).with_extension("zip");
        let tmp_dir = extract_zip_to_tmp(&zip_path)?;
        let truth_dir = fme_dir.join(&entry.truth_path);
        let result = png::write_png_truth(&tmp_dir, &truth_dir, entry.tiles.as_deref());
        fs::remove_dir_all(&tmp_dir).ok();
        result?;
        println!("wrote mvt_png/{} -> {}", id, truth_dir.display());
    }

    Ok(())
}

fn extract_zip_to_tmp(zip_path: &Path) -> Result<PathBuf, String> {
    let tmp_dir = std::env::temp_dir().join(format!("generate-truth-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).map_err(|e| format!("Failed to create tmp dir: {}", e))?;
    let file = fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open zip {:?}: {}", zip_path, e))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read zip {:?}: {}", zip_path, e))?;
    zip.extract(&tmp_dir)
        .map_err(|e| format!("Failed to extract zip {:?}: {}", zip_path, e))?;
    Ok(tmp_dir)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: generate-truth <profile.toml>");
        std::process::exit(1);
    }

    let profile_path = PathBuf::from(&args[1]);

    if let Err(e) = run(&profile_path) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
