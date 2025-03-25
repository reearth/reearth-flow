use std::{
    convert::Infallible,
    fs,
    hash::RandomState,
    io::BufWriter,
    path::Path,
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
};

use atlas_packer::{
    export::{AtlasExporter as _, WebpAtlasExporter},
    pack::AtlasPacker,
    place::{GuillotineTexturePlacer, TexturePlacerConfig},
    texture::{
        cache::{TextureCache, TextureSizeCache},
        DownsampleFactor, PolygonMappedTexture,
    },
};
use bytemuck::Zeroable;
use earcut::{utils3d::project3d_to_2d, Earcut};
use indexmap::IndexSet;
use itertools::Itertools;
use nusamai_citygml::schema::Schema;
use nusamai_projection::cartesian::geodetic_to_geocentric;
use rayon::prelude::*;
use reearth_flow_common::{
    texture::{apply_downsample_factor, get_texture_downsample_scale_of_polygon},
    uri::Uri,
};
use reearth_flow_gltf::calculate_normal;
use reearth_flow_runtime::executor_operation::Context;
use reearth_flow_types::{material, Feature};
use tempfile::tempdir;
use url::Url;

use super::tiling::{TileContent, TileTree};
use super::{
    slice::{slice_to_tiles, SlicedFeature},
    tiling,
};
use crate::file::mvt::tileid::TileIdMethod;

pub(super) fn geometry_slicing_stage(
    upstream: &[Feature],
    schema: &nusamai_citygml::schema::Schema,
    tile_id_conv: TileIdMethod,
    sender_sliced: mpsc::SyncSender<(u64, String, Vec<u8>)>,
    min_zoom: u8,
    max_zoom: u8,
    attach_texture: bool,
) -> crate::errors::Result<()> {
    let bincode_config = bincode::config::standard();
    upstream.iter().par_bridge().try_for_each(|parcel| {
        slice_to_tiles(
            parcel,
            schema,
            min_zoom,
            max_zoom,
            attach_texture,
            |(z, x, y), feature| {
                let bytes = bincode::serde::encode_to_vec(&feature, bincode_config)
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
                        "Failed to send sliced feature with error = {:?}",
                        e
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
                        "Failed to send sorted features with error = {:?}",
                        e
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
                    "Failed to sort features: {:?}",
                    err
                )));
            }
        }
    }

    Ok(())
}

