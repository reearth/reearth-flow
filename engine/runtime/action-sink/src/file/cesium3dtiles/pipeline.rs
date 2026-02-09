use std::{
    collections::HashMap,
    convert::Infallible,
    fs,
    io::BufWriter,
    path::Path,
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
};

use nusamai_citygml::schema::TypeRef;

use atlas_packer::{
    export::WebpAtlasExporter,
    pack::AtlasPacker,
    texture::cache::{TextureCache, TextureSizeCache},
};
use bytemuck::Zeroable;
use indexmap::IndexSet;
use itertools::Itertools;
use nusamai_citygml::schema::Schema;
use nusamai_projection::cartesian::geodetic_to_geocentric;
use rayon::prelude::*;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_types::Feature;
use tempfile::tempdir;

use super::tiling::{TileContent, TileTree};
use super::{slice::slice_to_tiles, tiling};
use crate::atlas::{
    encode_metadata, load_textures_into_packer, process_geometry_with_atlas_export, GltfFeature,
};
use crate::file::mvt::tileid::TileIdMethod;

pub(super) fn geometry_slicing_stage(
    upstream: &[Feature],
    tile_id_conv: TileIdMethod,
    sender_sliced: mpsc::SyncSender<(u64, String, Vec<u8>)>,
    min_zoom: u8,
    max_zoom: u8,
    attach_texture: bool,
) -> crate::errors::Result<()> {
    upstream.iter().par_bridge().try_for_each(|parcel| {
        slice_to_tiles(
            parcel,
            min_zoom,
            max_zoom,
            attach_texture,
            |(z, x, y), feature| {
                let bytes = serde_json::to_vec(&feature)
                    .map_err(|e| crate::errors::SinkError::cesium3dtiles_writer(e.to_string()))?;
                let Some(feature_type) = parcel.feature_type() else {
                    return Err(crate::errors::SinkError::cesium3dtiles_writer(
                        "Failed to get feature type",
                    ));
                };
                let tile_id = tile_id_conv.zxy_to_id(z, x, y);
                let serialized_feature = (tile_id, feature_type.to_string(), bytes);
                sender_sliced.send(serialized_feature).map_err(|e| {
                    crate::errors::SinkError::cesium3dtiles_writer(format!(
                        "Failed to send sliced feature with error = {e:?}"
                    ))
                })?;
                Ok(())
            },
        )
    })?;

    Ok(())
}

#[derive(
    bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Ord, PartialOrd, PartialEq, Eq, std::fmt::Debug,
)]
#[repr(C)]
pub(super) struct SortKey {
    tile_id: u64,
    type_seq: u64,
}

pub(super) fn feature_sorting_stage(
    receiver_sliced: mpsc::Receiver<(u64, String, Vec<u8>)>,
    sender_sorted: mpsc::SyncSender<(u64, String, Vec<Vec<u8>>)>,
) -> crate::errors::Result<()> {
    let mut typename_to_seq: IndexSet<String, ahash::RandomState> = Default::default();

    let config = kv_extsort::SortConfig::default().max_chunk_bytes(256 * 1024 * 1024); // TODO: Configurable

    let sorted_iter = kv_extsort::sort(
        receiver_sliced
            .into_iter()
            .map(|(tile_id, typename, body)| {
                let (idx, _) = typename_to_seq.insert_full(typename);
                let type_seq = idx as u64;
                std::result::Result::<_, Infallible>::Ok((SortKey { tile_id, type_seq }, body))
            }),
        config,
    );

    for ((_, key), grouped) in &sorted_iter.chunk_by(|feat| match feat {
        Ok((key, _)) => (false, *key),
        Err(_) => (true, SortKey::zeroed()),
    }) {
        let grouped = grouped
            .into_iter()
            .map_ok(|(_, serialized_feats)| serialized_feats)
            .collect::<kv_extsort::Result<Vec<_>, _>>();
        match grouped {
            Ok(serialized_feats) => {
                let tile_id = key.tile_id;
                let typename = typename_to_seq[key.type_seq as usize].clone();
                if let Err(e) = sender_sorted.send((tile_id, typename, serialized_feats)) {
                    return Err(crate::errors::SinkError::cesium3dtiles_writer(format!(
                        "Failed to send sorted features with error = {e:?}"
                    )));
                }
            }
            Err(kv_extsort::Error::Canceled) => {
                return Err(crate::errors::SinkError::cesium3dtiles_writer(
                    "Sort canceled",
                ));
            }
            Err(err) => {
                return Err(crate::errors::SinkError::cesium3dtiles_writer(format!(
                    "Failed to sort features: {err:?}"
                )));
            }
        }
    }

    Ok(())
}

