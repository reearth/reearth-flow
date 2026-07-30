//! New-geometry path for the OBJ Reader. Reads OBJ faces into a single
//! `PolygonMesh3D` per feature (Euclidean frame; OBJ is model-space), attaching
//! any `usemtl` material as a per-face `Material::Phong` appearance (with a
//! `diffuse_map` texture and per-corner UVs when the face supplies them).
//! Reuses `super`'s OBJ/MTL parser.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use bytes::Bytes;
use indexmap::IndexMap;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::{
    appearance::{
        ChannelId, Material as GeoMaterial, PhongMaterial, Raster, Sampler, Texture, ThemeId,
        UvSource,
    },
    coordinate::CoordinateFrame,
    polygon::Polygon3D,
    polygon_mesh::PolygonMesh3D,
    Euclidean3DGeometry, Geometry,
};
use reearth_flow_runtime::{
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use tokio::sync::mpsc::Sender;

use crate::errors::SourceError;
use crate::file::reader::runner::get_input_path;

use super::{Face, ObjData, ObjReaderCompiledParam};

pub(super) async fn read(
    ctx: &NodeContext,
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    content: &Bytes,
    params: &ObjReaderCompiledParam,
    sender: &Sender<(Port, IngestionMessage)>,
) -> Result<(), SourceError> {
    let obj_data = super::parse_obj_content(content)?;
    let materials = load_materials(ctx, storage_resolver, &obj_data, params).await?;

    if params.merge_groups {
        let feature = build_feature(
            &obj_data,
            &obj_data.faces,
            group_attrs_merged(&obj_data),
            &materials,
            params,
        );
        send_feature(sender, feature).await?;
    } else {
        for (name, faces) in group_faces(&obj_data) {
            let feature = build_feature(
                &obj_data,
                &faces,
                group_attrs_one(&obj_data, &name),
                &materials,
                params,
            );
            send_feature(sender, feature).await?;
        }
    }
    Ok(())
}

/// Load the OBJ's materials, mirroring old-world `read_obj`'s material-loading
/// block: an explicit `material_file` param overrides any `mtllib` directives in
/// the OBJ itself; otherwise every referenced `mtllib` is parsed and merged.
/// Returns an empty map when `parse_materials` is off, or if resolution fails
/// (a warning is logged; a missing/bad MTL never fails the read).
async fn load_materials(
    ctx: &NodeContext,
    storage_resolver: Arc<reearth_flow_storage::resolve::StorageResolver>,
    obj_data: &ObjData,
    params: &ObjReaderCompiledParam,
) -> Result<HashMap<String, super::Material>, SourceError> {
    if !params.parse_materials {
        return Ok(HashMap::new());
    }

    let obj_uri = get_input_path(&params.common)
        .map_err(SourceError::ObjReader)?
        .unwrap_or_else(|| Uri::from_str("file://./unknown.obj").unwrap());

    let mut all_materials = HashMap::new();

    if let Some(ref material_file) = params.material_file {
        let external_mtl = material_file
            .eval_string_env_only(ctx.env_vars.clone())
            .map_err(|e| {
                SourceError::ObjReader(format!("Failed to evaluate material_file: {e:?}"))
            })?;
        let mtl_uri =
            super::resolve_material_path(ctx, storage_resolver.clone(), &obj_uri, &external_mtl)
                .await?;
        if let Some(mtl_uri) = mtl_uri {
            match super::parse_mtl(ctx, storage_resolver.clone(), &mtl_uri).await {
                Ok(mats) => all_materials.extend(mats),
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse external material file {}: {}",
                        external_mtl,
                        e
                    );
                }
            }
        }
    } else if !obj_data.material_libs.is_empty() {
        for mtl_lib in &obj_data.material_libs {
            let mtl_uri =
                super::resolve_material_path(ctx, storage_resolver.clone(), &obj_uri, mtl_lib)
                    .await?;
            if let Some(mtl_uri) = mtl_uri {
                match super::parse_mtl(ctx, storage_resolver.clone(), &mtl_uri).await {
                    Ok(mats) => all_materials.extend(mats),
                    Err(e) => {
                        tracing::warn!("Failed to parse material file {}: {}", mtl_lib, e);
                    }
                }
            }
        }
    }

    Ok(all_materials)
}

/// Group faces by object (fallback group, then "default"), preserving the old
/// reader's grouping.
fn group_faces(obj_data: &ObjData) -> Vec<(String, Vec<Face>)> {
    let mut groups: IndexMap<String, Vec<Face>> = IndexMap::new();
    for face in &obj_data.faces {
        let key = face
            .object
            .clone()
            .or_else(|| face.group.clone())
            .unwrap_or_else(|| "default".to_string());
        groups.entry(key).or_default().push(face.clone());
    }
    groups.into_iter().collect()
}

