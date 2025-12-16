use plateau_tiles_test::align_cesium::{collect_geometries_by_gmlid, collect_glb_paths_from_tileset};
use reearth_flow_gltf::parse_gltf;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Extract a specific texture by source_idx and name from the tileset
fn extract_texture(
    tileset_dir: &Path,
    output_dir: &Path,
    source_idx: u32,
    texture_name: &str,
    extracted_cache: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Skip if already extracted
    if extracted_cache.contains(texture_name) {
        return Ok(());
    }

    let glb_paths = collect_glb_paths_from_tileset(tileset_dir)?;

    for glb_path in glb_paths {
        let content = fs::read(&glb_path)?;
        let gltf = parse_gltf(&bytes::Bytes::from(content))
            .map_err(|e| format!("Failed to parse GLB {:?}: {}", glb_path, e))?;

        // Find the image with matching source_idx
        if let Some(image) = gltf.images().find(|img| img.index() == source_idx as usize) {
            let image_data = match image.source() {
                gltf::image::Source::View { view, mime_type } => {
                    let buffer = gltf.blob.as_ref().ok_or("No blob in GLB")?;
                    let start = view.offset();
                    let end = start + view.length();

                    let ext = match mime_type.to_lowercase().as_str() {
                        "image/png" => "png",
                        "image/jpeg" => "jpg",
                        "image/webp" => "webp",
                        _ => "bin",
                    }
                    .to_string();

                    (buffer[start..end].to_vec(), ext)
                }
                gltf::image::Source::Uri { uri, .. } => {
                    let image_path = glb_path.parent().unwrap().join(uri);
                    let data = fs::read(&image_path)?;
                    let ext = image_path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("bin")
                        .to_string();
                    (data, ext)
                }
            };

            let output_filename = format!("{}.{}", texture_name, &image_data.1);
            let output_path = output_dir.join(&output_filename);

            fs::write(&output_path, &image_data.0)?;
            extracted_cache.insert(texture_name.to_string());
            return Ok(());
        }
    }

    Err(format!("Texture {} (source_idx={}) not found in tileset", texture_name, source_idx).into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <tileset_dir> [gml_id]", args[0]);
        std::process::exit(1);
    }

    let tileset_dir = PathBuf::from(&args[1]);
    let filter_gml_id = args.get(2);
    assert!(tileset_dir.exists());

    // Use the same function as the test
    let geometries = collect_geometries_by_gmlid(&tileset_dir)
        .map_err(|e| format!("Failed to collect geometries: {}", e))?;

    if let Some(filter_id) = filter_gml_id {
        // Show detailed info for specific feature
        if let Some(detail_levels) = geometries.get(filter_id) {
            println!("Feature: {}", filter_id);
            println!("  Detail levels: {}", detail_levels.len());
            for (i, level) in detail_levels.iter().enumerate() {
                println!("  Level {}:", i);
                println!("    Geometric error: {}", level.geometric_error);
                println!("    Has texture: {}", level.source_idx.is_some());
                if let Some(idx) = level.source_idx {
                    println!("    Texture source index: {}", idx);
                }
                if let Some(name) = &level.texture_name {
                    println!("    Texture name: {}", name);
                }
                println!("    Polygons: {}", level.multipolygon.0.len());
            }
        } else {
            println!("Error: gml_id '{}' not found in this tileset", filter_id);
        }
    } else {
        let mut extracted_cache = HashSet::new();
        let output_dir: Option<PathBuf> = std::env::var("TEXDIR").ok().map(|dir| {
            fs::create_dir_all(&dir).expect("Failed to create output directory");
            PathBuf::from(dir)
        });

        for (gml_id, detail_levels) in geometries.iter() {
            let mut names = HashSet::new();
            for level in detail_levels {
                if let (Some(idx), Some(name)) = (level.source_idx, &level.texture_name) {
                    names.insert(name.clone());
                    let Some(output_dir) = &output_dir else { continue; };
                    if let Err(e) = extract_texture(&tileset_dir, output_dir, idx, name, &mut extracted_cache) {
                        eprintln!("Warning: Failed to extract texture {}: {}", name, e);
                    }
                }
            }

            println!("{}: {} levels, texture={:?}", gml_id, detail_levels.len(), names);
        }

        if !extracted_cache.is_empty() {
            println!("Extracted {} unique textures", extracted_cache.len());
        }
    }

    Ok(())
}
