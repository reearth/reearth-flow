use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::PlateauProcessorError;

#[derive(Debug, Clone)]
struct FileHistogram {
    gml_file_path: String,
    room: i64,
    door: i64,
    ground_surface: i64,
    wall_surface: i64,
    building_furniture: i64,
    outer_floor_surface: i64,
    building_installation: i64,
    floor_surface: i64,
    window: i64,
    building: i64,
    closure_surface: i64,
    ceiling_surface: i64,
    roof_surface: i64,
    building_part: i64,
    interior_wall_surface: i64,
    int_building_installation: i64,
    outer_ceiling_surface: i64,
}

#[derive(Debug, Clone, Default)]
pub struct InstanceHistogramCreatorFactory;

impl ProcessorFactory for InstanceHistogramCreatorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.InstanceHistogramCreator"
    }

    fn description(&self) -> &str {
        "Creates instance histogram for PLATEAU4 building features"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(InstanceHistogramCreatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let _params: InstanceHistogramCreatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::InstanceHistogramCreatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::InstanceHistogramCreatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            InstanceHistogramCreatorParam::default()
        };

        let process = InstanceHistogramCreator {
            file_histograms: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct InstanceHistogramCreatorParam {}

impl Default for InstanceHistogramCreatorParam {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone)]
struct InstanceHistogramCreator {
    file_histograms: HashMap<String, FileHistogram>,
}

impl Processor for InstanceHistogramCreator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        // Get gmlPath for aggregation
        let gml_path = if let Some(AttributeValue::String(path)) = feature.get(&"gmlPath") {
            path.clone()
        } else {
            return Ok(()); // Skip if no gmlPath
        };

        // Get cityGmlAttributes
        let attributes_value = feature.get(&"cityGmlAttributes");
        let attributes = if let Some(AttributeValue::Map(attr_map)) = attributes_value {
            attr_map
        } else {
            return Ok(()); // Skip if no cityGmlAttributes
        };

        // Get or create histogram for this file
        let histogram = self
            .file_histograms
            .entry(gml_path.clone())
            .or_insert_with(|| FileHistogram {
                gml_file_path: gml_path.clone(),
                room: 0,
                door: 0,
                ground_surface: 0,
                wall_surface: 0,
                building_furniture: 0,
                outer_floor_surface: 0,
                building_installation: 0,
                floor_surface: 0,
                window: 0,
                building: 0,
                closure_surface: 0,
                ceiling_surface: 0,
                roof_surface: 0,
                building_part: 0,
                interior_wall_surface: 0,
                int_building_installation: 0,
                outer_ceiling_surface: 0,
            });

        // Aggregate counts
        histogram.room += count_rooms(attributes);
        histogram.door += count_doors(attributes);
        histogram.ground_surface += count_ground_surfaces(attributes);
        histogram.wall_surface += count_wall_surfaces(attributes);
        histogram.building_furniture += count_building_furniture(attributes);
        histogram.outer_floor_surface += count_outer_floor_surfaces(attributes);
        histogram.building_installation += count_building_installations(attributes);
        histogram.floor_surface += count_floor_surfaces(attributes);
        histogram.window += count_windows(attributes);
        histogram.closure_surface += count_closure_surfaces(attributes);
        histogram.ceiling_surface += count_ceiling_surfaces(attributes);
        histogram.roof_surface += count_roof_surfaces(attributes);
        histogram.building_part += count_building_parts(attributes);
        histogram.interior_wall_surface += count_interior_wall_surfaces(attributes);
        histogram.int_building_installation += count_int_building_installations(attributes);
        histogram.outer_ceiling_surface += count_outer_ceiling_surfaces(attributes);

        // Count Building instances
        if let Some(AttributeValue::String(gml_name)) = feature.get(&"gmlName") {
            if gml_name == "bldg:Building" {
                histogram.building += 1;
            }
        }

        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        // Output aggregated histograms
        for histogram in self.file_histograms.values() {
            let mut feature = Feature::new();

            // Add counts with the expected column names and order
            feature.insert(
                Attribute::new("ClosureSurface".to_string()),
                AttributeValue::Number(histogram.closure_surface.into()),
            );
            feature.insert(
                Attribute::new("IntBuildingInstallation".to_string()),
                AttributeValue::Number(histogram.int_building_installation.into()),
            );
            feature.insert(
                Attribute::new("GroundSurface".to_string()),
                AttributeValue::Number(histogram.ground_surface.into()),
            );
            feature.insert(
                Attribute::new("RoofSurface".to_string()),
                AttributeValue::Number(histogram.roof_surface.into()),
            );
            feature.insert(
                Attribute::new("OuterFloorSurface".to_string()),
                AttributeValue::Number(histogram.outer_floor_surface.into()),
            );
            feature.insert(
                Attribute::new("Door".to_string()),
                AttributeValue::Number(histogram.door.into()),
            );
            feature.insert(
                Attribute::new("BuildingPart".to_string()),
                AttributeValue::Number(histogram.building_part.into()),
            );
            feature.insert(
                Attribute::new("BuildingFurniture".to_string()),
                AttributeValue::Number(histogram.building_furniture.into()),
            );
            feature.insert(
                Attribute::new("Building".to_string()),
                AttributeValue::Number(histogram.building.into()),
            );
            feature.insert(
                Attribute::new("Room".to_string()),
                AttributeValue::Number(histogram.room.into()),
            );
            feature.insert(
                Attribute::new("GmlFilePath".to_string()),
                AttributeValue::String(histogram.gml_file_path.clone()),
            );
            feature.insert(
                Attribute::new("BuildingInstallation".to_string()),
                AttributeValue::Number(histogram.building_installation.into()),
            );
            feature.insert(
                Attribute::new("InteriorWallSurface".to_string()),
                AttributeValue::Number(histogram.interior_wall_surface.into()),
            );
            feature.insert(
                Attribute::new("Window".to_string()),
                AttributeValue::Number(histogram.window.into()),
            );
            feature.insert(
                Attribute::new("FloorSurface".to_string()),
                AttributeValue::Number(histogram.floor_surface.into()),
            );
            feature.insert(
                Attribute::new("OuterCeilingSurface".to_string()),
                AttributeValue::Number(histogram.outer_ceiling_surface.into()),
            );
            feature.insert(
                Attribute::new("WallSurface".to_string()),
                AttributeValue::Number(histogram.wall_surface.into()),
            );
            feature.insert(
                Attribute::new("CeilingSurface".to_string()),
                AttributeValue::Number(histogram.ceiling_surface.into()),
            );

            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "InstanceHistogramCreator"
    }
}

