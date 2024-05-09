use std::hash::Hash;

use nusamai_citygml::{object::ObjectStereotype, Color, GeometryType, Value};
use nusamai_plateau::Entity;
use nusamai_projection::crs::EpsgCode;
use reearth_flow_common::uri::Uri;
use serde::{Deserialize, Serialize};

use reearth_flow_geometry::types::multi_polygon::{MultiPolygon, MultiPolygon2D};

use crate::error::Error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Geometry {
    pub id: String,
    pub epsg: EpsgCode,
    pub entity: GeometryEntity,
    pub attributes: Option<serde_json::Value>,
}

impl TryFrom<Entity> for Geometry {
    type Error = Error;

    fn try_from(entity: Entity) -> Result<Self, Self::Error> {
        let app = entity.appearance_store.read().unwrap();
        let theme = {
            app.themes
                .get("rgbTexture")
                .or_else(|| app.themes.get("FMETheme"))
        };
        let geoms = entity.geometry_store.write().unwrap();
        let apperance = entity.appearance_store.read().unwrap();
        let epsg = geoms.epsg;
        // entity must be a Feature
        let Value::Object(obj) = &entity.root else {
            return Err(Error::unsupported_feature("no object found"));
        };
        let ObjectStereotype::Feature { id, geometries } = &obj.stereotype else {
            return Err(Error::unsupported_feature("no feature found"));
        };
        let attributes = entity.root.to_attribute_json();
        let mut mpoly = nusamai_geometry::MultiPolygon3::<f64>::new();
        let mut geometry_features = Vec::<GeometryFeature>::new();
        geometries.iter().for_each(|entry| match entry.ty {
            GeometryType::Solid
            | GeometryType::Surface
            | GeometryType::MultiSurface
            | GeometryType::Triangle => {
                geometry_features.push(entry.clone().into());
                for idx_poly in geoms
                    .multipolygon
                    .iter_range(entry.pos as usize..(entry.pos + entry.len) as usize)
                {
                    let poly = idx_poly.transform(|c| geoms.vertices[*c as usize]);
                    mpoly.push(&poly);
                }
            }
            GeometryType::Curve | GeometryType::MultiCurve => unimplemented!(),
            GeometryType::Point | GeometryType::MultiPoint => unimplemented!(),
            GeometryType::Tin => unimplemented!(),
        });

        let mut geometry_entity = GeometryEntity::new(
            geometry_features,
            Some(mpoly.into()),
            apperance
                .materials
                .iter()
                .cloned()
                .map(Into::into)
                .collect(),
            apperance.textures.iter().cloned().map(Into::into).collect(),
        );

        if let Some(theme) = theme {
            // find and apply materials
            {
                let mut poly_materials = vec![None; geoms.multipolygon.len()];
                for surface in &geoms.surface_spans {
                    if let Some(&mat) = theme.surface_id_to_material.get(&surface.id) {
                        for idx in surface.start..surface.end {
                            poly_materials[idx as usize] = Some(mat);
                        }
                    }
                }
                geometry_entity.polygon_materials = poly_materials;
            }
            // find and apply textures
            {
                let mut ring_id_iter = geoms.ring_ids.iter();
                let mut poly_textures = Vec::with_capacity(geoms.multipolygon.len());
                let mut poly_uvs = nusamai_geometry::MultiPolygon::new();

                for poly in &geoms.multipolygon {
                    for (i, ring) in poly.rings().enumerate() {
                        let tex = ring_id_iter
                            .next()
                            .unwrap()
                            .and_then(|ring_id| theme.ring_id_to_texture.get(&ring_id));

                        let mut add_dummy_texture = || {
                            let uv = [[0.0, 0.0]].into_iter().cycle().take(ring.len() + 1);
                            if i == 0 {
                                poly_textures.push(None);
                                poly_uvs.add_exterior(uv);
                            } else {
                                poly_uvs.add_interior(uv);
                            }
                        };

                        match tex {
                            Some((idx, uv)) if ring.len() == uv.len() => {
                                // texture found
                                if i == 0 {
                                    poly_textures.push(Some(*idx));
                                    poly_uvs.add_exterior(uv.iter_closed());
                                } else {
                                    poly_uvs.add_interior(uv.iter_closed());
                                }
                            }
                            Some((_, uv)) if uv.len() != ring.len() => {
                                // invalid texture found
                                add_dummy_texture();
                            }
                            _ => {
                                // no texture found
                                add_dummy_texture();
                            }
                        };
                    }
                }
                // apply textures to polygons
                geometry_entity.polygon_textures = poly_textures;
                geometry_entity.polygon_uv = Some(poly_uvs.into());
            }
        } else {
            // set 'null' appearance if no theme found
            geometry_entity.polygon_materials = vec![None; geoms.multipolygon.len()];
            geometry_entity.polygon_textures = vec![None; geoms.multipolygon.len()];
            let mut poly_uvs = nusamai_geometry::MultiPolygon::new();
            for poly in &geoms.multipolygon {
                for (i, ring) in poly.rings().enumerate() {
                    let uv = [[0.0, 0.0]].into_iter().cycle().take(ring.len() + 1);
                    if i == 0 {
                        poly_uvs.add_exterior(uv);
                    } else {
                        poly_uvs.add_interior(uv);
                    }
                }
            }
            geometry_entity.polygon_uv = Some(poly_uvs.into());
        }
        Ok(Geometry::new(
            id.to_string(),
            epsg,
            geometry_entity,
            Some(attributes),
        ))
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            epsg: nusamai_projection::crs::EPSG_JGD2011_GEOGRAPHIC_3D,
            entity: GeometryEntity::new(Vec::new(), None, Vec::new(), Vec::new()),
            attributes: None,
        }
    }
}

