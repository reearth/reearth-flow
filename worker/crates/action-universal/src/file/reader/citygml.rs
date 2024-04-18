use std::{
    io::{BufRead, BufReader, Cursor},
    sync::{Arc, RwLock},
};

use nusamai_citygml::{
    object::{Map, Object, ObjectStereotype},
    CityGmlElement, CityGmlReader, Envelope, GeometryStore, ParseError, SubTreeReader, Value,
};
use nusamai_plateau::{appearance::AppearanceStore, models, Entity};
use quick_xml::NsReader;
use reearth_flow_action::{error::Error, ActionContext, ActionValue, Result};
use reearth_flow_common::uri::Uri;

enum Parent {
    Feature { id: String, typename: String },
    Data { typename: String }, // Data stereotype does not have an id
    Object { id: String, typename: String },
}

pub(crate) async fn read_citygml(input_path: Uri, ctx: ActionContext) -> Result<ActionValue> {
    let code_resolver = nusamai_plateau::codelist::Resolver::new();
    let storage_resolver = Arc::clone(&ctx.storage_resolver);
    ctx.action_log(format!("Parsing CityGML file: {:?} ...", input_path));
    let storage = storage_resolver
        .resolve(&input_path)
        .map_err(Error::input)?;
    let result = storage
        .get(input_path.path().as_path())
        .await
        .map_err(Error::internal_runtime)?;
    let byte = result.bytes().await.map_err(Error::internal_runtime)?;
    let cursor = Cursor::new(byte);
    let buf_reader = BufReader::new(cursor);

    let mut xml_reader = NsReader::from_reader(buf_reader);
    let context = nusamai_citygml::ParseContext::new(input_path.into(), &code_resolver);
    let mut citygml_reader = CityGmlReader::new(context);
    let mut st = citygml_reader
        .start_root(&mut xml_reader)
        .map_err(Error::internal_runtime)?;
    let entities = parse_tree_reader(&mut st).map_err(Error::internal_runtime)?;
    let mut flattened_entities = Vec::new();
    for entity in entities {
        flatten_entity(
            entity.root,
            entity.geometry_store,
            entity.appearance_store,
            &mut flattened_entities,
            &None,
        );
    }
    let values = serde_json::to_value(flattened_entities).map_err(Error::internal_runtime)?;
    Ok(values.into())
}

fn parse_tree_reader<R: BufRead>(st: &mut SubTreeReader<R>) -> Result<Vec<Entity>, ParseError> {
    let mut entities = Vec::new();
    let mut global_appearances = AppearanceStore::default();

    st.parse_children(|st| {
        match st.current_path() {
            b"gml:boundedBy" => {
                // skip
                Ok(())
            }
            b"gml:boundedBy/gml:Envelope" => {
                let mut envelope = Envelope::default();
                envelope.parse(st)?;
                Ok(())
            }
            b"core:cityObjectMember" => {
                let mut cityobj: models::TopLevelCityObject = Default::default();
                cityobj.parse(st)?;
                let geometry_store = st.collect_geometries();

                if let Some(root) = cityobj.into_object() {
                    let entity = Entity {
                        root,
                        base_url: url::Url::parse("file:///dummy").unwrap(),
                        geometry_store: RwLock::new(geometry_store).into(),
                        appearance_store: Default::default(), // TODO: from local appearances
                    };
                    entities.push(entity);
                }
                Ok(())
            }
            b"app:appearanceMember" => {
                let mut app: models::appearance::AppearanceProperty = Default::default();
                app.parse(st)?;
                let models::appearance::AppearanceProperty::Appearance(app) = app else {
                    unreachable!();
                };
                global_appearances.update(app);
                Ok(())
            }
            other => Err(ParseError::SchemaViolation(format!(
                "Unrecognized element {}",
                String::from_utf8_lossy(other)
            ))),
        }
    })?;
    for entity in &entities {
        {
            let geom_store = entity.geometry_store.read().unwrap();
            entity.appearance_store.write().unwrap().merge_global(
                &mut global_appearances,
                &geom_store.ring_ids,
                &geom_store.surface_spans,
            );
        }
    }
    Ok(entities)
}

fn flatten_entity(
    value: Value,
    geom_store: Arc<RwLock<GeometryStore>>,
    appearance_store: Arc<RwLock<AppearanceStore>>,
    out: &mut Vec<Entity>,
    parent: &Option<Parent>,
) -> Option<Value> {
    match value {
        Value::Object(mut obj) => {
            let new_parent = match &obj.stereotype {
                ObjectStereotype::Feature { id, .. } => Some(Parent::Feature {
                    id: id.to_string(),
                    typename: obj.typename.to_string(),
                }),
                ObjectStereotype::Data => Some(Parent::Data {
                    typename: obj.typename.to_string(),
                }),
                ObjectStereotype::Object { id, .. } => Some(Parent::Object {
                    id: id.to_string(),
                    typename: obj.typename.to_string(),
                }),
            };

            // Attributes
            let mut new_attribs = Map::default();
            for (key, value) in obj.attributes.drain(..) {
                if let Some(v) = flatten_entity(
                    value,
                    geom_store.clone(),
                    appearance_store.clone(),
                    out,
                    &new_parent,
                ) {
                    new_attribs.insert(key, v);
                }
            }
            obj.attributes = new_attribs;

            if is_flatten_target(&obj) {
                // set parent id and type to attributes
                if let Some(parent) = parent {
                    match parent {
                        Parent::Feature { id, typename } => {
                            obj.attributes
                                .insert("parentId".to_string(), Value::String(id.to_string()));
                            obj.attributes.insert(
                                "parentType".to_string(),
                                Value::String(typename.to_string()),
                            );
                        }
                        Parent::Data { typename } => {
                            obj.attributes.insert(
                                "parentType".to_string(),
                                Value::String(typename.to_string()),
                            );
                        }
                        Parent::Object { id, typename } => {
                            obj.attributes
                                .insert("parentId".to_string(), Value::String(id.to_string()));
                            obj.attributes.insert(
                                "parentType".to_string(),
                                Value::String(typename.to_string()),
                            );
                        }
                    }
                }
                out.push(Entity {
                    root: Value::Object(obj),
                    base_url: url::Url::parse("file:///dummy").expect("should be valid"),
                    geometry_store: geom_store.clone(),
                    appearance_store: appearance_store.clone(),
                });
                return None;
            }

            Some(Value::Object(obj))
        }
        Value::Array(mut arr) => {
            let mut new_arr = Vec::with_capacity(arr.len());
            for value in arr.drain(..) {
                if let Some(v) = flatten_entity(
                    value,
                    geom_store.clone(),
                    appearance_store.clone(),
                    out,
                    parent,
                ) {
                    new_arr.push(v)
                }
            }
            if new_arr.is_empty() {
                None
            } else {
                Some(Value::Array(new_arr))
            }
        }
        _ => Some(value),
    }
}

fn is_flatten_target(obj: &Object) -> bool {
    if obj.typename == "gen:genericAttribute" {
        return false;
    }
    match obj.stereotype {
        ObjectStereotype::Feature { .. } => {
            !obj.typename.ends_with("Surface")
                && !obj.typename.ends_with(":Window")
                && !obj.typename.ends_with(":Door")
                && !obj.typename.ends_with("TrafficArea")
        }
        ObjectStereotype::Data => true,
        ObjectStereotype::Object { .. } => true,
    }
}