pub(super) fn tile_writing_stage(
    ctx: Context,
    output_path: Uri,
    receiver_sorted: mpsc::Receiver<(u64, String, Vec<Vec<u8>>)>,
    tile_id_conv: TileIdMethod,
    schema: &Schema,
    limit_texture_resolution: Option<bool>,
) -> crate::errors::Result<()> {
    let ellipsoid = nusamai_projection::ellipsoid::wgs84();
    let contents: Arc<Mutex<Vec<TileContent>>> = Default::default();
    let bincode_config = bincode::config::standard();

    // Texture cache
    // use default cache size
    let texture_cache = TextureCache::new(200_000_000);
    let texture_size_cache = TextureSizeCache::new();

    // Use a temporary directory for embedding in glb.
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
            let (tile_zoom, tile_x, tile_y) = tile_id_conv.id_to_zxy(tile_id);
            // Tile information
            let (mut content, translation) = {
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

                (content, translation)
            };

            let mut vertices: IndexSet<[u32; 9], RandomState> = IndexSet::default(); // [x, y, z, u, v, feature_id]
            let mut primitives: reearth_flow_gltf::Primitives = Default::default();

            let mut metadata_encoder = reearth_flow_gltf::MetadataEncoder::new(schema);

            let packer = Mutex::new(AtlasPacker::default());

            // transform features
            let features = {
                let mut features = Vec::new();
                for serialized_feat in feats.into_iter() {
                    let feature = {
                        let (mut feature, _): (SlicedFeature, _) =
                            bincode::serde::decode_from_slice(&serialized_feat, bincode_config)
                                .map_err(|e| {
                                    crate::errors::SinkError::cesium3dtiles_writer(format!(
                                        "Failed to decode_from_slice with {:?}",
                                        e
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

                        feature
                    };
                    features.push(feature);
                }
                features
            };

            // metadata encoding
            let features = features
                .iter()
                .filter(|feature| {
                    let result = metadata_encoder.add_feature(&typename, &feature.attributes);
                    if let Err(e) = result {
                        ctx.event_hub
                            .error_log(None, format!("Failed to add feature with error = {:?}", e));
                        false
                    } else {
                        true
                    }
                })
                .collect::<Vec<_>>();
            // A unique ID used when planning the atlas layout
            //  and when obtaining the UV coordinates after the layout has been completed
            let generate_texture_id = |z, x, y, feature_id, poly_count| {
                format!("{}_{}_{}_{}_{}", z, x, y, feature_id, poly_count)
            };

            // Check the size of all the textures and calculate the power of 2 of the largest size
            let mut max_width = 0;
            let mut max_height = 0;

            // Load all textures into the Packer
            for (feature_id, feature) in features.iter().enumerate() {
                for (poly_count, (mat, poly)) in feature
                    .polygons
                    .iter()
                    .zip_eq(feature.polygon_material_ids.iter())
                    .map(move |(poly, orig_mat_id)| {
                        (feature.materials[*orig_mat_id as usize].clone(), poly)
                    })
                    .enumerate()
                {
                    let t = mat.base_texture.clone();
                    if let Some(base_texture) = t {
                        // texture packing
                        let original_vertices = poly
                            .raw_coords()
                            .iter()
                            .map(|[x, y, z, u, v]| (*x, *y, *z, *u, *v))
                            .collect::<Vec<(f64, f64, f64, f64, f64)>>();

                        let uv_coords = original_vertices
                            .iter()
                            .map(|(_, _, _, u, v)| (*u, *v))
                            .collect::<Vec<(f64, f64)>>();

                        let texture_uri = base_texture.uri.to_file_path().map_err(|_| {
                            crate::errors::SinkError::cesium3dtiles_writer(
                                "Failed to convert texture URI to file path",
                            )
                        })?;
                        let texture_size = texture_size_cache.get_or_insert(&texture_uri);

                        let downsample_scale = if limit_texture_resolution.unwrap_or(false) {
                            get_texture_downsample_scale_of_polygon(
                                &original_vertices,
                                texture_size,
                            ) as f32
                        } else {
                            1.0
                        };

                        let geom_error = tiling::geometric_error(tile_zoom, tile_y);
                        let factor = apply_downsample_factor(geom_error, downsample_scale as f32);
                        let downsample_factor = DownsampleFactor::new(&factor);
                        let cropped_texture = PolygonMappedTexture::new(
                            &texture_uri,
                            texture_size,
                            &uv_coords,
                            downsample_factor,
                        );

                        let scaled_width = (texture_size.0 as f32 * factor) as u32;
                        let scaled_height = (texture_size.1 as f32 * factor) as u32;

                        max_width = max_width.max(scaled_width);
                        max_height = max_height.max(scaled_height);

                        // Unique id required for placement in atlas
                        let (z, x, y) = tile_id_conv.id_to_zxy(tile_id);
                        let texture_id = generate_texture_id(z, x, y, feature_id, poly_count);

                        packer
                            .lock()
                            .map_err(|_| {
                                crate::errors::SinkError::cesium3dtiles_writer(
                                    "Failed to lock the texture packer",
                                )
                            })?
                            .add_texture(texture_id, cropped_texture);
                    }
                }
            }

            let max_width = max_width.next_power_of_two();
            let max_height = max_height.next_power_of_two();

            // initialize texture packer
            // To reduce unnecessary draw calls, set the lower limit for max_width and max_height to 1024
            let config = TexturePlacerConfig {
                width: max_width.max(1024),
                height: max_height.max(1024),
                padding: 0,
            };

            let placer = GuillotineTexturePlacer::new(config.clone());
            let packer = packer.into_inner().map_err(|_| {
                crate::errors::SinkError::cesium3dtiles_writer("Failed to get the texture packer")
            })?;

            // Packing the loaded textures into an atlas
            let packed = packer.pack(placer);

            let exporter = WebpAtlasExporter::default();
            let ext = exporter.clone().get_extension().to_string();

            // Obtain the UV coordinates placed in the atlas by specifying the ID
            //  and apply them to the original polygon.
            for (feature_id, feature) in features.iter().enumerate() {
                for (poly_count, (mut mat, mut poly)) in feature
                    .polygons
                    .iter()
                    .zip_eq(feature.polygon_material_ids.iter())
                    .map(move |(poly, orig_mat_id)| {
                        (feature.materials[*orig_mat_id as usize].clone(), poly)
                    })
                    .enumerate()
                {
                    let original_vertices = poly
                        .raw_coords()
                        .iter()
                        .map(|[x, y, z, u, v]| (*x, *y, *z, *u, *v))
                        .collect::<Vec<(f64, f64, f64, f64, f64)>>();

                    let (z, x, y) = tile_id_conv.id_to_zxy(tile_id);
                    let texture_id = generate_texture_id(z, x, y, feature_id, poly_count);

                    if let Some(info) = packed.get_texture_info(&texture_id) {
                        // Place the texture in the atlas
                        let atlas_placed_uv_coords = info
                            .placed_uv_coords
                            .iter()
                            .map(|(u, v)| ({ *u }, { *v }))
                            .collect::<Vec<(f64, f64)>>();
                        let updated_vertices = original_vertices
                            .iter()
                            .zip(atlas_placed_uv_coords.iter())
                            .map(|((x, y, z, _, _), (u, v))| (*x, *y, *z, *u, *v))
                            .collect::<Vec<(f64, f64, f64, f64, f64)>>();

                        // Apply the UV coordinates placed in the atlas to the original polygon
                        poly.transform_inplace(|&[x, y, z, _, _]| {
                            let (u, v) = updated_vertices
                                .iter()
                                .find(|(x_, y_, z_, _, _)| {
                                    (*x_ - x).abs() < 1e-6
                                        && (*y_ - y).abs() < 1e-6
                                        && (*z_ - z).abs() < 1e-6
                                })
                                .map(|(_, _, _, u, v)| (*u, *v))
                                .unwrap();
                            [x, y, z, u, v]
                        });

                        let atlas_file_name = info.atlas_id.to_string();

                        let atlas_uri = atlas_dir
                            .join(z.to_string())
                            .join(x.to_string())
                            .join(y.to_string())
                            .join(atlas_file_name)
                            .with_extension(ext.clone());

                        // update material
                        mat = material::Material {
                            base_color: mat.base_color,
                            base_texture: Some(material::Texture {
                                uri: Url::from_file_path(atlas_uri).map_err(|_| {
                                    crate::errors::SinkError::cesium3dtiles_writer(
                                        "Failed to convert atlas URI to URL",
                                    )
                                })?,
                            }),
                        };
                    }

                    let primitive = primitives.entry(mat).or_default();
                    primitive.feature_ids.insert(feature_id as u32);

                    if let Some((nx, ny, nz)) =
                        calculate_normal(poly.exterior().iter().map(|v| [v[0], v[1], v[2]]))
                    {
                        let num_outer_points = match poly.hole_indices().first() {
                            Some(&v) => v as usize,
                            None => poly.raw_coords().len(),
                        };
                        let mut earcutter = Earcut::new();
                        let mut buf3d: Vec<[f64; 3]> = Vec::new();
                        let mut buf2d: Vec<[f64; 2]> = Vec::new();
                        let mut index_buf: Vec<u32> = Vec::new();

                        buf3d.clear();
                        buf3d.extend(poly.raw_coords().iter().map(|c| [c[0], c[1], c[2]]));

                        if project3d_to_2d(&buf3d, num_outer_points, &mut buf2d) {
                            // earcut
                            earcutter.earcut(
                                buf2d.iter().cloned(),
                                poly.hole_indices(),
                                &mut index_buf,
                            );

                            // collect triangles
                            primitive.indices.extend(index_buf.iter().map(|&idx| {
                                let [x, y, z, u, v] = poly.raw_coords()[idx as usize];
                                let vbits = [
                                    (x as f32).to_bits(),
                                    (y as f32).to_bits(),
                                    (z as f32).to_bits(),
                                    (nx as f32).to_bits(),
                                    (ny as f32).to_bits(),
                                    (nz as f32).to_bits(),
                                    (u as f32).to_bits(),
                                    // flip the texture v-coordinate
                                    ((1.0 - v) as f32).to_bits(),
                                    (feature_id as f32).to_bits(), // UNSIGNED_INT can't be used for vertex attribute
                                ];
                                let (index, _) = vertices.insert_full(vbits);
                                index as u32
                            }));
                        }
                    }
                }
            }

            // Write to atlas
            let (z, x, y) = tile_id_conv.id_to_zxy(tile_id);
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

            // Write to file
            let mut buffer = Vec::new();
            let writer = BufWriter::new(&mut buffer);
            let content_path = content.content_path.clone();
            contents.lock().unwrap().push(content);
            reearth_flow_gltf::write_gltf_glb(
                writer,
                Some(translation),
                vertices,
                primitives,
                features.len(),
                metadata_encoder,
            )
            .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;

            let storage = ctx
                .storage_resolver
                .resolve(&output_path)
                .map_err(crate::errors::SinkError::cesium3dtiles_writer)?;
            let output_path = output_path.path().join(Path::new(&content_path));
            storage
                .put_sync(Path::new(&output_path), bytes::Bytes::from(buffer))
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
        .map_err(crate::errors::SinkError::file_writer)?;

    Ok(())
}