impl Geometry {
    pub fn new(
        id: String,
        epsg: EpsgCode,
        entity: GeometryEntity,
        attributes: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id,
            epsg,
            entity,
            attributes,
        }
    }
}

#[derive(Debug, Serialize, Clone, Hash, PartialEq, Eq, Deserialize)]
pub struct Texture {
    pub uri: Uri,
}

impl From<nusamai_plateau::appearance::Texture> for Texture {
    fn from(texture: nusamai_plateau::appearance::Texture) -> Self {
        Self {
            uri: texture
                .image_url
                .try_into()
                .unwrap_or(Uri::for_test("file:///dummy")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Material {
    pub diffuse_color: Color,
    pub specular_color: Color,
    pub ambient_intensity: f64,
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.diffuse_color == other.diffuse_color
            && self.specular_color == other.specular_color
            && self.ambient_intensity == other.ambient_intensity
    }
}

impl Eq for Material {}

impl Hash for Material {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.diffuse_color.hash(state);
        self.specular_color.hash(state);
        self.ambient_intensity.to_bits().hash(state);
    }
}

impl From<nusamai_plateau::appearance::Material> for Material {
    fn from(material: nusamai_plateau::appearance::Material) -> Self {
        Self {
            diffuse_color: material.diffuse_color,
            specular_color: material.specular_color,
            ambient_intensity: material.ambient_intensity,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Appearance {
    pub material: Option<Material>,
}

impl Appearance {
    pub fn new(material: Option<Material>) -> Self {
        Self { material }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeometryEntity {
    pub features: Vec<GeometryFeature>,
    pub polygons: Option<MultiPolygon<f64>>,
    materials: Vec<Material>,
    textures: Vec<Texture>,
    pub polygon_materials: Vec<Option<u32>>,
    pub polygon_textures: Vec<Option<u32>>,
    pub polygon_uv: Option<MultiPolygon2D<f64>>,
}

impl GeometryEntity {
    pub fn new(
        features: Vec<GeometryFeature>,
        polygons: Option<MultiPolygon<f64>>,
        materials: Vec<Material>,
        textures: Vec<Texture>,
    ) -> Self {
        Self {
            features,
            polygons,
            materials,
            textures,
            polygon_materials: Vec::new(),
            polygon_textures: Vec::new(),
            polygon_uv: None,
        }
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }

    pub fn textures(&self) -> &[Texture] {
        &self.textures
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeometryFeature {
    #[serde(rename = "type")]
    pub ty: GeometryType,
    pub lod: Option<u8>,
    pub pos: u32,
    pub len: u32,
}

impl From<nusamai_citygml::geometry::GeometryRef> for GeometryFeature {
    fn from(geometry: nusamai_citygml::geometry::GeometryRef) -> Self {
        Self {
            ty: geometry.ty,
            lod: Some(geometry.lod),
            pos: geometry.pos,
            len: geometry.len,
        }
    }
}
