use std::{
    io::{BufRead, BufReader, Cursor},
    sync::{Arc, RwLock},
};

use nusamai_citygml::{CityGmlElement, CityGmlReader, Envelope, ParseError, SubTreeReader};
use nusamai_plateau::{appearance::AppearanceStore, models, Entity};
use quick_xml::NsReader;
use reearth_flow_action::{error::Error, geometry::Geometry, ActionContext, Result};
use reearth_flow_common::uri::Uri;
use url::Url;

pub(crate) async fn read_citygml(input_path: Uri, ctx: ActionContext) -> Result<Vec<Geometry>> {
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

    let base_url: Url = input_path.into();
    let mut xml_reader = NsReader::from_reader(buf_reader);
    let context = nusamai_citygml::ParseContext::new(base_url.clone(), &code_resolver);
    let mut citygml_reader = CityGmlReader::new(context);
    let mut st = citygml_reader
        .start_root(&mut xml_reader)
        .map_err(Error::internal_runtime)?;
    let entities = parse_tree_reader(&mut st, base_url).map_err(Error::internal_runtime)?;
    let mut result = Vec::<Geometry>::new();
    for entity in entities {
        result.push(entity.try_into()?);
    }
    Ok(result)
}

fn parse_tree_reader<R: BufRead>(
    st: &mut SubTreeReader<R>,
    base_url: Url,
) -> Result<Vec<Entity>, ParseError> {
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
                        base_url: base_url.clone(),
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
