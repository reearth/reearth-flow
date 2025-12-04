use nusamai_citygml::PropertyType;
use reearth_flow_geometry::types::coordinate::Coordinate3D;
use reearth_flow_geometry::types::polygon::Polygon3D;
use reearth_flow_types::geometry::{CityGmlGeometry, GeometryType, GmlGeometry};
use reearth_flow_types::lod::LodMask;

#[derive(Debug, Clone)]
pub struct GeometryEntry {
    pub lod: u8,
    pub property: Option<PropertyType>,
    pub element: GmlElement,
}

#[derive(Debug, Clone)]
pub enum GmlElement {
    Solid {
        id: Option<String>,
        surfaces: Vec<GmlSurface>,
    },
    MultiSurface {
        id: Option<String>,
        surfaces: Vec<GmlSurface>,
    },
    MultiCurve {
        id: Option<String>,
        curves: Vec<Vec<Coordinate3D<f64>>>,
    },
}

#[derive(Debug, Clone)]
pub struct GmlSurface {
    pub id: Option<String>,
    pub exterior: Vec<Coordinate3D<f64>>,
    pub interiors: Vec<Vec<Coordinate3D<f64>>>,
}

impl From<&Polygon3D<f64>> for GmlSurface {
    fn from(polygon: &Polygon3D<f64>) -> Self {
        Self {
            id: None,
            exterior: polygon.exterior().0.clone(),
            interiors: polygon
                .interiors()
                .iter()
                .map(|ring| ring.0.clone())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CityObjectType {
    Building,
    BuildingPart,
    Road,
    Railway,
    Track,
    Square,
    Bridge,
    BridgePart,
    Tunnel,
    TunnelPart,
    WaterBody,
    LandUse,
    SolitaryVegetationObject,
    PlantCover,
    CityFurniture,
    ReliefFeature,
    GenericCityObject,
}

impl CityObjectType {
    pub fn from_feature_type(feature_type: &str) -> Self {
        let normalized = feature_type.to_lowercase();
        if normalized.contains("buildingpart") {
            Self::BuildingPart
        } else if normalized.contains("building") {
            Self::Building
        } else if normalized.contains("railway") {
            Self::Railway
        } else if normalized.contains("road") {
            Self::Road
        } else if normalized.contains("track") {
            Self::Track
        } else if normalized.contains("square") {
            Self::Square
        } else if normalized.contains("bridgepart") {
            Self::BridgePart
        } else if normalized.contains("bridge") {
            Self::Bridge
        } else if normalized.contains("tunnelpart") {
            Self::TunnelPart
        } else if normalized.contains("tunnel") {
            Self::Tunnel
        } else if normalized.contains("waterbody") {
            Self::WaterBody
        } else if normalized.contains("landuse") {
            Self::LandUse
        } else if normalized.contains("solitaryvegetationobject") {
            Self::SolitaryVegetationObject
        } else if normalized.contains("plantcover") {
            Self::PlantCover
        } else if normalized.contains("cityfurniture") {
            Self::CityFurniture
        } else if normalized.contains("relieffeature") {
            Self::ReliefFeature
        } else {
            Self::GenericCityObject
        }
    }

    pub fn element_name(&self) -> &'static str {
        match self {
            Self::Building => "bldg:Building",
            Self::BuildingPart => "bldg:BuildingPart",
            Self::Road => "tran:Road",
            Self::Railway => "tran:Railway",
            Self::Track => "tran:Track",
            Self::Square => "tran:Square",
            Self::Bridge => "brid:Bridge",
            Self::BridgePart => "brid:BridgePart",
            Self::Tunnel => "tun:Tunnel",
            Self::TunnelPart => "tun:TunnelPart",
            Self::WaterBody => "wtr:WaterBody",
            Self::LandUse => "luse:LandUse",
            Self::SolitaryVegetationObject => "veg:SolitaryVegetationObject",
            Self::PlantCover => "veg:PlantCover",
            Self::CityFurniture => "frn:CityFurniture",
            Self::ReliefFeature => "dem:ReliefFeature",
            Self::GenericCityObject => "gen:GenericCityObject",
        }
    }

    pub fn namespace_prefix(&self) -> &'static str {
        match self {
            Self::Building | Self::BuildingPart => "bldg",
            Self::Road | Self::Railway | Self::Track | Self::Square => "tran",
            Self::Bridge | Self::BridgePart => "brid",
            Self::Tunnel | Self::TunnelPart => "tun",
            Self::WaterBody => "wtr",
            Self::LandUse => "luse",
            Self::SolitaryVegetationObject | Self::PlantCover => "veg",
            Self::CityFurniture => "frn",
            Self::ReliefFeature => "dem",
            Self::GenericCityObject => "gen",
        }
    }
}

pub fn convert_citygml_geometry(
    geometry: &CityGmlGeometry,
    lod_filter: &LodMask,
) -> Vec<GeometryEntry> {
    geometry
        .gml_geometries
        .iter()
        .filter_map(|gml_geom| {
            let lod = gml_geom.lod.unwrap_or(0);
            if !lod_filter.has_lod(lod) {
                return None;
            }
            let property = gml_geom.gml_trait.as_ref().map(|t| t.property);
            convert_gml_geometry(gml_geom).map(|elem| GeometryEntry {
                lod,
                property,
                element: elem,
            })
        })
        .collect()
}

fn convert_gml_geometry(gml_geom: &GmlGeometry) -> Option<GmlElement> {
    match gml_geom.ty {
        GeometryType::Solid => {
            if gml_geom.polygons.is_empty() {
                return None;
            }
            Some(GmlElement::Solid {
                id: gml_geom.id.clone(),
                surfaces: gml_geom.polygons.iter().map(GmlSurface::from).collect(),
            })
        }
        GeometryType::Surface | GeometryType::Triangle => {
            if gml_geom.polygons.is_empty() {
                return None;
            }
            Some(GmlElement::MultiSurface {
                id: gml_geom.id.clone(),
                surfaces: gml_geom.polygons.iter().map(GmlSurface::from).collect(),
            })
        }
        GeometryType::Curve => {
            if gml_geom.line_strings.is_empty() {
                return None;
            }
            Some(GmlElement::MultiCurve {
                id: gml_geom.id.clone(),
                curves: gml_geom
                    .line_strings
                    .iter()
                    .map(|ls| ls.0.clone())
                    .collect(),
            })
        }
        GeometryType::Point => None,
    }
}

pub fn format_pos_list(coords: &[Coordinate3D<f64>]) -> String {
    coords
        .iter()
        .map(|c| format!("{} {} {}", c.y, c.x, c.z))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn compute_envelope(geometry: &CityGmlGeometry) -> Option<BoundingEnvelope> {
    let vertices = geometry.get_vertices();
    if vertices.is_empty() {
        return None;
    }

    let (mut min_x, mut max_x) = (f64::MAX, f64::MIN);
    let (mut min_y, mut max_y) = (f64::MAX, f64::MIN);
    let (mut min_z, mut max_z) = (f64::MAX, f64::MIN);

    for v in &vertices {
        min_x = min_x.min(v.x);
        max_x = max_x.max(v.x);
        min_y = min_y.min(v.y);
        max_y = max_y.max(v.y);
        min_z = min_z.min(v.z);
        max_z = max_z.max(v.z);
    }

    Some(BoundingEnvelope {
        lower: Coordinate3D::new__(min_x, min_y, min_z),
        upper: Coordinate3D::new__(max_x, max_y, max_z),
    })
}

#[derive(Debug, Clone)]
pub struct BoundingEnvelope {
    pub lower: Coordinate3D<f64>,
    pub upper: Coordinate3D<f64>,
}

impl BoundingEnvelope {
    pub fn merge(&mut self, other: &BoundingEnvelope) {
        self.lower.x = self.lower.x.min(other.lower.x);
        self.lower.y = self.lower.y.min(other.lower.y);
        self.lower.z = self.lower.z.min(other.lower.z);
        self.upper.x = self.upper.x.max(other.upper.x);
        self.upper.y = self.upper.y.max(other.upper.y);
        self.upper.z = self.upper.z.max(other.upper.z);
    }

    pub fn lower_corner_str(&self) -> String {
        format!("{} {} {}", self.lower.y, self.lower.x, self.lower.z)
    }

    pub fn upper_corner_str(&self) -> String {
        format!("{} {} {}", self.upper.y, self.upper.x, self.upper.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::types::line_string::LineString3D;

    #[test]
    fn test_city_object_type_from_feature_type() {
        assert_eq!(
            CityObjectType::from_feature_type("bldg:Building"),
            CityObjectType::Building
        );
        assert_eq!(
            CityObjectType::from_feature_type("tran:Road"),
            CityObjectType::Road
        );
    }

    #[test]
    fn test_format_pos_list() {
        let coords = vec![
            Coordinate3D::new__(135.0, 35.0, 10.0),
            Coordinate3D::new__(135.1, 35.1, 11.0),
        ];
        let result = format_pos_list(&coords);
        assert_eq!(result, "35 135 10 35.1 135.1 11");
    }

    #[test]
    fn test_gml_surface_from_polygon() {
        let exterior = LineString3D::new(vec![
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, 0.0, 0.0),
        ]);
        let polygon = Polygon3D::new(exterior, vec![]);
        let surface = GmlSurface::from(&polygon);
        assert_eq!(surface.exterior.len(), 4);
        assert!(surface.interiors.is_empty());
    }
}
