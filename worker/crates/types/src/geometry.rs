use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

use nusamai_citygml::GeometryRef;
use nusamai_citygml::{object::ObjectStereotype, Color, GeometryType, Value};
use nusamai_plateau::Entity;
use nusamai_projection::crs::EpsgCode;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::polygon::Polygon3D;
use serde::{Deserialize, Serialize};

use reearth_flow_geometry::types::geometry::Geometry2D as FlowGeometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D as FlowGeometry3D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;

use crate::error::Error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GeometryValue {
    Null,
    CityGmlGeometry(CityGmlGeometry),
    FlowGeometry2D(FlowGeometry2D),
    FlowGeometry3D(FlowGeometry3D),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Geometry {
    pub id: String,
    pub name: Option<String>,
    pub epsg: Option<EpsgCode>,
    pub value: GeometryValue,
    pub attributes: Option<serde_json::Value>,
}

impl TryFrom<Entity> for Geometry {
    type Error = Error;

    fn try_from(entity: Entity) -> Result<Self, Self::Error> {
        let app = entity.appearance_store.read().unwrap();
        let name = entity.name.clone();
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
        let mut geometry_features = Vec::<GeometryFeature>::new();
        let operation = |geometry: &GeometryRef| -> Option<GeometryFeature> {
            match geometry.ty {
                GeometryType::Solid
                | GeometryType::Surface
                | GeometryType::MultiSurface
                | GeometryType::CompositeSurface
                | GeometryType::Triangle => {
                    let mut polygons = Vec::<Polygon3D<f64>>::new();
                    for idx_poly in geoms
                        .multipolygon
                        .iter_range(geometry.pos as usize..(geometry.pos + geometry.len) as usize)
                    {
                        let poly = idx_poly.transform(|c| geoms.vertices[*c as usize]);
                        polygons.push(poly.into());
                    }
                    let mut geometry_feature = GeometryFeature::from(geometry.clone());
                    geometry_feature.polygon.extend(polygons);
                    Some(geometry_feature)
                }
                GeometryType::Curve | GeometryType::MultiCurve => unimplemented!(),
                GeometryType::Point | GeometryType::MultiPoint => unimplemented!(),
                GeometryType::Tin => unimplemented!(),
            }
        };
        geometry_features.extend(geometries.iter().flat_map(operation));
        let bounded_map = entity
            .bounded
            .iter()
            .flat_map(|bound| {
                let id = bound.id.clone()?;
                Some((id, bound.clone()))
            })
            .collect::<HashMap<_, _>>();

        geometries
            .iter()
            .enumerate()
            .for_each(|(index, geometry)| match geometry.ty {
                GeometryType::Solid
                | GeometryType::Surface
                | GeometryType::MultiSurface
                | GeometryType::CompositeSurface
                | GeometryType::Triangle => {
                    if geometry.solid_ids.is_empty() {
                        return;
                    }
                    let Some(feature) = geometry_features.get_mut(index) else {
                        return;
                    };
                    geometry.solid_ids.iter().for_each(|solid_id| {
                        if let Some(bound) = bounded_map.get(solid_id) {
                            let mut polygons = Vec::<Polygon3D<f64>>::new();
                            for idx_poly in geoms
                                .multipolygon
                                .iter_range(bound.pos as usize..(bound.pos + bound.len) as usize)
                            {
                                let poly = idx_poly.transform(|c| geoms.vertices[*c as usize]);
                                polygons.push(poly.into());
                            }
                            feature.polygon.extend(polygons);
                        }
                    });
                }
                GeometryType::Curve | GeometryType::MultiCurve => unimplemented!(),
                GeometryType::Point | GeometryType::MultiPoint => unimplemented!(),
                GeometryType::Tin => unimplemented!(),
            });

        geometry_features.extend(entity.bounded.iter().flat_map(operation));

        let mut geometry_entity = CityGmlGeometry::new(
            geometry_features,
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
                            .clone()
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
            Some(name),
            epsg,
            GeometryValue::CityGmlGeometry(geometry_entity),
            Some(attributes),
        ))
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: Some("".to_string()),
            epsg: None,
            value: GeometryValue::Null,
            attributes: None,
        }
    }
}

impl Geometry {
    pub fn new(
        id: String,
        name: Option<String>,
        epsg: EpsgCode,
        value: GeometryValue,
        attributes: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id,
            name,
            epsg: Some(epsg),
            value,
            attributes,
        }
    }

    pub fn with_value(value: GeometryValue) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: None,
            epsg: None,
            value,
            attributes: None,
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
pub struct CityGmlGeometry {
    pub features: Vec<GeometryFeature>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub polygon_materials: Vec<Option<u32>>,
    pub polygon_textures: Vec<Option<u32>>,
    pub polygon_uv: Option<MultiPolygon2D<f64>>,
}

impl CityGmlGeometry {
    pub fn new(
        features: Vec<GeometryFeature>,
        materials: Vec<Material>,
        textures: Vec<Texture>,
    ) -> Self {
        Self {
            features,
            materials,
            textures,
            polygon_materials: Vec::new(),
            polygon_textures: Vec::new(),
            polygon_uv: None,
        }
    }

    pub fn split_feature(&self) -> Vec<CityGmlGeometry> {
        self.features
            .iter()
            .map(|feature| {
                CityGmlGeometry::new(
                    vec![feature.clone()],
                    self.materials.clone(),
                    self.textures.clone(),
                )
            })
            .collect()
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
    pub id: Option<String>,
    pub ty: GeometryType,
    pub lod: Option<u8>,
    pub pos: u32,
    pub len: u32,
    pub polygon: Vec<Polygon3D<f64>>,
}

impl Display for GeometryFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = format!("lod{}{:?}", self.lod.unwrap_or_default(), self.ty);
        write!(f, "{}", msg)
    }
}

impl From<nusamai_citygml::geometry::GeometryRef> for GeometryFeature {
    fn from(geometry: nusamai_citygml::geometry::GeometryRef) -> Self {
        let id = geometry.id.map(|id| id.value());
        Self {
            id,
            ty: geometry.ty,
            lod: Some(geometry.lod),
            pos: geometry.pos,
            len: geometry.len,
            polygon: Vec::new(),
        }
    }
}
