use nusamai_citygml::PropertyType;
use reearth_flow_geometry::types::coordinate::Coordinate3D;
use reearth_flow_geometry::types::polygon::Polygon3D;
use reearth_flow_types::geometry::{CityGmlGeometry, GeometryType, GmlGeometry};
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::{Attribute, AttributeValue, Feature};

#[derive(Debug, Clone)]
pub struct GeometryEntry {
    pub lod: u8,
    pub property: Option<PropertyType>,
    pub element: GmlElement,
}

#[derive(Debug, Clone)]
pub struct TargetData {
    pub uri: String,                      // Surface ID (Polygon gml:id)
    pub ring: String,                     // Ring ID (LinearRing gml:id)
    pub texture_coordinates: Vec<String>, // List of texture coordinate pairs as strings
}

#[derive(Debug, Clone)]
pub struct TextureData {
    pub uri: String,
    pub targets: Vec<TargetData>, // Targets associated with this specific texture
}

#[derive(Debug, Clone)]
pub struct AppearanceData {
    pub textures: Vec<TextureData>, // Each texture with its associated targets
    pub themes: Vec<String>,
}

/// Extract appearance data from feature attributes
pub fn extract_appearance_data(feature: &Feature) -> Option<AppearanceData> {
    let appearance_attr = Attribute::new("appearanceMember");
    let appearance_member = feature.attributes.get(&appearance_attr)?;

    match appearance_member {
        AttributeValue::Map(map) => {
            let mut textures = Vec::new();
            let mut themes = Vec::new();

            // Extract textures with their associated targets
            if let Some(AttributeValue::Array(tex_array)) = map.get("textures") {
                for tex in tex_array {
                    if let AttributeValue::Map(tex_map) = tex {
                        let uri = if let Some(AttributeValue::String(uri)) = tex_map.get("uri") {
                            uri.clone()
                        } else {
                            continue; // Skip textures without URI
                        };

                        // Extract targets specific to this texture
                        let mut texture_targets = Vec::new();
                        if let Some(AttributeValue::Array(target_array)) = tex_map.get("targets") {
                            for target in target_array {
                                if let AttributeValue::Map(target_map) = target {
                                    if let Some(AttributeValue::String(target_uri)) = target_map.get("uri")
                                    {
                                        let mut texture_coords = Vec::new();

                                        // Extract texture coordinates if available
                                        if let Some(AttributeValue::Array(coord_array)) =
                                            target_map.get("textureCoordinates")
                                        {
                                            for coord in coord_array {
                                                if let AttributeValue::String(coord_str) = coord {
                                                    texture_coords.push(coord_str.clone());
                                                }
                                            }
                                        }

                                        // Extract ring ID if available (new field)
                                        let ring_id = if let Some(AttributeValue::String(ring)) = target_map.get("ring") {
                                            ring.clone()
                                        } else {
                                            // Fallback: extract from URI (backward compatibility)
                                            target_uri.clone()
                                        };

                                        texture_targets.push(TargetData {
                                            uri: target_uri.clone(),
                                            ring: ring_id,
                                            texture_coordinates: texture_coords,
                                        });
                                    }
                                }
                            }
                        }

                        textures.push(TextureData {
                            uri,
                            targets: texture_targets,
                        });
                    }
                }
            }

            // Extract themes
            if let Some(AttributeValue::Array(theme_array)) = map.get("themes") {
                for theme in theme_array {
                    if let AttributeValue::Map(theme_map) = theme {
                        if let Some(AttributeValue::String(name)) = theme_map.get("name") {
                            themes.push(name.clone());
                        }
                    }
                }
            }

            Some(AppearanceData {
                textures,
                themes,
            })
        }
        _ => None,
    }
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

    pub fn id_prefix(&self) -> &'static str {
        match self {
            Self::Building => "bldg",
            Self::BuildingPart => "bldg_part",
            Self::Road => "road",
            Self::Railway => "rail",
            Self::Track => "track",
            Self::Square => "square",
            Self::Bridge => "brid",
            Self::BridgePart => "brid_part",
            Self::Tunnel => "tun",
            Self::TunnelPart => "tun_part",
            Self::WaterBody => "wtr",
            Self::LandUse => "luse",
            Self::SolitaryVegetationObject => "veg_sol",
            Self::PlantCover => "veg_plant",
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

/// Represents a CityGML attribute entry to be written to XML
#[derive(Debug, Clone)]
pub struct CityGmlAttribute {
    /// Full name including namespace prefix (e.g., "bldg:class", "uro:buildingID")
    pub name: String,
    /// The attribute value
    pub value: AttributeValue,
    /// Optional codeSpace attribute for Code types
    pub code_space: Option<String>,
    /// Optional uom attribute for Measure types
    pub uom: Option<String>,
}

/// Extract cityGmlAttributes from feature
/// Groups related attributes (base, _code, _codeSpace, _uom) into single CityGmlAttribute entries
pub fn extract_citygml_attributes(feature: &Feature) -> Vec<CityGmlAttribute> {
    let attr_key = Attribute::new("cityGmlAttributes");
    let Some(AttributeValue::Map(attrs_map)) = feature.attributes.get(&attr_key) else {
        return Vec::new();
    };

    // First pass: collect metadata (codeSpace and uom) for base attributes
    let mut code_spaces: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut uoms: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    
    for (key, value) in attrs_map.iter() {
        let lower_key = key.to_lowercase();
        
        // Extract codeSpace values
        if lower_key.ends_with("_codespace") {
            let base_key = &key[..key.len() - 10];
            if let AttributeValue::String(cs) = value {
                code_spaces.insert(base_key.to_string(), cs.clone());
            }
        }
        // Extract uom values  
        else if lower_key.ends_with("_uom") {
            let base_key = &key[..key.len() - 4];
            if let AttributeValue::String(u) = value {
                uoms.insert(base_key.to_string(), u.clone());
            }
        }
    }
    
    // Second pass: build CityGmlAttribute entries for base attributes
    let mut attributes = Vec::new();
    let mut processed_bases = std::collections::HashSet::new();
    
    for (key, value) in attrs_map.iter() {
        let lower_key = key.to_lowercase();
        
        // Skip metadata keys - they will be attached to base attributes
        if lower_key.ends_with("_codespace") || lower_key.ends_with("_uom") {
            continue;
        }
        
        // Get the base name (without _code suffix if present)
        let (base_name, is_code_value) = if key.ends_with("_code") {
            (key[..key.len() - 5].to_string(), true)
        } else {
            (key.clone(), false)
        };
        
        // Skip if we've already processed this base (can happen if both base and _code exist)
        if processed_bases.contains(&base_name) {
            continue;
        }
        processed_bases.insert(base_name.clone());
        
        // Determine the value to use for this attribute
        let final_value = if is_code_value {
            // This is a _code key, use it directly
            value.clone()
        } else {
            // This is a base key - check if there's a _code variant
            let code_key = format!("{}_code", key);
            if let Some(code_value) = attrs_map.get(&code_key) {
                // Use the code value instead of the human-readable value
                code_value.clone()
            } else {
                // No code variant, use the base value
                value.clone()
            }
        };
        
        // Get codeSpace and uom for this base attribute
        let code_space = code_spaces.get(&base_name).cloned();
        let uom = uoms.get(&base_name).cloned();
        
        attributes.push(CityGmlAttribute {
            name: base_name,
            value: final_value,
            code_space,
            uom,
        });
    }
    
    // Sort attributes to ensure consistent output
    attributes.sort_by(|a, b| a.name.cmp(&b.name));
    
    attributes
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
