use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use reearth_flow_common::collection::insert_map_element;
use reearth_flow_common::uri::Uri;
use reearth_flow_common::xml;
use reearth_flow_storage::resolve::StorageResolver;

use super::errors::PlateauProcessorError;

use super::types::{Schema, SchemaFeature};

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

pub(super) fn create_codelist_map(
    storage_resolver: Arc<StorageResolver>,
    dir: &Uri,
) -> super::errors::Result<HashMap<String, HashMap<String, String>>> {
    let storage = storage_resolver
        .resolve(dir)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let mut codelist_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    if storage
        .exists_sync(dir.path().as_path())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?
    {
        for f in storage
            .list_sync(Some(dir.path().as_path()), true)
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?
        {
            if !f.is_file() || f.extension().is_none() || f.extension().unwrap() != "xml" {
                continue;
            }
            let bytes = storage.get_sync(f.path().as_path()).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
            })?;
            let text = String::from_utf8(bytes.to_vec()).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
            })?;
            let document = xml::parse(text).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
            })?;
            let names = xml::evaluate(
                &document,
                "/gml:Dictionary/gml:dictionaryEntry/gml:Definition/gml:name/text()",
            )
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
            let descriptions = xml::evaluate(
                &document,
                "/gml:Dictionary/gml:dictionaryEntry/gml:Definition/gml:description/text()",
            )
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
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
    Ok(codelist_map)
}

pub(super) fn generate_xpath_to_properties(
    schema_json: String,
    dm_geom_to_xml: bool,
) -> super::errors::Result<HashMap<String, HashMap<String, SchemaFeature>>> {
    let schema: Schema = serde_json::from_str(&schema_json).map_err(|e| {
        PlateauProcessorError::DomainOfDefinitionValidator(format!(
            "Cannot parse schema with error = {e:?}"
        ))
    })?;
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

    let mut xpath_to_properties: HashMap<String, HashMap<String, SchemaFeature>> = HashMap::new();
    for (key, items) in schema.features.iter() {
        let mut complex_type = Vec::new();
        complex_type.extend(COMMON_ITEMS.clone());
        complex_type.extend(items.clone());
        complex_types.insert(key.clone(), complex_type);
        for obj in items.iter() {
            create_xpath(
                &complex_types,
                key.clone(),
                key.clone(),
                obj,
                &mut xpath_to_properties,
            )?;
        }
    }
    Ok(xpath_to_properties)
}

fn create_xpath(
    complex_types: &HashMap<String, Vec<SchemaFeature>>,
    key: String,
    xpath: String,
    item: &SchemaFeature,
    xpath_to_properties: &mut HashMap<String, HashMap<String, SchemaFeature>>,
) -> super::errors::Result<()> {
    let xpath = format!("{}/{}", xpath, item.name);
    match &item.children {
        Some(children) => {
            insert_map_element(
                xpath_to_properties,
                key.clone(),
                xpath.clone(),
                SchemaFeature {
                    name: item.name.clone(),
                    r#type: item.r#type.clone(),
                    min_occurs: item.min_occurs.clone(),
                    max_occurs: item.max_occurs.clone(),
                    flag: Some("role".to_string()),
                    children: item.children.clone(),
                },
            );
            for child in children {
                let xp = format!("{xpath}/{child}");
                insert_map_element(
                    xpath_to_properties,
                    key.clone(),
                    xp.clone(),
                    SchemaFeature {
                        name: child.clone(),
                        r#type: item.r#type.clone(),
                        min_occurs: item.min_occurs.clone(),
                        max_occurs: item.max_occurs.clone(),
                        flag: Some("parent".to_string()),
                        children: item.children.clone(),
                    },
                );
                let child = complex_types.get(child).cloned().unwrap_or_default();
                for c in child.iter() {
                    create_xpath(
                        complex_types,
                        key.clone(),
                        xp.clone(),
                        c,
                        xpath_to_properties,
                    )?;
                }
            }
        }
        None => {
            insert_map_element(
                xpath_to_properties,
                key.clone(),
                xpath.clone(),
                SchemaFeature {
                    name: item.name.clone(),
                    r#type: item.r#type.clone(),
                    min_occurs: item.min_occurs.clone(),
                    max_occurs: item.max_occurs.clone(),
                    flag: item.flag.clone(),
                    children: None,
                },
            );
        }
    }
    Ok(())
}