/// Convert a set of OBJ faces into one `PolygonMesh3D` geometry, attaching each
/// face's `usemtl` material (if any) as a `Material::Phong` appearance before
/// the faces are merged into the mesh.
fn build_geometry(
    obj_data: &ObjData,
    faces: &[Face],
    materials: &HashMap<String, super::Material>,
    params: &ObjReaderCompiledParam,
) -> Geometry {
    let theme = ThemeId(Arc::from("default"));
    let mut polygons: Vec<Polygon3D> = Vec::new();
    for face in faces {
        let Some(ring) = face_ring(obj_data, face) else {
            continue;
        };
        let mut polygon = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            ring,
            std::iter::empty::<Vec<[f64; 3]>>(),
        );

        if let Some(mat) = face.material.as_ref().and_then(|n| materials.get(n)) {
            let uv = face_uv(obj_data, face, params);
            let material = to_phong(mat, uv.is_some());
            if let Err(e) =
                polygon.set_appearance(theme.clone(), material, uv.map(UvSource::Explicit))
            {
                tracing::warn!("OBJ: skipping face appearance: {e:?}");
            }
        }
        polygons.push(polygon);
    }
    if polygons.is_empty() {
        return Geometry::None;
    }
    match PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, &polygons) {
        Ok(mesh) => Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(Box::new(mesh))),
        Err(e) => {
            tracing::warn!("OBJ: failed to build PolygonMesh, dropping feature: {e:?}");
            Geometry::None
        }
    }
}

/// Per-corner UVs for a face (`v` flipped to `1 - v`, top-left canonical
/// origin), or `None` if texcoords are disabled or any corner lacks a valid
/// `vt` index.
fn face_uv(
    obj_data: &ObjData,
    face: &Face,
    params: &ObjReaderCompiledParam,
) -> Option<Box<[[f64; 2]]>> {
    if !params._include_texcoords {
        return None;
    }
    let mut uv = Vec::with_capacity(face.vertices.len());
    for v in &face.vertices {
        let ti = v.texture_index?;
        let idx = if ti > 0 {
            (ti - 1) as usize
        } else {
            (obj_data.texcoords.len() as i32 + ti) as usize
        };
        let t = obj_data.texcoords.get(idx)?;
        uv.push([t[0], 1.0 - t[1]]);
    }
    Some(uv.into_boxed_slice())
}

/// Map an OBJ/MTL `Material` to a `Material::Phong`. `with_texture` gates the
/// `diffuse_map` (only attach when the face supplies UVs); a textured Phong
/// without a UV set is rejected by `Polygon3D::set_appearance`, so a face
/// lacking UVs always gets a colour-only material.
///
/// `transparency`: OBJ `d`/`Tr` is opacity (1 = opaque) but the model's
/// `transparency` is 0 = opaque, so this stores `1 - d`. The current MTL parser
/// stores whatever value follows `d` *or* `Tr` into the same field without
/// inverting for `Tr` (whose direction is already opposite `d`), so this is a
/// known approximation inherited from the existing parser, not new here.
fn to_phong(mat: &super::Material, with_texture: bool) -> GeoMaterial {
    let diffuse_map = if with_texture {
        mat.texture_uri.as_ref().map(|uri| Texture {
            raster: Arc::new(Raster::Uri(uri.clone())),
            sampler: Sampler::default(),
            transform: None,
            uv_channel: ChannelId(0),
        })
    } else {
        None
    };
    GeoMaterial::Phong(PhongMaterial {
        diffuse: mat.diffuse.unwrap_or([1.0, 1.0, 1.0]),
        specular: mat.specular.unwrap_or([0.0, 0.0, 0.0]),
        emissive: [0.0, 0.0, 0.0],
        ambient_intensity: mat
            .ambient
            .map(|a| (a[0] + a[1] + a[2]) / 3.0)
            .unwrap_or(0.0),
        shininess: mat.shininess.unwrap_or(0.0),
        transparency: mat.transparency.map(|d| 1.0 - d).unwrap_or(0.0),
        diffuse_map,
        emissive_map: None,
        normal_map: None,
    })
}

