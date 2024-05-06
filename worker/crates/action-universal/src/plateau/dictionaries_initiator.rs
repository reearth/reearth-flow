use std::vec;
use std::{collections::HashMap, str::FromStr, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_action::Attribute;
use reearth_flow_action::Dataframe;
use reearth_flow_action::Feature;
use reearth_flow_common::uri::Uri;
use reearth_flow_common::xml;
use reearth_flow_storage::resolve::StorageResolver;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error, ActionContext, ActionDataframe, ActionResult, AsyncAction, AttributeValue, Result,
    DEFAULT_PORT,
};

use super::types::SchemaFeature;
use super::types::Settings;
use super::types::DICTIONARIES_INITIATOR_SETTINGS_PORT;

static ADMIN_CODE_LIST: &str = "Common_localPublicAuthorities.xml";

static COMMON_ITEMS: Lazy<Vec<SchemaFeature>> = Lazy::new(|| {
    vec![
        SchemaFeature {
            name: "gml:description".to_string(),
            r#type: "gml:StringOrRefType".to_string(),
            min_occurs: "0".to_string(),
            max_occurs: "1".to_string(),
            flag: None,
            children: None,
        },
        SchemaFeature {
            name: "gml:name".to_string(),
            r#type: "gml:CodeType".to_string(),
            min_occurs: "0".to_string(),
            max_occurs: "1".to_string(),
            flag: None,
            children: None,
        },
        SchemaFeature {
            name: "core:creationDate".to_string(),
            r#type: "xs:date".to_string(),
            min_occurs: "0".to_string(),
            max_occurs: "1".to_string(),
            flag: None,
            children: None,
        },
        SchemaFeature {
            name: "core:terminationDate".to_string(),
            r#type: "xs:date".to_string(),
            min_occurs: "0".to_string(),
            max_occurs: "1".to_string(),
            flag: None,
            children: None,
        },
        SchemaFeature {
            name: "core:externalReference".to_string(),
            r#type: "ExternalReferenceType".to_string(),
            min_occurs: "0".to_string(),
            max_occurs: "unbounded".to_string(),
            flag: None,
            children: None,
        },
        SchemaFeature {
            name: "core:generalizesTo".to_string(),
            r#type: "GeneralizationRelationType".to_string(),
            min_occurs: "0".to_string(),
            max_occurs: "unbounded".to_string(),
            flag: None,
            children: None,
        },
        SchemaFeature {
            name: "core:relativeToTerrain".to_string(),
            r#type: "RelativeToTerrainType".to_string(),
            min_occurs: "0".to_string(),
            max_occurs: "1".to_string(),
            flag: None,
            children: None,
        },
    ]
});

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DictionariesInitiator {
    city_code: Option<String>,
    target_packages: Option<Vec<String>>,
    add_nsprefix_to_feature_types: Option<bool>,
    except_feature_types: Option<Vec<String>>,
    extract_dm_geometry_as_xml_fragment: Option<bool>,
    schema_json: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Schema {
    features: HashMap<String, Vec<SchemaFeature>>,
    complex_types: HashMap<String, Vec<SchemaFeature>>,
}

#[async_trait::async_trait]
#[typetag::serde(name = "PLATEAU.DictionariesInitiator")]
impl AsyncAction for DictionariesInitiator {
    async fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(error::Error::input("No Default Port"))?;
        let data = input;
        let mut res = ActionDataframe::new();

        let xpath_to_properties = {
            let schema_json = self
                .schema_json
                .clone()
                .ok_or(error::Error::input("No schema_json"))?;
            let dm_geom_to_xml = self.extract_dm_geometry_as_xml_fragment.unwrap_or_default();
            generate_xpath(schema_json, dm_geom_to_xml)?
        };
        let except_feature_types = self.except_feature_types.clone().unwrap_or_default();

        let data = {
            let mut codelists_map: HashMap<String, HashMap<String, HashMap<String, String>>> =
                HashMap::new();
            let mut result = Vec::<Feature>::new();
            for row in &data.features {
                let feature = &row.attributes;
                // Codelist dictionary creation
                let dir_codelists = match feature.get(&Attribute::new("dirCodelists")) {
                    Some(AttributeValue::String(dir)) => dir,
                    v => {
                        return Err(error::Error::input(format!(
                            "No dirCodelists value with {:?}",
                            v
                        )))
                    }
                };
                if !codelists_map.contains_key(dir_codelists) {
                    let dir = Uri::from_str(dir_codelists).map_err(|e| {
                        error::Error::input(format!("Cannot parse uri with error = {:?}", e))
                    })?;
                    if dir.is_dir() {
                        let codelists =
                            create_codelist_map(Arc::clone(&ctx.storage_resolver), &dir).await?;
                        if !codelists.is_empty() {
                            codelists_map.insert(dir_codelists.to_string(), codelists);
                        }
                    }
                }
                let mut result_value = feature.clone();
                // Municipality name acquisition
                if let Some(file) = codelists_map.get(dir_codelists) {
                    if let Some(city_code) = &self.city_code {
                        if let Some(name) = file.get(ADMIN_CODE_LIST) {
                            if let Some(city_name) = name.get(city_code) {
                                result_value.insert(
                                    Attribute::new("cityName"),
                                    AttributeValue::String(city_name.clone()),
                                );
                            }
                        }
                    }
                }

                result_value.insert(
                    Attribute::new("featureTypesWithPrefix"),
                    AttributeValue::Array(
                        xpath_to_properties
                            .keys()
                            .map(|v| AttributeValue::String(v.clone()))
                            .collect::<Vec<_>>(),
                    ),
                );
                let ftypes = xpath_to_properties.keys().collect::<Vec<_>>();
                let out_ftypes = ftypes
                    .iter()
                    .flat_map(|v| {
                        if !except_feature_types.contains(v) {
                            if let Some(true) = self.add_nsprefix_to_feature_types {
                                Some(AttributeValue::String(v.replace(':', "_")))
                            } else {
                                Some(AttributeValue::String(
                                    v.split(':')
                                        .map(|v| v.to_string())
                                        .nth(1)
                                        .unwrap_or_default(),
                                ))
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                result_value.insert(
                    Attribute::new("featureTypes"),
                    AttributeValue::Array(out_ftypes),
                );
                result.push(row.with_attributes(result_value));
            }
            let settings = Settings::new(
                xpath_to_properties,
                except_feature_types,
                codelists_map
                    .iter()
                    .fold(HashMap::new(), |mut acc, (_k, v)| {
                        acc.extend(v.clone());
                        acc
                    }),
            );
            let settings = serde_json::to_value(settings).map_err(|e| {
                error::Error::output(format!("Cannot convert to json with error = {:?}", e))
            })?;
            let settings: Feature = settings.into();
            res.insert(
                DICTIONARIES_INITIATOR_SETTINGS_PORT.clone(),
                Dataframe::new(vec![settings]),
            );
            result
        };
        res.insert(DEFAULT_PORT.clone(), Dataframe::new(data));
        Ok(res)
    }
}

async fn create_codelist_map(
    storage_resolver: Arc<StorageResolver>,
    dir: &Uri,
) -> Result<HashMap<String, HashMap<String, String>>> {
    let storage = storage_resolver
        .resolve(dir)
        .map_err(error::Error::internal_runtime)?;
    let mut codelist_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    if storage
        .exists(dir.path().as_path())
        .await
        .map_err(error::Error::internal_runtime)?
    {
        for f in storage
            .list_with_result(Some(dir.path().as_path()), true)
            .await
            .map_err(error::Error::internal_runtime)?
        {
            if f.is_file() {
                if let Some(extension) = f.extension() {
                    if extension == "xml" {
                        let result = storage
                            .get(f.path().as_path())
                            .await
                            .map_err(error::Error::internal_runtime)?;
                        let byte = result
                            .bytes()
                            .await
                            .map_err(error::Error::internal_runtime)?;
                        let text = String::from_utf8(byte.to_vec())
                            .map_err(error::Error::internal_runtime)?;
                        let document = xml::parse(text).map_err(error::Error::internal_runtime)?;
                        let names = xml::evaluate(
                            &document,
                            "/gml:Dictionary/gml:dictionaryEntry/gml:Definition/gml:name/text()",
                        )
                        .map_err(error::Error::internal_runtime)?;
                        let descriptions = xml::evaluate(&document, "/gml:Dictionary/gml:dictionaryEntry/gml:Definition/gml:description/text()").map_err(error::Error::internal_runtime)?;
                        let codelist = xml::collect_text_values(&names)
                            .into_iter()
                            .zip(xml::collect_text_values(&descriptions))
                            .collect::<HashMap<_, _>>();
                        codelist_map.insert(
                            f.file_name().unwrap().to_str().unwrap().to_string(),
                            codelist,
                        );
                    }
                }
            }
        }
    }
    Ok(codelist_map)
}

fn generate_xpath(
    schema_json: String,
    dm_geom_to_xml: bool,
) -> Result<HashMap<String, HashMap<String, SchemaFeature>>> {
    let schema: Schema = serde_json::from_str(&schema_json)
        .map_err(|e| error::Error::input(format!("Cannot parse schema with error = {:?}", e)))?;
    let mut xpath_to_properties: HashMap<String, HashMap<String, SchemaFeature>> = HashMap::new();
    let mut complex_types = schema
        .complex_types
        .iter()
        .map(|(key, value)| {
            if ["uro:DmGeometricAttribute", "uro:DmAnnotation"].contains(&key.as_str()) {
                let value = value
                    .iter()
                    .map(|v| {
                        if v.name.starts_with("uro:lod0") {
                            let mut v = v.clone();
                            v.flag = if dm_geom_to_xml {
                                Some("fragment".to_string())
                            } else {
                                Some("geometry".to_string())
                            };
                            return v;
                        }
                        v.clone()
                    })
                    .collect::<Vec<_>>();
                return (key.clone(), value);
            }
            (key.clone(), value.to_vec())
        })
        .collect::<HashMap<_, _>>();

    for (key, items) in schema.features.iter() {
        let mut complex_type = Vec::new();
        complex_type.extend(COMMON_ITEMS.clone());
        complex_type.extend(items.clone());
        complex_types.insert(key.clone(), complex_type);
        for obj in items.iter() {
            let properties = create_xpath(&complex_types, key.clone(), key.clone(), obj)?;
            xpath_to_properties.insert(
                key.to_string(),
                properties
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<_, _>>(),
            );
        }
    }

    Ok(xpath_to_properties)
}

fn create_xpath(
    complex_types: &HashMap<String, Vec<SchemaFeature>>,
    key: String,
    xpath: String,
    item: &SchemaFeature,
) -> Result<Vec<(String, SchemaFeature)>> {
    let xpath = format!("{}/{}", xpath, item.name);
    let mut xpath_to_properties = Vec::<(String, SchemaFeature)>::new();
    match &item.children {
        Some(children) => {
            xpath_to_properties.push((
                xpath.clone(),
                SchemaFeature {
                    name: item.name.clone(),
                    r#type: item.r#type.clone(),
                    min_occurs: item.min_occurs.clone(),
                    max_occurs: item.max_occurs.clone(),
                    flag: Some("role".to_string()),
                    children: item.children.clone(),
                },
            ));
            for child in children {
                let xp = format!("{}/{}", xpath, child);
                xpath_to_properties.push((
                    xp.clone(),
                    SchemaFeature {
                        name: child.clone(),
                        r#type: item.r#type.clone(),
                        min_occurs: item.min_occurs.clone(),
                        max_occurs: item.max_occurs.clone(),
                        flag: Some("parent".to_string()),
                        children: item.children.clone(),
                    },
                ));
                let child = match complex_types.get(child) {
                    Some(child) => child,
                    None => continue,
                };
                for c in child {
                    let properties = create_xpath(complex_types, key.clone(), xpath.clone(), c)?;
                    properties.iter().for_each(|(xpath, item)| {
                        xpath_to_properties.push((xpath.clone(), item.clone()));
                    });
                }
            }
        }
        None => {
            xpath_to_properties.push((
                xpath.clone(),
                SchemaFeature {
                    name: item.name.clone(),
                    r#type: item.r#type.clone(),
                    min_occurs: item.min_occurs.clone(),
                    max_occurs: item.max_occurs.clone(),
                    flag: item.flag.clone(),
                    children: None,
                },
            ));
        }
    }
    Ok(xpath_to_properties)
}
