use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

use super::converter::{
    format_pos_list, AppearanceData, BoundingEnvelope, CityGmlAttribute, CityObjectType,
    GeometryEntry, GmlElement, GmlSurface, TargetData, TextureData,
};
use crate::errors::SinkError;

const CITYGML_2_NAMESPACES: &[(&str, &str)] = &[
    ("xmlns:core", "http://www.opengis.net/citygml/2.0"),
    ("xmlns:gml", "http://www.opengis.net/gml"),
    ("xmlns:bldg", "http://www.opengis.net/citygml/building/2.0"),
    (
        "xmlns:tran",
        "http://www.opengis.net/citygml/transportation/2.0",
    ),
    ("xmlns:brid", "http://www.opengis.net/citygml/bridge/2.0"),
    ("xmlns:tun", "http://www.opengis.net/citygml/tunnel/2.0"),
    ("xmlns:wtr", "http://www.opengis.net/citygml/waterbody/2.0"),
    ("xmlns:luse", "http://www.opengis.net/citygml/landuse/2.0"),
    ("xmlns:veg", "http://www.opengis.net/citygml/vegetation/2.0"),
    (
        "xmlns:frn",
        "http://www.opengis.net/citygml/cityfurniture/2.0",
    ),
    ("xmlns:dem", "http://www.opengis.net/citygml/relief/2.0"),
    ("xmlns:gen", "http://www.opengis.net/citygml/generics/2.0"),
    ("xmlns:app", "http://www.opengis.net/citygml/appearance/2.0"),
    ("xmlns:xlink", "http://www.w3.org/1999/xlink"),
    ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    // PLATEAU Urban Object extension (uro)
    ("xmlns:uro", "https://www.geospatial.jp/iur/uro/3.1"),
];

pub struct CityGmlXmlWriter<W: Write> {
    writer: Writer<W>,
    srs_name: String,
    id_counter: u64,
}

impl<W: Write> CityGmlXmlWriter<W> {
    pub fn new(inner: W, pretty: bool, srs_name: String) -> Self {
        let writer = if pretty {
            Writer::new_with_indent(inner, b' ', 2)
        } else {
            Writer::new(inner)
        };
        Self {
            writer,
            srs_name,
            id_counter: 0,
        }
    }

    fn generate_gml_id(&mut self, prefix: &str) -> String {
        self.id_counter += 1;
        format!("{}_{}", prefix, self.id_counter)
    }

    pub fn write_header(&mut self, envelope: Option<&BoundingEnvelope>) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let mut city_model = BytesStart::new("core:CityModel");
        for (prefix, uri) in CITYGML_2_NAMESPACES {
            city_model.push_attribute((*prefix, *uri));
        }
        self.writer
            .write_event(Event::Start(city_model))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        if let Some(env) = envelope {
            self.write_bounded_by(env)?;
        }