struct TileContext {
    content: TileContent,
    translation: [f64; 3],
}

fn initialize_tile_context(
    tile_id_conv: &TileIdMethod,
    tile_id: u64,
    typename: &str,
) -> TileContext {
    let ellipsoid = nusamai_projection::ellipsoid::wgs84();
    let (tile_zoom, tile_x, tile_y) = tile_id_conv.id_to_zxy(tile_id);

    let (min_lat, max_lat) = tiling::y_slice_range(tile_zoom, tile_y);
    let (min_lng, max_lng) =
        tiling::x_slice_range(tile_zoom, tile_x as i32, tiling::x_step(tile_zoom, tile_y));

    // Use the tile center as the translation of the glTF mesh
    let translation = {
        let (tx, ty, tz) = geodetic_to_geocentric(
            &ellipsoid,
            (min_lng + max_lng) / 2.0,
            (min_lat + max_lat) / 2.0,
            0.,
        );
        // z-up to y-up
        let [tx, ty, tz] = [tx, tz, -ty];
        // double-precision to single-precision
        [(tx as f32) as f64, (ty as f32) as f64, (tz as f32) as f64]
    };

    let content_path = {
        let normalized_typename = typename.replace(':', "_");
        format!("{tile_zoom}/{tile_x}/{tile_y}_{normalized_typename}.glb")
    };

    let content = TileContent {
        zxy: (tile_zoom, tile_x, tile_y),
        content_path,
        min_lng: f64::MAX,
        max_lng: f64::MIN,
        min_lat: f64::MAX,
        max_lat: f64::MIN,
        min_height: f64::MAX,
        max_height: f64::MIN,
    };

    TileContext {
        content,
        translation,
    }
}

fn transform_features(
    feats: Vec<Vec<u8>>,
    content: &mut TileContent,
    translation: [f64; 3],
) -> crate::errors::Result<Vec<GltfFeature>> {
    let ellipsoid = nusamai_projection::ellipsoid::wgs84();
    let mut features = Vec::new();

    for serialized_feat in feats.into_iter() {
        let mut feature: GltfFeature = serde_json::from_slice(&serialized_feat).map_err(|e| {
            crate::errors::SinkError::cesium3dtiles_writer(format!(
                "Failed to decode_from_slice with {e:?}"
            ))
        })?;

        feature
            .polygons
            .transform_inplace(|&[lng, lat, height, u, v]| {
                // Update tile boundary
                content.min_lng = content.min_lng.min(lng);
                content.max_lng = content.max_lng.max(lng);
                content.min_lat = content.min_lat.min(lat);
                content.max_lat = content.max_lat.max(lat);
                content.min_height = content.min_height.min(height);
                content.max_height = content.max_height.max(height);

                // Coordinate transformation
                // - geographic to geocentric
                // - z-up to y-up
                // - subtract the translation
                // - The origin of atlas-packer is in the lower right.
                let (x, y, z) = geodetic_to_geocentric(&ellipsoid, lng, lat, height);
                [
                    x - translation[0],
                    z - translation[1],
                    -y - translation[2],
                    u,
                    v,
                ]
            });

        features.push(feature);
    }

    Ok(features)
}

/// Property metadata for tileset.json properties
#[derive(Debug, Clone, Default)]
struct PropertyMetadata {
    minimum: Option<serde_json::Number>,
    maximum: Option<serde_json::Number>,
}

impl PropertyMetadata {
    fn merge(&mut self, other: &PropertyMetadata) {
        // Merge minimum
        if let Some(other_min) = &other.minimum {
            match &self.minimum {
                Some(self_min) => {
                    if let (Some(a), Some(b)) = (other_min.as_f64(), self_min.as_f64()) {
                        if a < b {
                            self.minimum = Some(other_min.clone());
                        }
                    }
                }
                None => self.minimum = Some(other_min.clone()),
            }
        }
        // Merge maximum
        if let Some(other_max) = &other.maximum {
            match &self.maximum {
                Some(self_max) => {
                    if let (Some(a), Some(b)) = (other_max.as_f64(), self_max.as_f64()) {
                        if a > b {
                            self.maximum = Some(other_max.clone());
                        }
                    }
                }
                None => self.maximum = Some(other_max.clone()),
            }
        }
    }
}

