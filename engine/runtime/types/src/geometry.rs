use std::fmt::Display;
use std::hash::Hash;

use nusamai_citygml::{GmlGeometryType, PropertyType};
use nusamai_projection::vshift::Jgd2011ToWgs84;
use reearth_flow_geometry::types::coordinate::Coordinate3D;
use reearth_flow_geometry::types::coordnum::CoordNum;
use reearth_flow_geometry::types::line_string::{LineString2D, LineString3D};
use reearth_flow_geometry::types::traits::Elevation;

use nusamai_projection::crs::EpsgCode;
use reearth_flow_geometry::algorithm::hole::HoleCounter;
use reearth_flow_geometry::types::multi_line_string::MultiLineString2D;
use reearth_flow_geometry::types::multi_point::MultiPoint2D;
use reearth_flow_geometry::types::point::{Point, Point2D};
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use serde::{Deserialize, Serialize};

use reearth_flow_geometry::types::geometry::Geometry2D as FlowGeometry2D;
use reearth_flow_geometry::types::geometry::Geometry3D as FlowGeometry3D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;

use crate::material::{Texture, X3DMaterial};

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
    pub polygon_uvs: MultiPolygon2D<f64>,
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
            polygon_uvs: MultiPolygon2D::default(),
        }
    }

    pub fn split_feature(&self) -> Vec<CityGmlGeometry> {
        self.gml_geometries
            .iter()
            .map(|feature| {
                let pos = feature.pos as usize;
                let len = feature.len as usize;

                // Extract only the relevant slice of polygon_materials for this geometry
                let polygon_materials = if pos + len <= self.polygon_materials.len() {
                    self.polygon_materials[pos..pos + len].to_vec()
                } else {
                    Vec::new()
                };

                // Extract only the relevant slice of polygon_textures for this geometry
                let polygon_textures = if pos + len <= self.polygon_textures.len() {
                    self.polygon_textures[pos..pos + len].to_vec()
                } else {
                    Vec::new()
                };

                // Extract only the relevant slice of polygon_uvs for this geometry
                let polygon_uvs = if pos + len <= self.polygon_uvs.0.len() {
                    MultiPolygon2D::new(self.polygon_uvs.0[pos..pos + len].to_vec())
                } else {
                    MultiPolygon2D::default()
                };

                // Clone the feature and reset pos to 0 since it's now the only geometry
                let mut cloned_feature = feature.clone();
                cloned_feature.pos = 0;

                CityGmlGeometry {
                    gml_geometries: vec![cloned_feature],
                    materials: self.materials.clone(),
                    textures: self.textures.clone(),
                    polygon_materials,
                    polygon_textures,
                    polygon_uvs,
                }
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

    /// Transforms the X/Y coordinates of all geometries using the provided function.
    /// The Z coordinate is passed through unchanged.
    /// Returns an error if any transformation fails.
    pub fn transform_horizontal<F, E>(&mut self, transform_fn: F) -> Result<(), E>
    where
        F: Fn(f64, f64) -> Result<(f64, f64), E>,
    {
        for gml_geometry in &mut self.gml_geometries {
            gml_geometry.transform_horizontal(&transform_fn)?;
        }
        Ok(())
    }

    pub fn get_vertices(&self) -> Vec<Coordinate3D<f64>> {
        let mut vertices = Vec::new();
        for gml_geometry in &self.gml_geometries {
            for polygon in &gml_geometry.polygons {
                for line in &polygon.rings() {
                    for point in line {
                        vertices.push(*point);
                    }
                }
            }
            for line_string in &gml_geometry.line_strings {
                for point in line_string {
                    vertices.push(*point);
                }
            }
        }
        vertices
    }

    /// Filters gml_geometries by a predicate and extracts corresponding polygon data
    /// with properly remapped indices.
    pub fn filter_by_lod<F>(&self, predicate: F) -> CityGmlGeometry
    where
        F: Fn(&GmlGeometry) -> bool,
    {
        let mut filtered_gml_geometries = Vec::new();
        let mut filtered_polygon_materials = Vec::new();
        let mut filtered_polygon_textures = Vec::new();
        let mut filtered_polygon_uvs = Vec::new();
        let mut new_pos: u32 = 0;

        for gml_geom in &self.gml_geometries {
            if !predicate(gml_geom) {
                continue;
            }

            let pos = gml_geom.pos as usize;
            let len = gml_geom.len as usize;

            // Verify that polygon count matches len - this is critical for data consistency
            // len represents the number of polygons in polygon_materials/textures/uvs arrays
            // polygons.len() is the actual count of Polygon3D objects
            if gml_geom.polygons.len() != len {
                tracing::warn!(
                    "Skipping geometry with mismatched counts: polygons.len()={} != len={}",
                    gml_geom.polygons.len(),
                    len
                );
                continue;
            }

            // Verify bounds before extraction - skip geometry if data is missing
            let has_materials = pos + len <= self.polygon_materials.len();
            let has_textures = pos + len <= self.polygon_textures.len();
            let has_uvs = pos + len <= self.polygon_uvs.0.len();

            // Extract polygon_materials for this geometry
            if has_materials {
                filtered_polygon_materials
                    .extend_from_slice(&self.polygon_materials[pos..pos + len]);
            } else if !self.polygon_materials.is_empty() {
                // Fill with None if source has materials but this range is out of bounds
                filtered_polygon_materials.extend(std::iter::repeat_n(None, len));
            }

            // Extract polygon_textures for this geometry
            if has_textures {
                filtered_polygon_textures.extend_from_slice(&self.polygon_textures[pos..pos + len]);
            } else if !self.polygon_textures.is_empty() {
                // Fill with None if source has textures but this range is out of bounds
                filtered_polygon_textures.extend(std::iter::repeat_n(None, len));
            }

            // Extract polygon_uvs for this geometry, verifying vertex counts match
            if has_uvs {
                // Check if UVs are compatible with polygons (vertex counts must match)
                let uvs_slice = &self.polygon_uvs.0[pos..pos + len];
                let mut all_match = true;
                for (poly, uv) in gml_geom.polygons.iter().zip(uvs_slice.iter()) {
                    let poly_ext_len = poly.exterior().0.len();
                    let uv_ext_len = uv.exterior().0.len();
                    if poly_ext_len != uv_ext_len {
                        all_match = false;
                        break;
                    }
                    // Check interiors
                    if poly.interiors().len() != uv.interiors().len() {
                        all_match = false;
                        break;
                    }
                    for (poly_int, uv_int) in poly.interiors().iter().zip(uv.interiors().iter()) {
                        if poly_int.0.len() != uv_int.0.len() {
                            all_match = false;
                            break;
                        }
                    }
                    if !all_match {
                        break;
                    }
                }

                if all_match {
                    filtered_polygon_uvs.extend(uvs_slice.iter().cloned());
                } else {
                    // UV vertex counts don't match, create new UVs matching polygon structure
                    tracing::debug!(
                        "Creating new UVs for geometry with mismatched vertex counts (lod={:?}, pos={}, len={})",
                        gml_geom.lod, pos, len
                    );
                    for poly in &gml_geom.polygons {
                        let exterior_len = poly.exterior().0.len();
                        let exterior_uv: Vec<_> =
                            std::iter::repeat_n([0.0, 0.0].into(), exterior_len).collect();
                        let interiors_uv: Vec<Vec<_>> = poly
                            .interiors()
                            .iter()
                            .map(|interior| {
                                std::iter::repeat_n([0.0, 0.0].into(), interior.0.len()).collect()
                            })
                            .collect();
                        filtered_polygon_uvs.push(Polygon2D::new(
                            LineString2D::new(exterior_uv),
                            interiors_uv.into_iter().map(LineString2D::new).collect(),
                        ));
                    }
                }
            } else if !self.polygon_uvs.0.is_empty() {
                // Create dummy UVs matching the polygon structure if source has UVs but range is out of bounds
                for poly in &gml_geom.polygons {
                    let exterior_len = poly.exterior().0.len();
                    let exterior_uv: Vec<_> =
                        std::iter::repeat_n([0.0, 0.0].into(), exterior_len).collect();
                    let interiors_uv: Vec<Vec<_>> = poly
                        .interiors()
                        .iter()
                        .map(|interior| {
                            std::iter::repeat_n([0.0, 0.0].into(), interior.0.len()).collect()
                        })
                        .collect();
                    filtered_polygon_uvs.push(Polygon2D::new(
                        LineString2D::new(exterior_uv),
                        interiors_uv.into_iter().map(LineString2D::new).collect(),
                    ));
                }
            }

            // Clone the geometry and update pos to the new position
            let mut cloned_geom = gml_geom.clone();
            cloned_geom.pos = new_pos;
            new_pos += len as u32;

            filtered_gml_geometries.push(cloned_geom);
        }

        CityGmlGeometry {
            gml_geometries: filtered_gml_geometries,
            materials: self.materials.clone(),
            textures: self.textures.clone(),
            polygon_materials: filtered_polygon_materials,
            polygon_textures: filtered_polygon_textures,
            polygon_uvs: MultiPolygon2D::new(filtered_polygon_uvs),
        }
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
        let mut line_strings = Vec::<LineString2D<f64>>::new();
        let mut points = Vec::<Point2D<f64>>::new();

        for gml_geometry in geometry.gml_geometries {
            for polygon in gml_geometry.polygons {
                polygons.push(polygon.into());
            }
            for line_string in gml_geometry.line_strings {
                line_strings.push(line_string.into());
            }
            for point in gml_geometry.points {
                // Convert Coordinate3D to Point2D: Coordinate3D -> Point3D -> Point2D
                let point_3d: Point<f64, f64> = point.into();
                points.push(point_3d.into());
            }
        }

        let has_polygons: u8 = if !polygons.is_empty() { 1 } else { 0 };
        let has_line_strings: u8 = if !line_strings.is_empty() { 1 } else { 0 };
        let has_points: u8 = if !points.is_empty() { 1 } else { 0 };
        if has_polygons + has_line_strings + has_points > 1 {
            tracing::warn!("CityGML feature contains multiple geometry types.");
        }

        // Return geometry based on what's available
        if !polygons.is_empty() {
            Self::MultiPolygon(MultiPolygon2D::from(polygons))
        } else if !line_strings.is_empty() {
            Self::MultiLineString(MultiLineString2D::new(line_strings))
        } else if !points.is_empty() {
            Self::MultiPoint(MultiPoint2D::from(points))
        } else {
            tracing::warn!("CityGML feature contains no supported geometries.");
            Self::MultiPolygon(MultiPolygon2D::from(Vec::new()))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GmlGeometry {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub ty: GeometryType,
    pub gml_trait: Option<GmlGeometryTrait>,
    pub lod: Option<u8>,
    pub pos: u32,
    pub len: u32,
    pub polygons: Vec<Polygon3D<f64>>,
    pub line_strings: Vec<LineString3D<f64>>,
    pub points: Vec<Coordinate3D<f64>>,
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
        self.points
            .iter_mut()
            .for_each(|point| point.transform_inplace(jgd2wgs));
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.polygons
            .iter_mut()
            .for_each(|poly| poly.transform_offset(x, y, z));
        self.line_strings
            .iter_mut()
            .for_each(|line| line.transform_offset(x, y, z));
        self.points
            .iter_mut()
            .for_each(|point| point.transform_offset(x, y, z));
    }

    /// Transforms the X/Y coordinates of all geometries using the provided function.
    /// The Z coordinate is passed through unchanged.
    /// Returns an error if any transformation fails.
    pub fn transform_horizontal<F, E>(&mut self, transform_fn: &F) -> Result<(), E>
    where
        F: Fn(f64, f64) -> Result<(f64, f64), E>,
    {
        for poly in &mut self.polygons {
            poly.transform_horizontal(transform_fn)?;
        }
        for line in &mut self.line_strings {
            line.transform_horizontal(transform_fn)?;
        }
        for point in &mut self.points {
            let (new_x, new_y) = transform_fn(point.x, point.y)?;
            point.x = new_x;
            point.y = new_y;
        }
        for composite in &mut self.composite_surfaces {
            composite.transform_horizontal(transform_fn)?;
        }
        Ok(())
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
            gml_trait: GmlGeometryTrait::maybe_new(
                geometry.property_name,
                geometry.gml_geometry_type,
            ),
            lod: Some(geometry.lod),
            pos: geometry.pos,
            len: geometry.len,
            polygons: Vec::new(),
            line_strings: Vec::new(),
            points: Vec::new(),
            feature_id: geometry.feature_id,
            feature_type: geometry.feature_type,
            composite_surfaces: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GmlGeometryTrait {
    pub property: PropertyType,
    pub gml_geometry_type: GmlGeometryType,
}

impl GmlGeometryTrait {
    pub fn maybe_new(
        property: Option<PropertyType>,
        gml_geometry_type: Option<GmlGeometryType>,
    ) -> Option<Self> {
        match (property, gml_geometry_type) {
            (Some(prop), Some(ty)) => Some(Self {
                property: prop,
                gml_geometry_type: ty,
            }),
            _ => None,
        }
    }
}
