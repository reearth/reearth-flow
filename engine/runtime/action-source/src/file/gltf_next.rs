//! New-geometry path for the glTF Reader.
//!
//! Declared as a child module of `gltf.rs` so it reuses that module's scene
//! traversal, buffer loading, and old-world triangle extraction via `super::`.
//! The old extraction (`reearth_flow_gltf::create_geometry_from_primitives_with_transform`)
//! still returns the old `Geometry3D`; we convert that into the new
//! `reearth_flow_geometry::Geometry`. glTF vertices are in model space with no
//! CRS, so every leaf uses `CoordinateFrame::Euclidean` (no axis-swap / no
//! reprojection, unlike the GeoPackage/GeoJSON readers).

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::{image::MimeType, uri::Uri};
use reearth_flow_geometry::{
    appearance::{
        AlphaMode, ChannelId, FaceBinding, Filter, Material, MaterialIndex, PbrMaterial, Raster,
        RasterData, Sampler, Texture, TextureTransform, ThemeId, UvSource, WrapMode,
    },
    coordinate::CoordinateFrame,
    triangular_mesh::TriangularMesh3D,
    Euclidean3DGeometry, Geometry,
};
use reearth_flow_runtime::{
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use tokio::sync::mpsc::Sender;

use crate::{errors::SourceError, file::reader::runner::get_input_path};

use super::{load_buffers, GltfReaderCompiledParam, MeshInfo};

/// New-geometry glTF read: mirrors the old-world `read_gltf` traversal, converting
/// each extracted geometry into the new model and streaming features to `sender` as
/// they are produced (no buffering of the full feature list), matching `read_gltf`.
pub(super) async fn read(
    ctx: &NodeContext,
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    content: &Bytes,
    params: &GltfReaderCompiledParam,
    sender: &Sender<(Port, IngestionMessage)>,
) -> Result<(), SourceError> {
    let gltf_uri = get_input_path(&params.common)
        .map_err(SourceError::GltfReader)?
        .unwrap_or_else(|| Uri::from_str("file://./unknown.gltf").unwrap());

    let gltf = gltf::Gltf::from_slice(content)
        .map_err(|e| SourceError::GltfReader(format!("Failed to parse glTF: {e}")))?;

    let buffer_data = load_buffers(&gltf, ctx, storage_resolver, &gltf_uri, content).await?;

    // Decoded once per document: every split feature below looks up its row in
    // here by (propertyTable index, feature ID).
    let structural_metadata = reearth_flow_gltf::read_structural_metadata(&gltf, &buffer_data)
        .map_err(|e| {
            SourceError::GltfReader(format!("Failed to read glTF structural metadata: {e}"))
        })?;

    // Collect lightweight mesh info with transforms (traversal only; heavy geometry
    // processing happens per-mesh below), same as the old-world path.
    let mut mesh_infos = Vec::new();
    for scene in gltf.scenes() {
        reearth_flow_gltf::traverse_scene(
            &scene,
            |node, world_transform| -> Result<(), SourceError> {
                if let Some(mesh) = node.mesh() {
                    let primitives: Vec<_> = mesh.primitives().collect();
                    if !primitives.is_empty() {
                        mesh_infos.push(MeshInfo {
                            primitives,
                            mesh_name: mesh.name().map(|s| s.to_string()),
                            node_name: if params.include_nodes {
                                node.name().map(|s| s.to_string())
                            } else {
                                None
                            },
                            transform: world_transform.clone(),
                        });
                    }
                }
                Ok(())
            },
        )?;
    }

    // Meshes that carry their own EXT_mesh_features feature IDs always split
    // into one Flow feature per ID, streamed immediately, regardless of
    // `merge_meshes` (splitting takes precedence over merging; logged once
    // below). Meshes without feature IDs keep today's behaviour: one feature
    // each, or accumulated here to merge into a single feature at the end.
    let mut merge_candidates = Vec::new();
    let mut merge_mesh_names: HashSet<String> = HashSet::new();
    let mut merge_node_names: HashSet<String> = HashSet::new();
    let mut merge_primitive_count = 0usize;
    let mut merge_override_logged = false;
    let mut metadata_no_features_logged = false;

    for mesh_info in mesh_infos {
        let build = extract_mesh_build(
            &mesh_info.primitives,
            &buffer_data,
            Some(&mesh_info.transform),
            &gltf_uri,
        )?;

        let mesh_names = mesh_info.mesh_name.map(|n| vec![n]).unwrap_or_default();
        let node_names = mesh_info.node_name.map(|n| vec![n]).unwrap_or_default();
        let primitive_count = mesh_info.primitives.len();
        let has_feature_ids = build.tri_feature_id.iter().any(Option::is_some);

        if !has_feature_ids && structural_metadata.is_some() && !metadata_no_features_logged {
            tracing::warn!(
                "glTF: EXT_structural_metadata is present but this mesh has no \
                 EXT_mesh_features feature IDs; per-object metadata was not surfaced"
            );
            metadata_no_features_logged = true;
        }

        if has_feature_ids {
            if params.merge_meshes && !merge_override_logged {
                tracing::warn!(
                    "glTF: merge_meshes is set, but EXT_mesh_features feature IDs are \
                     present; splitting by feature ID takes precedence and merge was \
                     overridden"
                );
                merge_override_logged = true;
            }
            for feature in split_features(
                build,
                structural_metadata.as_ref(),
                &mesh_names,
                &node_names,
                primitive_count,
                params,
            ) {
                send_feature(sender, feature).await?;
            }
        } else if !params.merge_meshes {
            // Stream each mesh's feature as it is produced (no full-list buffering).
            let feature = build_feature(
                build_geometry(build),
                &mesh_names,
                &node_names,
                primitive_count,
                params,
                IndexMap::new(),
            );
            send_feature(sender, feature).await?;
        } else {
            merge_mesh_names.extend(mesh_names);
            merge_node_names.extend(node_names);
            merge_primitive_count += primitive_count;
            merge_candidates.push(build);
        }
    }

    if !merge_candidates.is_empty() {
        let merged = merge_builds(merge_candidates);
        let merged_mesh_names: Vec<String> = merge_mesh_names.into_iter().collect();
        let merged_node_names: Vec<String> = merge_node_names.into_iter().collect();

        let feature = build_feature(
            build_geometry(merged),
            &merged_mesh_names,
            &merged_node_names,
            merge_primitive_count,
            params,
            IndexMap::new(),
        );
        send_feature(sender, feature).await?;
    }

    Ok(())
}

async fn send_feature(
    sender: &Sender<(Port, IngestionMessage)>,
    feature: Feature,
) -> Result<(), SourceError> {
    sender
        .send((
            FEATURES_PORT.clone(),
            IngestionMessage::OperationEvent { feature },
        ))
        .await
        .map_err(|e| SourceError::GltfReader(format!("Failed to send feature: {e}")))
}

/// The flattened, per-triangle result of reading a glTF mesh's primitives: a
/// triangle soup (three vertices per triangle, in corner order) plus, parallel to
/// it, the per-triangle material slot and per-corner UV, and the distinct material
/// palette. This is the shape `TriangularMesh3D` (via `from_soup`) and its
/// appearance setters consume; keeping it flat lets the merge-meshes path simply
/// concatenate builds (offsetting the palette).
#[derive(Default, Debug)]
struct MeshBuild {
    /// Triangle soup: three coordinates per triangle, in corner order.
    soup: Vec<[f64; 3]>,
    /// Per triangle: palette slot in `materials`, or `None` for the glTF default
    /// material (left unpainted so the writer's neutral default applies).
    tri_material: Vec<Option<u32>>,
    /// Per UV channel, one entry per corner (aligned to `soup`, i.e. every
    /// channel's buffer has the same length as the whole mesh's corner count).
    /// `[0, 0]` at corners whose triangle's material doesn't sample that
    /// channel (untextured, or textured on a different channel).
    corner_uv: BTreeMap<ChannelId, Vec<[f64; 2]>>,
    /// Distinct authored materials; `tri_material` indexes this.
    materials: Vec<Material>,
    /// UV channels sampled by any textured material (drives the appearance's UV
    /// sets). Empty when nothing is textured.
    channels: BTreeSet<ChannelId>,
    /// Per triangle: the `EXT_mesh_features` feature ID of its first corner (see
    /// `extract_mesh_build`), or `None` when the primitive that produced it
    /// carries no feature-ID set. All-`None` (including empty) means the mesh
    /// isn't split; otherwise the reader groups triangles by this value into
    /// one Flow `Feature` per distinct ID (see `split_features`).
    tri_feature_id: Vec<Option<u32>>,
    /// The `propertyTable` index named by the mesh's feature-ID set (first one
    /// found across its primitives), used to look up each split feature's
    /// `EXT_structural_metadata` row. `None` when no primitive named one
    /// (equivalently, no primitive carries a feature-ID set at all).
    property_table_index: Option<usize>,
}

/// Read every primitive of one glTF mesh into a flat [`MeshBuild`]: positions
/// (with the node's world transform baked in), per-triangle material, and
/// per-corner UV. Replicates the crate's triangle expansion (Triangles / Strip /
/// Fan, indexed and non-indexed) while additionally tracking material + UV per
/// output triangle, which the geometry-only extraction discards.
fn extract_mesh_build(
    primitives: &[gltf::Primitive],
    buffer_data: &[Vec<u8>],
    transform: Option<&reearth_flow_gltf::Transform>,
    base_uri: &Uri,
) -> Result<MeshBuild, SourceError> {
    let mut build = MeshBuild::default();
    // glTF material index -> palette slot, so a material shared by several
    // primitives is converted (and its image decoded) once.
    let mut palette_by_index: HashMap<usize, u32> = HashMap::new();

    // Pass 1: resolve/convert every distinct material up front, so the full
    // per-mesh UV channel set (`build.channels`) is known before any per-corner
    // UV buffer is written in pass 2 — a channel introduced by a later
    // primitive's material must still get zero-filled entries for every corner
    // that came before it.
    for primitive in primitives {
        if primitive
            .extension_value("KHR_draco_mesh_compression")
            .is_some()
        {
            return Err(SourceError::GltfReader(
                "KHR_draco_mesh_compression is not yet supported \
                 (Draco decode pending; see issue #2311)"
                    .to_string(),
            ));
        }

        let material = primitive.material();
        if let Some(index) = material.index() {
            palette_by_index.entry(index).or_insert_with(|| {
                let (converted, channels) = convert_material(&material, buffer_data, base_uri);
                let slot = build.materials.len() as u32;
                build.materials.push(converted);
                build.channels.extend(channels);
                slot
            });
        }
    }

    // Pass 2: expand triangles. For each corner, every known channel
    // (`build.channels`) gets an entry — the real tex-coord when the corner's
    // own material samples that channel, `[0, 0]` otherwise — so every
    // channel's buffer stays aligned to the whole mesh's corner soup.
    // `warned_feature_id_disagreement` caps the corner-mismatch warning to once
    // per mesh (i.e. once per `extract_mesh_build` call), not once per triangle.
    let mut warned_feature_id_disagreement = false;
    for primitive in primitives {
        let feature_ids =
            reearth_flow_gltf::read_mesh_features(primitive, buffer_data).map_err(|e| {
                SourceError::GltfReader(format!("Failed to read EXT_mesh_features: {e}"))
            })?;
        if feature_ids.is_some() && build.property_table_index.is_none() {
            build.property_table_index = mesh_features_property_table(primitive);
        }

        let pos_accessor = primitive
            .get(&gltf::Semantic::Positions)
            .ok_or_else(|| SourceError::GltfReader("Primitive has no positions".to_string()))?;
        let positions =
            reearth_flow_gltf::read_positions_with_transform(&pos_accessor, buffer_data, transform)
                .map_err(|e| SourceError::GltfReader(format!("Failed to read positions: {e}")))?;

        let material = primitive.material();
        let slot = material.index().map(|index| palette_by_index[&index]);
        let sampled_channels: BTreeSet<ChannelId> = slot
            .map(|s| build.materials[s as usize].referenced_channels())
            .unwrap_or_default();

        let reader = primitive.reader(|b| buffer_data.get(b.index()).map(|v| v.as_slice()));
        let mut uv_by_channel: HashMap<ChannelId, Vec<[f32; 2]>> = HashMap::new();
        for &channel in &sampled_channels {
            if let Some(tex_coords) = reader.read_tex_coords(channel.0) {
                uv_by_channel.insert(channel, tex_coords.into_f32().collect());
            }
        }
        let indices: Option<Vec<usize>> = reader
            .read_indices()
            .map(|i| i.into_u32().map(|v| v as usize).collect());

        for [a, b, c] in triangle_corners(primitive.mode(), indices.as_deref(), positions.len())? {
            for &i in &[a, b, c] {
                let p = positions[i];
                build.soup.push([p.x, p.y, p.z]);
                for &channel in &build.channels {
                    let corner = uv_by_channel
                        .get(&channel)
                        .and_then(|u| u.get(i))
                        .map(|&[u, v]| [u as f64, v as f64])
                        .unwrap_or([0.0, 0.0]);
                    build.corner_uv.entry(channel).or_default().push(corner);
                }
            }
            build.tri_material.push(slot);

            let fid = feature_ids.as_ref().map(|ids| {
                // A `constant` feature-ID set (see `read_mesh_features`) comes
                // back as a single-element vec applying to every vertex; a
                // per-vertex `attribute` set has one entry per vertex.
                let id_at = |vertex_index: usize| -> u32 {
                    if ids.len() == 1 {
                        ids[0]
                    } else {
                        ids.get(vertex_index).copied().unwrap_or(ids[0])
                    }
                };
                let (fa, fb, fc) = (id_at(a), id_at(b), id_at(c));
                if (fa != fb || fa != fc) && !warned_feature_id_disagreement {
                    tracing::warn!(
                        "glTF: a triangle's corners disagree on their EXT_mesh_features \
                         feature ID ({fa}, {fb}, {fc}); using the first corner's ID"
                    );
                    warned_feature_id_disagreement = true;
                }
                fa
            });
            build.tri_feature_id.push(fid);
        }
    }

    Ok(build)
}

/// The `propertyTable` index declared on a primitive's first `EXT_mesh_features`
/// feature-ID set, defaulting to `0` when the set doesn't name one explicitly
/// (per the extension's spec). `None` when the primitive carries no
/// `EXT_mesh_features` feature-ID set at all.
fn mesh_features_property_table(primitive: &gltf::Primitive) -> Option<usize> {
    let mesh_features = primitive.extension_value("EXT_mesh_features")?;
    let feature_ids = mesh_features.get("featureIds")?.as_array()?;
    let first = feature_ids.first()?.as_object()?;
    Some(
        first
            .get("propertyTable")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
    )
}

/// The vertex-index triples of a primitive's triangles, replicating the crate's
/// expansion for every supported mode (see `reearth_flow_gltf`'s geometry path).
/// Indexed modes index into the primitive's vertices via `indices`; non-indexed
/// `Triangles` walks the vertices directly.
fn triangle_corners(
    mode: gltf::mesh::Mode,
    indices: Option<&[usize]>,
    vertex_count: usize,
) -> Result<Vec<[usize; 3]>, SourceError> {
    use gltf::mesh::Mode;
    let mut tris = Vec::new();
    match indices {
        Some(idx) => match mode {
            Mode::Triangles => {
                for chunk in idx.chunks(3) {
                    if let &[a, b, c] = chunk {
                        tris.push([a, b, c]);
                    }
                }
            }
            Mode::TriangleStrip => {
                for i in 0..idx.len().saturating_sub(2) {
                    if i % 2 == 0 {
                        tris.push([idx[i], idx[i + 1], idx[i + 2]]);
                    } else {
                        tris.push([idx[i], idx[i + 2], idx[i + 1]]);
                    }
                }
            }
            Mode::TriangleFan => {
                for i in 1..idx.len().saturating_sub(1) {
                    tris.push([idx[0], idx[i], idx[i + 1]]);
                }
            }
            other => {
                return Err(SourceError::GltfReader(format!(
                    "Unsupported primitive mode: {other:?}"
                )))
            }
        },
        None => match mode {
            Mode::Triangles => {
                for chunk in (0..vertex_count).collect::<Vec<_>>().chunks(3) {
                    if let &[a, b, c] = chunk {
                        tris.push([a, b, c]);
                    }
                }
            }
            other => {
                return Err(SourceError::GltfReader(format!(
                    "Unsupported non-indexed primitive mode: {other:?}"
                )))
            }
        },
    }
    Ok(tris)
}

/// Resolve one glTF texture reference (its image, sampler, and optional
/// `KHR_texture_transform`) to a new-geometry [`Texture`] plus the UV channel it
/// samples. Shared by every PBR map slot (`Info`, `NormalTexture`,
/// `OcclusionTexture` all expose the same `texture()` / `tex_coord()` /
/// `texture_transform()` shape, just as distinct gltf-rs types). `None` when the
/// image can't be resolved (unsupported format, missing buffer view, ...).
///
/// When `KHR_texture_transform` carries its own `tex_coord` override, that wins
/// over the texture-info's `tex_coord`, per the extension's spec.
fn convert_texture(
    texture: gltf::texture::Texture,
    tex_coord: u32,
    khr_transform: Option<gltf::texture::TextureTransform>,
    buffer_data: &[Vec<u8>],
    base_uri: &Uri,
) -> Option<(Texture, ChannelId)> {
    let raster = resolve_image(texture.source().source(), buffer_data, base_uri)?;
    let uv_channel = ChannelId(
        khr_transform
            .as_ref()
            .and_then(|t| t.tex_coord())
            .unwrap_or(tex_coord),
    );
    let transform = khr_transform.map(|t| TextureTransform {
        offset: t.offset(),
        rotation: t.rotation(),
        scale: t.scale(),
    });
    Some((
        Texture {
            raster: Arc::new(raster),
            sampler: convert_sampler(&texture.sampler()),
            transform,
            uv_channel,
        },
        uv_channel,
    ))
}

/// Convert a glTF PBR material to the new-geometry [`Material`], resolving every
/// texture slot (base colour, metallic-roughness, normal, occlusion, emissive) to
/// an embedded raster (or an external URI) when present, folding
/// `KHR_materials_emissive_strength` into the emissive factor, and reading
/// `KHR_texture_transform` on each map. Returns every UV channel sampled by any
/// of its textures so the caller knows which UV sets to read.
fn convert_material(
    material: &gltf::Material,
    buffer_data: &[Vec<u8>],
    base_uri: &Uri,
) -> (Material, Vec<ChannelId>) {
    let pbr = material.pbr_metallic_roughness();
    let mut channels: Vec<ChannelId> = Vec::new();

    let mut record = |resolved: Option<(Texture, ChannelId)>| -> Option<Texture> {
        resolved.map(|(texture, channel)| {
            channels.push(channel);
            texture
        })
    };

    let base_color_map = record(pbr.base_color_texture().and_then(|info| {
        convert_texture(
            info.texture(),
            info.tex_coord(),
            info.texture_transform(),
            buffer_data,
            base_uri,
        )
    }));
    let metallic_roughness_map = record(pbr.metallic_roughness_texture().and_then(|info| {
        convert_texture(
            info.texture(),
            info.tex_coord(),
            info.texture_transform(),
            buffer_data,
            base_uri,
        )
    }));
    let normal_map = record(material.normal_texture().and_then(|normal| {
        convert_texture(
            normal.texture(),
            normal.tex_coord(),
            normal.texture_transform(),
            buffer_data,
            base_uri,
        )
    }));
    let occlusion_map = record(material.occlusion_texture().and_then(|occlusion| {
        convert_texture(
            occlusion.texture(),
            occlusion.tex_coord(),
            occlusion.texture_transform(),
            buffer_data,
            base_uri,
        )
    }));
    let emissive_map = record(material.emissive_texture().and_then(|info| {
        convert_texture(
            info.texture(),
            info.tex_coord(),
            info.texture_transform(),
            buffer_data,
            base_uri,
        )
    }));

    // KHR_materials_emissive_strength scales emissiveFactor beyond its normal
    // [0, 1] range; fold it in now so downstream consumers see one linear
    // emissive colour without needing to know about the extension.
    let strength = material.emissive_strength().unwrap_or(1.0);
    let emissive_factor = material.emissive_factor();
    let emissive = [
        emissive_factor[0] * strength,
        emissive_factor[1] * strength,
        emissive_factor[2] * strength,
    ];

    let converted = Material::Pbr(PbrMaterial {
        base_color: pbr.base_color_factor(),
        metallic: pbr.metallic_factor(),
        roughness: pbr.roughness_factor(),
        emissive,
        base_color_map,
        metallic_roughness_map,
        normal_map,
        occlusion_map,
        emissive_map,
        alpha_mode: convert_alpha_mode(material.alpha_mode(), material.alpha_cutoff()),
        double_sided: material.double_sided(),
    });

    (converted, channels)
}

/// Resolve a glTF image to a [`Raster`]: embedded (`View`) and `data:` URIs become
/// in-memory bytes; a plain external URI is carried as a location resolved against
/// the glTF's directory. `None` when the bytes/mime can't be determined.
fn resolve_image(
    source: gltf::image::Source,
    buffer_data: &[Vec<u8>],
    base_uri: &Uri,
) -> Option<Raster> {
    match source {
        gltf::image::Source::View { view, mime_type } => {
            let buffer = buffer_data.get(view.buffer().index())?;
            let bytes = buffer
                .get(view.offset()..view.offset() + view.length())?
                .to_vec();
            let mime = mime_type_from(Some(mime_type), &bytes)?;
            Some(Raster::InMemory(RasterData {
                mime_type: mime,
                bytes: Bytes::from(bytes),
            }))
        }
        gltf::image::Source::Uri { uri, mime_type } => {
            if uri.starts_with("data:") {
                let bytes = reearth_flow_gltf::decode_data_uri(uri).ok()?;
                let mime = mime_type_from(mime_type, &bytes)?;
                Some(Raster::InMemory(RasterData {
                    mime_type: mime,
                    bytes: Bytes::from(bytes),
                }))
            } else {
                Some(Raster::Uri(join_relative_uri(base_uri, uri)?))
            }
        }
    }
}

/// Map a glTF image mime type (or sniff the leading magic bytes when absent) to a
/// [`MimeType`]. `None` for formats the raster model can't represent.
fn mime_type_from(declared: Option<&str>, bytes: &[u8]) -> Option<MimeType> {
    match declared {
        Some("image/png") => return Some(MimeType::ImagePng),
        Some("image/jpeg") => return Some(MimeType::ImageJpeg),
        Some("image/webp") => return Some(MimeType::ImageWebp),
        _ => {}
    }
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        Some(MimeType::ImagePng)
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        Some(MimeType::ImageJpeg)
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some(MimeType::ImageWebp)
    } else {
        tracing::warn!("glTF: unsupported texture image format; rendering colour-only");
        None
    }
}

