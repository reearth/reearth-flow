use std::fmt::Display;
use std::hash::Hash;

use nusamai_projection::vshift::Jgd2011ToWgs84;
use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_geometry::types::line_string::LineString3D;
use reearth_flow_geometry::types::traits::Elevation;

use nusamai_projection::crs::EpsgCode;
use reearth_flow_geometry::algorithm::hole::HoleCounter;
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_geometry::utils::are_points_coplanar;
use serde::{Deserialize, Serialize};

use reearth_flow_geometry::types::geometry::Geometry2D as FlowGeometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D as FlowGeometry3D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;

use crate::material::{Texture, X3DMaterial};

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.value, GeometryValue::None)
    }

    pub fn new_with(epsg: EpsgCode, value: GeometryValue) -> Self {
        Self {
            epsg: Some(epsg),
            value,
        }
    }

    pub fn with_value(value: GeometryValue) -> Self {
        Self { epsg: None, value }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CityGmlGeometry {
    pub gml_geometries: Vec<GmlGeometry>,
    pub materials: Vec<X3DMaterial>,
    pub textures: Vec<Texture>,
    pub polygon_materials: Vec<Option<u32>>,
    pub polygon_textures: Vec<Option<u32>>,
    pub polygon_uvs: flatgeom::MultiPolygon<'static, [f64; 2]>,
}

impl CityGmlGeometry {
    pub fn new(
        gml_geometries: Vec<GmlGeometry>,
        materials: Vec<X3DMaterial>,
        textures: Vec<Texture>,
    ) -> Self {
        Self {
            gml_geometries,
            materials,
            textures,
            polygon_materials: Vec::new(),
            polygon_textures: Vec::new(),
            polygon_uvs: flatgeom::MultiPolygon::default(),
        }
    }

    pub fn split_feature(&self) -> Vec<CityGmlGeometry> {
        self.gml_geometries
            .iter()
            .map(|feature| CityGmlGeometry {
                gml_geometries: vec![feature.clone()],
                ..self.clone()
            })
            .collect()
    }

    pub fn materials(&self) -> &[X3DMaterial] {
        &self.materials
    }

    pub fn textures(&self) -> &[Texture] {
        &self.textures
    }

    pub fn hole_count(&self) -> usize {
        self.gml_geometries
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
        self.gml_geometries.iter().all(|feature| {
            feature.polygons.iter().all(|poly| {
                let result = are_points_coplanar(poly.clone().into(), EPSILON);
                result.is_some()
            })
        })
    }

    pub fn elevation(&self) -> f64 {
        self.gml_geometries
            .first()
            .and_then(|feature| feature.polygons.first())
            .and_then(|poly| poly.exterior().0.first())
            .map_or(0.0, |p| p.z)
    }

    pub fn is_elevation_zero(&self) -> bool {
        self.gml_geometries
            .iter()
            .all(|feature| feature.polygons.iter().all(|poly| poly.is_elevation_zero()))
    }

    pub fn max_min_vertice(&self) -> MaxMinVertice {
        let mut max_min = MaxMinVertice::default();
        for gml_geometry in &self.gml_geometries {
            for polygon in &gml_geometry.polygons {
                for line in &polygon.rings() {
                    for point in line {
                        if point.x < max_min.min_lng {
                            max_min.min_lng = point.x;
                        }
                        if point.x > max_min.max_lng {
                            max_min.max_lng = point.x;
                        }
                        if point.y < max_min.min_lat {
                            max_min.min_lat = point.y;
                        }
                        if point.y > max_min.max_lat {
                            max_min.max_lat = point.y;
                        }
                        if point.z < max_min.min_height {
                            max_min.min_height = point.z;
                        }
                        if point.z > max_min.max_height {
                            max_min.max_height = point.z;
                        }
                    }
                }
            }
        }
        max_min
    }

    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.gml_geometries
            .iter_mut()
            .for_each(|feature| feature.transform_inplace(jgd2wgs));
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.gml_geometries
            .iter_mut()
            .for_each(|feature| feature.transform_offset(x, y, z));
    }
}

#[derive(Clone, Debug)]
pub struct MaxMinVertice {
    pub min_lng: f64,
    pub max_lng: f64,
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_height: f64,
    pub max_height: f64,
}

