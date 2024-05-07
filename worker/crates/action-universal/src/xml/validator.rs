use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use reearth_flow_common::{
    uri::{Uri, PROTOCOL_SEPARATOR},
    xml::{self, XmlDocument, XmlNamespace},
};
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, Attribute, AttributeValue,
    Dataframe, Feature, Result, SyncAction, DEFAULT_PORT, REJECTED_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum XmlInputType {
    File,
    Text,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum ValidationType {
    Syntax,
    SyntaxAndNamespace,
    SyntaxAndSchema,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct XmlValidator {
    attribute: Attribute,
    input_type: XmlInputType,
    validation_type: ValidationType,
}

#[typetag::serde(name = "XMLValidator")]
impl SyncAction for XmlValidator {
    fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;

        let targets = &input.features;
        let mut success = Vec::<Feature>::new();
        let mut failed = Vec::<Feature>::new();

        match self.validation_type {
            ValidationType::Syntax => {
                for feature in targets {
                    let xml_content = self.get_xml_content(&ctx, feature)?;
                    let Ok(document) = xml::parse(xml_content) else {
                        failed.push(feature.clone());
                        continue;
                    };
                    let Ok(_) = xml::get_root_node(&document) else {
                        failed.push(feature.clone());
                        continue;
                    };
                    success.push(feature.clone());
                }
            }
            ValidationType::SyntaxAndNamespace => {
                let result = targets
                    .par_iter()
                    .map(|feature| {
                        ctx.action_log(format!("Validating feature: {:?}", feature));
                        let xml_content = self.get_xml_content(&ctx, feature).unwrap();
                        let Ok(document) = xml::parse(xml_content) else {
                            return (false, feature.clone());
                        };
                        let Ok(root_node) = xml::get_root_node(&document) else {
                            return (false, feature.clone());
                        };
                        if XmlValidator::recursive_check_namespace(
                            &root_node,
                            &root_node.get_namespace_declarations(),
                        ) {
                            (true, feature.clone())
                        } else {
                            (false, feature.clone())
                        }
                    })
                    .collect::<Vec<_>>();
                for (result, feature) in result {
                    if result {
                        success.push(feature);
                    } else {
                        failed.push(feature);
                    }
                }
            }
            ValidationType::SyntaxAndSchema => {
                let mut schema_store = HashMap::<String, xml::XmlSchemaValidationContext>::new();
                for feature in targets {
                    ctx.action_log(format!("Validating feature: {:?}", feature));
                    let xml_content = self.get_xml_content(&ctx, feature)?;
                    let Ok(document) = xml::parse(xml_content) else {
                        failed.push(feature.clone());
                        continue;
                    };
                    if let Ok(true) = self.check_schema(feature, &ctx, &document, &mut schema_store)
                    {
                        success.push(feature.clone());
                    } else {
                        failed.push(feature.clone());
                    }
                }
            }
        }
        Ok(ActionDataframe::from([
            (DEFAULT_PORT.clone(), Dataframe::new(success)),
            (REJECTED_PORT.clone(), Dataframe::new(failed)),
        ]))
    }
}

impl XmlValidator {
    fn get_base_path(&self, feature: &Feature) -> String {
        match self.input_type {
            XmlInputType::File => feature
                .attributes
                .get(&self.attribute)
                .and_then(|v| {
                    if let AttributeValue::String(s) = v {
                        match Uri::from_str(s) {
                            Ok(uri) => {
                                if uri.is_dir() {
                                    Some(uri.to_string())
                                } else if let Some(parent) = uri.parent() {
                                    Some(parent.to_string())
                                } else {
                                    Some("".to_string())
                                }
                            }
                            Err(_) => None,
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or("".to_string()),
            XmlInputType::Text => "".to_string(),
        }
    }

    fn get_xml_content(&self, ctx: &ActionContext, feature: &Feature) -> Result<String> {
        match self.input_type {
            XmlInputType::File => {
                let uri = feature
                    .attributes
                    .get(&self.attribute)
                    .ok_or(Error::input("No Attribute"))?;
                let uri = match uri {
                    AttributeValue::String(s) => {
                        Uri::from_str(s).map_err(|_| Error::input("Invalid URI"))?
                    }
                    _ => return Err(Error::input("Invalid Attribute")),
                };
                let storage = ctx.resolve_uri(&uri)?;
                let content = storage
                    .get_sync(uri.path().as_path())
                    .map_err(Error::internal_runtime)?;
                String::from_utf8(content.to_vec()).map_err(|_| Error::input("Invalid UTF-8"))
            }
            XmlInputType::Text => {
                let content = feature
                    .attributes
                    .get(&self.attribute)
                    .ok_or(Error::input("No Attribute"))?;
                let content = match content {
                    AttributeValue::String(s) => s,
                    _ => return Err(Error::input("Invalid Attribute")),
                };
                Ok(content.to_string())
            }
        }
    }

    fn recursive_check_namespace(node: &xml::XmlNode, namespaces: &Vec<XmlNamespace>) -> bool {
        let result = match node.get_namespace() {
            Some(ns) => namespaces.iter().any(|n| n.get_prefix() == ns.get_prefix()),
            None => {
                let tag = xml::get_node_tag(node);
                if tag.contains(':') {
                    let prefix = tag.split(':').collect::<Vec<&str>>()[0];
                    namespaces.iter().any(|n| n.get_prefix() == prefix)
                } else {
                    true
                }
            }
        };
        node.get_child_nodes()
            .iter()
            .filter(|n| {
                if let Some(typ) = n.get_type() {
                    typ == xml::XmlNodeType::ElementNode
                } else {
                    false
                }
            })
            .all(|n| XmlValidator::recursive_check_namespace(n, namespaces))
            && result
    }

    fn check_schema(
        &self,
        feature: &Feature,
        ctx: &ActionContext,
        document: &XmlDocument,
        schema_store: &mut HashMap<String, xml::XmlSchemaValidationContext>,
    ) -> Result<bool> {
        let schema_locations =
            xml::parse_schema_locations(document).map_err(Error::internal_runtime)?;
        let target_locations = schema_locations
            .difference(&HashSet::from_iter(schema_store.keys().cloned()))
            .cloned()
            .collect::<Vec<_>>();
        if !target_locations.is_empty() {
            for location in target_locations {
                let target = if !location.contains(PROTOCOL_SEPARATOR) && !location.starts_with('/')
                {
                    format!("{}/{}", self.get_base_path(feature), location.clone())
                } else {
                    location.clone()
                };
                if target.is_empty() {
                    continue;
                }
                let schema_context = match xml::create_xml_schema_validation_context(target) {
                    Ok(ctx) => ctx,
                    Err(e) => {
                        ctx.action_log(&format!("Failed to create schema context: {:?}", e));
                        continue;
                    }
                };
                schema_store.insert(location, schema_context);
            }
        }
        for location in schema_locations {
            let schema_context = match schema_store.get_mut(&location) {
                Some(ctx) => ctx,
                None => continue,
            };
            if xml::validate_document_by_schema_context(document, schema_context).is_err() {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_action::{
        Attribute, AttributeValue, Dataframe, Feature, Port, DEFAULT_PORT, REJECTED_PORT,
    };
    use rstest::*;
    use std::collections::HashMap;
    use uuid::uuid;

    static VALID_XML: &str = r#"<?xml version="1.0"?>
    <?xml version="1.0" encoding="UTF-8"?>
    <core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.0" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd">
            <gml:boundedBy>
                    <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
                            <gml:lowerCorner>35.83407435187811 139.81150449360632 0</gml:lowerCorner>
                            <gml:upperCorner>35.835160640918005 139.81296464368188 0</gml:upperCorner>
                    </gml:Envelope>
            </gml:boundedBy>
            <core:cityObjectMember>
                    <tran:Road gml:id="tran_ca6e219f-8f9f-42a0-9fdc-f9c919a5aadd">
                            <core:creationDate>2024-03-29</core:creationDate>
                            <tran:class codeSpace="../../codelists/TransportationComplex_class.xml">1040</tran:class>
                            <tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
                            <tran:lod1MultiSurface>
                                    <gml:MultiSurface>
                                            <gml:surfaceMember>
                                                    <gml:Polygon>
                                                            <gml:exterior>
                                                                    <gml:LinearRing>
                                                                            <gml:posList>35.835082923688766 139.81240430913275 0 35.83513857251635 139.81235524863882 0 35.83509452854166 139.81227524096641 0 35.835016266236615 139.81213169621427 0 35.834974791796974 139.81206152957924 0 35.8349334984113 139.8119960119602 0 35.83488670628952 139.81192783932084 0 35.83485488130023 139.81188644974807 0 35.834775816152764 139.81179216310065 0 35.83471604447279 139.81172388354543 0 35.83469278541443 139.8117002022056 0 35.83467024842746 139.81168205517872 0 35.834640950803276 139.81166103206388 0 35.83460471263021 139.81163912533526 0 35.83456676193032 139.8116174404727 0 35.8345069075908 139.81159100194915 0 35.83442478918868 139.81156047403294 0 35.834401818667246 139.81163718815566 0 35.83448573990736 139.81166860116923 0 35.83454063647314 139.8116931593648 0 35.83457570272412 139.8117141809001 0 35.834608875959844 139.81173420675853 0 35.83463456769947 139.8117525743116 0 35.834652507161884 139.81176718051807 0 35.83467152905465 139.8117863247246 0 35.834729227182116 139.81185228035764 0 35.83480666952048 139.8119446857135 0 35.83483624060701 139.8119831979447 0 35.834880868892455 139.8120481611355 0 35.83492071968172 139.81211135462706 0 35.83496084166052 139.8121791971029 0 35.835038202251226 139.81232097099297 0 35.835082923688766 139.81240430913275 0</gml:posList>
                                                                    </gml:LinearRing>
                                                            </gml:exterior>
                                                    </gml:Polygon>
                                            </gml:surfaceMember>
                                    </gml:MultiSurface>
                            </tran:lod1MultiSurface>
                    </tran:Road>
            </core:cityObjectMember>
    </core:CityModel>
"#;

    static INVALID_SYNTAX_XML: &str = r#"fsahoges.>fsafsafsa"#;

    static INVALID_NAMESPACE_XML: &str = r#"<?xml version="1.0"?>
<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.0" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd">
        <invalid:cityObjectMember>
                <tran:Road gml:id="tran_ca6e219f-8f9f-42a0-9fdc-f9c919a5aadd">
                        <core:creationDate>2024-03-29</core:creationDate>
                        <tran:class codeSpace="../../codelists/TransportationComplex_class.xml">1040</tran:class>
                        <tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function>
                        <tran:lod1MultiSurface>
                                <gml:MultiSurface>
                                        <gml:surfaceMember>
                                                <gml:Polygon>
                                                        <gml:exterior>
                                                                <gml:LinearRing>
                                                                        <gml:posList>35.835082923688766 139.81240430913275 0 35.83513857251635 139.81235524863882 0 35.83509452854166 139.81227524096641 0 35.835016266236615 139.81213169621427 0 35.834974791796974 139.81206152957924 0 35.8349334984113 139.8119960119602 0 35.83488670628952 139.81192783932084 0 35.83485488130023 139.81188644974807 0 35.834775816152764 139.81179216310065 0 35.83471604447279 139.81172388354543 0 35.83469278541443 139.8117002022056 0 35.83467024842746 139.81168205517872 0 35.834640950803276 139.81166103206388 0 35.83460471263021 139.81163912533526 0 35.83456676193032 139.8116174404727 0 35.8345069075908 139.81159100194915 0 35.83442478918868 139.81156047403294 0 35.834401818667246 139.81163718815566 0 35.83448573990736 139.81166860116923 0 35.83454063647314 139.8116931593648 0 35.83457570272412 139.8117141809001 0 35.834608875959844 139.81173420675853 0 35.83463456769947 139.8117525743116 0 35.834652507161884 139.81176718051807 0 35.83467152905465 139.8117863247246 0 35.834729227182116 139.81185228035764 0 35.83480666952048 139.8119446857135 0 35.83483624060701 139.8119831979447 0 35.834880868892455 139.8120481611355 0 35.83492071968172 139.81211135462706 0 35.83496084166052 139.8121791971029 0 35.835038202251226 139.81232097099297 0 35.835082923688766 139.81240430913275 0</gml:posList>
                                                                </gml:LinearRing>
                                                        </gml:exterior>
                                                </gml:Polygon>
                                        </gml:surfaceMember>
                                </gml:MultiSurface>
                        </tran:lod1MultiSurface>
                </tran:Road>
        </invalid:cityObjectMember>
</core:CityModel>
"#;

    static INVALID_SCHEMA_XML: &str = r#"<?xml version="1.0"?>
<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.0" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="http://www.opengis.net/citygml/transportation/2.0 http://schemas.opengis.net/citygml/transportation/2.0/transportation.xsd">
        <gml:hogehoge>
            <gml:lowerCorner>35.83407435187811 139.81150449360632 0</gml:lowerCorner>
            <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
                <gml:upperCorner>35.835160640918005 139.81296464368188 0</gml:upperCorner>
            </gml:Envelope>
        </gml:hogehoge>
        <core:cityObjectMembers>
                <tran:Road gml:id="tran_ca6e219f-8f9f-42a0-9fdc-f9c919a5aadd">
                        <core:creationDate>2024-03-29</core:creationDate>
                        <tran:class codeSpace="../../codelists/TransportationComplex_class.xml"><tran:function codeSpace="../../codelists/Road_function.xml">9020</tran:function></tran:class>
                        <tran:hoge codeSpace="../../codelists/Road_function.xml">9020</tran:hoge>
                        <tran:lod1MultiSurface>
                        </tran:lod1MultiSurface>
                </tran:Road>
        </core:cityObjectMembers>
</core:CityModel>
"#;

    #[fixture]
    fn inputs() -> Vec<Feature> {
        vec![
            Feature::new_with_id_and_attributes(
                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4a"),
                HashMap::from([(
                    Attribute::new("xml"),
                    AttributeValue::String(VALID_XML.to_string()),
                )]),
            ),
            Feature::new_with_id_and_attributes(
                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4b"),
                HashMap::from([(
                    Attribute::new("xml"),
                    AttributeValue::String(INVALID_SYNTAX_XML.to_string()),
                )]),
            ),
            Feature::new_with_id_and_attributes(
                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4c"),
                HashMap::from([(
                    Attribute::new("xml"),
                    AttributeValue::String(INVALID_NAMESPACE_XML.to_string()),
                )]),
            ),
            Feature::new_with_id_and_attributes(
                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4d"),
                HashMap::from([(
                    Attribute::new("xml"),
                    AttributeValue::String(INVALID_SCHEMA_XML.to_string()),
                )]),
            ),
        ]
    }

    #[fixture]
    fn expecteds() -> HashMap<String, HashMap<Port, Vec<Feature>>> {
        HashMap::from([
            (
                "syntax".to_string(),
                HashMap::from([
                    (
                        DEFAULT_PORT.clone(),
                        vec![
                            Feature::new_with_id_and_attributes(
                                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4a"),
                                HashMap::from([(
                                    Attribute::new("xml"),
                                    AttributeValue::String(VALID_XML.to_string()),
                                )]),
                            ),
                            Feature::new_with_id_and_attributes(
                                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4c"),
                                HashMap::from([(
                                    Attribute::new("xml"),
                                    AttributeValue::String(INVALID_NAMESPACE_XML.to_string()),
                                )]),
                            ),
                            Feature::new_with_id_and_attributes(
                                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4d"),
                                HashMap::from([(
                                    Attribute::new("xml"),
                                    AttributeValue::String(INVALID_SCHEMA_XML.to_string()),
                                )]),
                            ),
                        ],
                    ),
                    (
                        REJECTED_PORT.clone(),
                        vec![Feature::new_with_id_and_attributes(
                            uuid!("2830de29-b6bd-4783-9a89-042a587c2b4b"),
                            HashMap::from([(
                                Attribute::new("xml"),
                                AttributeValue::String(INVALID_SYNTAX_XML.to_string()),
                            )]),
                        )],
                    ),
                ]),
            ),
            (
                "syntax_and_namespace".to_string(),
                HashMap::from([
                    (
                        DEFAULT_PORT.clone(),
                        vec![
                            Feature::new_with_id_and_attributes(
                                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4a"),
                                HashMap::from([(
                                    Attribute::new("xml"),
                                    AttributeValue::String(VALID_XML.to_string()),
                                )]),
                            ),
                            Feature::new_with_id_and_attributes(
                                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4d"),
                                HashMap::from([(
                                    Attribute::new("xml"),
                                    AttributeValue::String(INVALID_SCHEMA_XML.to_string()),
                                )]),
                            ),
                        ],
                    ),
                    (
                        REJECTED_PORT.clone(),
                        vec![
                            Feature::new_with_id_and_attributes(
                                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4b"),
                                HashMap::from([(
                                    Attribute::new("xml"),
                                    AttributeValue::String(INVALID_SYNTAX_XML.to_string()),
                                )]),
                            ),
                            Feature::new_with_id_and_attributes(
                                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4c"),
                                HashMap::from([(
                                    Attribute::new("xml"),
                                    AttributeValue::String(INVALID_NAMESPACE_XML.to_string()),
                                )]),
                            ),
                        ],
                    ),
                ]),
            ),
            (
                "syntax_and_schema".to_string(),
                HashMap::from([
                    (
                        DEFAULT_PORT.clone(),
                        vec![
                            Feature::new_with_id_and_attributes(
                                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4a"),
                                HashMap::from([(
                                    Attribute::new("xml"),
                                    AttributeValue::String(VALID_XML.to_string()),
                                )]),
                            ),
                            Feature::new_with_id_and_attributes(
                                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4c"),
                                HashMap::from([(
                                    Attribute::new("xml"),
                                    AttributeValue::String(INVALID_NAMESPACE_XML.to_string()),
                                )]),
                            ),
                            Feature::new_with_id_and_attributes(
                                uuid!("2830de29-b6bd-4783-9a89-042a587c2b4d"),
                                HashMap::from([(
                                    Attribute::new("xml"),
                                    AttributeValue::String(INVALID_SCHEMA_XML.to_string()),
                                )]),
                            ),
                        ],
                    ),
                    (
                        REJECTED_PORT.clone(),
                        vec![Feature::new_with_id_and_attributes(
                            uuid!("2830de29-b6bd-4783-9a89-042a587c2b4b"),
                            HashMap::from([(
                                Attribute::new("xml"),
                                AttributeValue::String(INVALID_SYNTAX_XML.to_string()),
                            )]),
                        )],
                    ),
                ]),
            ),
        ])
    }

    #[rstest]
    #[test]
    #[case::syntax(ValidationType::Syntax, "syntax".to_string())]
    #[case::syntax_and_namespace(ValidationType::SyntaxAndNamespace, "syntax_and_namespace".to_string())]
    fn test_xml_validator(
        #[case] arg: ValidationType,
        #[case] case: String,
        inputs: Vec<Feature>,
        expecteds: HashMap<String, HashMap<Port, Vec<Feature>>>,
    ) {
        let inputs = vec![(DEFAULT_PORT.clone(), Dataframe::new(inputs))]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let expected_output = expecteds
            .get(&case)
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), Dataframe::new(v.clone())))
            .collect::<HashMap<_, _>>();
        let validator = XmlValidator {
            attribute: Attribute::new("xml"),
            input_type: XmlInputType::Text,
            validation_type: arg,
        };
        let result = validator.run(ActionContext::default(), inputs);
        assert_eq!(result.unwrap(), expected_output);
    }
}