/// Join a texture's relative URI against the glTF file's directory, matching how
/// the crate resolves external buffer URIs.
fn join_relative_uri(base_uri: &Uri, relative: &str) -> Option<Uri> {
    let base = base_uri.to_string();
    let joined = match base.rfind('/') {
        Some(slash) => format!("{}/{}", &base[..slash], relative),
        None => relative.to_string(),
    };
    Uri::from_str(&joined).ok()
}

/// Map a glTF sampler's wrap/filter enums to the new-geometry [`Sampler`].
fn convert_sampler(sampler: &gltf::texture::Sampler) -> Sampler {
    use gltf::texture::{MagFilter, MinFilter, WrappingMode};
    let wrap = |w: WrappingMode| match w {
        WrappingMode::ClampToEdge => WrapMode::ClampToEdge,
        WrappingMode::MirroredRepeat => WrapMode::MirroredRepeat,
        WrappingMode::Repeat => WrapMode::Repeat,
    };
    Sampler {
        wrap_s: wrap(sampler.wrap_s()),
        wrap_t: wrap(sampler.wrap_t()),
        mag_filter: match sampler.mag_filter() {
            Some(MagFilter::Nearest) => Filter::Nearest,
            _ => Filter::Linear,
        },
        min_filter: match sampler.min_filter() {
            Some(MinFilter::Nearest) => Filter::Nearest,
            Some(MinFilter::Linear) => Filter::Linear,
            Some(MinFilter::NearestMipmapNearest) | Some(MinFilter::NearestMipmapLinear) => {
                Filter::NearestMipmap
            }
            _ => Filter::LinearMipmap,
        },
    }
}

