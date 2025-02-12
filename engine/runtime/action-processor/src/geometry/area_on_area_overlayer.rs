use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_geometry::{
    algorithm::bool_ops::BooleanOps, types::multi_polygon::MultiPolygon2D,
};
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static AREA_PORT: Lazy<Port> = Lazy::new(|| Port::new("area"));
pub static REMNANTS_PORT: Lazy<Port> = Lazy::new(|| Port::new("remnants"));

#[derive(Debug, Clone, Default)]
pub struct AreaOnAreaOverlayerFactory;

impl ProcessorFactory for AreaOnAreaOverlayerFactory {
    fn name(&self) -> &str {
        "AreaOnAreaOverlayer"
    }

    fn description(&self) -> &str {
        "Overlays an area on another area"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AreaOnAreaOverlayerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            AREA_PORT.clone(),
            REMNANTS_PORT.clone(),
            REJECTED_PORT.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: AreaOnAreaOverlayerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to serialize 'with' parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::AreaOnAreaOverlayerFactory(format!(
                    "Failed to deserialize 'with' parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::AreaOnAreaOverlayerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = AreaOnAreaOverlayer {
            group_by: param.group_by,
            buffer: HashMap::new(),
        };

        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AreaOnAreaOverlayerParam {
    group_by: Option<Vec<Attribute>>,
}

#[derive(Debug, Clone)]
pub struct AreaOnAreaOverlayer {
    group_by: Option<Vec<Attribute>>,
    buffer: HashMap<AttributeValue, Vec<Feature>>,
}

impl Processor for AreaOnAreaOverlayer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let geometry = &feature.geometry;
        if geometry.is_empty() {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), REJECTED_PORT.clone()));
            return Ok(());
        }
        match &geometry.value {
            GeometryValue::None => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
            GeometryValue::FlowGeometry2D(_) => {
                let key = if let Some(group_by) = &self.group_by {
                    AttributeValue::Array(
                        group_by
                            .iter()
                            .filter_map(|attr| feature.attributes.get(attr).cloned())
                            .collect(),
                    )
                } else {
                    AttributeValue::Null
                };

                if !self.buffer.contains_key(&key) {
                    let overlayed = self.overlay();
                    for feature in &overlayed.area {
                        fw.send(ctx.new_with_feature_and_port(feature.clone(), AREA_PORT.clone()));
                    }
                    for feature in &overlayed.remnant {
                        fw.send(
                            ctx.new_with_feature_and_port(feature.clone(), REMNANTS_PORT.clone()),
                        );
                    }
                    self.buffer.clear();
                }

                self.buffer
                    .entry(key.clone())
                    .or_default()
                    .push(feature.clone());
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let overlayed = self.overlay();
        for feature in &overlayed.area {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                AREA_PORT.clone(),
            ));
        }
        for feature in &overlayed.remnant {
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                REMNANTS_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "AreaOnAreaOverlayer"
    }
}

#[derive(Debug, Clone)]
struct SubPolygon {
    polygon: MultiPolygon2D<f64>,
    parents: Vec<usize>,
}

enum SubPolygonType {
    None,
    Area(Vec<usize>),
    Remnant(usize),
}

impl SubPolygon {
    fn get_type(&self) -> SubPolygonType {
        match self.parents.len() {
            0 => SubPolygonType::None,
            1 => SubPolygonType::Remnant(self.parents[0]),
            _ => SubPolygonType::Area(self.parents.clone()),
        }
    }
}

impl AreaOnAreaOverlayer {
    fn overlay(&self) -> OverlayedFeatures {
        let mut overlayed = OverlayedFeatures::new();
        for buffer in self.buffer.values() {
            let buffered_features_2d = buffer
                .iter()
                .filter(|f| matches!(&f.geometry.value, GeometryValue::FlowGeometry2D(_)))
                .collect::<Vec<_>>();
            overlayed.extend(self.overlay_2d(buffered_features_2d));
        }
        overlayed
    }