        Ok(())
    }

    fn write_bounded_by(&mut self, envelope: &BoundingEnvelope) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:boundedBy")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let mut env_elem = BytesStart::new("gml:Envelope");
        env_elem.push_attribute(("srsName", self.srs_name.as_str()));
        env_elem.push_attribute(("srsDimension", "3"));
        self.writer
            .write_event(Event::Start(env_elem))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.write_text_element("gml:lowerCorner", &envelope.lower_corner_str())?;
        self.write_text_element("gml:upperCorner", &envelope.upper_corner_str())?;

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:Envelope")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("gml:boundedBy")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    pub fn write_city_object(
        &mut self,
        city_type: CityObjectType,
        geometries: &[GeometryEntry],
        gml_id: Option<&str>,
        attributes: &[CityGmlAttribute],
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("core:cityObjectMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let element_name = city_type.element_name();
        let mut city_obj_elem = BytesStart::new(element_name);
        let obj_id = gml_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.generate_gml_id(city_type.id_prefix()));
        city_obj_elem.push_attribute(("gml:id", obj_id.as_str()));
        self.writer
            .write_event(Event::Start(city_obj_elem))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Write attributes before geometry
        if !attributes.is_empty() {
            self.write_citygml_attributes(attributes)?;
        }

        for entry in geometries {
            self.write_lod_geometry(city_type, entry)?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new(element_name)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("core:cityObjectMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    /// Write CityGML attributes as XML elements
    fn write_citygml_attributes(
        &mut self,
        attributes: &[CityGmlAttribute],
    ) -> Result<(), SinkError> {
        for attr in attributes {
            // Skip geometry-related attributes that are not actual attributes
            // These include surface elements like groundSurface, roofSurface, wallSurface
            // which are part of the geometry structure, not attributes
            if Self::is_geometry_element(&attr.name) {
                continue;
            }
            // Skip nested attributes that should be inside parent elements
            // The reader flattens these, but they should not be written as top-level elements
            if Self::is_nested_attribute(&attr.name) {
                continue;
            }
            self.write_attribute_value(&attr.name, &attr.value, attr.code_space.as_deref(), attr.uom.as_deref())?;
        }
        Ok(())
    }

    /// Check if an element name is a geometry element (not an attribute)
    fn is_geometry_element(name: &str) -> bool {
        let local_name = name.split(':').next_back().unwrap_or(name).to_lowercase();
        matches!(local_name.as_str(),
            "groundsurface" | "roofsurface" | "wallsurface" | "interiorwallsurface" |
            "ceilingsurface" | "floorsurface" | "outersurface" | "closuresurface" |
            "lod0roofedge" | "lod0groundsurface"
        )
    }

    /// Check if an element should be nested inside another element (not written as top-level)
    /// These are sub-types that the reader extracts but should not be written as separate elements
    fn is_nested_attribute(name: &str) -> bool {
        let local_name = name.split(':').next_back().unwrap_or(name);
        matches!(local_name,
            "TsunamiRiskAttribute" | "InlandFloodingRiskAttribute" | 
            "HighTideRiskAttribute" | "ReservoirFloodingRiskAttribute" |
            "RiverFloodingRiskAttribute" | "LandSlideRiskAttribute"
        )
    }

    /// Mapping from type names to property names
    /// The reader stores type names but the schema requires specific property names
    fn type_name_to_property_name(name: &str) -> String {
        match name {
            // uro namespace - Building related attributes
            "uro:DataQualityAttribute" => "uro:bldgDataQualityAttribute".to_string(),
            "uro:InlandFloodingRiskAttribute" => "uro:bldgDisasterRiskAttribute".to_string(),
            // These already match
            "uro:BuildingDetailAttribute" => "uro:buildingDetailAttribute".to_string(),
            "uro:BuildingIDAttribute" => "uro:buildingIDAttribute".to_string(),
            // For other names, normalize (lowercase first letter)
            _ => Self::normalize_element_name(name),
        }
    }

    /// Convert element name to CityGML schema compliant format
    /// The reader normalizes names to start with lowercase (camelCase)
    /// e.g., "uro:BuildingDetailAttribute" -> "uro:buildingDetailAttribute"
    fn normalize_element_name(name: &str) -> String {
        if let Some(colon_pos) = name.find(':') {
            let prefix = &name[..colon_pos + 1];
            let local_name = &name[colon_pos + 1..];
            
            // Convert first character of local name to lowercase
            if let Some(first_char) = local_name.chars().next() {
                let normalized = first_char.to_lowercase().to_string() + &local_name[first_char.len_utf8()..];
                return format!("{}{}", prefix, normalized);
            }
        }
        // For names without prefix, just lowercase the first char
        if let Some(first_char) = name.chars().next() {
            return first_char.to_lowercase().to_string() + &name[first_char.len_utf8()..];
        }
        name.to_string()
    }

    /// Write a single attribute value recursively
    /// Element names are normalized to comply with CityGML schema (camelCase)
    fn write_attribute_value(
        &mut self,
        name: &str,
        value: &reearth_flow_types::AttributeValue,
        code_space: Option<&str>,
        uom: Option<&str>,
    ) -> Result<(), SinkError> {
        use reearth_flow_types::AttributeValue;
        
        // Normalize element name for CityGML schema compliance
        let normalized_name = Self::normalize_element_name(name);
        
        match value {
            AttributeValue::Null => {
                // Write empty element for null values
                let mut elem = BytesStart::new(&normalized_name);
                if let Some(cs) = code_space {
                    elem.push_attribute(("codeSpace", cs));
                }
                if let Some(u) = uom {
                    elem.push_attribute(("uom", u));
                }
                self.writer.write_event(Event::Empty(elem))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            AttributeValue::Bool(v) => {
                let mut elem = BytesStart::new(&normalized_name);
                if let Some(cs) = code_space {
                    elem.push_attribute(("codeSpace", cs));
                }
                self.writer.write_event(Event::Start(elem))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                let text = if *v { "true" } else { "false" };
                self.writer.write_event(Event::Text(BytesText::new(text)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                self.writer.write_event(Event::End(BytesEnd::new(&normalized_name)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            AttributeValue::Number(n) => {
                let mut elem = BytesStart::new(&normalized_name);
                if let Some(cs) = code_space {
                    elem.push_attribute(("codeSpace", cs));
                }
                if let Some(u) = uom {
                    elem.push_attribute(("uom", u));
                }
                self.writer.write_event(Event::Start(elem))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                self.writer.write_event(Event::Text(BytesText::new(&n.to_string())))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                self.writer.write_event(Event::End(BytesEnd::new(&normalized_name)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            AttributeValue::String(s) => {
                let mut elem = BytesStart::new(&normalized_name);
                if let Some(cs) = code_space {
                    elem.push_attribute(("codeSpace", cs));
                }
                if let Some(u) = uom {
                    elem.push_attribute(("uom", u));
                }
                self.writer.write_event(Event::Start(elem))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                self.writer.write_event(Event::Text(BytesText::new(s)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                self.writer.write_event(Event::End(BytesEnd::new(&normalized_name)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            AttributeValue::DateTime(dt) => {
                let mut elem = BytesStart::new(&normalized_name);
                if let Some(cs) = code_space {
                    elem.push_attribute(("codeSpace", cs));
                }
                self.writer.write_event(Event::Start(elem))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                let formatted = dt.to_rfc3339();
                self.writer.write_event(Event::Text(BytesText::new(&formatted)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                self.writer.write_event(Event::End(BytesEnd::new(&normalized_name)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            AttributeValue::Array(arr) => {
                // For arrays of complex types (Maps), we need two levels of wrapping:
                // 1. Property element (e.g., uro:bldgDataQualityAttribute)
                // 2. Type wrapper element (e.g., uro:DataQualityAttribute)
                if arr.len() == 1 && matches!(&arr[0], AttributeValue::Map(_)) {
                    // Get property name from type name mapping
                    // The input 'name' is the type name from the reader (e.g., uro:BuildingDetailAttribute)
                    let property_name = Self::type_name_to_property_name(name);
                    
                    // Write property element
                    self.writer.write_event(Event::Start(BytesStart::new(&property_name)))
                        .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                    
                    // Type name is the original name (capitalized) from the reader
                    // e.g., uro:BuildingDetailAttribute, uro:DataQualityAttribute
                    let type_name = name.to_string();
                    
                    // Write type wrapper element
                    self.writer.write_event(Event::Start(BytesStart::new(&type_name)))
                        .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                    
                    // Write the children directly from the Map
                    // Need to combine _code values with their base elements
                    if let AttributeValue::Map(map) = &arr[0] {
                        // First pass: collect codeSpace and uom values
                        let mut child_code_spaces: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                        let mut child_uoms: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                        for (child_name, child_value) in map.iter() {
                            let child_lower = child_name.to_lowercase();
                            if child_lower.ends_with("_codespace") {
                                if let AttributeValue::String(cs) = child_value {
                                    child_code_spaces.insert(child_name[..child_name.len() - 10].to_string(), cs.clone());
                                }
                            } else if child_lower.ends_with("_uom") {
                                if let AttributeValue::String(u) = child_value {
                                    child_uoms.insert(child_name[..child_name.len() - 4].to_string(), u.clone());
                                }
                            }
                        }
                        
                        // Second pass: write base attributes (skipping _code, _codespace, _uom)
                        let mut processed_children = std::collections::HashSet::new();
                        for (child_name, child_value) in map.iter() {
                            let child_lower = child_name.to_lowercase();
                            // Skip metadata keys
                            if child_lower.ends_with("_codespace") || child_lower.ends_with("_uom") {
                                continue;
                            }
                            
                            // Get base name (without _code suffix)
                            let base_child_name = if child_name.ends_with("_code") {
                                child_name[..child_name.len() - 5].to_string()
                            } else {
                                child_name.clone()
                            };
                            
                            // Skip if already processed (when both base and _code exist)
                            if processed_children.contains(&base_child_name) {
                                continue;
                            }
                            processed_children.insert(base_child_name.clone());
                            
                            // Determine the value to use (code value if available)
                            let final_value = if child_name.ends_with("_code") {
                                child_value.clone()
                            } else {
                                let code_key = format!("{}_code", child_name);
                                if let Some(code_value) = map.get(&code_key) {
                                    code_value.clone()
                                } else {
                                    child_value.clone()
                                }
                            };
                            
                            // Get codeSpace and uom for this child
                            let child_cs = child_code_spaces.get(&base_child_name).cloned();
                            let child_u = child_uoms.get(&base_child_name).cloned();
                            
                            self.write_attribute_value(&base_child_name, &final_value, child_cs.as_deref(), child_u.as_deref())?;
                        }
                    }
                    
                    // Close type wrapper element
                    self.writer.write_event(Event::End(BytesEnd::new(&type_name)))
                        .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                    
                    // Close property element
                    self.writer.write_event(Event::End(BytesEnd::new(&property_name)))
                        .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                } else {
                    // Default: write each element with the same tag name
                    for item in arr {
                        self.write_attribute_value(name, item, code_space, uom)?;
                    }
                }
            }
            AttributeValue::Map(map) => {
                // For maps (complex types), create a wrapper element and write children
                self.writer.write_event(Event::Start(BytesStart::new(&normalized_name)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                
                // Write each child element
                for (child_name, child_value) in map {
                    // Skip metadata keys (codeSpace, uom)
                    let lower_name = child_name.to_lowercase();
                    if lower_name.ends_with("_codespace") || lower_name.ends_with("_uom") {
                        continue;
                    }
                    self.write_attribute_value(child_name, child_value, None, None)?;
                }
                
                self.writer.write_event(Event::End(BytesEnd::new(&normalized_name)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            AttributeValue::Bytes(_) => {
                // Skip binary data for now
            }
        }
        Ok(())
    }

    /// Write global appearance members at CityModel level (before city objects)
    pub fn write_global_appearances(
        &mut self,
        appearance_data: &AppearanceData,
    ) -> Result<(), SinkError> {
        let total_targets: usize = appearance_data.textures.iter().map(|t| t.targets.len()).sum();
        tracing::info!(
            "write_global_appearances: textures={}, themes={}, total_targets={}",
            appearance_data.textures.len(),
            appearance_data.themes.len(),
            total_targets
        );
        
        // Only write appearance members if there are textures
        if !appearance_data.textures.is_empty() {
            // Start appearance member
            self.writer
                .write_event(Event::Start(BytesStart::new("app:appearanceMember")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            // Start Appearance
            self.writer
                .write_event(Event::Start(BytesStart::new("app:Appearance")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            // Write theme if available
            for theme_name in &appearance_data.themes {
                self.write_text_element("app:theme", theme_name)?;
            }

            // Write each texture as a surface data member (with its own targets)
            for texture_data in &appearance_data.textures {
                self.write_surface_data_member(texture_data)?;
            }

            // Close Appearance
            self.writer
                .write_event(Event::End(BytesEnd::new("app:Appearance")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            // Close appearanceMember
            self.writer
                .write_event(Event::End(BytesEnd::new("app:appearanceMember")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        }
        Ok(())
    }

    fn write_surface_data_member(
        &mut self,
        texture_data: &TextureData,
    ) -> Result<(), SinkError> {
        tracing::debug!(
            "write_surface_data_member: texture_uri={}, targets={}",
            texture_data.uri,
            texture_data.targets.len()
        );
        
        // Extract just the directory and filename part from the full URI
        let path_parts: Vec<&str> = texture_data.uri.split('/').collect();
        let simplified_path = if path_parts.len() >= 2 {
            // Take the last two parts: directory and filename
            format!(
                "{}/{}",
                path_parts[path_parts.len() - 2],
                path_parts[path_parts.len() - 1]
            )
        } else {
            // If there are less than 2 parts, just use the original
            texture_data.uri.to_string()
        };

        // Write surface data member with ParameterizedTexture
        self.writer
            .write_event(Event::Start(BytesStart::new("app:surfaceDataMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("app:ParameterizedTexture")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Write image URI with just the directory and filename
        self.write_text_element("app:imageURI", &simplified_path)?;

        // Write mime type (assuming jpg for now, could be extracted from URI extension)
        let mime_type = if simplified_path.ends_with(".png") {
            "image/png"
        } else if simplified_path.ends_with(".gif") {
            "image/gif"
        } else {
            "image/jpg"
        };
        self.write_text_element("app:mimeType", mime_type)?;

        // Write target elements specific to this texture only
        if !texture_data.targets.is_empty() {
            tracing::debug!("write_surface_data_member: writing {} targets for this texture", texture_data.targets.len());
            for target in &texture_data.targets {
                self.write_target_element(target)?;
            }
        }

        // Close ParameterizedTexture
        self.writer
            .write_event(Event::End(BytesEnd::new("app:ParameterizedTexture")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Close surfaceDataMember
        self.writer
            .write_event(Event::End(BytesEnd::new("app:surfaceDataMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_target_element(&mut self, target: &TargetData) -> Result<(), SinkError> {
        tracing::debug!(
            "write_target_element: uri={}, coords_count={}",
            target.uri,
            target.texture_coordinates.len()
        );
        
        // Start target element with URI attribute
        let mut target_start = BytesStart::new("app:target");
        target_start.push_attribute(("uri", target.uri.as_str()));
        self.writer
            .write_event(Event::Start(target_start))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Write texture coordinate list if available
        if !target.texture_coordinates.is_empty() {
            tracing::debug!("write_target_element: writing TexCoordList with {} coordinates", target.texture_coordinates.len());
            self.writer
                .write_event(Event::Start(BytesStart::new("app:TexCoordList")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            // Join all texture coordinates into a single space-separated string
            // CityGML requires the first and last coordinate to be the same (closed loop)
            tracing::debug!("write_target_element: joining coordinates...");
            let mut coord_string = target.texture_coordinates.join(" ");
            
            // Append the first coordinate at the end to close the loop
            if let Some(first_coord) = target.texture_coordinates.first() {
                coord_string.push(' ');
                coord_string.push_str(first_coord);
            }
            
            tracing::debug!("write_target_element: joined {} bytes of coordinates", coord_string.len());

            // Use the ring field directly (which contains the ring ID, not surface ID)
            // The ring attribute must have a # prefix to be valid LocalId reference
            let ring_ref = if target.ring.starts_with('#') {
                target.ring.clone()
            } else {
                format!("#{}", target.ring)
            };
            
            let mut tex_coord = BytesStart::new("app:textureCoordinates");
            tex_coord.push_attribute(("ring", ring_ref.as_str()));
            self.writer
                .write_event(Event::Start(tex_coord))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            self.writer
                .write_event(Event::Text(BytesText::new(&coord_string)))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            self.writer
                .write_event(Event::End(BytesEnd::new("app:textureCoordinates")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            self.writer
                .write_event(Event::End(BytesEnd::new("app:TexCoordList")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        }

        // Close target element
        self.writer
            .write_event(Event::End(BytesEnd::new("app:target")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_lod_geometry(
        &mut self,
        city_type: CityObjectType,
        entry: &GeometryEntry,
    ) -> Result<(), SinkError> {
        let ns = city_type.namespace_prefix();
        let lod_elem = self.get_geometry_element_name(ns, entry, city_type);

        match &entry.element {
            GmlElement::Solid { id, surfaces } => {
                self.writer
                    .write_event(Event::Start(BytesStart::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                self.write_solid(id.as_deref(), surfaces)?;

                self.writer
                    .write_event(Event::End(BytesEnd::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            GmlElement::MultiSurface { id, surfaces } => {
                self.writer
                    .write_event(Event::Start(BytesStart::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                self.write_multi_surface(id.as_deref(), surfaces)?;

                self.writer
                    .write_event(Event::End(BytesEnd::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            GmlElement::MultiCurve { id, curves } => {
                self.writer
                    .write_event(Event::Start(BytesStart::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                self.write_multi_curve(id.as_deref(), curves)?;

                self.writer
                    .write_event(Event::End(BytesEnd::new(&lod_elem)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
        }

        Ok(())
    }

    fn get_geometry_element_name(
        &self,
        ns: &str,
        entry: &GeometryEntry,
        city_type: CityObjectType,
    ) -> String {
        if let Some(property) = &entry.property {
            format!("{}:{}", ns, property)
        } else {
            // GenericCityObject uses lodXGeometry, not lodXMultiSurface/lodXSolid
            if city_type == CityObjectType::GenericCityObject {
                format!("{}:lod{}Geometry", ns, entry.lod)
            } else {
                let geom_type = match &entry.element {
                    GmlElement::Solid { .. } => "Solid",
                    GmlElement::MultiSurface { .. } => "MultiSurface",
                    GmlElement::MultiCurve { .. } => "MultiCurve",
                };
                format!("{}:lod{}{}", ns, entry.lod, geom_type)
            }
        }
    }

    fn write_solid(&mut self, id: Option<&str>, surfaces: &[GmlSurface]) -> Result<(), SinkError> {
        let mut solid = BytesStart::new("gml:Solid");
        if let Some(gml_id) = id {
            solid.push_attribute(("gml:id", gml_id));
        }
        // Add srsName to geometry for proper CRS reference
        solid.push_attribute(("srsName", self.srs_name.as_str()));
        solid.push_attribute(("srsDimension", "3"));
        self.writer
            .write_event(Event::Start(solid))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("gml:exterior")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:CompositeSurface")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for surface in surfaces {
            self.write_surface_member(surface)?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:CompositeSurface")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("gml:exterior")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("gml:Solid")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_multi_surface(
        &mut self,
        id: Option<&str>,
        surfaces: &[GmlSurface],
    ) -> Result<(), SinkError> {
        let mut ms = BytesStart::new("gml:MultiSurface");
        if let Some(gml_id) = id {
            ms.push_attribute(("gml:id", gml_id));
        }
        // Add srsName to geometry for proper CRS reference
        ms.push_attribute(("srsName", self.srs_name.as_str()));
        ms.push_attribute(("srsDimension", "3"));
        self.writer
            .write_event(Event::Start(ms))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for surface in surfaces {
            self.write_surface_member(surface)?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:MultiSurface")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_surface_member(&mut self, surface: &GmlSurface) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:surfaceMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let mut polygon = BytesStart::new("gml:Polygon");
        if let Some(id) = &surface.id {
            polygon.push_attribute(("gml:id", id.as_str()));
        }
        self.writer
            .write_event(Event::Start(polygon))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Exterior ring
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:exterior")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.write_linear_ring(&surface.exterior)?;
        self.writer
            .write_event(Event::End(BytesEnd::new("gml:exterior")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        // Interior rings (holes)
        for interior in &surface.interiors {
            self.writer
                .write_event(Event::Start(BytesStart::new("gml:interior")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.write_linear_ring(interior)?;
            self.writer
                .write_event(Event::End(BytesEnd::new("gml:interior")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:Polygon")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("gml:surfaceMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_linear_ring(
        &mut self,
        coords: &[reearth_flow_geometry::types::coordinate::Coordinate3D<f64>],
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("gml:LinearRing")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.write_text_element("gml:posList", &format_pos_list(coords))?;

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:LinearRing")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_multi_curve(
        &mut self,
        id: Option<&str>,
        curves: &[Vec<reearth_flow_geometry::types::coordinate::Coordinate3D<f64>>],
    ) -> Result<(), SinkError> {
        let mut mc = BytesStart::new("gml:MultiCurve");
        if let Some(gml_id) = id {
            mc.push_attribute(("gml:id", gml_id));
        }
        // Add srsName to geometry for proper CRS reference
        mc.push_attribute(("srsName", self.srs_name.as_str()));
        mc.push_attribute(("srsDimension", "3"));
        self.writer
            .write_event(Event::Start(mc))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for curve in curves {
            self.writer
                .write_event(Event::Start(BytesStart::new("gml:curveMember")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::Start(BytesStart::new("gml:LineString")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

            self.write_text_element("gml:posList", &format_pos_list(curve))?;

            self.writer
                .write_event(Event::End(BytesEnd::new("gml:LineString")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::End(BytesEnd::new("gml:curveMember")))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("gml:MultiCurve")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_text_element(&mut self, tag: &str, text: &str) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new(tag)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::Text(BytesText::new(text)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new(tag)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        Ok(())
    }

    pub fn write_footer(&mut self) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::End(BytesEnd::new("core:CityModel")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        Ok(())
    }

    /// Flush the underlying writer to ensure all data is written.
    pub fn flush(&mut self) -> Result<(), SinkError> {
        self.writer
            .get_mut()
            .flush()
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))
    }
}
