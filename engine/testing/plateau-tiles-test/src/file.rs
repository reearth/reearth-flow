use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Zips the contents of `src_dir` into `zip_path` (no top-level directory prefix).
pub fn zip_dir(src_dir: &Path, zip_path: &Path) -> Result<(), String> {
    let file = fs::File::create(zip_path)
        .map_err(|e| format!("Failed to create zip {:?}: {}", zip_path, e))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let rel = path.strip_prefix(src_dir).unwrap();
            let name = rel.to_string_lossy().to_string();
            zip.start_file(&name, options)
                .map_err(|e| format!("Failed to start zip entry {}: {}", name, e))?;
            let content =
                fs::read(path).map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
            zip.write_all(&content)
                .map_err(|e| format!("Failed to write zip entry {}: {}", name, e))?;
        }
    }

    zip.finish()
        .map_err(|e| format!("Failed to finish zip {:?}: {}", zip_path, e))?;
    Ok(())
}

/// Extracts `zip_path` into a fresh temporary directory and returns its path.
pub fn extract_zip_to_tmp(zip_path: &Path) -> Result<PathBuf, String> {
    let tmp_dir = std::env::temp_dir().join(format!("plateau-tiles-{}", std::process::id()));
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

/// Copies all items from `source_dir` into `output_dir`.
/// - `.zip` files are extracted into a subdirectory named after the zip stem.
/// - Directories are copied recursively.
/// - Other files are copied directly.
pub fn extract_dir(source_dir: &Path, output_dir: &Path) {
    if !source_dir.exists() {
        return;
    }
    fs::create_dir_all(output_dir).unwrap();

    for entry in fs::read_dir(source_dir).unwrap().filter_map(|e| e.ok()) {
        let path = entry.path();
        let dest = output_dir.join(path.file_name().unwrap());
        if path.extension().is_some_and(|e| e == "zip") {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let out = output_dir.join(stem);
            let _ = fs::remove_dir_all(&out);
            fs::create_dir_all(&out).unwrap();
            let mut zip = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
            zip.extract(&out).unwrap();
        } else if path.is_dir() {
            copy_dir_recursive(&path, &dest);
        } else {
            fs::copy(&path, &dest).unwrap();
        }
    }
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap().filter_map(|e| e.ok()) {
        let path = entry.path();
        let dest = dst.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_dir_recursive(&path, &dest);
        } else {
            fs::copy(&path, &dest).unwrap();
        }
    }
}
