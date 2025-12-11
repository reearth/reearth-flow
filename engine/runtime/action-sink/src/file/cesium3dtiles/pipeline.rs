use std::{
    convert::Infallible,
    fs,
    io::BufWriter,
    path::Path,
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
};

use ahash::RandomState;
use atlas_packer::{
    export::{AtlasExporter as _, WebpAtlasExporter},
    pack::AtlasPacker,
    place::{GuillotineTexturePlacer, TexturePlacerConfig},
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
use super::{
    slice::{slice_to_tiles, SlicedFeature},
    tiling,
};
use crate::file::mvt::tileid::TileIdMethod;
use crate::atlas::{compute_max_texture_size, load_textures_into_packer, process_geometry_with_atlas, encode_metadata};

pub(super) fn geometry_slicing_stage(
    upstream: &[Feature],
    schema: &nusamai_citygml::schema::Schema,
    tile_id_conv: TileIdMethod,
    sender_sliced: mpsc::SyncSender<(u64, String, Vec<u8>)>,
    min_zoom: u8,
    max_zoom: u8,
    attach_texture: bool,
) -> crate::errors::Result<()> {
    upstream.iter().par_bridge().try_for_each(|parcel| {
        slice_to_tiles(
            parcel,
            schema,
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
    let (min_lng, max_lng) = tiling::x_slice_range(
        tile_zoom,
        tile_x as i32,
        tiling::x_step(tile_zoom, tile_y),
    );

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
) -> crate::errors::Result<Vec<SlicedFeature>> {
    let ellipsoid = nusamai_projection::ellipsoid::wgs84();
    let mut features = Vec::new();

    for serialized_feat in feats.into_iter() {
        let mut feature: SlicedFeature = serde_json::from_slice(&serialized_feat)
            .map_err(|e| {
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
                let (x, y, z) =
                    geodetic_to_geocentric(&ellipsoid, lng, lat, height);
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

            let mut vertices: IndexSet<[u32; 9], RandomState> = IndexSet::default();
            let mut primitives: reearth_flow_gltf::Primitives = Default::default();
            let mut metadata_encoder = reearth_flow_gltf::MetadataEncoder::new(schema);

            // Transform features
            let features = transform_features(feats, &mut tile_ctx.content, tile_ctx.translation)?;

            // Encode metadata and filter valid features
            let valid_features = encode_metadata(&features, &typename, &mut metadata_encoder);

            // Prepare texture packing
            let (z, x, y) = tile_id_conv.id_to_zxy(tile_id);
            let geom_error = tiling::geometric_error(tile_zoom, tile_y);

            let packer = Mutex::new(AtlasPacker::default());

            let geom_error_opt = if limit_texture_resolution.unwrap_or(false) {
                Some(geom_error)
            } else {
                None
            };

            let wrapping_textures = load_textures_into_packer(
                &valid_features,
                &packer,
                &texture_size_cache,
                &|feature_id, poly_count| format!("{z}_{x}_{y}_{feature_id}_{poly_count}"),
                geom_error_opt,
            )?;

            let (max_width, max_height) = compute_max_texture_size(
                &valid_features,
                &texture_size_cache,
                &|feature_id, poly_count| format!("{z}_{x}_{y}_{feature_id}_{poly_count}"),
                &wrapping_textures,
                geom_error_opt,
            )?;

            // Initialize texture packer config
            let config = TexturePlacerConfig {
                width: max_width.max(1024),
                height: max_height.max(1024),
                padding: 0,
            };

            let placer = GuillotineTexturePlacer::new(config.clone());
            let packer = packer.into_inner().map_err(|_| {
                crate::errors::SinkError::cesium3dtiles_writer("Failed to get the texture packer")
            })?;

            // Pack textures into atlas
            let packed = packer.pack(placer);

            let exporter = WebpAtlasExporter::default();
            let ext = exporter.clone().get_extension().to_string();

            // Process geometry and triangulate
            process_geometry_with_atlas(
                &valid_features,
                &packed,
                &wrapping_textures,
                &ext,
                |feature_id, poly_count| format!("{z}_{x}_{y}_{feature_id}_{poly_count}"),
                |atlas_id| {
                    atlas_dir
                        .join(z.to_string())
                        .join(x.to_string())
                        .join(y.to_string())
                        .join(atlas_id.to_string())
                },
                &mut primitives,
                &mut vertices,
            )?;

            // Export atlas textures
            let atlas_path = atlas_dir
                .join(z.to_string())
                .join(x.to_string())
                .join(y.to_string());
            fs::create_dir_all(&atlas_path)
                .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;
            packed.export(
                exporter,
                &atlas_path,
                &texture_cache,
                config.width,
                config.height,
            );

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

    let tileset = cesiumtiles::tileset::Tileset {
        asset: cesiumtiles::tileset::Asset {
            version: "1.1".to_string(),
            ..Default::default()
        },
        root: tree.into_tileset_root(),
        geometric_error: 1e+100,
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
