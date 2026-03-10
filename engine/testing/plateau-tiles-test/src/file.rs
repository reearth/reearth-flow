use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn perr(p: &Path, err: impl std::fmt::Display) -> String {
    format!("{:?}: {}", p, err)
}

/// Zips the contents of `src_dir` into `zip_path` (no top-level directory prefix).
pub fn zip_dir(src_dir: &Path, zip_path: &Path) -> Result<(), String> {
    let file = fs::File::create(zip_path).map_err(|e| perr(zip_path, e))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|r| r.ok()) {
        let path = entry.path();
        if path.is_file() {
            let rel = path.strip_prefix(src_dir).map_err(|e| perr(path, e))?;
            let name = rel.to_string_lossy().to_string();
            zip.start_file(&name, options).map_err(|e| perr(path, e))?;
            let content = fs::read(path).map_err(|e| perr(path, e))?;
            zip.write_all(&content).map_err(|e| perr(path, e))?;
        }
    }

    zip.finish().map_err(|e| perr(zip_path, e))?;
    Ok(())
}

/// Extracts `zip_path` into a fresh temporary directory and returns its path.
pub fn extract_zip_to_tmp(zip_path: &Path) -> Result<PathBuf, String> {
    let tmp_dir = std::env::temp_dir().join(format!("plateau-tiles-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).map_err(|e| perr(&tmp_dir, e))?;
    let file = fs::File::open(zip_path).map_err(|e| perr(zip_path, e))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| perr(zip_path, e))?;
    zip.extract(&tmp_dir).map_err(|e| perr(zip_path, e))?;
    Ok(tmp_dir)
}

/// Copies all items from `source_dir` into `output_dir`.
/// - `.zip` files are extracted into a subdirectory named after the zip stem.
/// - Directories are copied recursively.
/// - Other files are copied directly.
pub fn extract_dir(source_dir: &Path, output_dir: &Path) -> Result<(), String> {
    if !source_dir.exists() {
        return Ok(());
    }
    fs::create_dir_all(output_dir).map_err(|err| perr(output_dir, err))?;
    for entry in fs::read_dir(source_dir)
        .map_err(|err| perr(source_dir, err))?
        .filter_map(|r| r.ok())
    {
        let path = entry.path();
        let dest = output_dir.join(
            path.file_name()
                .ok_or_else(|| format!("no filename: {:?}", path))?,
        );
        if path.extension().is_some_and(|ext| ext == "zip") {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| format!("bad stem: {:?}", path))?;
            let out = output_dir.join(stem);
            let _ = fs::remove_dir_all(&out);
            fs::create_dir_all(&out).map_err(|err| perr(&out, err))?;
            zip::ZipArchive::new(fs::File::open(&path).map_err(|err| perr(&path, err))?)
                .map_err(|err| perr(&path, err))?
                .extract(&out)
                .map_err(|err| perr(&path, err))?;
        } else if path.is_dir() {
            copy_dir_recursive(&path, &dest)?;
        } else {
            fs::copy(&path, &dest).map_err(|err| perr(&path, err))?;
        }
    }
    Ok(())
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|err| perr(dst, err))?;
    for entry in fs::read_dir(src)
        .map_err(|err| perr(src, err))?
        .filter_map(|r| r.ok())
    {
        let path = entry.path();
        let dest = dst.join(
            path.file_name()
                .ok_or_else(|| format!("no filename: {:?}", path))?,
        );
        if path.is_dir() {
            copy_dir_recursive(&path, &dest)?;
        } else {
            fs::copy(&path, &dest).map_err(|err| perr(&path, err))?;
        }
    }
    Ok(())
}
