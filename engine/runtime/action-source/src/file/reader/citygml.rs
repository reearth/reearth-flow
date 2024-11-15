use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Cursor},
    sync::{Arc, RwLock},
};

use nusamai_citygml::{CityGmlElement, CityGmlReader, Envelope, ParseError, SubTreeReader};
use nusamai_plateau::{
    appearance::AppearanceStore, models, Entity, FlattenTreeTransform, GeometricMergedownTransform,
};
use quick_xml::NsReader;
use reearth_flow_common::{str::to_hash, uri::Uri};
use reearth_flow_runtime::node::{IngestionMessage, Port, DEFAULT_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{
    geometry::Geometry, metadata::Metadata, Attribute, AttributeValue, Feature,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CityGmlReaderParam {
    pub(super) flatten: Option<bool>,
}

pub(crate) async fn read_citygml(
    input_path: Uri,
    params: &CityGmlReaderParam,
    storage_resolver: Arc<StorageResolver>,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let code_resolver = nusamai_plateau::codelist::Resolver::new();
    let storage = storage_resolver
        .resolve(&input_path)
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let result = storage
        .get(input_path.path().as_path())
        .await
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let byte = result
        .bytes()
        .await
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let cursor = Cursor::new(byte);
    let buf_reader = BufReader::new(cursor);

    let base_url: Url = input_path.into();
    let mut xml_reader = NsReader::from_reader(buf_reader);
    let context = nusamai_citygml::ParseContext::new(base_url.clone(), &code_resolver);
    let mut citygml_reader = CityGmlReader::new(context);
    let mut st = citygml_reader
        .start_root(&mut xml_reader)
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    parse_tree_reader(&mut st, base_url, params.flatten.unwrap_or(false), sender)
        .await
        .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    Ok(())
}

async fn parse_tree_reader<'a, 'b, R: BufRead>(
    st: &mut SubTreeReader<'a, 'b, R>,
    base_url: Url,
    flatten: bool,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let mut entities = Vec::new();
    let mut global_appearances = AppearanceStore::default();
    let mut envelope = Envelope::default();

    st.parse_children(|st| {
        let path: &[u8] = &st.current_path();
        match path {
            b"gml:boundedBy" => {
                // skip
                Ok(())
            }
            b"gml:boundedBy/gml:Envelope" => {
                envelope.parse(st)?;
                Ok(())
            }
            b"core:cityObjectMember" => {
                let mut cityobj: models::TopLevelCityObject = Default::default();
                cityobj.parse(st)?;
                let geometry_store = st.collect_geometries(envelope.crs_uri.clone());
                let id = cityobj.id();
                let typename = cityobj.name();
                let bounded_by = cityobj.bounded_by();
                if let Some(root) = cityobj.into_object() {
                    let entity = Entity {
                        id: Some(id.to_string()),
                        typename: Some(typename.to_string()),
                        root,
                        base_url: base_url.clone(),
                        geometry_store: RwLock::new(geometry_store).into(),
                        appearance_store: Default::default(),
                        bounded_by,
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
    })
    .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
    let mut transformer = GeometricMergedownTransform::new();
    for entity in entities {
        {
            let geom_store = entity.geometry_store.read().unwrap();
            entity.appearance_store.write().unwrap().merge_global(
                &mut global_appearances,
                &geom_store.ring_ids,
                &geom_store.surface_spans,
            );
        }
        {
            let mut geom_store = entity.geometry_store.write().unwrap();
            geom_store.vertices.iter_mut().for_each(|v| {
                // Swap x and y (lat, lng -> lng, lat)
                (v[0], v[1], v[2]) = (v[1], v[0], v[2]);
            });
        }
        let attributes = entity.root.to_attribute_json();
        let gml_id = entity
            .root
            .id()
            .map(|id| AttributeValue::String(id.to_string()))
            .unwrap_or(AttributeValue::Null);
        let name = entity
            .root
            .typename()
            .map(|name| AttributeValue::String(name.to_string()))
            .unwrap_or(AttributeValue::Null);
        let attributes = HashMap::<Attribute, AttributeValue>::from([
            (Attribute::new("cityGmlAttributes"), attributes.into()),
            (Attribute::new("gmlName"), name),
            (Attribute::new("gmlId"), gml_id),
            (
                Attribute::new("gmlRootId"),
                AttributeValue::String(format!("root_{}", to_hash(base_url.as_str()))),
            ),
        ]);
        let metadata = Metadata {
            feature_id: entity.root.id().map(|id| id.to_string()),
            feature_type: entity.root.typename().map(|name| name.to_string()),
        };
        let entities = if flatten {
            FlattenTreeTransform::transform(entity)
        } else {
            vec![entity]
        };

        for mut ent in entities {
            transformer.transform(&mut ent);
            let geometry: Geometry = ent
                .try_into()
                .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
            let mut feature: Feature = geometry.into();
            feature.extend(attributes.clone());
            feature.metadata = metadata.clone();
            sender
                .send((
                    DEFAULT_PORT.clone(),
                    IngestionMessage::OperationEvent { feature },
                ))
                .await
                .map_err(|e| crate::errors::SourceError::FileReader(format!("{:?}", e)))?;
        }
    }
    Ok(())
}
