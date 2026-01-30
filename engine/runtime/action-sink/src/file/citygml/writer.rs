use std::collections::HashMap;
use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

use super::converter::{
    format_pos_list, BoundingEnvelope, CityObjectType, GeometryEntry, GmlElement, GmlSurface,
};
use crate::errors::SinkError;
use reearth_flow_types::attribute::AttributeValue;

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
        feature_attributes: Option<&HashMap<String, AttributeValue>>,
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

        // Write feature attributes first (like creationDate, class, usage, etc.)
        if let Some(attrs) = feature_attributes {
            self.write_feature_attributes(attrs)?;
        }

        // Then write geometries
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

    fn write_feature_attributes(
        &mut self,
        attributes: &HashMap<String, AttributeValue>,
    ) -> Result<(), SinkError> {
        for (key, value) in attributes {
            // Handle special attributes like creationDate, class, usage, etc.
            match key.as_str() {
                "cityGmlAttributes" => {
                    if let AttributeValue::Map(ref attr_map) = value {
                        self.write_citygml_attributes(attr_map)?;
                    }
                }
                "gmlName" => {
                    // Skip gmlName as it's handled via feature type
                }
                "gmlId" => {
                    // Skip gmlId as it's handled separately
                }
                "gmlRootId" => {
                    // Skip gmlRootId as it's not needed in output
                }
                _ => {
                    // Handle other attributes as needed
                    self.write_generic_attribute(key, value)?;
                }
            }
        }
        Ok(())
    }

    fn write_citygml_attributes(
        &mut self,
        attributes: &HashMap<String, AttributeValue>,
    ) -> Result<(), SinkError> {
        for (key, value) in attributes {
            match key.as_str() {
                // Core attributes
                "core:creationDate" => {
                    if let AttributeValue::String(date_str) = value {
                        self.write_text_element("core:creationDate", date_str)?;
                    }
                }
                // Building attributes
                "bldg:class" => {
                    if let AttributeValue::String(class_str) = value {
                        self.write_text_element_with_attribute(
                            "bldg:class",
                            class_str,
                            "codeSpace",
                            "../../codelists/Building_class.xml",
                        )?;
                    }
                }
                "bldg:usage" => {
                    if let AttributeValue::String(usage_str) = value {
                        self.write_text_element_with_attribute(
                            "bldg:usage",
                            usage_str,
                            "codeSpace",
                            "../../codelists/Building_usage.xml",
                        )?;
                    }
                }
                "bldg:measuredHeight" => {
                    if let AttributeValue::Number(height_num) = value {
                        self.write_text_element_with_attribute(
                            "bldg:measuredHeight",
                            &height_num.to_string(),
                            "uom",
                            "m",
                        )?;
                    }
                }
                "bldg:storeysAboveGround" => {
                    if let AttributeValue::Number(storeys_num) = value {
                        self.write_text_element(
                            "bldg:storeysAboveGround",
                            &storeys_num.to_string(),
                        )?;
                    }
                }
                // LOD0 and LOD1 geometry
                "bldg:lod0RoofEdge" | "bldg:lod1Solid" => {
                    // These are handled separately as geometries
                }
                // URO attributes
                "uro:bldgDataQualityAttribute" => {
                    if let AttributeValue::Map(ref quality_attr) = value {
                        self.write_bldg_data_quality_attribute(quality_attr)?;
                    }
                }
                "uro:bldgDisasterRiskAttribute" => {
                    if let AttributeValue::Array(ref risks) = value {
                        for risk in risks {
                            if let AttributeValue::Map(ref risk_map) = risk {
                                self.write_bldg_disaster_risk_attribute(risk_map)?;
                            }
                        }
                    }
                }
                "uro:buildingDetailAttribute" => {
                    if let AttributeValue::Map(ref detail_attr) = value {
                        self.write_building_detail_attribute(detail_attr)?;
                    }
                }
                "uro:buildingIDAttribute" => {
                    if let AttributeValue::Map(ref id_attr) = value {
                        self.write_building_id_attribute(id_attr)?;
                    }
                }
                _ => {
                    // Handle other attributes generically
                    self.write_generic_attribute(key, value)?;
                }
            }
        }
        Ok(())
    }

    fn write_bldg_data_quality_attribute(
        &mut self,
        quality_attr: &HashMap<String, AttributeValue>,
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new(
                "uro:bldgDataQualityAttribute",
            )))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("uro:DataQualityAttribute")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for (key, value) in quality_attr {
            match key.as_str() {
                "uro:geometrySrcDescLod0" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:geometrySrcDescLod0",
                            code,
                            "codeSpace",
                            "../../codelists/DataQualityAttribute_geometrySrcDesc.xml",
                        )?;
                    }
                }
                "uro:geometrySrcDescLod1" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:geometrySrcDescLod1",
                            code,
                            "codeSpace",
                            "../../codelists/DataQualityAttribute_geometrySrcDesc.xml",
                        )?;
                    }
                }
                "uro:geometrySrcDescLod2" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:geometrySrcDescLod2",
                            code,
                            "codeSpace",
                            "../../codelists/DataQualityAttribute_geometrySrcDesc.xml",
                        )?;
                    }
                }
                "uro:thematicSrcDesc" => {
                    if let AttributeValue::Array(ref codes) = value {
                        for code_val in codes {
                            if let AttributeValue::String(code) = code_val {
                                self.write_text_element_with_attribute(
                                    "uro:thematicSrcDesc",
                                    code,
                                    "codeSpace",
                                    "../../codelists/DataQualityAttribute_thematicSrcDesc.xml",
                                )?;
                            }
                        }
                    } else if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:thematicSrcDesc",
                            code,
                            "codeSpace",
                            "../../codelists/DataQualityAttribute_thematicSrcDesc.xml",
                        )?;
                    }
                }
                "uro:appearanceSrcDescLod2" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:appearanceSrcDescLod2",
                            code,
                            "codeSpace",
                            "../../codelists/DataQualityAttribute_appearanceSrcDesc.xml",
                        )?;
                    }
                }
                "uro:lod1HeightType" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:lod1HeightType",
                            code,
                            "codeSpace",
                            "../../codelists/DataQualityAttribute_lod1HeightType.xml",
                        )?;
                    }
                }
                "uro:publicSurveyDataQualityAttribute" => {
                    if let AttributeValue::Map(ref survey_attr) = value {
                        self.write_public_survey_data_quality_attribute(survey_attr)?;
                    }
                }
                _ => {
                    self.write_generic_attribute(key, value)?;
                }
            }
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("uro:DataQualityAttribute")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("uro:bldgDataQualityAttribute")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_public_survey_data_quality_attribute(
        &mut self,
        survey_attr: &HashMap<String, AttributeValue>,
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new(
                "uro:publicSurveyDataQualityAttribute",
            )))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.writer
            .write_event(Event::Start(BytesStart::new(
                "uro:PublicSurveyDataQualityAttribute",
            )))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for (key, value) in survey_attr {
            match key.as_str() {
                "uro:srcScaleLod0" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:srcScaleLod0",
                            code,
                            "codeSpace",
                            "../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml",
                        )?;
                    }
                }
                "uro:srcScaleLod1" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:srcScaleLod1",
                            code,
                            "codeSpace",
                            "../../codelists/PublicSurveyDataQualityAttribute_srcScale.xml",
                        )?;
                    }
                }
                "uro:publicSurveySrcDescLod0" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute("uro:publicSurveySrcDescLod0", code, "codeSpace", "../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml")?;
                    }
                }
                "uro:publicSurveySrcDescLod1" => {
                    if let AttributeValue::Array(ref codes) = value {
                        for code_val in codes {
                            if let AttributeValue::String(code) = code_val {
                                self.write_text_element_with_attribute("uro:publicSurveySrcDescLod1", code, "codeSpace", "../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml")?;
                            }
                        }
                    } else if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute("uro:publicSurveySrcDescLod1", code, "codeSpace", "../../codelists/PublicSurveyDataQualityAttribute_publicSurveySrcDesc.xml")?;
                    }
                }
                _ => {
                    self.write_generic_attribute(key, value)?;
                }
            }
        }

        self.writer
            .write_event(Event::End(BytesEnd::new(
                "uro:PublicSurveyDataQualityAttribute",
            )))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new(
                "uro:publicSurveyDataQualityAttribute",
            )))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_bldg_disaster_risk_attribute(
        &mut self,
        risk_attr: &HashMap<String, AttributeValue>,
    ) -> Result<(), SinkError> {
        // Find the type of risk attribute (RiverFloodingRiskAttribute, InlandFloodingRiskAttribute, etc.)
        for (key, value) in risk_attr {
            if key.starts_with("uro:") && key.contains("RiskAttribute") {
                self.writer
                    .write_event(Event::Start(BytesStart::new(format!(
                        "uro:{}",
                        key.replace("uro:", "")
                    ))))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                if let AttributeValue::Map(ref risk_details) = value {
                    for (detail_key, detail_value) in risk_details {
                        match detail_key.as_str() {
                            "uro:description" => {
                                if let AttributeValue::String(code) = detail_value {
                                    self.write_text_element_with_attribute("uro:description", code, "codeSpace", "../../codelists/RiverFloodingRiskAttribute_description.xml")?;
                                }
                            }
                            "uro:rank" => {
                                if let AttributeValue::String(rank) = detail_value {
                                    self.write_text_element_with_attribute(
                                        "uro:rank",
                                        rank,
                                        "codeSpace",
                                        "../../codelists/RiverFloodingRiskAttribute_rank.xml",
                                    )?;
                                }
                            }
                            "uro:depth" => {
                                if let AttributeValue::Number(depth_num) = detail_value {
                                    self.write_text_element_with_attribute(
                                        "uro:depth",
                                        &depth_num.to_string(),
                                        "uom",
                                        "m",
                                    )?;
                                }
                            }
                            "uro:adminType" => {
                                if let AttributeValue::String(code) = detail_value {
                                    self.write_text_element_with_attribute(
                                        "uro:adminType",
                                        code,
                                        "codeSpace",
                                        "../../codelists/RiverFloodingRiskAttribute_adminType.xml",
                                    )?;
                                }
                            }
                            "uro:scale" => {
                                if let AttributeValue::String(scale) = detail_value {
                                    self.write_text_element_with_attribute(
                                        "uro:scale",
                                        scale,
                                        "codeSpace",
                                        "../../codelists/RiverFloodingRiskAttribute_scale.xml",
                                    )?;
                                }
                            }
                            "uro:duration" => {
                                if let AttributeValue::Number(duration_num) = detail_value {
                                    self.write_text_element_with_attribute(
                                        "uro:duration",
                                        &duration_num.to_string(),
                                        "uom",
                                        "hour",
                                    )?;
                                }
                            }
                            _ => {
                                self.write_generic_attribute(detail_key, detail_value)?;
                            }
                        }
                    }
                }

                self.writer
                    .write_event(Event::End(BytesEnd::new(format!(
                        "uro:{}",
                        key.replace("uro:", "")
                    ))))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
        }

        Ok(())
    }

    fn write_building_detail_attribute(
        &mut self,
        detail_attr: &HashMap<String, AttributeValue>,
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("uro:buildingDetailAttribute")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("uro:BuildingDetailAttribute")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for (key, value) in detail_attr {
            match key.as_str() {
                "uro:totalFloorArea" => {
                    if let AttributeValue::Number(area_num) = value {
                        self.write_text_element_with_attribute(
                            "uro:totalFloorArea",
                            &area_num.to_string(),
                            "uom",
                            "m2",
                        )?;
                    }
                }
                "uro:buildingRoofEdgeArea" => {
                    if let AttributeValue::Number(area_num) = value {
                        self.write_text_element_with_attribute(
                            "uro:buildingRoofEdgeArea",
                            &area_num.to_string(),
                            "uom",
                            "m2",
                        )?;
                    }
                }
                "uro:buildingStructureType" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:buildingStructureType",
                            code,
                            "codeSpace",
                            "../../codelists/BuildingDetailAttribute_buildingStructureType.xml",
                        )?;
                    }
                }
                "uro:buildingStructureOrgType" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:buildingStructureOrgType",
                            code,
                            "codeSpace",
                            "../../codelists/BuildingDetailAttribute_buildingStructureOrgType.xml",
                        )?;
                    }
                }
                "uro:detailedUsage" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:detailedUsage",
                            code,
                            "codeSpace",
                            "../../codelists/BuildingDetailAttribute_detailedUsage.xml",
                        )?;
                    }
                }
                "uro:surveyYear" => {
                    if let AttributeValue::String(year) = value {
                        self.write_text_element("uro:surveyYear", year)?;
                    }
                }
                _ => {
                    self.write_generic_attribute(key, value)?;
                }
            }
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("uro:BuildingDetailAttribute")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("uro:buildingDetailAttribute")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_building_id_attribute(
        &mut self,
        id_attr: &HashMap<String, AttributeValue>,
    ) -> Result<(), SinkError> {
        self.writer
            .write_event(Event::Start(BytesStart::new("uro:buildingIDAttribute")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        self.writer
            .write_event(Event::Start(BytesStart::new("uro:BuildingIDAttribute")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        for (key, value) in id_attr {
            match key.as_str() {
                "uro:buildingID" => {
                    if let AttributeValue::String(id) = value {
                        self.write_text_element("uro:buildingID", id)?;
                    }
                }
                "uro:prefecture" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:prefecture",
                            code,
                            "codeSpace",
                            "../../codelists/Common_localPublicAuthorities.xml",
                        )?;
                    }
                }
                "uro:city" => {
                    if let AttributeValue::String(code) = value {
                        self.write_text_element_with_attribute(
                            "uro:city",
                            code,
                            "codeSpace",
                            "../../codelists/Common_localPublicAuthorities.xml",
                        )?;
                    }
                }
                _ => {
                    self.write_generic_attribute(key, value)?;
                }
            }
        }

        self.writer
            .write_event(Event::End(BytesEnd::new("uro:BuildingIDAttribute")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new("uro:buildingIDAttribute")))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

        Ok(())
    }

    fn write_text_element_with_attribute(
        &mut self,
        tag: &str,
        text: &str,
        attr_name: &str,
        attr_value: &str,
    ) -> Result<(), SinkError> {
        let mut elem = BytesStart::new(tag);
        elem.push_attribute((attr_name, attr_value));
        self.writer
            .write_event(Event::Start(elem))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::Text(BytesText::new(text)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        self.writer
            .write_event(Event::End(BytesEnd::new(tag)))
            .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
        Ok(())
    }

    fn write_generic_attribute(
        &mut self,
        key: &str,
        value: &AttributeValue,
    ) -> Result<(), SinkError> {
        match value {
            AttributeValue::String(s) => {
                self.write_text_element(key, s)?;
            }
            AttributeValue::Number(n) => {
                self.write_text_element(key, &n.to_string())?;
            }
            AttributeValue::Bool(b) => {
                self.write_text_element(key, &b.to_string())?;
            }
            AttributeValue::Array(arr) => {
                for item in arr {
                    self.write_generic_attribute(key, item)?;
                }
            }
            AttributeValue::Map(map) => {
                // For nested maps, we'll write them as nested elements
                self.writer
                    .write_event(Event::Start(BytesStart::new(key)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;

                for (sub_key, sub_value) in map {
                    self.write_generic_attribute(sub_key, sub_value)?;
                }

                self.writer
                    .write_event(Event::End(BytesEnd::new(key)))
                    .map_err(|e| SinkError::CityGmlWriter(e.to_string()))?;
            }
            _ => {
                // For other types, convert to string representation
                self.write_text_element(key, &value.to_string())?;
            }
        }
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
}