/// Map a glTF alpha mode + cutoff to the new-geometry [`AlphaMode`].
fn convert_alpha_mode(mode: gltf::material::AlphaMode, cutoff: Option<f32>) -> AlphaMode {
    match mode {
        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
        gltf::material::AlphaMode::Mask => AlphaMode::Mask {
            cutoff: cutoff.unwrap_or(0.5),
        },
        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
    }
}

/// Concatenate per-mesh builds for the `merge_meshes` path: append each build's
/// soup / UV / triangle-material, offsetting palette slots into the combined
/// material list and unioning the sampled channels.
fn merge_builds(builds: Vec<MeshBuild>) -> MeshBuild {
    // The union of every build's channels, decided up front: a channel a given
    // build never sampled still needs `[0, 0]`-filled entries for that build's
    // corners, to keep every channel's buffer aligned to the merged soup.
    let channels: BTreeSet<ChannelId> = builds
        .iter()
        .flat_map(|b| b.channels.iter().copied())
        .collect();
    let mut out = MeshBuild {
        channels,
        ..MeshBuild::default()
    };

    for build in builds {
        let base = out.materials.len() as u32;
        let corner_count = build.soup.len();
        out.materials.extend(build.materials);
        out.soup.extend(build.soup);
        out.tri_material
            .extend(build.tri_material.into_iter().map(|s| s.map(|s| s + base)));

        for &channel in &out.channels {
            let buf = out.corner_uv.entry(channel).or_default();
            match build.corner_uv.get(&channel) {
                Some(values) => buf.extend(values.iter().copied()),
                None => buf.extend(std::iter::repeat_n([0.0, 0.0], corner_count)),
            }
        }
    }
    out
}