// Helper functions to count instances
fn count_rooms(attributes: &HashMap<String, AttributeValue>) -> i64 {
    let mut count = 0;
    if let Some(rooms) = attributes
        .get("bldg:Room")
        .or_else(|| attributes.get("bldg:interiorRoom"))
    {
        match rooms {
            AttributeValue::Array(arr) => count = arr.len() as i64,
            _ => count = 1,
        }
    }
    count
}

fn count_doors(attributes: &HashMap<String, AttributeValue>) -> i64 {
    search_nested_count(attributes, "bldg:Door")
}

fn count_windows(attributes: &HashMap<String, AttributeValue>) -> i64 {
    search_nested_count(attributes, "bldg:Window")
}

fn search_nested_count(obj: &HashMap<String, AttributeValue>, target_name: &str) -> i64 {
    let mut count = 0;

    for (key, value) in obj {
        if key == target_name {
            match value {
                AttributeValue::Array(arr) => count += arr.len() as i64,
                AttributeValue::Null => {}
                _ => count += 1,
            }
        } else {
            match value {
                AttributeValue::Map(nested_map) => {
                    count += search_nested_count(nested_map, target_name);
                }
                AttributeValue::Array(arr) => {
                    for item in arr {
                        if let AttributeValue::Map(item_map) = item {
                            count += search_nested_count(item_map, target_name);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    count
}

fn count_ground_surfaces(attributes: &HashMap<String, AttributeValue>) -> i64 {
    count_direct_attribute(attributes, "bldg:GroundSurface")
}

fn count_wall_surfaces(attributes: &HashMap<String, AttributeValue>) -> i64 {
    count_direct_attribute(attributes, "bldg:WallSurface")
}

fn count_building_furniture(attributes: &HashMap<String, AttributeValue>) -> i64 {
    let mut count = 0;
    if let Some(rooms) = attributes
        .get("bldg:Room")
        .or_else(|| attributes.get("bldg:interiorRoom"))
    {
        match rooms {
            AttributeValue::Array(arr) => {
                for room in arr {
                    if let AttributeValue::Map(room_map) = room {
                        count += count_direct_attribute(room_map, "bldg:BuildingFurniture");
                        count += count_direct_attribute(room_map, "bldg:interiorFurniture");
                    }
                }
            }
            AttributeValue::Map(room_map) => {
                count += count_direct_attribute(room_map, "bldg:BuildingFurniture");
                count += count_direct_attribute(room_map, "bldg:interiorFurniture");
            }
            _ => {}
        }
    }
    count
}

fn count_outer_floor_surfaces(attributes: &HashMap<String, AttributeValue>) -> i64 {
    count_direct_attribute(attributes, "bldg:OuterFloorSurface")
}

fn count_building_installations(attributes: &HashMap<String, AttributeValue>) -> i64 {
    let mut count = count_direct_attribute(attributes, "bldg:BuildingInstallation");
    count += count_direct_attribute(attributes, "bldg:outerBuildingInstallation");
    count
}

fn count_floor_surfaces(attributes: &HashMap<String, AttributeValue>) -> i64 {
    let mut count = 0;
    if let Some(rooms) = attributes
        .get("bldg:Room")
        .or_else(|| attributes.get("bldg:interiorRoom"))
    {
        match rooms {
            AttributeValue::Array(arr) => {
                for room in arr {
                    if let AttributeValue::Map(room_map) = room {
                        count += count_direct_attribute(room_map, "bldg:FloorSurface");
                    }
                }
            }
            AttributeValue::Map(room_map) => {
                count += count_direct_attribute(room_map, "bldg:FloorSurface");
            }
            _ => {}
        }
    }
    count
}

fn count_closure_surfaces(attributes: &HashMap<String, AttributeValue>) -> i64 {
    count_direct_attribute(attributes, "bldg:ClosureSurface")
}

fn count_ceiling_surfaces(attributes: &HashMap<String, AttributeValue>) -> i64 {
    let mut count = 0;
    if let Some(rooms) = attributes
        .get("bldg:Room")
        .or_else(|| attributes.get("bldg:interiorRoom"))
    {
        match rooms {
            AttributeValue::Array(arr) => {
                for room in arr {
                    if let AttributeValue::Map(room_map) = room {
                        count += count_direct_attribute(room_map, "bldg:CeilingSurface");
                    }
                }
            }
            AttributeValue::Map(room_map) => {
                count += count_direct_attribute(room_map, "bldg:CeilingSurface");
            }
            _ => {}
        }
    }
    count
}

fn count_roof_surfaces(attributes: &HashMap<String, AttributeValue>) -> i64 {
    count_direct_attribute(attributes, "bldg:RoofSurface")
}

fn count_building_parts(attributes: &HashMap<String, AttributeValue>) -> i64 {
    count_direct_attribute(attributes, "bldg:BuildingPart")
}

fn count_interior_wall_surfaces(attributes: &HashMap<String, AttributeValue>) -> i64 {
    let mut count = 0;
    if let Some(rooms) = attributes
        .get("bldg:Room")
        .or_else(|| attributes.get("bldg:interiorRoom"))
    {
        match rooms {
            AttributeValue::Array(arr) => {
                for room in arr {
                    if let AttributeValue::Map(room_map) = room {
                        count += count_direct_attribute(room_map, "bldg:InteriorWallSurface");
                    }
                }
            }
            AttributeValue::Map(room_map) => {
                count += count_direct_attribute(room_map, "bldg:InteriorWallSurface");
            }
            _ => {}
        }
    }
    count
}

fn count_int_building_installations(attributes: &HashMap<String, AttributeValue>) -> i64 {
    let mut count = 0;
    if let Some(rooms) = attributes
        .get("bldg:Room")
        .or_else(|| attributes.get("bldg:interiorRoom"))
    {
        match rooms {
            AttributeValue::Array(arr) => {
                for room in arr {
                    if let AttributeValue::Map(room_map) = room {
                        count += count_direct_attribute(room_map, "bldg:IntBuildingInstallation");
                        count +=
                            count_direct_attribute(room_map, "bldg:interiorBuildingInstallation");
                    }
                }
            }
            AttributeValue::Map(room_map) => {
                count += count_direct_attribute(room_map, "bldg:IntBuildingInstallation");
                count += count_direct_attribute(room_map, "bldg:interiorBuildingInstallation");
            }
            _ => {}
        }
    }
    count
}

fn count_outer_ceiling_surfaces(attributes: &HashMap<String, AttributeValue>) -> i64 {
    count_direct_attribute(attributes, "bldg:OuterCeilingSurface")
}

fn count_direct_attribute(attributes: &HashMap<String, AttributeValue>, attr_name: &str) -> i64 {
    if let Some(value) = attributes.get(attr_name) {
        match value {
            AttributeValue::Array(arr) => arr.len() as i64,
            AttributeValue::Null => 0,
            _ => 1,
        }
    } else {
        0
    }
}
