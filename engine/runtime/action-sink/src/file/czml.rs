use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;
use std::vec;

use bytes::Bytes;
use indexmap::IndexMap;
use itertools::Itertools;
use nusamai_czml::{
    CzmlBoolean, CzmlPolygon, Packet, PositionList, PositionListOfLists,
    PositionListOfListsProperties, PositionListProperties, StringProperties, StringValueType,
};
use rayon::iter::{ParallelBridge, ParallelIterator};
use reearth_flow_common::str::to_hash;
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::types::polygon::Polygon3D;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{Context, ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::SinkError;

#[derive(Debug, Clone, Default)]
pub(crate) struct CzmlWriterFactory;

impl SinkFactory for CzmlWriterFactory {
    fn name(&self) -> &str {
        "CzmlWriter"
    }

    fn description(&self) -> &str {
        "Export Features as CZML for Cesium Visualization"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CzmlWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::CzmlWriterFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::CzmlWriterFactory(format!("Failed to deserialize `with` parameter: {e}"))
            })?
        } else {
            return Err(SinkError::CzmlWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let sink = CzmlWriter {
            params,
            buffer: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CzmlWriter {
    pub(super) params: CzmlWriterParam,
    pub(super) buffer: HashMap<AttributeValue, Vec<Feature>>,
}

/// # CzmlWriter Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CzmlWriterParam {
    /// # Output File Path
    /// Path where the CZML file will be written
    pub(super) output: Expr,
    /// # Group By Attributes
    /// Attributes used to group features into separate CZML files
    pub(super) group_by: Option<Vec<Attribute>>,
}

impl Sink for CzmlWriter {
    fn name(&self) -> &str {
        "CzmlWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let key = if let Some(group_by) = &self.params.group_by {
            let key = group_by
                .iter()
                .map(|k| feature.get(&k).cloned().unwrap_or(AttributeValue::Null))
                .collect::<Vec<_>>();
            AttributeValue::Array(key)
        } else {
            AttributeValue::Null
        };
        self.buffer.entry(key).or_default().push(feature.clone());
        Ok(())
    }
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let output = self.params.output.clone();
        let scope = expr_engine.new_scope();
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
        let output = Uri::from_str(path.as_str())?;

        for (key, features) in self.buffer.iter() {
            let file_path = if *key == AttributeValue::Null {
                output.clone()
            } else {
                output.join(format!("{}.json", to_hash(key.to_string().as_str())))?
            };
            let storage = storage_resolver
                .resolve(&file_path)
                .map_err(crate::errors::SinkError::file_writer)?;
            let (sender, receiver) = std::sync::mpsc::sync_channel(1000);
            let gctx = ctx.as_context();

            let (ra, rb) = rayon::join(
                || {
                    // Convert Entity to CzmlPolygon objects
                    features
                        .iter()
                        .par_bridge()
                        .try_for_each_with(sender, |sender, feature| {
                            let packets = feature_to_packets(&gctx, feature);
                            for packet in packets {
                                let bytes = serde_json::to_vec(&packet).unwrap();
                                if sender.send(bytes).is_err() {
                                    return Err(SinkError::CzmlWriter(
                                        "Failed to send packet".to_string(),
                                    ));
                                };
                            }
                            Ok(())
                        })
                },
                || {
                    let doc = Packet {
                        id: Some("document".into()),
                        version: Some("1.0".into()),
                        ..Default::default()
                    };
                    let mut buffer =
                        Vec::from(format!(r#"[{},"#, serde_json::to_string(&doc).unwrap()));

                    let mut iter = receiver.into_iter().peekable();
                    while let Some(bytes) = iter.next() {
                        buffer
                            .write(&bytes)
                            .map_err(crate::errors::SinkError::czml_writer)?;
                        if iter.peek().is_some() {
                            buffer
                                .write(b",")
                                .map_err(crate::errors::SinkError::czml_writer)?;
                        };
                    }

                    // Write the FeautureCollection footer and EOL
                    buffer
                        .write(b"]\n")
                        .map_err(crate::errors::SinkError::czml_writer)?;
                    storage
                        .put_sync(file_path.path().as_path(), Bytes::from(buffer))
                        .map_err(crate::errors::SinkError::czml_writer)
                },
            );
            ra?;
            rb?;
        }
        Ok(())
    }
}

fn map_to_html_table(map: &IndexMap<Attribute, AttributeValue>) -> String {
    let mut html = String::new();
    html.push_str("<table>");
    for (key, value) in map {
        let value: serde_json::Value = value.clone().into();
        html.push_str(&format!("<tr><td>{key}</td><td>{value}</td></tr>"));
    }
    html.push_str("</table>");
    html
}

fn polygon_to_czml_polygon(poly: &Polygon3D<f64>) -> CzmlPolygon {
    let mut czml_polygon = CzmlPolygon::default();

    let exteriors = poly
        .exterior()
        .iter()
        .flat_map(|coord| vec![coord.x, coord.y, coord.z])
        .collect_vec();
    czml_polygon.positions = Some(PositionList::Object(PositionListProperties {
        cartographic_degrees: Some(exteriors),
        ..Default::default()
    }));

    let interiors = poly
        .interiors()
        .iter()
        .flat_map(|line| line.iter().map(|coord| vec![coord.x, coord.y, coord.z]))
        .collect_vec();
    czml_polygon.holes = Some(PositionListOfLists::Object(PositionListOfListsProperties {
        cartographic_degrees: Some(interiors),
        ..Default::default()
    }));

    czml_polygon
}

fn feature_to_packets(ctx: &Context, feature: &Feature) -> Vec<Packet> {
    let Some(parent_id) = feature.metadata.feature_id.clone() else {
        ctx.event_hub
            .warn_log(None, "Feature does not have a feature_id".to_string());
        return vec![];
    };

    let properties = map_to_html_table(&feature.attributes);

    let GeometryValue::CityGmlGeometry(geometry) = &feature.geometry.value else {
        ctx.event_hub.warn_log(
            None,
            format!(
                "Geometry is not a CityGML geometry with: feature_id={}",
                feature.id
            ),
        );
        return vec![];
    };

    let polygons = geometry
        .gml_geometries
        .iter()
        .filter(|geometry| geometry.lod.unwrap_or(0) > 0)
        .flat_map(|geometry| geometry.polygons.clone())
        .collect_vec();

    if polygons.is_empty() {
        ctx.event_hub.warn_log(
            None,
            format!(
                "Geometry does not contain any polygons: feature_id={}",
                feature.id
            ),
        );
        return vec![];
    }

    // Create a Packet that retains attributes and references it from child features
    let properties_packet = Packet {
        id: Some(parent_id.clone()),
        description: Some(StringValueType::String(properties)),
        ..Default::default()
    };
    let mut packets: Vec<Packet> = vec![properties_packet];

    for poly in polygons {
        let mut czml_polygon = polygon_to_czml_polygon(&poly);
        // In Cesium, if perPositionHeight is false, the polygon height is fixed
        czml_polygon.per_position_height = CzmlBoolean::Boolean(true);

        let packet = Packet {
            polygon: Some(czml_polygon),
            description: Some(StringValueType::Object(StringProperties {
                reference: Some(format!("{parent_id}#description")),
                ..Default::default()
            })),
            parent: Some(parent_id.clone()),
            ..Default::default()
        };
        packets.push(packet);
    }

    packets
}
