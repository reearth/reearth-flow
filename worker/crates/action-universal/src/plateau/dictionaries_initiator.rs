use std::{collections::HashMap, str::FromStr, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_common::xml;
use reearth_flow_storage::resolve::StorageResolver;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error, Action, ActionContext, ActionDataframe, ActionResult, ActionValue, Result, DEFAULT_PORT,
};

use super::types::SchemaFeature;
use super::types::Settings;
use super::types::DICTIONARIES_INITIATOR_SETTINGS_PORT;

static ADMIN_CODE_LIST: &str = "Common_localPublicAuthorities";

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
pub struct DictionariesInitiator;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Schema {
    features: HashMap<String, Vec<SchemaFeature>>,
    complex_types: HashMap<String, Vec<SchemaFeature>>,
}

#[async_trait::async_trait]
#[typetag::serde(name = "PLATEAU.DictionariesInitiator")]
impl Action for DictionariesInitiator {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(error::Error::input("No Input"))?;
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(error::Error::input("No Default Port"))?;
        let data = input.as_ref().ok_or(error::Error::input("No Value"))?;
        let mut res = ActionDataframe::new();
        let data = match data {
            ActionValue::Array(data) => {
                let first = data.first().ok_or(error::Error::input("No Value"))?;
                // XPath-Property Dictionary Creation
                let xpath_to_properties = if let ActionValue::Map(row) = first {
                    match (
                        row.get("schemaJson")
                            .ok_or(error::Error::input("No schema json value"))?,
                        row.get("extractDmGeometryAsXmlFragment")
                            .ok_or(error::Error::input(
                                "No extractDmGeometryAsXmlFragment value",
                            ))?,
                    ) {
                        (ActionValue::String(schema_json), ActionValue::Bool(dm_geom_to_xml)) => {
                            generate_xpath(schema_json.clone(), *dm_geom_to_xml)?
                        }
                        _ => {
                            return Err(error::Error::input("Invalid Input. supported only String"))
                        }
                    }
                } else {
                    return Err(error::Error::input("Invalid Input. supported only Map"));
                };
                let except_feature_types = if let ActionValue::Map(row) = first {
                    let ftypes = row.get("exceptFeatureTypes");
                    match ftypes {
                        Some(ActionValue::Array(ftypes)) => ftypes
                            .iter()
                            .filter_map(|ft| {
                                if let ActionValue::String(ft) = ft {
                                    Some(ft.clone())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>(),
                        _ => vec![],
                    }
                } else {
                    return Err(error::Error::input("Invalid Input. supported only Map"));
                };

                let mut codelists_map: HashMap<String, HashMap<String, HashMap<String, String>>> =
                    HashMap::new();
                let mut result = Vec::<ActionValue>::new();
                for row in data {
                    let feature = match row {
                        ActionValue::Map(row) => row,
                        _ => return Err(error::Error::input("Invalid Input. supported only Map")),
                    };
                    // Codelist dictionary creation
                    let dir_codelists = match feature.get("dirCodelists") {
                        Some(ActionValue::String(dir)) => dir,
                        _ => return Err(error::Error::input("No dirCodelists value")),
                    };
                    if codelists_map.get(dir_codelists).is_none() {
                        let dir = Uri::from_str(dir_codelists).map_err(|e| {
                            error::Error::input(format!("Cannot parse uri with error = {:?}", e))
                        })?;
                        if dir.is_dir() {
                            let codelists =
                                create_codelist_map(Arc::clone(&ctx.storage_resolver), &dir)
                                    .await?;
                            if !codelists.is_empty() {
                                codelists_map.insert(dir_codelists.to_string(), codelists);
                            }
                        }
                    }
                    let mut result_value = feature.clone();
                    // Municipality name acquisition
                    if let Some(file) = codelists_map.get(dir_codelists) {
                        if let Some(ActionValue::String(city_code)) = feature.get("cityCode") {
                            if let Some(name) = file.get(ADMIN_CODE_LIST) {
                                if let Some(city_name) = name.get(city_code) {
                                    result_value.insert(
                                        "cityName".to_string(),
                                        ActionValue::String(city_name.clone()),
                                    );
                                }
                            }
                        }
                    }

                    result_value.insert(
                        "featureTypesWithPrefix".to_string(),
                        ActionValue::Array(
                            xpath_to_properties
                                .keys()
                                .map(|v| ActionValue::String(v.clone()))
                                .collect::<Vec<_>>(),
                        ),
                    );
                    let ftypes = xpath_to_properties.keys().collect::<Vec<_>>();
                    let out_ftypes = if let Some(ActionValue::Bool(true)) =
                        feature.get("addNsprefixToFeatureTypes")
                    {
                        except_feature_types
                            .iter()
                            .flat_map(|v| {
                                if ftypes.contains(&v) {
                                    Some(ActionValue::String(v.replace(':', "_")))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    } else {
                        except_feature_types
                            .iter()
                            .flat_map(|v| {
                                if ftypes.contains(&v) {
                                    Some(ActionValue::String(
                                        v.split(':')
                                            .map(|v| v.to_string())
                                            .nth(1)
                                            .unwrap_or_default(),
                                    ))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    };
                    result_value.insert("featureTypes".to_string(), ActionValue::Array(out_ftypes));
                    result.push(ActionValue::Map(result_value));
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
                res.insert(
                    DICTIONARIES_INITIATOR_SETTINGS_PORT.clone(),
                    Some(settings.into()),
                );
                result
            }
            _ => return Err(error::Error::input("Invalid Input. supported only Array")),
        };
        res.insert(DEFAULT_PORT.clone(), Some(ActionValue::Array(data)));
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
    if storage
        .exists(dir.path().as_path())
        .await
        .map_err(error::Error::internal_runtime)?
    {
        let mut codelist_map: HashMap<String, HashMap<String, String>> = HashMap::new();
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
    Ok(HashMap::new())
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
            if ["uro:DmGeometricAttribute", "uro:DmAnnotation"].contains(&key.as_ref()) {
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