/// Collect property statistics from features based on schema type information
fn collect_property_stats(
    features: &[&GltfFeature],
    typename: &str,
    schema: &Schema,
) -> HashMap<String, PropertyMetadata> {
    use nusamai_citygml::schema::TypeDef;
    use reearth_flow_types::AttributeValue;

    let mut stats: HashMap<String, PropertyMetadata> = HashMap::new();

    let Some(TypeDef::Feature(feature_def)) = schema.types.get(typename) else {
        return stats;
    };

    // Initialize all properties from schema with empty metadata
    for key in feature_def.attributes.keys() {
        stats.insert(key.clone(), PropertyMetadata::default());
    }

    // Update min/max only for numeric types
    for feature in features {
        for (key, value) in &feature.attributes {
            let Some(attr_def) = feature_def.attributes.get(key) else {
                continue;
            };

            // Only collect stats for numeric types
            let numeric_value: Option<serde_json::Number> = match attr_def.type_ref {
                TypeRef::Integer => match value {
                    AttributeValue::Number(n) => n.as_i64().map(serde_json::Number::from),
                    AttributeValue::String(s) => {
                        s.parse::<i64>().ok().map(serde_json::Number::from)
                    }
                    _ => None,
                },
                TypeRef::NonNegativeInteger => match value {
                    AttributeValue::Number(n) => n.as_u64().map(serde_json::Number::from),
                    AttributeValue::String(s) => {
                        s.parse::<u64>().ok().map(serde_json::Number::from)
                    }
                    _ => None,
                },
                TypeRef::Double | TypeRef::Measure => match value {
                    AttributeValue::Number(n) => Some(n.clone()),
                    AttributeValue::String(s) => serde_json::from_str::<serde_json::Number>(s).ok(),
                    _ => None,
                },
                _ => None,
            };

            if let Some(num) = numeric_value {
                let metadata = stats.entry(key.clone()).or_default();
                // Update minimum
                match &metadata.minimum {
                    Some(current) => {
                        if let (Some(new_val), Some(cur_val)) = (num.as_f64(), current.as_f64()) {
                            if new_val < cur_val {
                                metadata.minimum = Some(num.clone());
                            }
                        }
                    }
                    None => metadata.minimum = Some(num.clone()),
                }
                // Update maximum
                match &metadata.maximum {
                    Some(current) => {
                        if let (Some(new_val), Some(cur_val)) = (num.as_f64(), current.as_f64()) {
                            if new_val > cur_val {
                                metadata.maximum = Some(num.clone());
                            }
                        }
                    }
                    None => metadata.maximum = Some(num),
                }
            }
        }
    }

    stats
}

/// Merge per-tile property stats into global stats
fn merge_property_stats(
    global: &mut HashMap<String, PropertyMetadata>,
    local: HashMap<String, PropertyMetadata>,
) {
    for (key, local_meta) in local {
        global.entry(key).or_default().merge(&local_meta);
    }
}