/// Resolve a face's vertex ring to `[f64;3]` coordinates (OBJ 1-based / negative
/// indices), or `None` if any index is out of bounds or the ring has < 3 vertices.
fn face_ring(obj_data: &ObjData, face: &Face) -> Option<Vec<[f64; 3]>> {
    let mut ring = Vec::with_capacity(face.vertices.len());
    for v in &face.vertices {
        let idx = if v.vertex_index > 0 {
            (v.vertex_index - 1) as usize
        } else {
            (obj_data.vertices.len() as i32 + v.vertex_index) as usize
        };
        ring.push(*obj_data.vertices.get(idx)?);
    }
    (ring.len() >= 3).then_some(ring)
}

fn build_feature(
    obj_data: &ObjData,
    faces: &[Face],
    mut attributes: IndexMap<Attribute, AttributeValue>,
    materials: &HashMap<String, super::Material>,
    params: &ObjReaderCompiledParam,
) -> Feature {
    attributes.insert(
        Attribute::new("faceCount"),
        AttributeValue::Number(serde_json::Number::from(faces.len())),
    );
    Feature::new_with_attributes_and_geometry(
        attributes,
        build_geometry(obj_data, faces, materials, params),
    )
}

fn base_attrs() -> IndexMap<Attribute, AttributeValue> {
    let mut a = IndexMap::new();
    a.insert(
        Attribute::new("source"),
        AttributeValue::String("OBJ".to_string()),
    );
    a
}

fn group_attrs_one(obj_data: &ObjData, name: &str) -> IndexMap<Attribute, AttributeValue> {
    let mut a = base_attrs();
    let key = if obj_data.objects.iter().any(|o| o == name) {
        "object"
    } else {
        "group"
    };
    a.insert(
        Attribute::new(key),
        AttributeValue::String(name.to_string()),
    );
    a
}

fn group_attrs_merged(obj_data: &ObjData) -> IndexMap<Attribute, AttributeValue> {
    let mut a = base_attrs();
    let arr = |v: &[String]| {
        AttributeValue::Array(v.iter().cloned().map(AttributeValue::String).collect())
    };
    if !obj_data.objects.is_empty() {
        a.insert(Attribute::new("objects"), arr(&obj_data.objects));
    }
    if !obj_data.groups.is_empty() {
        a.insert(Attribute::new("groups"), arr(&obj_data.groups));
    }
    a
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
        .map_err(|e| SourceError::ObjReader(format!("Failed to send feature: {e}")))
}

