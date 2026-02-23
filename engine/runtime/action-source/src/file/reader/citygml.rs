use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Cursor},
    sync::RwLock,
};

use bytes::Bytes;
use nusamai_citygml::{CityGmlElement, CityGmlReader, Envelope, ParseError, SubTreeReader};
use nusamai_plateau::{
    appearance::AppearanceStore, models, Entity, FlattenTreeTransform, GeometricMergedownTransform,
};
use quick_xml::NsReader;
use reearth_flow_common::{str::to_hash, uri::Uri};
use reearth_flow_runtime::node::{IngestionMessage, Port, DEFAULT_PORT};
use reearth_flow_types::{
    geometry::Geometry, lod::LodMask, metadata::Metadata, Attribute, AttributeValue, Feature,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use url::Url;

/// # CityGmlReader Parameters
///
/// Configuration for reading CityGML files as a data source.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CityGmlReaderParam {
    pub(super) flatten: Option<bool>,
}

pub(crate) async fn read_citygml(
    content: &Bytes,
    input_path: Option<Uri>,
    params: &CityGmlReaderParam,
    sender: Sender<(Port, IngestionMessage)>,
) -> Result<(), crate::errors::SourceError> {
    let code_resolver = nusamai_plateau::codelist::Resolver::new();
    let cursor = Cursor::new(content);
    let buf_reader = BufReader::new(cursor);

    let base_url: Url = if let Some(input_path) = input_path {
        input_path.into()
    } else {
        Url::parse(".")
            .map_err(|e| crate::errors::SourceError::CityGmlFileReader(format!("{e:?}")))?
    };
    let mut xml_reader = NsReader::from_reader(buf_reader);
    let context = nusamai_citygml::ParseContext::new(base_url.clone(), &code_resolver);
    let mut citygml_reader = CityGmlReader::new(context);
    let mut st = citygml_reader
        .start_root(&mut xml_reader)
        .map_err(|e| crate::errors::SourceError::CityGmlFileReader(format!("{e:?}")))?;
    parse_tree_reader(&mut st, base_url, params.flatten.unwrap_or(false), sender)
        .await
        .map_err(|e| crate::errors::SourceError::CityGmlFileReader(format!("{e:?}")))?;
    Ok(())
}