pub(super) fn tile_writing_stage(
    ctx: Context,
    output_path: Uri,
    receiver_sorted: mpsc::Receiver<(u64, String, Vec<Vec<u8>>)>,
    tile_id_conv: TileIdMethod,
    schema: &Schema,
    limit_texture_resolution: Option<bool>,
    draco_compression: bool,
) -> crate::errors::Result<()> {
    let contents: Arc<Mutex<Vec<TileContent>>> = Default::default();
    let property_stats: Arc<Mutex<HashMap<String, PropertyMetadata>>> = Default::default();

    // Texture cache (use default cache size)
    let texture_cache = TextureCache::new(200_000_000);
    let texture_size_cache = TextureSizeCache::new();

    // Use a temporary directory for embedding in glb
    let binding =
        tempdir().map_err(|e| crate::errors::SinkError::cesium3dtiles_writer(e.to_string()))?;
    let folder_path = binding.path();
    let texture_folder_name = "textures";
    let atlas_dir = folder_path.join(texture_folder_name);
    std::fs::create_dir_all(&atlas_dir).map_err(crate::errors::SinkError::cesium3dtiles_writer)?;

    // Make a glTF (.glb) file for each tile
    receiver_sorted
        .into_iter()
        .par_bridge()
        .try_for_each(|(tile_id, typename, feats)| {
            let (tile_zoom, _tile_x, tile_y) = tile_id_conv.id_to_zxy(tile_id);

            // Initialize tile context
            let mut tile_ctx = initialize_tile_context(&tile_id_conv, tile_id, &typename);

            let mut metadata_encoder = reearth_flow_gltf::MetadataEncoder::new(schema);

            // Transform features
            let features = transform_features(feats, &mut tile_ctx.content, tile_ctx.translation)?;

            // Encode metadata and filter valid features
            let valid_features = encode_metadata(&features, &typename, &mut metadata_encoder);

            // Collect property stats from valid features only
            let tile_stats = collect_property_stats(&valid_features, &typename, schema);
            merge_property_stats(&mut property_stats.lock().unwrap(), tile_stats);

            // Prepare texture packing
            let (z, x, y) = tile_id_conv.id_to_zxy(tile_id);
            let geom_error = tiling::geometric_error(tile_zoom, tile_y);

            // DEBUG: check features have textures before packing
            let feats_with_tex = valid_features.iter().filter(|f| f.materials.iter().any(|m| m.base_texture.is_some())).count();
            eprintln!("[DEBUG tile {z}/{x}/{y}] before packing: {feats_with_tex}/{} features have textures, geom_error={geom_error}",
                valid_features.len());

            let packer = Mutex::new(AtlasPacker::default());

            let (max_width, max_height) = load_textures_into_packer(
                &valid_features,
                &packer,
                &texture_size_cache,
                &|feature_id, poly_count| format!("{z}_{x}_{y}_{feature_id}_{poly_count}"),
                geom_error,
                limit_texture_resolution.unwrap_or(false),
            )?;

            // DEBUG: after loading
            eprintln!("[DEBUG tile {z}/{x}/{y}] after load_textures_into_packer: max=({max_width},{max_height})");

            // Export atlas textures
            let atlas_path = atlas_dir
                .join(z.to_string())
                .join(x.to_string())
                .join(y.to_string());
            fs::create_dir_all(&atlas_path)
                .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;

            // To reduce unnecessary draw calls, set the lower limit for max_width and max_height to 1024
            let (primitives, vertices) = process_geometry_with_atlas_export(
                &valid_features,
                packer,
                (max_width.max(1024), max_height.max(1024)),
                WebpAtlasExporter::default(),
                &atlas_path,
                &texture_cache,
                |feature_id, poly_count| format!("{z}_{x}_{y}_{feature_id}_{poly_count}"),
            )?;

            // DEBUG: check primitives after atlas export
            let mats_with_tex = primitives.keys().filter(|m| m.base_texture.is_some()).count();
            eprintln!("[DEBUG tile {z}/{x}/{y}] after process_geometry: {} primitives, {} with texture",
                primitives.len(), mats_with_tex);

            // Write glTF to storage
            let content_path = tile_ctx.content.content_path.clone();
            contents.lock().unwrap().push(tile_ctx.content);

            let mut buffer = Vec::new();
            let writer = BufWriter::new(&mut buffer);

            reearth_flow_gltf::write_gltf_glb(
                writer,
                Some(tile_ctx.translation),
                vertices,
                primitives,
                valid_features.len(),
                metadata_encoder,
                draco_compression,
            )
            .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;

            let storage = ctx
                .storage_resolver
                .resolve(&output_path)
                .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;
            let output_file_path = output_path.path().join(Path::new(&content_path));
            storage
                .put_sync(Path::new(&output_file_path), bytes::Bytes::from(buffer))
                .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;

            Ok::<(), crate::errors::SinkError>(())
        })?;

    // Generate tileset.json
    let mut tree = TileTree::default();
    for content in contents.lock().unwrap().drain(..) {
        tree.add_content(content);
    }

    // Convert property stats to tileset properties format
    let properties = {
        let stats = property_stats.lock().unwrap();
        if stats.is_empty() {
            None
        } else {
            let props: HashMap<String, serde_json::Value> = stats
                .iter()
                .map(|(key, meta)| {
                    let mut obj = serde_json::Map::new();
                    if let Some(min) = &meta.minimum {
                        obj.insert(
                            "minimum".to_string(),
                            serde_json::Value::Number(min.clone()),
                        );
                    }
                    if let Some(max) = &meta.maximum {
                        obj.insert(
                            "maximum".to_string(),
                            serde_json::Value::Number(max.clone()),
                        );
                    }
                    (key.clone(), serde_json::Value::Object(obj))
                })
                .collect();
            if props.is_empty() {
                None
            } else {
                Some(props)
            }
        }
    };

    let tileset = cesiumtiles::tileset::Tileset {
        asset: cesiumtiles::tileset::Asset {
            version: "1.1".to_string(),
            ..Default::default()
        },
        root: tree.into_tileset_root(),
        geometric_error: 1e+100,
        properties,
        ..Default::default()
    };

    let storage = ctx
        .storage_resolver
        .resolve(&output_path)
        .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;

    let root_tileset_path = output_path
        .join(Path::new("tileset.json"))
        .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;
    let tileset_json = serde_json::to_string_pretty(&tileset)
        .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;
    storage
        .put_sync(Path::new(&root_tileset_path.path()), tileset_json.into())
        .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;

    Ok(())
}
