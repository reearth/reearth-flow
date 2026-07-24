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
    collections::{BTreeMap, BTreeSet, HashMap},
    str::FromStr,
    sync::Arc,
};

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::{image::MimeType, uri::Uri};
use reearth_flow_geometry::{
    appearance::{
        AlphaMode, ChannelId, FaceBinding, Filter, Material, MaterialIndex, PbrMaterial, Raster,
        RasterData, Sampler, Texture, ThemeId, UvSource, WrapMode,
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

    if !params.merge_meshes {
        // Stream each mesh's feature as it is produced (no full-list buffering).
        for mesh_info in mesh_infos {
            let build = extract_mesh_build(
                &mesh_info.primitives,
                &buffer_data,
                Some(&mesh_info.transform),
                &gltf_uri,
            )?;

            let mesh_names = mesh_info.mesh_name.map(|n| vec![n]).unwrap_or_default();
            let node_names = mesh_info.node_name.map(|n| vec![n]).unwrap_or_default();

            let feature = build_feature(
                build_geometry(build),
                &mesh_names,
                &node_names,
                mesh_info.primitives.len(),
                params,
            );
            send_feature(sender, feature).await?;
        }
    } else if !mesh_infos.is_empty() {
        let mut builds = Vec::new();
        let mut all_mesh_names = std::collections::HashSet::new();
        let mut all_node_names = std::collections::HashSet::new();
        let mut total_primitives = 0;

        for mesh_info in mesh_infos {
            builds.push(extract_mesh_build(
                &mesh_info.primitives,
                &buffer_data,
                Some(&mesh_info.transform),
                &gltf_uri,
            )?);
            if let Some(name) = mesh_info.mesh_name {
                all_mesh_names.insert(name);
            }
            if let Some(name) = mesh_info.node_name {
                all_node_names.insert(name);
            }
            total_primitives += mesh_info.primitives.len();
        }

        let merged = merge_builds(builds);
        let merged_mesh_names: Vec<String> = all_mesh_names.into_iter().collect();
        let merged_node_names: Vec<String> = all_node_names.into_iter().collect();

        let feature = build_feature(
            build_geometry(merged),
            &merged_mesh_names,
            &merged_node_names,
            total_primitives,
            params,
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
#[derive(Default)]
struct MeshBuild {
    /// Triangle soup: three coordinates per triangle, in corner order.
    soup: Vec<[f64; 3]>,
    /// Per triangle: palette slot in `materials`, or `None` for the glTF default
    /// material (left unpainted so the writer's neutral default applies).
    tri_material: Vec<Option<u32>>,
    /// Per corner (three per triangle), aligned to `soup`. `[0, 0]` where a
    /// triangle's material is untextured (the slot is never sampled then).
    corner_uv: Vec<[f64; 2]>,
    /// Distinct authored materials; `tri_material` indexes this.
    materials: Vec<Material>,
    /// UV channels sampled by any textured material (drives the appearance's UV
    /// sets). Empty when nothing is textured.
    channels: BTreeSet<ChannelId>,
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

    for primitive in primitives {
        let pos_accessor = primitive
            .get(&gltf::Semantic::Positions)
            .ok_or_else(|| SourceError::GltfReader("Primitive has no positions".to_string()))?;
        let positions =
            reearth_flow_gltf::read_positions_with_transform(&pos_accessor, buffer_data, transform)
                .map_err(|e| SourceError::GltfReader(format!("Failed to read positions: {e}")))?;

        // Resolve (and dedup) this primitive's material, learning which UV channel
        // its base-colour texture samples (if any).
        let material = primitive.material();
        let (slot, channel) = match material.index() {
            Some(index) => {
                let slot = match palette_by_index.get(&index) {
                    Some(&slot) => slot,
                    None => {
                        let (converted, channel) =
                            convert_material(&material, buffer_data, base_uri);
                        let slot = build.materials.len() as u32;
                        build.materials.push(converted);
                        palette_by_index.insert(index, slot);
                        if let Some(channel) = channel {
                            build.channels.insert(channel);
                        }
                        slot
                    }
                };
                (
                    Some(slot),
                    textured_channel(&build.materials[slot as usize]),
                )
            }
            // The glTF default material: leave faces unpainted.
            None => (None, None),
        };

        let reader = primitive.reader(|b| buffer_data.get(b.index()).map(|v| v.as_slice()));
        let uv: Option<Vec<[f32; 2]>> =
            channel.and_then(|c| reader.read_tex_coords(c.0).map(|t| t.into_f32().collect()));
        let indices: Option<Vec<usize>> = reader
            .read_indices()
            .map(|i| i.into_u32().map(|v| v as usize).collect());

        for [a, b, c] in triangle_corners(primitive.mode(), indices.as_deref(), positions.len())? {
            for &i in &[a, b, c] {
                let p = positions[i];
                build.soup.push([p.x, p.y, p.z]);
                let corner = uv
                    .as_ref()
                    .and_then(|u| u.get(i))
                    .map(|&[u, v]| [u as f64, v as f64])
                    .unwrap_or([0.0, 0.0]);
                build.corner_uv.push(corner);
            }
            build.tri_material.push(slot);
        }
    }

    Ok(build)
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
                    if let [a, b, c] = *chunk {
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
                    if let [a, b, c] = *chunk {
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

/// The UV channel a converted material's base-colour texture samples, if textured.
fn textured_channel(material: &Material) -> Option<ChannelId> {
    match material {
        Material::Pbr(m) => m.base_color_map.as_ref().map(|t| t.uv_channel),
        Material::Phong(m) => m.diffuse_map.as_ref().map(|t| t.uv_channel),
    }
}

/// Convert a glTF PBR material to the new-geometry [`Material`], resolving its
/// base-colour texture to an embedded raster (or an external URI) when present.
/// Returns the UV channel the texture samples so the caller can read that set.
fn convert_material(
    material: &gltf::Material,
    buffer_data: &[Vec<u8>],
    base_uri: &Uri,
) -> (Material, Option<ChannelId>) {
    let pbr = material.pbr_metallic_roughness();

    let mut channel = None;
    let base_color_map = pbr.base_color_texture().and_then(|info| {
        let raster = resolve_image(info.texture().source().source(), buffer_data, base_uri)?;
        let uv_channel = ChannelId(info.tex_coord());
        channel = Some(uv_channel);
        Some(Texture {
            raster: Arc::new(raster),
            sampler: convert_sampler(&info.texture().sampler()),
            transform: None,
            uv_channel,
        })
    });

    let converted = Material::Pbr(PbrMaterial {
        base_color: pbr.base_color_factor(),
        metallic: pbr.metallic_factor(),
        roughness: pbr.roughness_factor(),
        emissive: material.emissive_factor(),
        base_color_map,
        metallic_roughness_map: None,
        normal_map: None,
        occlusion_map: None,
        emissive_map: None,
        alpha_mode: convert_alpha_mode(material.alpha_mode(), material.alpha_cutoff()),
        double_sided: material.double_sided(),
    });

    (converted, channel)
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
    let mut out = MeshBuild::default();
    for build in builds {
        let base = out.materials.len() as u32;
        out.materials.extend(build.materials);
        out.soup.extend(build.soup);
        out.corner_uv.extend(build.corner_uv);
        out.tri_material
            .extend(build.tri_material.into_iter().map(|s| s.map(|s| s + base)));
        out.channels.extend(build.channels);
    }
    out
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
        // One `Explicit` UV buffer (per-corner, aligned to the triangles) reused
        // for every sampled channel; a given triangle only ever binds one
        // material, so its corner slot already holds that material's UV.
        let uvs: BTreeMap<ChannelId, UvSource> = build
            .channels
            .iter()
            .map(|&channel| {
                (
                    channel,
                    UvSource::Explicit(build.corner_uv.clone().into_boxed_slice()),
                )
            })
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

fn build_feature(
    geometry: Geometry,
    mesh_names: &[String],
    node_names: &[String],
    primitive_count: usize,
    params: &GltfReaderCompiledParam,
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

    let mut feature = Feature::new_with_attributes(attributes);
    feature.geometry = Arc::new(geometry);
    feature
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
    fn bare_build(soup: Vec<[f64; 3]>) -> MeshBuild {
        let tris = soup.len() / 3;
        MeshBuild {
            corner_uv: vec![[0.0, 0.0]; soup.len()],
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
            corner_uv: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            materials: vec![material],
            channels: BTreeSet::from([ChannelId(0)]),
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
        assert_eq!(build.corner_uv, vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]);

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
}