async fn parse_tree_reader<R: BufRead>(
    st: &mut SubTreeReader<'_, '_, R>,
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
                if let Some(root) = cityobj.into_object() {
                    let entity = Entity {
                        id: Some(id.to_string()),
                        typename: Some(typename.to_string()),
                        root,
                        base_url: base_url.clone(),
                        geometry_store: RwLock::new(geometry_store).into(),
                        appearance_store: Default::default(),
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
    .map_err(|e| crate::errors::SourceError::CityGmlFileReader(format!("{e:?}")))?;
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
        let attributes = AttributeValue::from_nusamai_citygml_value(&entity.root);
        let city_gml_attributes = match attributes.len() {
            0 => AttributeValue::Null,
            1 => attributes.values().next().unwrap().clone(),
            _ => AttributeValue::Map(attributes),
        };
        let city_gml_attributes = city_gml_attributes.flatten();
        let gml_id = entity.root.id();
        let name = entity.root.typename();
        let attributes = HashMap::<Attribute, AttributeValue>::from([
            (Attribute::new("cityGmlAttributes"), city_gml_attributes),
            (
                Attribute::new("gmlName"),
                name.map(|s| AttributeValue::String(s.to_string()))
                    .unwrap_or(AttributeValue::Null),
            ),
            (
                Attribute::new("gmlId"),
                gml_id
                    .map(|s| AttributeValue::String(s.to_string()))
                    .unwrap_or(AttributeValue::Null),
            ),
            (
                Attribute::new("gmlRootId"),
                AttributeValue::String(format!("root_{}", to_hash(base_url.as_str()))),
            ),
        ]);
        let lod = LodMask::find_lods_by_citygml_value(&entity.root);
        let metadata = Metadata {
            feature_id: gml_id.map(|id| id.to_string()),
            feature_type: name.map(|name| name.to_string()),
            lod: Some(lod),
        };
        let entities = if flatten {
            FlattenTreeTransform::transform(entity)
        } else {
            vec![entity]
        };

        for mut ent in entities {
            transformer.transform(&mut ent);

            // Extract appearance data BEFORE try_into() consumes ent
            let appearance_member_data =
                convert_appearance_store_to_attribute_value(&ent.appearance_store.read().unwrap());

            let geometry: Geometry = ent
                .try_into()
                .map_err(|e| crate::errors::SourceError::CityGmlFileReader(format!("{e:?}")))?;
            let mut feature: Feature = geometry.into();
            feature.extend(attributes.clone());
            feature.metadata = metadata.clone();

            // Insert appearance member data
            feature.insert("appearanceMember", appearance_member_data);

            sender
                .send((
                    DEFAULT_PORT.clone(),
                    IngestionMessage::OperationEvent { feature },
                ))
                .await
                .map_err(|e| crate::errors::SourceError::CityGmlFileReader(format!("{e:?}")))?;
        }
    }
    Ok(())
}

// Helper function to convert AppearanceStore to AttributeValue
fn convert_appearance_store_to_attribute_value(
    appearance_store: &AppearanceStore,
) -> AttributeValue {
    use std::collections::HashMap;
    let mut appearance_attrs = HashMap::new();

    // Add textures if available
    if !appearance_store.textures.is_empty() {
        let textures: Vec<AttributeValue> = appearance_store
            .textures
            .iter()
            .enumerate()
            .map(|(idx, texture)| {
                let mut texture_map = HashMap::new();
                texture_map.insert(
                    "uri".to_string(),
                    AttributeValue::String(texture.image_url.to_string()),
                );

                // Build targets using surface_id_to_rings mapping
                // This ensures target.uri references the surface (Polygon) ID,
                // while the ring attribute references the ring (LinearRing) ID
                let mut all_targets = Vec::new();
                for (_theme_name, theme) in &appearance_store.themes {
                    // Iterate through surface-to-rings mapping
                    for (surface_id, ring_ids) in &theme.surface_id_to_rings {
                        for ring_id in ring_ids {
                            // Look up the texture index and coordinates for this ring
                            if let Some((tex_idx, line_string)) =
                                theme.ring_id_to_texture.get(ring_id)
                            {
                                if *tex_idx == idx as u32 {
                                    let mut target_map = HashMap::new();

                                    // Use SURFACE ID for target URI (correct!)
                                    let uri = format!("#{}", surface_id.0);
                                    target_map
                                        .insert("uri".to_string(), AttributeValue::String(uri));

                                    // Use RING ID for the ring attribute (correct!)
                                    target_map.insert(
                                        "ring".to_string(),
                                        AttributeValue::String(format!("#{}", ring_id.0)),
                                    );

                                    // Add texture coordinates from the line string
                                    let mut coord_strings = Vec::new();
                                    for point in line_string.iter() {
                                        coord_strings.push(format!("{} {}", point[0], point[1]));
                                    }
                                    if !coord_strings.is_empty() {
                                        let tex_coords: Vec<AttributeValue> = coord_strings
                                            .iter()
                                            .map(|coord| AttributeValue::String(coord.clone()))
                                            .collect();
                                        target_map.insert(
                                            "textureCoordinates".to_string(),
                                            AttributeValue::Array(tex_coords),
                                        );
                                    }

                                    all_targets.push(AttributeValue::Map(target_map));
                                }
                            }
                        }
                    }
                }

                if !all_targets.is_empty() {
                    texture_map.insert("targets".to_string(), AttributeValue::Array(all_targets));
                }

                AttributeValue::Map(texture_map)
            })
            .collect();
        appearance_attrs.insert("textures".to_string(), AttributeValue::Array(textures));
    }

    // Add materials if available
    if !appearance_store.materials.is_empty() {
        let materials: Vec<AttributeValue> = appearance_store
            .materials
            .iter()
            .map(|material| {
                let mut material_map = HashMap::new();
                // Add diffuse color
                let mut diffuse_color_map = HashMap::new();
                // Convert f64 to serde_json::Number using serde_json::Number::from_f64
                diffuse_color_map.insert(
                    "red".to_string(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(material.diffuse_color.r)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                diffuse_color_map.insert(
                    "green".to_string(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(material.diffuse_color.g)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                diffuse_color_map.insert(
                    "blue".to_string(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(material.diffuse_color.b)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                // nusamai_citygml::Color doesn't have alpha, so we'll use the red value as a placeholder
                diffuse_color_map.insert(
                    "alpha".to_string(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(material.diffuse_color.r)
                            .unwrap_or_else(|| serde_json::Number::from(1)),
                    ),
                ); // Default alpha to 1
                material_map.insert(
                    "diffuseColor".to_string(),
                    AttributeValue::Map(diffuse_color_map),
                );

                // Add specular color
                let mut specular_color_map = HashMap::new();
                specular_color_map.insert(
                    "red".to_string(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(material.specular_color.r)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                specular_color_map.insert(
                    "green".to_string(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(material.specular_color.g)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                specular_color_map.insert(
                    "blue".to_string(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(material.specular_color.b)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                // nusamai_citygml::Color doesn't have alpha, so we'll use the red value as a placeholder
                specular_color_map.insert(
                    "alpha".to_string(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(material.specular_color.r)
                            .unwrap_or_else(|| serde_json::Number::from(1)),
                    ),
                ); // Default alpha to 1
                material_map.insert(
                    "specularColor".to_string(),
                    AttributeValue::Map(specular_color_map),
                );

                // Add ambient intensity
                material_map.insert(
                    "ambientIntensity".to_string(),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(material.ambient_intensity)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );

                AttributeValue::Map(material_map)
            })
            .collect();
        appearance_attrs.insert("materials".to_string(), AttributeValue::Array(materials));
    }

    // Add themes if available
    if !appearance_store.themes.is_empty() {
        let themes: Vec<AttributeValue> = appearance_store
            .themes
            .iter()
            .map(|(theme_name, theme)| {
                let mut theme_map = HashMap::new();
                theme_map.insert(
                    "name".to_string(),
                    AttributeValue::String(theme_name.clone()),
                );

                // Add surface mappings if available
                if !theme.surface_id_to_material.is_empty() {
                    let surface_mappings: Vec<AttributeValue> = theme
                        .surface_id_to_material
                        .iter()
                        .map(|(surface_id, material_idx)| {
                            let mut mapping_map = HashMap::new();
                            mapping_map.insert(
                                "surfaceId".to_string(),
                                AttributeValue::String(surface_id.0.clone()),
                            );
                            mapping_map.insert(
                                "materialIndex".to_string(),
                                AttributeValue::Number((*material_idx).into()),
                            );
                            AttributeValue::Map(mapping_map)
                        })
                        .collect();
                    theme_map.insert(
                        "surfaceMappings".to_string(),
                        AttributeValue::Array(surface_mappings),
                    );
                }

                AttributeValue::Map(theme_map)
            })
            .collect();
        appearance_attrs.insert("themes".to_string(), AttributeValue::Array(themes));
    }

    AttributeValue::Map(appearance_attrs)
}