/// Extract the triangles at `tri_indices` (0-based into `build`'s per-triangle
/// arrays) into their own [`MeshBuild`], sharing the parent's material palette
/// and channel set. Used by [`split_features`] to turn one node's mesh into
/// one sub-mesh per distinct `EXT_mesh_features` feature ID; `build_geometry`'s
/// `from_soup` then dedups vertices within each sub-mesh independently, so
/// per-face material/UV bindings still line up.
fn sub_build(build: &MeshBuild, tri_indices: &[usize]) -> MeshBuild {
    let mut soup = Vec::with_capacity(tri_indices.len() * 3);
    let mut tri_material = Vec::with_capacity(tri_indices.len());
    let mut corner_uv: BTreeMap<ChannelId, Vec<[f64; 2]>> = build
        .channels
        .iter()
        .map(|&channel| (channel, Vec::with_capacity(tri_indices.len() * 3)))
        .collect();

    for &tri in tri_indices {
        soup.extend_from_slice(&build.soup[tri * 3..tri * 3 + 3]);
        for (&channel, buf) in corner_uv.iter_mut() {
            buf.extend_from_slice(&build.corner_uv[&channel][tri * 3..tri * 3 + 3]);
        }
        tri_material.push(build.tri_material[tri]);
    }

    MeshBuild {
        soup,
        tri_material,
        corner_uv,
        materials: build.materials.clone(),
        channels: build.channels.clone(),
        tri_feature_id: Vec::new(),
        property_table_index: None,
    }
}

/// Split `build` into one Flow [`Feature`] per distinct `EXT_mesh_features`
/// feature ID (grouping its triangles by [`MeshBuild::tri_feature_id`]),
/// decorating each with `featureId` and, when `structural_metadata` resolves a
/// property-table row for that ID, that row's properties. When
/// `params.feature_class_attribute` names a key, the property table's class
/// name (if any) is additionally inserted under that key — the user opted
/// into it under a key of their choosing, so it is a plain insert (last
/// write wins) rather than going through the `meta_` collision rule below.
/// Only called once the caller has confirmed `build.tri_feature_id` has at
/// least one `Some` entry.
///
/// A triangle whose primitive carried no feature-ID set (a mix within one
/// mesh) falls into its own feature with no `featureId`, so its geometry
/// isn't silently dropped; this shouldn't occur on well-formed PLATEAU
/// exports, where every primitive in a mesh shares the same extension usage.
/// If `extra` already holds `key` (a decoded structural-metadata property
/// that happens to share a name as `split_features`'s own reserved
/// `featureId` attribute), move it out from under `key` to `meta_<key>`
/// first, so the reserved attribute can then be inserted under `key` without
/// silently dropping the metadata value. Mirrors `build_feature`'s `meta_`
/// collision rule against the base attributes, just applied here against
/// this reader-reserved key before that later merge.
fn reserve_extra_attr(extra: &mut IndexMap<Attribute, AttributeValue>, key: &str) {
    if let Some(value) = extra.shift_remove(&Attribute::new(key)) {
        extra.insert(Attribute::new(format!("meta_{key}")), value);
    }
}

fn split_features(
    build: MeshBuild,
    structural_metadata: Option<&reearth_flow_gltf::PropertyTables>,
    mesh_names: &[String],
    node_names: &[String],
    primitive_count: usize,
    params: &GltfReaderCompiledParam,
) -> Vec<Feature> {
    let mut groups: BTreeMap<Option<u32>, Vec<usize>> = BTreeMap::new();
    for (tri, &fid) in build.tri_feature_id.iter().enumerate() {
        groups.entry(fid).or_default().push(tri);
    }

    groups
        .into_iter()
        .map(|(fid, tri_indices)| {
            let sub = sub_build(&build, &tri_indices);
            let mut extra = IndexMap::new();

            match fid {
                Some(feature_id) => {
                    if let Some(tables) = structural_metadata {
                        let table_index = build.property_table_index.unwrap_or(0);
                        match tables.tables.get(table_index) {
                            Some(table) if (feature_id as usize) < table.count => {
                                for (name, value) in reearth_flow_gltf::feature_properties(
                                    tables,
                                    table_index,
                                    feature_id,
                                ) {
                                    extra.insert(Attribute::new(name), value);
                                }
                                if let (Some(key), Some(class_name)) =
                                    (params.feature_class_attribute.as_deref(), &table.class)
                                {
                                    // The user chose this key explicitly, so overwriting
                                    // whatever decoded property may already be there is
                                    // expected (no `meta_` renaming, unlike `featureId`
                                    // below).
                                    extra.insert(
                                        Attribute::new(key),
                                        AttributeValue::String(class_name.clone()),
                                    );
                                }
                            }
                            Some(table) => tracing::warn!(
                                "glTF: feature ID {feature_id} is out of range for \
                                 structural-metadata property table {table_index} \
                                 ({} rows); attaching only featureId",
                                table.count
                            ),
                            None => tracing::warn!(
                                "glTF: EXT_mesh_features references property table \
                                 {table_index}, but EXT_structural_metadata only has \
                                 {} table(s); attaching only featureId",
                                tables.tables.len()
                            ),
                        }
                    }
                    // Always set last so the reader's own numeric ID wins the
                    // slot; a metadata property that happens to be named
                    // `featureId` is preserved (renamed, not dropped) by the
                    // `reserve_extra_attr` call above/below rather than being
                    // silently overwritten.
                    reserve_extra_attr(&mut extra, "featureId");
                    extra.insert(
                        Attribute::new("featureId"),
                        AttributeValue::Number(serde_json::Number::from(feature_id)),
                    );
                }
                None => tracing::warn!(
                    "glTF: mesh mixes EXT_mesh_features feature IDs with feature-ID-less \
                     primitives; the latter's triangles are emitted as one feature with \
                     no featureId"
                ),
            }

            build_feature(
                build_geometry(sub),
                mesh_names,
                node_names,
                primitive_count,
                params,
                extra,
            )
        })
        .collect()
}

/// Build the new-geometry [`Geometry`] from a [`MeshBuild`]: a single
/// `Euclidean3D::TriangularMesh` (`from_soup` deduplicates shared vertices while
/// preserving winding; glTF is model-space, so the frame is `Euclidean`), with an
/// appearance attached when the mesh carries authored materials.
fn build_geometry(build: MeshBuild) -> Geometry {
    if build.soup.is_empty() {
        return Geometry::None;
    }

    let mut mesh = TriangularMesh3D::from_soup(CoordinateFrame::Euclidean, build.soup);

    if !build.materials.is_empty() {
        // One `Explicit` UV buffer per sampled channel, built from that
        // channel's own per-corner buffer (extract_mesh_build/merge_builds keep
        // each channel's buffer aligned to the full corner soup already).
        let uvs: BTreeMap<ChannelId, UvSource> = build
            .corner_uv
            .into_iter()
            .map(|(channel, buf)| (channel, UvSource::Explicit(buf.into_boxed_slice())))
            .collect();
        let binding = FaceBinding::PerFace(
            build
                .tri_material
                .iter()
                .map(|&slot| slot.and_then(MaterialIndex::new))
                .collect(),
        );

        if let Err(e) = mesh.set_appearance_with_binding(
            ThemeId(Arc::from("default")),
            build.materials,
            binding,
            uvs,
        ) {
            tracing::warn!("glTF: failed to attach appearance, emitting bare mesh: {e:?}");
        }
    }

    Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(mesh)))
}

/// Build a Flow [`Feature`] from `geometry` plus the reader's base attributes
/// (`source`, `mesh`/`meshes`, `node`/`nodes`, `primitiveCount`), then merge in
/// `extra_attributes` (e.g. `featureId` + decoded structural-metadata
/// properties from [`split_features`]) on top. A colliding key is written
/// under `meta_<name>` instead of overwriting the base attribute, so nothing
/// from either side is silently lost.
fn build_feature(
    geometry: Geometry,
    mesh_names: &[String],
    node_names: &[String],
    primitive_count: usize,
    params: &GltfReaderCompiledParam,
    extra_attributes: IndexMap<Attribute, AttributeValue>,
) -> Feature {
    let mut attributes = IndexMap::new();

    attributes.insert(
        Attribute::new("source"),
        AttributeValue::String("glTF".to_string()),
    );

    if !mesh_names.is_empty() {
        let key = if mesh_names.len() == 1 {
            "mesh"
        } else {
            "meshes"
        };
        attributes.insert(Attribute::new(key), string_or_array(mesh_names));
    }

    if params.include_nodes && !node_names.is_empty() {
        let key = if node_names.len() == 1 {
            "node"
        } else {
            "nodes"
        };
        attributes.insert(Attribute::new(key), string_or_array(node_names));
    }

    attributes.insert(
        Attribute::new("primitiveCount"),
        AttributeValue::Number(serde_json::Number::from(primitive_count)),
    );

    for (key, value) in extra_attributes {
        if attributes.contains_key(&key) {
            attributes.insert(Attribute::new(format!("meta_{}", key.inner())), value);
        } else {
            attributes.insert(key, value);
        }
    }

    Feature::new_with_attributes_and_geometry(attributes, geometry)
}