    fn overlay_2d(&self, buffered_features_2d: Vec<&Feature>) -> OverlayedFeatures {
        let polygons_incoming = buffered_features_2d
            .iter()
            .filter_map(|f| f.geometry.value.as_flow_geometry_2d())
            .filter_map(|g| {
                g.as_polygon()
                    .map(|polygon| MultiPolygon2D::new(vec![polygon]))
            })
            .collect::<Vec<_>>();

        let polygon_mbrs = polygons_incoming
            .iter()
            .map(|polygon| polygon.bounding_box())
            .collect::<Vec<_>>();

        let mut subpolygons: Vec<SubPolygon> = Vec::new();

        for i in 0..polygons_incoming.len() {
            let polygon_incoming = &polygons_incoming[i];
            let polygon_mbr = if let Some(polygon_mbr) = &polygon_mbrs[i] {
                polygon_mbr
            } else {
                continue;
            };

            let mut new_subpolygons = Vec::new();

            if subpolygons.is_empty() {
                new_subpolygons.push(SubPolygon {
                    polygon: polygon_incoming.clone(),
                    parents: vec![i],
                });
            } else {
                for overlayed in &subpolygons {
                    let overlayed_mbr =
                        if let Some(overlayed_mbr) = overlayed.polygon.bounding_box() {
                            overlayed_mbr
                        } else {
                            continue;
                        };
                    if polygon_mbr.overlap(&overlayed_mbr) {
                        let intersected = polygon_incoming.intersection(&overlayed.polygon);
                        new_subpolygons.push(SubPolygon {
                            polygon: intersected,
                            parents: overlayed
                                .parents
                                .clone()
                                .into_iter()
                                .chain(vec![i])
                                .collect(),
                        });
                        let difference = overlayed.polygon.difference(polygon_incoming);
                        new_subpolygons.push(SubPolygon {
                            polygon: difference,
                            parents: overlayed.parents.clone(),
                        });
                    } else {
                        new_subpolygons.push(overlayed.clone());
                    }
                }

                let mut rest = polygon_incoming.clone();

                for j in 0..i {
                    let mbr = &polygon_mbrs[j];
                    if mbr.is_none() || !polygon_mbr.overlap(&mbr.unwrap()) {
                        continue;
                    }
                    rest = rest.difference(&polygons_incoming[j]);
                }
                new_subpolygons.push(SubPolygon {
                    polygon: rest,
                    parents: vec![i],
                });
            }
            subpolygons = new_subpolygons;
        }

        OverlayedFeatures::from_subpolygons(
            subpolygons,
            buffered_features_2d
                .iter()
                .map(|f| f.attributes.clone())
                .collect(),
            &self.group_by,
        )
    }
}

struct OverlayedFeatures {
    area: Vec<Feature>,
    remnant: Vec<Feature>,
}

impl OverlayedFeatures {
    fn new() -> Self {
        Self {
            area: Vec::new(),
            remnant: Vec::new(),
        }
    }

    fn from_subpolygons(
        subpolygons: Vec<SubPolygon>,
        base_attributes: Vec<HashMap<Attribute, AttributeValue>>,
        group_by: &Option<Vec<Attribute>>,
    ) -> Self {
        let mut area = Vec::new();
        let mut remnant = Vec::new();
        for subpolygon in subpolygons {
            match subpolygon.get_type() {
                SubPolygonType::None => {}
                SubPolygonType::Area(parents) => {
                    let mut feature = Feature::new();
                    let last_feature = &base_attributes[*parents.last().unwrap()];
                    if let Some(group_by) = group_by {
                        feature.attributes = group_by
                            .iter()
                            .filter_map(|attr| {
                                let value = last_feature.get(attr).cloned()?;
                                Some((attr.clone(), value))
                            })
                            .collect::<HashMap<_, _>>();
                    } else {
                        feature.attributes = HashMap::new();
                    }

                    feature.geometry.value =
                        GeometryValue::FlowGeometry2D(subpolygon.polygon.into());
                    area.push(feature);
                }
                SubPolygonType::Remnant(parent) => {
                    let mut feature = Feature::new();
                    feature.attributes = base_attributes[parent].clone();
                    feature.geometry.value =
                        GeometryValue::FlowGeometry2D(subpolygon.polygon.into());
                    remnant.push(feature);
                }
            }
        }
        Self { area, remnant }
    }

    fn extend(&mut self, other: Self) {
        self.area.extend(other.area);
        self.remnant.extend(other.remnant);
    }
}
