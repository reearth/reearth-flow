use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use tracing::warn;

use super::converter::{
    format_pos_list, BoundingEnvelope, CityObjectType, GeometryEntry, GmlElement, GmlSurface,
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
];

const PLATEAU_NAMESPACES: &[(&str, &str)] = &[
    ("xmlns:uro", "https://www.geospatial.jp/iur/uro/3.1"),
    ("xmlns:urf", "https://www.geospatial.jp/iur/urf/3.1"),
];

pub struct CityGmlXmlWriter<W: Write> {
    writer: Writer<W>,
    srs_name: String,
    include_plateau: bool,
    codelist_path: String,
}

impl<W: Write> CityGmlXmlWriter<W> {
    pub fn new(
        inner: W,
        pretty: bool,
        srs_name: String,
        include_plateau: bool,
        codelist_path: String,
    ) -> Self {
        let writer = if pretty {
            Writer::new_with_indent(inner, b' ', 2)
        } else {
            Writer::new(inner)
        };
        Self {
            writer,
            srs_name,
            include_plateau,
            codelist_path,
        }
    }

    pub fn write_header(&mut self, envelope: Option<&BoundingEnvelope>) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let mut city_model = BytesStart::new("core:CityModel");
        for (prefix, uri) in CITYGML_2_NAMESPACES {
            city_model.push_attribute((*prefix, *uri));
        }
        if self.include_plateau {
            for (prefix, uri) in PLATEAU_NAMESPACES {
                city_model.push_attribute((*prefix, *uri));
            }
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
        feature: &Feature,
        city_type: CityObjectType,
        geometries: &[GeometryEntry],
        write_attributes: bool,
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("core:cityObjectMember")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        let element_name = city_type.element_name();
        let mut obj_elem = BytesStart::new(element_name);
        if let Some(gml_id) = &feature.metadata.feature_id {
            obj_elem.push_attribute(("gml:id", gml_id.as_str()));
        }
        self.writer
            .write_event(Event::Start(obj_elem))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        if write_attributes {
            self.write_attributes(feature, city_type)?;
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

    fn write_attributes(
        &mut self,
        feature: &Feature,
        city_type: CityObjectType,
    ) -> Result<(), SinkError> {
        let ns = city_type.namespace_prefix();

        // Get cityGmlAttributes from feature
        let citygml_attrs = feature.attributes.get(&Attribute::new("cityGmlAttributes"));

        if let Some(AttributeValue::Map(attrs)) = citygml_attrs {
            // Write standard CityGML attributes first
            self.write_standard_citygml_attributes(ns, attrs)?;
            // Write PLATEAU extension attributes
            self.write_plateau_extension_attributes(attrs)?;
        }

        Ok(())
    }

    fn write_standard_citygml_attributes(
        &mut self,
        ns: &str,
        attrs: &std::collections::HashMap<String, AttributeValue>,
    ) -> Result<(), SinkError> {
        // Standard CityGML building attributes in order
        let standard_attrs = [
            ("creationDate", "core:creationDate"),
            ("class", &format!("{}:class", ns)),
            ("class_code", ""),
            ("function", &format!("{}:function", ns)),
            ("function_code", ""),
            ("usage", &format!("{}:usage", ns)),
            ("usage_code", ""),
            ("yearOfConstruction", &format!("{}:yearOfConstruction", ns)),
            ("yearOfDemolition", &format!("{}:yearOfDemolition", ns)),
            ("roofType", &format!("{}:roofType", ns)),
            ("roofType_code", ""),
            ("measuredHeight", &format!("{}:measuredHeight", ns)),
            ("storeysAboveGround", &format!("{}:storeysAboveGround", ns)),
            ("storeysBelowGround", &format!("{}:storeysBelowGround", ns)),
        ];

        for (attr_key, tag) in standard_attrs {
            if tag.is_empty() {
                continue; // Skip _code attributes
            }
            if let Some(value) = attrs.get(attr_key) {
                self.write_citygml_attribute_value(tag, attr_key, value, attrs)?;
            }
        }

        Ok(())
    }

    fn write_citygml_attribute_value(
        &mut self,
        tag: &str,
        attr_key: &str,
        value: &AttributeValue,
        attrs: &std::collections::HashMap<String, AttributeValue>,
    ) -> Result<(), SinkError> {
        let text = match value {
            AttributeValue::String(s) => s.clone(),
            AttributeValue::Number(n) => n.to_string(),
            AttributeValue::Bool(b) => b.to_string(),
            _ => return Ok(()),
        };

        // Check if there's a corresponding code value for Code types
        let code_key = format!("{}_code", attr_key);
        if let Some(AttributeValue::String(code)) = attrs.get(&code_key) {
            // Write with codeSpace attribute
            let mut elem = BytesStart::new(tag);
            elem.push_attribute((
                "codeSpace",
                format!("{}{}.xml", self.codelist_path, attr_key).as_str(),
            ));
            self.writer
                .write_event(Event::Start(elem))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::Text(BytesText::new(code)))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::End(BytesEnd::new(tag)))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        } else if attr_key == "measuredHeight" {
            // Write with uom attribute for measurements
            let mut elem = BytesStart::new(tag);
            elem.push_attribute(("uom", "m"));
            self.writer
                .write_event(Event::Start(elem))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::Text(BytesText::new(&text)))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::End(BytesEnd::new(tag)))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        } else {
            self.write_text_element(tag, &text)?;
        }

        Ok(())
    }

    fn write_plateau_extension_attributes(
        &mut self,
        attrs: &std::collections::HashMap<String, AttributeValue>,
    ) -> Result<(), SinkError> {
        // Sort keys for deterministic output
        let mut keys: Vec<_> = attrs
            .keys()
            .filter(|k| k.starts_with("uro:") || k.starts_with("urf:"))
            .collect();
        keys.sort();

        for key in keys {
            if let Some(value) = attrs.get(key) {
                self.write_plateau_attribute(key, value)?;
            }
        }
        Ok(())
    }

    fn write_plateau_attribute(
        &mut self,
        key: &str,
        value: &AttributeValue,
    ) -> Result<(), SinkError> {
        // key is like "uro:BuildingIDAttribute" containing Array of Map
        if let AttributeValue::Array(arr) = value {
            for item in arr {
                if let AttributeValue::Map(map) = item {
                    // Write wrapper element: uro:buildingIDAttribute
                    let wrapper_name = to_attribute_property_name(key);
                    self.writer
                        .write_event(Event::Start(BytesStart::new(&wrapper_name)))
                        .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                    // Write type element: uro:BuildingIDAttribute
                    self.writer
                        .write_event(Event::Start(BytesStart::new(key)))
                        .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                    let ns = key.split(':').next().unwrap_or("uro");
                    // Sort keys for deterministic output
                    let mut child_keys: Vec<_> = map
                        .keys()
                        .filter(|k| !k.ends_with("_uom") && !k.ends_with("_code"))
                        .collect();
                    child_keys.sort();

                    for attr_key in child_keys {
                        if let Some(attr_val) = map.get(attr_key) {
                            self.write_plateau_child_attribute(ns, attr_key, attr_val, map)?;
                        }
                    }

                    self.writer
                        .write_event(Event::End(BytesEnd::new(key)))
                        .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                    self.writer
                        .write_event(Event::End(BytesEnd::new(&wrapper_name)))
                        .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
                }
            }
        }
        Ok(())
    }

    fn write_plateau_child_attribute(
        &mut self,
        ns: &str,
        attr_key: &str,
        value: &AttributeValue,
        all_attrs: &std::collections::HashMap<String, AttributeValue>,
    ) -> Result<(), SinkError> {
        // attr_key may or may not have namespace prefix
        let tag = if attr_key.contains(':') {
            attr_key.to_string()
        } else {
            format!("{}:{}", ns, attr_key)
        };
        let bare_key = attr_key.split(':').next_back().unwrap_or(attr_key);

        let text = match value {
            AttributeValue::String(s) => s.clone(),
            AttributeValue::Number(n) => n.to_string(),
            AttributeValue::Bool(b) => b.to_string(),
            _ => return Ok(()),
        };

        // Check for codeSpace (_code) and uom (_uom)
        let code_key = format!("{}_code", attr_key);
        let uom_key = format!("{}_uom", attr_key);

        if let Some(AttributeValue::String(code)) = all_attrs.get(&code_key) {
            // Write with codeSpace
            let (codelist, is_fallback) = map_attribute_to_codelist(bare_key);
            if is_fallback {
                warn!(
                    "Using fallback codelist pattern for attribute '{}': {}",
                    bare_key, codelist
                );
            }
            let mut elem = BytesStart::new(&tag);
            elem.push_attribute((
                "codeSpace",
                format!("{}{}.xml", self.codelist_path, codelist).as_str(),
            ));
            self.writer
                .write_event(Event::Start(elem))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::Text(BytesText::new(code)))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::End(BytesEnd::new(&tag)))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        } else if let Some(AttributeValue::String(uom)) = all_attrs.get(&uom_key) {
            // Write with uom
            let mut elem = BytesStart::new(&tag);
            elem.push_attribute(("uom", uom.as_str()));
            self.writer
                .write_event(Event::Start(elem))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::Text(BytesText::new(&text)))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            self.writer
                .write_event(Event::End(BytesEnd::new(&tag)))
                .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        } else {
            self.write_text_element(&tag, &text)?;
        }

        Ok(())
    }

    fn write_lod_geometry(
        &mut self,
        city_type: CityObjectType,
        entry: &GeometryEntry,
    ) -> Result<(), SinkError> {
        let ns = city_type.namespace_prefix();
        let lod_elem = self.get_geometry_element_name(ns, entry);

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

    fn get_geometry_element_name(&self, ns: &str, entry: &GeometryEntry) -> String {
        if let Some(property) = &entry.property {
            format!("{}:{}", ns, property)
        } else {
            let geom_type = match &entry.element {
                GmlElement::Solid { .. } => "Solid",
                GmlElement::MultiSurface { .. } => "MultiSurface",
                GmlElement::MultiCurve { .. } => "MultiCurve",
            };
            format!("{}:lod{}{}", ns, entry.lod, geom_type)
        }
    }

    fn write_solid(&mut self, id: Option<&str>, surfaces: &[GmlSurface]) -> Result<(), SinkError> {
        let mut solid = BytesStart::new("gml:Solid");
        if let Some(gml_id) = id {
            solid.push_attribute(("gml:id", gml_id));
        }
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
}

fn to_attribute_property_name(type_name: &str) -> String {
    // "uro:BuildingIDAttribute" -> "uro:buildingIDAttribute"
    let parts: Vec<&str> = type_name.split(':').collect();
    if parts.len() == 2 {
        let ns = parts[0];
        let name = parts[1];
        if name.is_empty() {
            return type_name.to_string();
        }
        let lowered = name
            .chars()
            .next()
            .map(|c| c.to_lowercase().to_string())
            .unwrap_or_default()
            + &name[1..];
        format!("{}:{}", ns, lowered)
    } else {
        type_name.to_string()
    }
}

/// Maps PLATEAU attribute names to codelist file names.
/// Returns (codelist_name, is_fallback) where is_fallback indicates if a generic pattern was used.
fn map_attribute_to_codelist(attr_key: &str) -> (String, bool) {
    match attr_key {
        "buildingStructureType" => (
            "BuildingDetailAttribute_buildingStructureType".to_string(),
            false,
        ),
        "fireproofStructureType" => (
            "BuildingDetailAttribute_fireproofStructureType".to_string(),
            false,
        ),
        "detailedUsage" => ("BuildingDetailAttribute_detailedUsage".to_string(), false),
        "landUseType" => ("Common_landUseType".to_string(), false),
        "prefecture" | "city" => ("Common_localPublicAuthorities".to_string(), false),
        "geometrySrcDescLod0" | "geometrySrcDescLod1" | "geometrySrcDescLod2" => {
            ("DataQualityAttribute_geometrySrcDesc".to_string(), false)
        }
        "thematicSrcDesc" => ("DataQualityAttribute_thematicSrcDesc".to_string(), false),
        "lod1HeightType" => ("DataQualityAttribute_lod1HeightType".to_string(), false),
        "srcScaleLod0" | "srcScaleLod1" => (
            "PublicSurveyDataQualityAttribute_srcScale".to_string(),
            false,
        ),
        "publicSurveySrcDescLod0" | "publicSurveySrcDescLod1" => (
            "PublicSurveyDataQualityAttribute_publicSurveySrcDesc".to_string(),
            false,
        ),
        _ => (format!("{}_codelist", attr_key), true),
    }
}