impl Default for MaxMinVertice {
    fn default() -> Self {
        Self {
            min_lng: f64::MAX,
            max_lng: f64::MIN,
            min_lat: f64::MAX,
            max_lat: f64::MIN,
            min_height: f64::MAX,
            max_height: f64::MIN,
        }
    }
}

impl From<CityGmlGeometry> for FlowGeometry2D {
    fn from(geometry: CityGmlGeometry) -> Self {
        let mut polygons = Vec::<Polygon2D<f64>>::new();
        for gml_geometry in geometry.gml_geometries {
            for polygon in gml_geometry.polygons {
                polygons.push(polygon.into());
            }
        }
        Self::MultiPolygon(MultiPolygon2D::from(polygons))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GmlGeometry {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub ty: GeometryType,
    pub lod: Option<u8>,
    pub pos: u32,
    pub len: u32,
    pub polygons: Vec<Polygon3D<f64>>,
    pub line_strings: Vec<LineString3D<f64>>,
    pub feature_id: Option<String>,
    pub feature_type: Option<String>,
    pub composite_surfaces: Vec<GmlGeometry>,
}

impl GmlGeometry {
    pub fn name(&self) -> &str {
        self.ty.name()
    }

    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.polygons
            .iter_mut()
            .for_each(|poly| poly.transform_inplace(jgd2wgs));
        self.line_strings
            .iter_mut()
            .for_each(|line| line.transform_inplace(jgd2wgs));
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.polygons
            .iter_mut()
            .for_each(|poly| poly.transform_offset(x, y, z));
        self.line_strings
            .iter_mut()
            .for_each(|line| line.transform_offset(x, y, z));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GeometryType {
    /// Polygons (solids)
    Solid,
    /// Polygons (surfaces)
    Surface,
    /// Polygons (triangles)
    Triangle,
    /// Line-strings
    Curve,
    /// Points
    Point,
}

impl From<nusamai_citygml::geometry::GeometryType> for GeometryType {
    fn from(ty: nusamai_citygml::geometry::GeometryType) -> Self {
        match ty {
            nusamai_citygml::geometry::GeometryType::Solid => Self::Solid,
            nusamai_citygml::geometry::GeometryType::Surface => Self::Surface,
            nusamai_citygml::geometry::GeometryType::Triangle => Self::Triangle,
            nusamai_citygml::geometry::GeometryType::Curve => Self::Curve,
            nusamai_citygml::geometry::GeometryType::Point => Self::Point,
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
            reearth_flow_geometry::types::geometry::Geometry::MultiPoint(_) => Self::Point,
            reearth_flow_geometry::types::geometry::Geometry::Point(_) => Self::Point,
            reearth_flow_geometry::types::geometry::Geometry::Line(_) => Self::Curve,
            reearth_flow_geometry::types::geometry::Geometry::LineString(_) => Self::Curve,
            reearth_flow_geometry::types::geometry::Geometry::Polygon(_) => Self::Surface,
            reearth_flow_geometry::types::geometry::Geometry::MultiLineString(_) => Self::Curve,
            reearth_flow_geometry::types::geometry::Geometry::MultiPolygon(_) => Self::Surface,
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
            Self::Surface => "Surface",
            Self::Triangle => "Triangle",
            Self::Curve => "Curve",
            Self::Point => "Point",
        }
    }
}

impl Display for GmlGeometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = format!("lod{}{:?}", self.lod.unwrap_or_default(), self.ty);
        write!(f, "{msg}")
    }
}

impl From<nusamai_citygml::geometry::GeometryRef> for GmlGeometry {
    fn from(geometry: nusamai_citygml::geometry::GeometryRef) -> Self {
        let id = geometry.id.map(|id| id.value());
        Self {
            id,
            ty: geometry.ty.into(),
            lod: Some(geometry.lod),
            pos: geometry.pos,
            len: geometry.len,
            polygons: Vec::new(),
            line_strings: Vec::new(),
            feature_id: geometry.feature_id,
            feature_type: geometry.feature_type,
            composite_surfaces: Vec::new(),
        }
    }
}