#[cfg(test)]
fn test_params() -> ObjReaderCompiledParam {
    use crate::file::reader::runner::FileReaderCompiledParam;

    ObjReaderCompiledParam {
        parse_materials: false,
        material_file: None,
        triangulate: false,
        merge_groups: false,
        _include_normals: true,
        _include_texcoords: true,
        common: FileReaderCompiledParam {
            dataset: None,
            inline: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build ObjData directly via the shared parser.
    fn parse(obj: &str) -> super::super::ObjData {
        super::super::parse_obj_content(&bytes::Bytes::copy_from_slice(obj.as_bytes())).unwrap()
    }

    fn mesh_of(geom: Geometry) -> reearth_flow_geometry::polygon_mesh::PolygonMesh3D {
        match geom {
            Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(m)) => *m,
            other => panic!("expected Euclidean3D PolygonMesh, got {other:?}"),
        }
    }

    #[test]
    fn quad_face_becomes_one_polygon_mesh_face_euclidean() {
        let data = parse("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n");
        let params = test_params();
        let mesh = mesh_of(build_geometry(
            &data,
            &data.faces,
            &std::collections::HashMap::new(),
            &params,
        ));
        assert_eq!(*mesh.frame(), CoordinateFrame::Euclidean);
        assert_eq!(
            mesh.num_faces(),
            1,
            "one quad face preserved as one n-gon face"
        );
        assert!(
            mesh.appearance().is_none(),
            "no materials used by this face"
        );
    }

    #[test]
    fn face_with_material_gets_phong_appearance() {
        use reearth_flow_geometry::appearance::Material;

        let data = parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nusemtl red\nf 1 2 3\n");
        let mut materials = std::collections::HashMap::new();
        materials.insert(
            "red".to_string(),
            super::super::Material {
                name: "red".to_string(),
                diffuse: Some([0.8, 0.1, 0.1]),
                ..Default::default()
            },
        );
        let params = test_params();
        let mesh = mesh_of(build_geometry(&data, &data.faces, &materials, &params));
        let app = mesh.appearance().as_ref().expect("appearance attached");
        assert_eq!(app.materials().len(), 1);
        match &app.materials()[0] {
            Material::Phong(m) => assert_eq!(m.diffuse, [0.8, 0.1, 0.1]),
            other => panic!("expected Phong, got {other:?}"),
        }
    }

    #[test]
    fn textured_face_with_uvs_gets_diffuse_map_and_flipped_v() {
        use reearth_flow_geometry::appearance::{Material, UvSource};

        // Third corner's vt (0.25, 0.75) should end up as [0.25, 1.0 - 0.75] = [0.25, 0.25].
        let data = parse(
            "v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0.1 0.2\nvt 0.9 0.3\nvt 0.25 0.75\nusemtl tex\nf 1/1 2/2 3/3\n",
        );
        let mut materials = std::collections::HashMap::new();
        materials.insert(
            "tex".to_string(),
            super::super::Material {
                name: "tex".to_string(),
                diffuse: Some([1.0, 1.0, 1.0]),
                texture_uri: Some(
                    reearth_flow_common::uri::Uri::from_str("file:///t.png").unwrap(),
                ),
                ..Default::default()
            },
        );
        let params = test_params();
        let mesh = mesh_of(build_geometry(&data, &data.faces, &materials, &params));
        let app = mesh.appearance().as_ref().expect("appearance attached");

        match &app.materials()[0] {
            Material::Phong(m) => {
                assert!(
                    m.diffuse_map.is_some(),
                    "complete UVs -> textured diffuse_map"
                )
            }
            other => panic!("expected Phong, got {other:?}"),
        }

        let uv_set = app.themes()[0]
            .uv_sets
            .iter()
            .find_map(|set| match &set.uv {
                UvSource::Explicit(coords) => Some(coords.clone()),
                UvSource::WorldToTexture(_) => None,
            })
            .expect("an explicit UV set is present");
        assert_eq!(
            uv_set[2],
            [0.25, 0.25],
            "v flipped to 1 - v (top-left canonical origin)"
        );
    }

    // detailed.obj's faces with `vt` indices use materials without `map_Kd`, and
    // its one `map_Kd` material (green_material) is applied to a face lacking
    // `vt`, so `to_phong`'s `with_texture` gating never attaches a `diffuse_map`
    // here. This test therefore verifies a real OBJ+MTL pair reads into a
    // PolygonMesh carrying a colour-only Phong appearance, not a textured one;
    // the genuinely-textured (diffuse_map + UV-flip) path is covered by the
    // synthetic `textured_face_with_uvs_gets_diffuse_map_and_flipped_v` test above.
    #[test]
    fn real_detailed_obj_reads_as_polygon_mesh_with_material() {
        use reearth_flow_geometry::appearance::Material;

        let bytes = include_bytes!("../../../tests/fixture/testdata/obj/detailed.obj");
        let data = super::super::parse_obj_content(&bytes::Bytes::from_static(bytes)).unwrap();
        // Materials from the sibling materials.mtl, resolved relative to the mtl dir.
        let mtl = include_str!("../../../tests/fixture/testdata/obj/materials.mtl");
        let mtl_uri =
            reearth_flow_common::uri::Uri::from_str("file:///fixture/obj/materials.mtl").unwrap();
        let materials = super::super::parse_mtl_str(mtl, &mtl_uri);
        let params = test_params();
        let mesh = mesh_of(build_geometry(&data, &data.faces, &materials, &params));
        assert!(mesh.num_faces() > 0);
        let appearance = mesh
            .appearance()
            .as_ref()
            .expect("detailed.obj uses materials");
        assert!(
            appearance
                .materials()
                .iter()
                .any(|m| matches!(m, Material::Phong(_))),
            "detailed.obj's usemtl materials should map to at least one Phong appearance"
        );
    }

    #[test]
    fn textured_face_without_uvs_falls_back_to_colour_only() {
        use reearth_flow_geometry::appearance::Material;

        // Material has a texture Uri but the face has no vt indices.
        let data = parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nusemtl tex\nf 1 2 3\n");
        let mut materials = std::collections::HashMap::new();
        materials.insert(
            "tex".to_string(),
            super::super::Material {
                name: "tex".to_string(),
                diffuse: Some([1.0, 1.0, 1.0]),
                texture_uri: Some(
                    reearth_flow_common::uri::Uri::from_str("file:///t.png").unwrap(),
                ),
                ..Default::default()
            },
        );
        let params = test_params();
        let mesh = mesh_of(build_geometry(&data, &data.faces, &materials, &params));
        match &mesh.appearance().as_ref().unwrap().materials()[0] {
            Material::Phong(m) => assert!(m.diffuse_map.is_none(), "no UVs -> colour-only"),
            other => panic!("expected Phong, got {other:?}"),
        }
    }
}
