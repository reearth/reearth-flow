use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

use nusamai_citygml::GeometryRef;
use nusamai_citygml::{object::ObjectStereotype, Color, GeometryType, Value};
use nusamai_plateau::Entity;
use nusamai_projection::crs::EpsgCode;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::algorithm::hole::HoleCounter;
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_geometry::utils::are_points_coplanar;
use serde::{Deserialize, Serialize};

use reearth_flow_geometry::types::geometry::Geometry2D as FlowGeometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D as FlowGeometry3D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;

use crate::error::Error;

static EPSILON: f64 = 1e-10;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum GeometryValue {
    None,
    CityGmlGeometry(CityGmlGeometry),
    FlowGeometry2D(FlowGeometry2D),
    FlowGeometry3D(FlowGeometry3D),
}

impl GeometryValue {
    pub fn as_flow_geometry_2d(&self) -> Option<&FlowGeometry2D> {
        match self {
            Self::FlowGeometry2D(geometry) => Some(geometry),
            _ => None,
        }
    }

    pub fn as_flow_geometry_3d(&self) -> Option<&FlowGeometry3D> {
        match self {
            Self::FlowGeometry3D(geometry) => Some(geometry),
            _ => None,
        }
    }

    pub fn as_citygml_geometry(&self) -> Option<&CityGmlGeometry> {
        match self {
            Self::CityGmlGeometry(geometry) => Some(geometry),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Geometry {
    pub epsg: Option<EpsgCode>,
    pub value: GeometryValue,
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
        let ObjectStereotype::Feature { id: _, geometries } = &obj.stereotype else {
            return Err(Error::unsupported_feature("no feature found"));
        };
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
                    geometry_feature.polygons.extend(polygons);
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
                            feature.polygons.extend(polygons);
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
            epsg,
            GeometryValue::CityGmlGeometry(geometry_entity),
        ))
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            epsg: None,
            value: GeometryValue::None,
        }
    }
}

impl Geometry {
    pub fn new(epsg: EpsgCode, value: GeometryValue) -> Self {
        Self {
            epsg: Some(epsg),
            value,
        }
    }

    pub fn with_value(value: GeometryValue) -> Self {
        Self { epsg: None, value }
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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct Appearance {
    pub material: Option<Material>,
}

impl Appearance {
    pub fn new(material: Option<Material>) -> Self {
        Self { material }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
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

    pub fn hole_count(&self) -> usize {
        self.features
            .iter()
            .map(|feature| {
                feature
                    .polygons
                    .iter()
                    .map(|poly| poly.hole_count())
                    .sum::<usize>()
            })
            .sum()
    }
    pub fn are_points_coplanar(&self) -> bool {
        self.features.iter().all(|feature| {
            feature.polygons.iter().all(|poly| {
                let result = are_points_coplanar(poly.clone().into(), EPSILON);
                result.is_some()
            })
        })
    }

    pub fn elevation(&self) -> f64 {
        self.features
            .first()
            .and_then(|feature| feature.polygons.first())
            .and_then(|poly| poly.exterior().0.first())
            .map_or(0.0, |p| p.z)
    }
}

impl From<CityGmlGeometry> for FlowGeometry2D {
    fn from(geometry: CityGmlGeometry) -> Self {
        let mut polygons = Vec::<Polygon2D<f64>>::new();
        for feature in geometry.features {
            for polygon in feature.polygons {
                polygons.push(polygon.into());
            }
        }
        Self::MultiPolygon(MultiPolygon2D::from(polygons))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeometryFeature {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub ty: GeometryFeatureType,
    pub lod: Option<u8>,
    pub pos: u32,
    pub len: u32,
    pub polygons: Vec<Polygon3D<f64>>,
}

impl GeometryFeature {
    pub fn name(&self) -> &str {
        self.ty.name()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GeometryFeatureType {
    /// Polygons (solids)
    Solid,
    /// Polygons (surfaces)
    MultiSurface,
    /// Composite surface
    CompositeSurface,
    Surface,
    /// Polygons (triangles)
    Triangle,
    /// Line-strings
    MultiCurve,
    Curve,
    /// Points
    MultiPoint,
    Point,
    /// Tin
    Tin,
}

impl From<nusamai_citygml::geometry::GeometryType> for GeometryFeatureType {
    fn from(ty: nusamai_citygml::geometry::GeometryType) -> Self {
        match ty {
            nusamai_citygml::geometry::GeometryType::Solid => Self::Solid,
            nusamai_citygml::geometry::GeometryType::MultiSurface => Self::MultiSurface,
            nusamai_citygml::geometry::GeometryType::CompositeSurface => Self::CompositeSurface,
            nusamai_citygml::geometry::GeometryType::Surface => Self::Surface,
            nusamai_citygml::geometry::GeometryType::Triangle => Self::Triangle,
            nusamai_citygml::geometry::GeometryType::MultiCurve => Self::MultiCurve,
            nusamai_citygml::geometry::GeometryType::Curve => Self::Curve,
            nusamai_citygml::geometry::GeometryType::MultiPoint => Self::MultiPoint,
            nusamai_citygml::geometry::GeometryType::Point => Self::Point,
            nusamai_citygml::geometry::GeometryType::Tin => Self::Tin,
        }
    }
}

impl GeometryFeatureType {
    pub fn all_type_names() -> Vec<String> {
        [
            "Solid",
            "MultiSurface",
            "CompositeSurface",
            "Surface",
            "Triangle",
            "MultiCurve",
            "Curve",
            "MultiPoint",
            "Point",
            "Tin",
        ]
        .iter()
        .map(|name| name.to_string())
        .collect()
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Solid => "Solid",
            Self::MultiSurface => "MultiSurface",
            Self::CompositeSurface => "CompositeSurface",
            Self::Surface => "Surface",
            Self::Triangle => "Triangle",
            Self::MultiCurve => "MultiCurve",
            Self::Curve => "Curve",
            Self::MultiPoint => "MultiPoint",
            Self::Point => "Point",
            Self::Tin => "Tin",
        }
    }
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
            ty: geometry.ty.into(),
            lod: Some(geometry.lod),
            pos: geometry.pos,
            len: geometry.len,
            polygons: Vec::new(),
        }
    }
}