fn string_or_array(values: &[String]) -> AttributeValue {
    if values.len() == 1 {
        AttributeValue::String(values[0].clone())
    } else {
        AttributeValue::Array(
            values
                .iter()
                .map(|v| AttributeValue::String(v.clone()))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unwrap a `Geometry` known to be a single `TriangularMesh`.
    fn mesh_of(geom: Geometry) -> TriangularMesh3D {
        match geom {
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(m)) => *m,
            other => panic!("expected Euclidean3D TriangularMesh, got {other:?}"),
        }
    }

    /// A material-less `MeshBuild` from a raw triangle soup (three coords/triangle).
    /// No materials means no sampled channels, so `corner_uv` stays empty.
    fn bare_build(soup: Vec<[f64; 3]>) -> MeshBuild {
        let tris = soup.len() / 3;
        MeshBuild {
            tri_material: vec![None; tris],
            soup,
            ..MeshBuild::default()
        }
    }

    #[test]
    fn build_geometry_single_triangle_preserves_z_and_winding() {
        let ext = [[0.0, 0.0, 1.0], [1.0, 0.0, 2.0], [0.0, 1.0, 3.0]];
        let mesh = mesh_of(build_geometry(bare_build(ext.to_vec())));

        assert_eq!(*mesh.frame(), CoordinateFrame::Euclidean);
        assert_eq!(mesh.num_triangles(), 1);
        // `from_soup` assigns vertex indices in first-seen order, so a single
        // triangle keeps its original winding v0, v1, v2 (glTF front faces are CCW;
        // this must not be reordered).
        assert_eq!(mesh.triangles().collect::<Vec<_>>(), vec![[0, 1, 2]]);
        // The three distinct vertices survive in order, per-vertex Z included.
        assert_eq!(mesh.vertices(), ext.as_slice());
        assert!(mesh.appearance().is_none(), "no materials -> bare mesh");
    }

    #[test]
    fn build_geometry_dedups_shared_vertices() {
        // Two triangles sharing the edge (1,0,0)-(0,1,0): 4 distinct vertices, not 6.
        let soup = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let mesh = mesh_of(build_geometry(bare_build(soup)));
        assert_eq!(mesh.num_triangles(), 2, "both triangles kept");
        assert_eq!(
            mesh.vertices().len(),
            4,
            "shared vertices deduplicated into one pool (not 6)"
        );
    }

    #[test]
    fn build_geometry_attaches_pbr_texture_appearance() {
        let texture = Texture {
            raster: Arc::new(Raster::InMemory(RasterData {
                mime_type: MimeType::ImagePng,
                bytes: Bytes::from_static(b"not-a-real-png-but-enough-for-the-model"),
            })),
            sampler: Sampler::default(),
            transform: None,
            uv_channel: ChannelId(0),
        };
        let material = Material::Pbr(PbrMaterial {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 1.0,
            roughness: 1.0,
            emissive: [0.0, 0.0, 0.0],
            base_color_map: Some(texture),
            metallic_roughness_map: None,
            normal_map: None,
            occlusion_map: None,
            emissive_map: None,
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
        });
        let build = MeshBuild {
            soup: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            tri_material: vec![Some(0)],
            corner_uv: BTreeMap::from([(ChannelId(0), vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]])]),
            materials: vec![material],
            channels: BTreeSet::from([ChannelId(0)]),
            ..MeshBuild::default()
        };

        let mesh = mesh_of(build_geometry(build));
        let appearance = mesh.appearance().as_ref().expect("appearance attached");
        assert_eq!(appearance.materials().len(), 1);
        match &appearance.materials()[0] {
            Material::Pbr(m) => assert!(
                m.base_color_map.is_some(),
                "base-colour texture survived onto the mesh"
            ),
            other => panic!("expected Pbr material, got {other:?}"),
        }
        // The textured material's UV channel must reach the theme as a UV set.
        assert!(
            appearance.themes()[0]
                .uv_sets
                .iter()
                .any(|set| set.channel == ChannelId(0)),
            "theme carries a UV set for the sampled channel"
        );
    }

    /// Parse a real `.glb` (embedded single buffer) through the live extraction,
    /// combining every mesh's primitives; per-node transforms don't affect the
    /// count/dedup/appearance assertions, so a `None` transform is fine.
    fn extract(bytes: &[u8]) -> MeshBuild {
        let gltf = gltf::Gltf::from_slice(bytes).expect("parse glb");
        let buffer_data = vec![gltf
            .blob
            .as_ref()
            .expect("glb has an embedded buffer")
            .clone()];
        let primitives: Vec<_> = gltf.meshes().flat_map(|m| m.primitives()).collect();
        let base = Uri::from_str("file:///model.glb").unwrap();
        extract_mesh_build(&primitives, &buffer_data, None, &base).expect("extract mesh build")
    }

    // Minimal self-contained glTF 2.0: one triangle with distinct per-vertex Z
    // (1, 2, 3), POSITION (VEC3 f32) + indices (u16) in an embedded data-URI buffer.
    const TRIANGLE_GLTF: &str = r#"{
      "asset": {"version": "2.0"},
      "scenes": [{"nodes": [0]}],
      "nodes": [{"mesh": 0}],
      "meshes": [{"name": "tri", "primitives": [{"attributes": {"POSITION": 0}, "indices": 1, "mode": 4}]}],
      "accessors": [
        {"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0.0, 0.0, 1.0], "max": [1.0, 1.0, 3.0]},
        {"bufferView": 1, "componentType": 5123, "count": 3, "type": "SCALAR"}
      ],
      "bufferViews": [
        {"buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962},
        {"buffer": 0, "byteOffset": 36, "byteLength": 6, "target": 34963}
      ],
      "buffers": [{"byteLength": 42, "uri": "data:application/octet-stream;base64,AAAAAAAAAAAAAIA/AACAPwAAAAAAAABAAAAAAAAAgD8AAEBAAAABAAIA"}]
    }"#;

    /// Real glTF parse -> extraction -> build on an embedded-buffer triangle. Unlike
    /// the synthetic `build_geometry` tests, this exercises the actual glTF parsing
    /// path (positions accessor, indices, triangle expansion).
    #[test]
    fn real_gltf_triangle_reads_as_triangular_mesh_preserving_z() {
        let gltf = gltf::Gltf::from_slice(TRIANGLE_GLTF.as_bytes()).expect("parse glTF");

        // Build the embedded buffer's exact bytes (positions VEC3 f32 at 0, indices
        // u16 at 36) so the test is independent of the reader's buffer loading.
        let mut buf = Vec::new();
        for xyz in [[0.0f32, 0.0, 1.0], [1.0, 0.0, 2.0], [0.0, 1.0, 3.0]] {
            for c in xyz {
                buf.extend_from_slice(&c.to_le_bytes());
            }
        }
        for i in [0u16, 1, 2] {
            buf.extend_from_slice(&i.to_le_bytes());
        }
        let buffer_data = vec![buf];
        let primitives: Vec<_> = gltf
            .meshes()
            .next()
            .expect("one mesh")
            .primitives()
            .collect();
        let base = Uri::from_str("file:///tri.gltf").unwrap();

        let build =
            extract_mesh_build(&primitives, &buffer_data, None, &base).expect("extract mesh build");
        let mesh = mesh_of(build_geometry(build));

        assert_eq!(*mesh.frame(), CoordinateFrame::Euclidean);
        assert_eq!(mesh.num_triangles(), 1);
        let zs: Vec<f64> = mesh.vertices().iter().map(|v| v[2]).collect();
        for z in [1.0_f64, 2.0, 3.0] {
            assert!(zs.contains(&z), "z={z} missing from mesh vertices {zs:?}");
        }
    }

    #[test]
    fn draco_compressed_primitive_errors_clearly() {
        // Minimal glTF: one mesh, one primitive carrying KHR_draco_mesh_compression.
        let json = r#"{
          "asset": {"version": "2.0"},
          "extensionsUsed": ["KHR_draco_mesh_compression"],
          "meshes": [{"primitives": [{
            "attributes": {"POSITION": 0},
            "extensions": {"KHR_draco_mesh_compression": {"bufferView": 0, "attributes": {"POSITION": 0}}}
          }]}],
          "nodes": [{"mesh": 0}],
          "scenes": [{"nodes": [0]}],
          "scene": 0,
          "accessors": [{"componentType": 5126, "count": 3, "type": "VEC3", "min": [0.0, 0.0, 0.0], "max": [0.0, 0.0, 0.0]}],
          "bufferViews": [{"buffer": 0, "byteLength": 12}],
          "buffers": [{"byteLength": 12}]
        }"#;
        let gltf = gltf::Gltf::from_slice(json.as_bytes()).expect("parse");
        let prim: Vec<_> = gltf.meshes().next().unwrap().primitives().collect();
        let base = Uri::from_str("file://./x.gltf").unwrap();
        let err = extract_mesh_build(&prim, &[vec![0u8; 12]], None, &base).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("KHR_draco_mesh_compression"), "got: {msg}");
        assert!(
            msg.contains("2311"),
            "should point to follow-up issue: {msg}"
        );
    }

    // A textured glTF: one triangle with TEXCOORD_0 and a PBR material whose
    // base-colour texture is an embedded (`data:`) PNG. The buffer's own URI is a
    // placeholder (from_slice does not decode it); the accessor data comes from the
    // `buffer_data` built below. The image bytes are the ASCII "hello" so the test
    // can assert they round-trip verbatim.
    const TEXTURED_GLTF: &str = r#"{
      "asset": {"version": "2.0"},
      "scenes": [{"nodes": [0]}],
      "nodes": [{"mesh": 0}],
      "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "TEXCOORD_0": 1}, "indices": 2, "material": 0, "mode": 4}]}],
      "materials": [{"pbrMetallicRoughness": {"baseColorFactor": [1.0, 1.0, 1.0, 1.0], "baseColorTexture": {"index": 0, "texCoord": 0}}}],
      "textures": [{"source": 0}],
      "images": [{"mimeType": "image/png", "uri": "data:image/png;base64,aGVsbG8="}],
      "accessors": [
        {"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0.0, 0.0, 0.0], "max": [1.0, 1.0, 0.0]},
        {"bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC2"},
        {"bufferView": 2, "componentType": 5123, "count": 3, "type": "SCALAR"}
      ],
      "bufferViews": [
        {"buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962},
        {"buffer": 0, "byteOffset": 36, "byteLength": 24, "target": 34962},
        {"buffer": 0, "byteOffset": 60, "byteLength": 6, "target": 34963}
      ],
      "buffers": [{"byteLength": 66, "uri": "data:application/octet-stream;base64,AA=="}]
    }"#;

    #[test]
    fn textured_gltf_reads_material_uv_and_embedded_image() {
        let gltf = gltf::Gltf::from_slice(TEXTURED_GLTF.as_bytes()).expect("parse glTF");

        // Buffer: positions (VEC3 f32) @0, uv (VEC2 f32) @36, indices (u16) @60.
        let mut buf = Vec::new();
        for xyz in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            for c in xyz {
                buf.extend_from_slice(&c.to_le_bytes());
            }
        }
        for uv in [[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]] {
            for c in uv {
                buf.extend_from_slice(&c.to_le_bytes());
            }
        }
        for i in [0u16, 1, 2] {
            buf.extend_from_slice(&i.to_le_bytes());
        }
        let buffer_data = vec![buf];
        let primitives: Vec<_> = gltf
            .meshes()
            .next()
            .expect("one mesh")
            .primitives()
            .collect();
        let base = Uri::from_str("file:///textured.gltf").unwrap();

        let build =
            extract_mesh_build(&primitives, &buffer_data, None, &base).expect("extract mesh build");

        assert_eq!(build.materials.len(), 1, "one authored material");
        assert!(
            build.channels.contains(&ChannelId(0)),
            "UV channel 0 sampled"
        );
        assert_eq!(
            build.tri_material,
            vec![Some(0)],
            "triangle binds material 0"
        );
        // UVs read from TEXCOORD_0 survive per corner (f32 -> f64), in corner order.
        assert_eq!(
            build.corner_uv.get(&ChannelId(0)),
            Some(&vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]])
        );

        match &build.materials[0] {
            Material::Pbr(m) => {
                let texture = m.base_color_map.as_ref().expect("base-colour texture");
                assert_eq!(texture.uv_channel, ChannelId(0));
                match &*texture.raster {
                    Raster::InMemory(data) => {
                        assert_eq!(data.mime_type, MimeType::ImagePng);
                        assert_eq!(&data.bytes[..], b"hello", "embedded image bytes round-trip");
                    }
                    other => panic!("expected embedded raster, got {other:?}"),
                }
            }
            other => panic!("expected Pbr material, got {other:?}"),
        }

        // And the whole thing assembles into a textured mesh appearance.
        let mesh = mesh_of(build_geometry(build));
        assert!(
            mesh.appearance().is_some(),
            "textured mesh carries appearance"
        );
    }

    /// Real GLB: `minimal_rectangle.glb` is a unit rectangle authored as two
    /// triangles that share the diagonal edge, proving real binary-glTF parsing and
    /// the shared-vertex pool (4 corners, not 6). It carries no material, so the
    /// mesh stays bare.
    #[test]
    fn real_glb_rectangle_reads_as_triangular_mesh_with_shared_vertices() {
        let mesh = mesh_of(build_geometry(extract(include_bytes!(
            "../../testdata/minimal_rectangle.glb"
        ))));
        assert_eq!(*mesh.frame(), CoordinateFrame::Euclidean);
        assert_eq!(mesh.num_triangles(), 2, "rectangle = 2 triangles");
        assert_eq!(
            mesh.vertices().len(),
            4,
            "the two triangles share the diagonal, so 4 corners, not 6"
        );
        assert!(mesh.appearance().is_none(), "fixture has no material");
    }

    /// Real-world PLATEAU export (also carries `EXT_structural_metadata`, which we
    /// parse past but do not consume): must read as a non-trivial, vertex-shared
    /// TriangularMesh rather than failing or degenerating.
    #[test]
    fn real_plateau_building_glb_reads_as_shared_vertex_triangular_mesh() {
        let mesh = mesh_of(build_geometry(extract(include_bytes!(
            "../../testdata/test_data_39255_tran_AuxiliaryTrafficArea.glb"
        ))));
        assert_eq!(*mesh.frame(), CoordinateFrame::Euclidean);
        assert!(mesh.num_triangles() > 0, "real mesh has triangles");
        assert!(!mesh.vertices().is_empty(), "real mesh has vertices");
        assert!(
            mesh.vertices().len() < 3 * mesh.num_triangles(),
            "expected shared-vertex dedup on real data: {} verts vs {} triangles",
            mesh.vertices().len(),
            mesh.num_triangles(),
        );
    }

    /// Fixtures for the full PBR material-extension tests: a self-contained glTF
    /// JSON document (no buffers/meshes needed) whose sole material wires up
    /// base-colour, normal and emissive textures plus `KHR_texture_transform` and
    /// `KHR_materials_emissive_strength`.
    mod fixtures {
        /// A valid, minimal 1x1 transparent PNG, reused as the image payload for
        /// every texture slot; `convert_material` only cares that the bytes
        /// round-trip, not that they decode to real pixels.
        const ONE_PX_PNG_DATA_URI: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";

        /// A material-only glTF document: one image/texture (reused by all three
        /// map slots), and one material with `baseColorTexture` (carrying
        /// `KHR_texture_transform`), `normalTexture`, `emissiveTexture`, and
        /// `KHR_materials_emissive_strength.emissiveStrength = 2.0`. No
        /// meshes/accessors/buffers: `convert_material` never touches them.
        pub fn material_glb_full() -> (gltf::Gltf, Vec<Vec<u8>>) {
            let json = format!(
                r#"{{
                  "asset": {{"version": "2.0"}},
                  "extensionsUsed": ["KHR_texture_transform", "KHR_materials_emissive_strength"],
                  "images": [{{"mimeType": "image/png", "uri": "{uri}"}}],
                  "textures": [{{"source": 0}}],
                  "materials": [{{
                    "pbrMetallicRoughness": {{
                      "baseColorFactor": [1.0, 1.0, 1.0, 1.0],
                      "baseColorTexture": {{
                        "index": 0,
                        "texCoord": 0,
                        "extensions": {{
                          "KHR_texture_transform": {{"offset": [0.1, 0.2], "rotation": 0.5, "scale": [2.0, 2.0]}}
                        }}
                      }}
                    }},
                    "normalTexture": {{"index": 0, "texCoord": 0, "scale": 1.0}},
                    "emissiveTexture": {{"index": 0, "texCoord": 0}},
                    "emissiveFactor": [1.0, 1.0, 1.0],
                    "extensions": {{
                      "KHR_materials_emissive_strength": {{"emissiveStrength": 2.0}}
                    }}
                  }}]
                }}"#,
                uri = ONE_PX_PNG_DATA_URI
            );
            let gltf = gltf::Gltf::from_slice(json.as_bytes()).expect("parse glTF");
            (gltf, Vec::new())
        }
    }

    #[test]
    fn convert_material_reads_all_maps_transform_and_emissive_strength() {
        let (gltf, buffers) = fixtures::material_glb_full();
        let mat = gltf.materials().next().unwrap();
        let base = Uri::from_str("file://./x.gltf").unwrap();
        let (converted, channels) = convert_material(&mat, &buffers, &base);
        let Material::Pbr(p) = converted else {
            panic!("expected PBR")
        };
        assert!(p.base_color_map.is_some());
        assert!(p.normal_map.is_some(), "normal map populated");
        assert!(p.emissive_map.is_some(), "emissive map populated");
        assert!(
            p.base_color_map.as_ref().unwrap().transform.is_some(),
            "KHR_texture_transform read"
        );
        // emissiveStrength = 2.0 scales emissive_factor
        assert!(
            p.emissive.iter().any(|&c| c > 1.0),
            "emissive scaled by strength"
        );
        assert!(!channels.is_empty());
    }

    /// Runs the full new-geometry read path (`read`) over `bytes` (an
    /// embedded-buffer `.glb`) and collects every emitted `Feature`. Both
    /// fixtures embed their buffer as the GLB's binary chunk, so `load_buffers`
    /// never touches the storage resolver, letting tests use a bare default
    /// `NodeContext`/`StorageResolver` with no real I/O.
    fn read_all_features_for_test(bytes: &[u8]) -> Vec<Feature> {
        read_all_features_for_test_with_class(bytes, None)
    }

    /// Same as [`read_all_features_for_test`], but lets the caller set
    /// `feature_class_attribute` to exercise the opt-in class-attribute
    /// behaviour.
    fn read_all_features_for_test_with_class(
        bytes: &[u8],
        feature_class_attribute: Option<&str>,
    ) -> Vec<Feature> {
        let params = GltfReaderCompiledParam {
            common: crate::file::reader::runner::FileReaderCompiledParam {
                dataset: None,
                inline: None,
            },
            _triangulate: true,
            merge_meshes: false,
            include_nodes: true,
            feature_class_attribute: feature_class_attribute.map(|s| s.to_string()),
        };
        let ctx = NodeContext::default();
        let storage_resolver = Arc::new(reearth_flow_storage::resolve::StorageResolver::default());
        let content = Bytes::copy_from_slice(bytes);

        let rt = tokio::runtime::Runtime::new().expect("build tokio runtime");
        rt.block_on(async {
            let (tx, mut rx) = tokio::sync::mpsc::channel(64);
            read(&ctx, storage_resolver, &content, &params, &tx)
                .await
                .expect("read succeeds");
            drop(tx);

            let mut features = Vec::new();
            while let Some((_, msg)) = rx.recv().await {
                let IngestionMessage::OperationEvent { feature } = msg;
                features.push(feature);
            }
            features
        })
    }

    #[test]
    fn plateau_glb_splits_into_features_with_metadata() {
        let bytes = include_bytes!("../../testdata/test_data_39255_tran_AuxiliaryTrafficArea.glb");
        let features = read_all_features_for_test(bytes);
        assert!(
            features.len() > 1,
            "metadata glb splits into multiple features: {}",
            features.len()
        );
        let f = &features[0];
        assert!(f.attributes.contains_key(&Attribute::new("featureId")));
        // at least one decoded structural-metadata property present
        assert!(
            f.attributes.keys().any(|k| {
                let n = k.to_string();
                n != "featureId"
                    && n != "source"
                    && n != "mesh"
                    && n != "meshes"
                    && n != "nodes"
                    && n != "node"
                    && n != "primitiveCount"
            }),
            "a metadata property is surfaced"
        );

        // Regression guard for the EXT_structural_metadata bufferView-offset
        // fix: a wrong offset/buffer resolution would still leave `featureId`
        // present and *a* property surfaced (the checks above), but with
        // garbage values. Locate a specific feature by its stable `gml_id`
        // (hand-verified against this fixture, same values independently
        // confirmed via the old blob-based `extract_feature_properties` in
        // `runtime/gltf/src/metadata/decode.rs`'s own `test_extract_feature_properties`)
        // and assert several of its STRING properties decode exactly.
        //
        // This fixture's `EXT_structural_metadata` schema has no NUMERIC
        // properties at all (every property is STRING) — see
        // `split_features_decodes_numeric_property_at_nonzero_buffer_offset`
        // below for the numeric-decode regression guard, using a small
        // synthetic fixture built for that purpose instead.
        let target = features
            .iter()
            .find(|f| {
                f.attributes.get(&Attribute::new("gml_id"))
                    == Some(&AttributeValue::String(
                        "tran_4d448e8a-db1d-48ef-8f04-feb24b49b701".to_string(),
                    ))
            })
            .expect("fixture has a feature with the expected gml_id");

        assert_eq!(
            target.attributes.get(&Attribute::new("meshcode")),
            Some(&AttributeValue::String("54401008".to_string()))
        );
        assert_eq!(
            target.attributes.get(&Attribute::new("tran:class")),
            Some(&AttributeValue::String("road traffic".to_string()))
        );
        assert_eq!(
            target.attributes.get(&Attribute::new("feature_type")),
            Some(&AttributeValue::String(
                "tran:AuxiliaryTrafficArea".to_string()
            ))
        );
        assert_eq!(
            target.attributes.get(&Attribute::new("core:creationDate")),
            Some(&AttributeValue::String("2024-03-19".to_string()))
        );
        assert_eq!(
            target.attributes.get(&Attribute::new("city_code")),
            Some(&AttributeValue::String("08220".to_string()))
        );
        assert_eq!(
            target.attributes.get(&Attribute::new("city_name")),
            Some(&AttributeValue::String("茨城県つくば市".to_string()))
        );
        assert_eq!(
            target.attributes.get(&Attribute::new("tran:function")),
            Some(&AttributeValue::String("路肩".to_string()))
        );
    }

    /// Regression guard for the `EXT_structural_metadata` bufferView-offset
    /// fix (`resolve_metadata_buffer_view` in
    /// `runtime/gltf/src/metadata/decode.rs`): the real PLATEAU fixture used
    /// above has no NUMERIC property to assert against (every property in its
    /// schema is STRING), so this hand-built glTF supplies one. Two
    /// primitives (`constant` feature IDs 0 and 1) share one buffer whose
    /// numeric property's bufferView sits at a **non-zero** byte offset,
    /// after the geometry data — exactly the layout that broke before the
    /// fix (bufferView index used to be read as a *buffer* index, and even a
    /// correct buffer index alone would miss a non-zero `byteOffset`).
    #[test]
    fn split_features_decodes_numeric_property_at_nonzero_buffer_offset() {
        // Triangle A's geometry, then triangle B's, then the numeric
        // property's own bytes — built programmatically so the JSON below can
        // reference exact offsets without hand arithmetic.
        let mut buf = Vec::new();

        let pos_a_offset = buf.len();
        for xyz in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            for c in xyz {
                buf.extend_from_slice(&c.to_le_bytes());
            }
        }
        let idx_a_offset = buf.len();
        for i in [0u16, 1, 2] {
            buf.extend_from_slice(&i.to_le_bytes());
        }
        while buf.len() % 4 != 0 {
            buf.push(0);
        }

        let pos_b_offset = buf.len();
        for xyz in [[2.0f32, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]] {
            for c in xyz {
                buf.extend_from_slice(&c.to_le_bytes());
            }
        }
        let idx_b_offset = buf.len();
        for i in [0u16, 1, 2] {
            buf.extend_from_slice(&i.to_le_bytes());
        }
        while buf.len() % 4 != 0 {
            buf.push(0);
        }

        // The "height" UINT32 property, one row per feature: [111, 222].
        // Its bufferView offset (`height_offset`) is well past zero, unlike
        // every earlier synthetic fixture in this crate's tests.
        let height_offset = buf.len();
        for v in [111u32, 222u32] {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        let height_length = buf.len() - height_offset;

        use base64::Engine;
        let buffer_data_uri = format!(
            "data:application/octet-stream;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(&buf)
        );

        let json = format!(
            r#"{{
              "asset": {{"version": "2.0"}},
              "extensionsUsed": ["EXT_mesh_features", "EXT_structural_metadata"],
              "scene": 0,
              "scenes": [{{"nodes": [0]}}],
              "nodes": [{{"mesh": 0}}],
              "meshes": [{{"primitives": [
                {{
                  "attributes": {{"POSITION": 0}},
                  "indices": 1,
                  "mode": 4,
                  "extensions": {{"EXT_mesh_features": {{"featureIds": [
                    {{"featureCount": 2, "constant": 0, "propertyTable": 0}}
                  ]}}}}
                }},
                {{
                  "attributes": {{"POSITION": 2}},
                  "indices": 3,
                  "mode": 4,
                  "extensions": {{"EXT_mesh_features": {{"featureIds": [
                    {{"featureCount": 2, "constant": 1, "propertyTable": 0}}
                  ]}}}}
                }}
              ]}}],
              "accessors": [
                {{"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0.0, 0.0, 0.0], "max": [1.0, 1.0, 0.0]}},
                {{"bufferView": 1, "componentType": 5123, "count": 3, "type": "SCALAR"}},
                {{"bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC3", "min": [2.0, 0.0, 0.0], "max": [3.0, 1.0, 0.0]}},
                {{"bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR"}}
              ],
              "bufferViews": [
                {{"buffer": 0, "byteOffset": {pos_a_offset}, "byteLength": 36, "target": 34962}},
                {{"buffer": 0, "byteOffset": {idx_a_offset}, "byteLength": 6, "target": 34963}},
                {{"buffer": 0, "byteOffset": {pos_b_offset}, "byteLength": 36, "target": 34962}},
                {{"buffer": 0, "byteOffset": {idx_b_offset}, "byteLength": 6, "target": 34963}},
                {{"buffer": 0, "byteOffset": {height_offset}, "byteLength": {height_length}}}
              ],
              "buffers": [{{"byteLength": {buf_len}, "uri": "{buffer_data_uri}"}}],
              "extensions": {{
                "EXT_structural_metadata": {{
                  "schema": {{
                    "id": "S",
                    "classes": {{"T": {{"properties": {{
                      "height": {{"type": "SCALAR", "componentType": "UINT32"}}
                    }}}}}}
                  }},
                  "propertyTables": [
                    {{"class": "T", "count": 2, "properties": {{"height": {{"values": 4}}}}}}
                  ]
                }}
              }}
            }}"#,
            pos_a_offset = pos_a_offset,
            idx_a_offset = idx_a_offset,
            pos_b_offset = pos_b_offset,
            idx_b_offset = idx_b_offset,
            height_offset = height_offset,
            height_length = height_length,
            buf_len = buf.len(),
            buffer_data_uri = buffer_data_uri,
        );

        let features = read_all_features_for_test(json.as_bytes());
        assert_eq!(features.len(), 2, "one feature per constant feature ID");

        let by_feature_id = |id: u32| -> &Feature {
            features
                .iter()
                .find(|f| {
                    f.attributes.get(&Attribute::new("featureId"))
                        == Some(&AttributeValue::Number(serde_json::Number::from(id)))
                })
                .unwrap_or_else(|| panic!("no feature with featureId {id}"))
        };

        assert_eq!(
            by_feature_id(0).attributes.get(&Attribute::new("height")),
            Some(&AttributeValue::Number(111u64.into())),
            "numeric property at a non-zero bufferView offset decodes correctly"
        );
        assert_eq!(
            by_feature_id(1).attributes.get(&Attribute::new("height")),
            Some(&AttributeValue::Number(222u64.into()))
        );
    }

    /// Unit-level guard for Finding 2: a metadata property literally named
    /// `featureId` must not be silently clobbered when the reader inserts its
    /// own reserved `featureId` attribute — it should survive renamed to
    /// `meta_featureId`, the same collision rule `build_feature` applies
    /// against the base attributes.
    #[test]
    fn split_features_preserves_a_metadata_property_literally_named_feature_id() {
        let build = MeshBuild {
            soup: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            tri_material: vec![None],
            tri_feature_id: vec![Some(0)],
            property_table_index: Some(0),
            ..MeshBuild::default()
        };

        let mut properties = IndexMap::new();
        properties.insert(
            "featureId".to_string(),
            reearth_flow_gltf::PropertyData {
                values: vec![AttributeValue::String("colliding-value".to_string())],
            },
        );
        let tables = reearth_flow_gltf::PropertyTables {
            schema: serde_json::json!({}),
            tables: vec![reearth_flow_gltf::PropertyTable {
                class: None,
                count: 1,
                properties,
            }],
        };

        let params = GltfReaderCompiledParam {
            common: crate::file::reader::runner::FileReaderCompiledParam {
                dataset: None,
                inline: None,
            },
            _triangulate: true,
            merge_meshes: false,
            include_nodes: true,
            feature_class_attribute: None,
        };
        let empty: Vec<String> = Vec::new();

        let features = split_features(build, Some(&tables), &empty, &empty, 1, &params);
        assert_eq!(features.len(), 1);
        let f = &features[0];

        assert_eq!(
            f.attributes.get(&Attribute::new("featureId")),
            Some(&AttributeValue::Number(serde_json::Number::from(0u32))),
            "the reader's own numeric featureId wins the reserved slot"
        );
        assert_eq!(
            f.attributes.get(&Attribute::new("meta_featureId")),
            Some(&AttributeValue::String("colliding-value".to_string())),
            "colliding metadata property preserved under meta_ prefix, not dropped"
        );
    }

    /// Builds a single-triangle, single-feature `MeshBuild` plus a
    /// `PropertyTables` whose one table names class `"MyClass"`, for the
    /// `feature_class_attribute` opt-in tests below.
    fn build_and_tables_with_class() -> (MeshBuild, reearth_flow_gltf::PropertyTables) {
        let build = MeshBuild {
            soup: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            tri_material: vec![None],
            tri_feature_id: vec![Some(0)],
            property_table_index: Some(0),
            ..MeshBuild::default()
        };
        let tables = reearth_flow_gltf::PropertyTables {
            schema: serde_json::json!({}),
            tables: vec![reearth_flow_gltf::PropertyTable {
                class: Some("MyClass".to_string()),
                count: 1,
                properties: IndexMap::new(),
            }],
        };
        (build, tables)
    }

    fn params_with_feature_class_attribute(
        feature_class_attribute: Option<&str>,
    ) -> GltfReaderCompiledParam {
        GltfReaderCompiledParam {
            common: crate::file::reader::runner::FileReaderCompiledParam {
                dataset: None,
                inline: None,
            },
            _triangulate: true,
            merge_meshes: false,
            include_nodes: true,
            feature_class_attribute: feature_class_attribute.map(|s| s.to_string()),
        }
    }

    /// Default behaviour (`feature_class_attribute: None`): the property
    /// table's class name is never surfaced as an attribute, even though the
    /// table names one. This is the opt-in change requested in review —
    /// previously the reader always injected a `class` attribute (and bumped
    /// any pre-existing `class` property to `meta_class`).
    #[test]
    fn split_features_does_not_inject_class_by_default() {
        let (build, tables) = build_and_tables_with_class();
        let params = params_with_feature_class_attribute(None);
        let empty: Vec<String> = Vec::new();

        let features = split_features(build, Some(&tables), &empty, &empty, 1, &params);
        assert_eq!(features.len(), 1);
        let f = &features[0];

        assert!(
            f.attributes.get(&Attribute::new("class")).is_none(),
            "class must not be injected unless feature_class_attribute is set"
        );
        assert!(
            f.attributes
                .keys()
                .all(|k| k.to_string() != "MyClass" && !k.to_string().contains("class")),
            "no class-derived attribute should be present at all when opted out"
        );
    }

    /// `feature_class_attribute: Some("myClass")`: the property table's class
    /// name is inserted under the user-chosen key, as a plain (overwriting)
    /// insert.
    #[test]
    fn split_features_injects_class_under_configured_key_when_set() {
        let (build, tables) = build_and_tables_with_class();
        let params = params_with_feature_class_attribute(Some("myClass"));
        let empty: Vec<String> = Vec::new();

        let features = split_features(build, Some(&tables), &empty, &empty, 1, &params);
        assert_eq!(features.len(), 1);
        let f = &features[0];

        assert_eq!(
            f.attributes.get(&Attribute::new("myClass")),
            Some(&AttributeValue::String("MyClass".to_string())),
            "class name surfaced under the configured attribute key"
        );
    }

    #[test]
    fn no_metadata_glb_stays_single_feature() {
        let bytes = include_bytes!("../../testdata/minimal_rectangle.glb");
        let features = read_all_features_for_test(bytes);
        assert_eq!(features.len(), 1);
    }
}
