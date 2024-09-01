use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_geometry::types::traits::Elevation;
use std::fmt::Display;
use std::{hash::Hash, path::Path};
use url::Url;

use nusamai_citygml::Color;
use nusamai_projection::crs::EpsgCode;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::algorithm::hole::HoleCounter;
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_geometry::utils::are_points_coplanar;
use serde::{Deserialize, Serialize};

use reearth_flow_geometry::types::geometry::Geometry2D as FlowGeometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D as FlowGeometry3D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;

static EPSILON: f64 = 1e-10;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum GeometryValue {
    None,
    CityGmlGeometry(CityGmlGeometry),
    FlowGeometry2D(FlowGeometry2D<f64>),
    FlowGeometry3D(FlowGeometry3D<f64>),
}

impl GeometryValue {
    pub fn as_flow_geometry_2d(&self) -> Option<&FlowGeometry2D<f64>> {
        match self {
            Self::FlowGeometry2D(geometry) => Some(geometry),
            _ => None,
        }
    }

    pub fn as_flow_geometry_3d(&self) -> Option<&FlowGeometry3D<f64>> {
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

impl Texture {
    pub fn to_gltf(
        &self,
        images: &mut IndexSet<Image, ahash::RandomState>,
    ) -> nusamai_gltf_json::Texture {
        let (image_index, _) = images.insert_full(Image {
            uri: self.uri.clone().into(),
        });
        nusamai_gltf_json::Texture {
            source: Some(image_index as u32),
            ..Default::default()
        }
    }
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

use indexmap::IndexSet;
use nusamai_gltf::nusamai_gltf_json;
use nusamai_gltf::nusamai_gltf_json::{BufferView, MimeType};

impl Material {
    pub fn to_gltf(
        &self,
        texture_set: &mut IndexSet<Texture, ahash::RandomState>,
        texture: Option<&Texture>,
    ) -> nusamai_gltf_json::Material {
        let tex = if let Some(texture) = texture {
            let (tex_idx, _) = texture_set.insert_full(texture.clone());
            Some(nusamai_gltf_json::TextureInfo {
                index: tex_idx as u32,
                tex_coord: 0,
                ..Default::default()
            })
        } else {
            None
        };
        nusamai_gltf_json::Material {
            pbr_metallic_roughness: Some(nusamai_gltf_json::MaterialPbrMetallicRoughness {
                base_color_factor: to_f64x4(self.diffuse_color.into()),
                metallic_factor: 0.2,
                roughness_factor: 0.5,
                base_color_texture: tex,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
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
            .map(|feature| CityGmlGeometry {
                features: vec![feature.clone()],
                ..self.clone()
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

    pub fn is_elevation_zero(&self) -> bool {
        self.features
            .iter()
            .all(|feature| feature.polygons.iter().all(|poly| poly.is_elevation_zero()))
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
    pub ty: GeometryType,
    pub lod: Option<u8>,
    pub pos: u32,
    pub len: u32,
    pub polygons: Vec<Polygon3D<f64>>,
    pub feature_id: Option<String>,
    pub feature_type: Option<String>,
    pub composite_surfaces: Vec<GeometryFeature>,
}

impl GeometryFeature {
    pub fn name(&self) -> &str {
        self.ty.name()
    }
}

impl From<GeometryFeature> for Vec<geojson::Value> {
    fn from(feature: GeometryFeature) -> Self {
        feature
            .polygons
            .into_iter()
            .map(|poly| poly.into())
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GeometryType {
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

impl From<nusamai_citygml::geometry::GeometryType> for GeometryType {
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

impl<T: CoordNum, Z: CoordNum> From<&reearth_flow_geometry::types::geometry::Geometry<T, Z>>
    for GeometryType
{
    fn from(geometry: &reearth_flow_geometry::types::geometry::Geometry<T, Z>) -> Self {
        match geometry {
            reearth_flow_geometry::types::geometry::Geometry::Solid(_) => Self::Solid,
            reearth_flow_geometry::types::geometry::Geometry::Triangle(_) => Self::Triangle,
            reearth_flow_geometry::types::geometry::Geometry::MultiPoint(_) => Self::MultiPoint,
            reearth_flow_geometry::types::geometry::Geometry::Point(_) => Self::Point,
            reearth_flow_geometry::types::geometry::Geometry::Line(_) => Self::Curve,
            reearth_flow_geometry::types::geometry::Geometry::LineString(_) => Self::MultiCurve,
            reearth_flow_geometry::types::geometry::Geometry::Polygon(_) => Self::Surface,
            reearth_flow_geometry::types::geometry::Geometry::MultiLineString(_) => {
                Self::MultiCurve
            }
            reearth_flow_geometry::types::geometry::Geometry::MultiPolygon(_) => Self::MultiSurface,
            reearth_flow_geometry::types::geometry::Geometry::Rect(_) => Self::Surface,
            _ => unreachable!(),
        }
    }
}

impl GeometryType {
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
            feature_id: geometry.feature_id,
            feature_type: geometry.feature_type,
            composite_surfaces: Vec::new(),
        }
    }
}

fn to_f64x4(c: [f32; 4]) -> [f64; 4] {
    [
        f64::from(c[0]),
        f64::from(c[1]),
        f64::from(c[2]),
        f64::from(c[3]),
    ]
}

#[derive(Debug, Serialize, Clone, Hash, PartialEq, Eq, Deserialize)]
pub struct Image {
    pub uri: Url,
}

impl Image {
    pub fn to_gltf(
        &self,
        buffer_views: &mut Vec<BufferView>,
        bin_content: &mut Vec<u8>,
    ) -> std::io::Result<nusamai_gltf_json::Image> {
        if let Ok(path) = self.uri.to_file_path() {
            // NOTE: temporary implementation
            let (content, mime_type) = load_image(&path)?;

            buffer_views.push(BufferView {
                byte_offset: bin_content.len() as u32,
                byte_length: content.len() as u32,
                ..Default::default()
            });

            bin_content.extend(content);

            Ok(nusamai_gltf_json::Image {
                mime_type: Some(mime_type),
                buffer_view: Some(buffer_views.len() as u32 - 1),
                ..Default::default()
            })
        } else {
            Ok(nusamai_gltf_json::Image {
                uri: Some(self.uri.to_string()),
                ..Default::default()
            })
        }
    }
}

// NOTE: temporary implementation
fn load_image(path: &Path) -> std::io::Result<(Vec<u8>, MimeType)> {
    if let Some(ext) = path.extension() {
        match ext.to_ascii_lowercase().to_str() {
            Some("tif" | "tiff" | "png") => {
                let image = image::open(path)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

                let mut writer = std::io::Cursor::new(Vec::new());
                let encoder = image::codecs::png::PngEncoder::new(&mut writer);
                image
                    .write_with_encoder(encoder)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

                Ok((writer.into_inner(), MimeType::ImagePng))
            }
            Some("jpg" | "jpeg") => Ok((std::fs::read(path)?, MimeType::ImageJpeg)),
            _ => {
                let err = format!("Unsupported image format: {:?}", path);
                Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
            }
        }
    } else {
        let err = format!("Unsupported image format: {:?}", path);
        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }
}
