use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use indexmap::IndexMap;
use once_cell::sync::Lazy;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use reearth_flow_common::dir::project_temp_dir;
use reearth_flow_geometry::algorithm::line_intersection::LineIntersection;
use reearth_flow_geometry::algorithm::line_string_ops::{
    LineStringOps, LineStringSplitResult, LineStringWithTree2D,
};
use reearth_flow_geometry::algorithm::GeoFloat;
use reearth_flow_geometry::types::coordinate::Coordinate2D;
use reearth_flow_geometry::types::geometry::Geometry2D;
use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::no_value::NoValue;
use reearth_flow_geometry::types::point::Point;
use reearth_flow_runtime::node::REJECTED_PORT;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

use super::errors::GeometryProcessorError;

pub static POINT_PORT: Lazy<Port> = Lazy::new(|| Port::new("point"));
pub static LINE_PORT: Lazy<Port> = Lazy::new(|| Port::new("line"));

#[derive(Debug, Clone, Default)]
pub struct LineOnLineOverlayerFactory;

impl ProcessorFactory for LineOnLineOverlayerFactory {
    fn name(&self) -> &str {
        "LineOnLineOverlayer"
    }

    fn description(&self) -> &str {
        "Intersection points are turned into point features that can contain the merged list of attributes of the original intersected lines."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(LineOnLineOverlayerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![POINT_PORT.clone(), LINE_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: LineOnLineOverlayerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::LineOnLineOverlayerFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::LineOnLineOverlayerFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::LineOnLineOverlayerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        Ok(Box::new(LineOnLineOverlayer {
            group_by: params.group_by,
            tolerance: params.tolerance,
            overlaid_lists_attr_name: params
                .overlaid_lists_attr_name
                .unwrap_or_else(|| "overlaidLists".to_string()),
            group_map: HashMap::new(),
            group_count: 0,
            current_writer: None,
            temp_dir: None,
        }))
    }
}

/// # LineOnLineOverlayer Parameters
///
/// Configuration for finding intersection points between line features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LineOnLineOverlayerParam {
    group_by: Option<Vec<Attribute>>,
    tolerance: f64,
    /// Name of the attribute to store the overlaid lists. Defaults to "overlaidLists".
    overlaid_lists_attr_name: Option<String>,
}

#[allow(clippy::type_complexity)]
pub struct LineOnLineOverlayer {
    group_by: Option<Vec<Attribute>>,
    tolerance: f64,
    overlaid_lists_attr_name: String,
    // Disk-backed state
    group_map: HashMap<AttributeValue, usize>,
    group_count: usize,
    current_writer: Option<(usize, BufWriter<File>)>, // (group_idx, writer) - only ONE open at a time
    temp_dir: Option<PathBuf>,
}

impl std::fmt::Debug for LineOnLineOverlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineOnLineOverlayer")
            .field("group_count", &self.group_count)
            .finish_non_exhaustive()
    }
}

impl Clone for LineOnLineOverlayer {
    fn clone(&self) -> Self {
        Self {
            group_by: self.group_by.clone(),
            tolerance: self.tolerance,
            overlaid_lists_attr_name: self.overlaid_lists_attr_name.clone(),
            group_map: HashMap::new(),
            group_count: 0,
            current_writer: None,
            temp_dir: None,
        }
    }
}

impl Drop for LineOnLineOverlayer {
    fn drop(&mut self) {
        if let Some(ref dir) = self.temp_dir {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

impl LineOnLineOverlayer {
    fn ensure_temp_dir(&mut self) -> Result<&PathBuf, BoxedError> {
        if self.temp_dir.is_none() {
            let dir = project_temp_dir(uuid::Uuid::new_v4().to_string().as_str())?;
            self.temp_dir = Some(dir);
        }
        Ok(self.temp_dir.as_ref().unwrap())
    }

    fn group_file_path(&self, group_idx: usize) -> PathBuf {
        self.temp_dir
            .as_ref()
            .unwrap()
            .join(format!("group_{group_idx:06}.jsonl"))
    }

    fn close_current_writer(&mut self) -> Result<(), BoxedError> {
        if let Some((_, mut writer)) = self.current_writer.take() {
            writer.flush()?;
        }
        Ok(())
    }

    fn ensure_writer_for_group(&mut self, group_idx: usize) -> Result<(), BoxedError> {
        // If we already have this group's writer open, nothing to do
        if let Some((current_idx, _)) = &self.current_writer {
            if *current_idx == group_idx {
                return Ok(());
            }
        }

        // Close current writer before opening new one
        self.close_current_writer()?;

        // Open new writer (append mode)
        let path = self.group_file_path(group_idx);
        let file = File::options().create(true).append(true).open(path)?;
        self.current_writer = Some((group_idx, BufWriter::new(file)));
        Ok(())
    }

    fn write_feature(&mut self, group_idx: usize, feature: &Feature) -> Result<(), BoxedError> {
        self.ensure_writer_for_group(group_idx)?;
        if let Some((_, ref mut writer)) = self.current_writer {
            serde_json::to_writer(&mut *writer, feature)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
        Ok(())
    }

    fn read_features_for_group(&self, group_idx: usize) -> Result<Vec<Feature>, BoxedError> {
        let path = self.group_file_path(group_idx);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut features = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if !line.is_empty() {
                features.push(serde_json::from_str(&line)?);
            }
        }
        Ok(features)
    }

    fn overlay_group(&self, features: &[Feature]) -> Result<OverlayedFeatures, BoxedError> {
        let buffered_features_2d: Vec<&Feature> = features
            .iter()
            .filter(|f| matches!(&f.geometry.value, GeometryValue::FlowGeometry2D(_)))
            .collect();
        self.overlay_2d(buffered_features_2d)
    }

    fn overlay_2d(&self, features_2d: Vec<&Feature>) -> Result<OverlayedFeatures, BoxedError> {
        // mapping from line string index to feature index
        let mut map_ls_to_features = Vec::new();
        let line_strings = features_2d
            .iter()
            .enumerate()
            .filter_map(|(i, f)| f.geometry.value.as_flow_geometry_2d().map(|f| (i, f)))
            .filter_map(|(i, g)| {
                if let Geometry2D::LineString(line) = g {
                    map_ls_to_features.push(i);
                    Some(vec![line.clone()])
                } else if let Geometry2D::MultiLineString(multi_line) = g {
                    // Add one entry for each line in the multi-line
                    for _ in 0..multi_line.0.len() {
                        map_ls_to_features.push(i);
                    }
                    Some(multi_line.0.clone())
                } else if let Geometry2D::Polygon(polygon) = g {
                    // Extract exterior ring as LineString
                    map_ls_to_features.push(i);
                    Some(vec![polygon.exterior().clone()])
                } else if let Geometry2D::MultiPolygon(multi_polygon) = g {
                    // Extract all exterior rings as LineStrings
                    // Add one entry for each polygon in the multi-polygon
                    for _ in 0..multi_polygon.0.len() {
                        map_ls_to_features.push(i);
                    }
                    Some(
                        multi_polygon
                            .0
                            .iter()
                            .map(|p| p.exterior().clone())
                            .collect(),
                    )
                } else {
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        let line_string_intersection_result =
            line_string_intersection_2d(&line_strings, self.tolerance);

        let mut overlaid = OverlayedFeatures::new();

        for ls in &line_string_intersection_result.line_strings_with_metadata {
            let LineString2DWithMetadata {
                line_string: result_ls,
                overlay_count,
                overlay_ids,
            } = ls;
            let mut feature = Feature::new_with_attributes(Attributes::new());
            feature.attributes_mut().insert(
                Attribute::new("overlayCount"),
                AttributeValue::Number(Number::from(*overlay_count)),
            );
            feature.attributes_mut().insert(
                Attribute::new(&self.overlaid_lists_attr_name),
                AttributeValue::Array(
                    overlay_ids
                        .iter()
                        .map(|&id| {
                            AttributeValue::Map(
                                features_2d[map_ls_to_features[id]]
                                    .attributes
                                    .iter()
                                    .map(|(k, v)| (k.inner(), v.clone()))
                                    .collect::<HashMap<_, _>>(),
                            )
                        })
                        .collect::<Vec<_>>(),
                ),
            );

            // Add common attributes. These are attributes that are listed in `group_by` and exist in all overlaid features.
            if let Some(group_by) = &self.group_by {
                let attr = &features_2d[map_ls_to_features[overlay_ids[0]]].attributes;
                for group_by in group_by {
                    if let Some(value) = attr.get(group_by) {
                        feature
                            .attributes_mut()
                            .insert(group_by.clone(), value.clone());
                    } else {
                        return Err(Box::new(
                            GeometryProcessorError::LineOnLineOverlayerFactory(
                                "Group by attribute not found in feature".to_string(),
                            ),
                        ));
                    }
                }
            };
            feature.geometry_mut().value =
                GeometryValue::FlowGeometry2D(Geometry2D::LineString(result_ls.clone()));
            overlaid.line.push(feature);
        }

        let last_feature = features_2d.last().unwrap();

        for result_coords in line_string_intersection_result.split_coords {
            let attrs = if let Some(group_by) = &self.group_by {
                group_by
                    .iter()
                    .filter_map(|attr| {
                        let value = last_feature.get(attr).cloned()?;
                        Some((attr.clone(), value))
                    })
                    .collect::<IndexMap<_, _>>()
            } else {
                IndexMap::new()
            };
            let mut feature = Feature::new_with_attributes(attrs);

            feature.geometry_mut().value =
                GeometryValue::FlowGeometry2D(Geometry2D::Point(Point(result_coords)));
            overlaid.point.push(feature);
        }

        Ok(overlaid)
    }
}

impl Processor for LineOnLineOverlayer {
    fn is_accumulating(&self) -> bool {
        true
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
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
                self.ensure_temp_dir()?;

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

                let group_idx = if let Some(&idx) = self.group_map.get(&key) {
                    idx
                } else {
                    let idx = self.group_count;
                    self.group_map.insert(key, idx);
                    self.group_count += 1;
                    idx
                };

                self.write_feature(group_idx, feature)?;
            }
            _ => {
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Close current writer
        self.close_current_writer()?;

        // Process each group one at a time to minimize memory usage
        for group_idx in 0..self.group_count {
            let features = self.read_features_for_group(group_idx)?;
            if features.is_empty() {
                continue;
            }

            let overlaid = self.overlay_group(&features)?;

            for feature in overlaid.line {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature,
                    LINE_PORT.clone(),
                ));
            }
            for feature in overlaid.point {
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature,
                    POINT_PORT.clone(),
                ));
            }
            // Features for this group are dropped here, freeing memory before next group
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "LineOnLineOverlayer"
    }
}

struct OverlayedFeatures {
    point: Vec<Feature>,
    line: Vec<Feature>,
}

impl OverlayedFeatures {
    fn new() -> Self {
        Self {
            point: Vec::new(),
            line: Vec::new(),
        }
    }
}

/// Coordinates of the intersection point to output
#[derive(Debug, Clone)]
struct OverlayResultCoordinates {
    i_by: usize,
    i_other: usize,
    coordinates: Coordinate2D<f64>,
}

#[derive(Debug, Clone)]
struct LineString2DWithMetadata<T: GeoFloat> {
    line_string: LineString2D<T>,
    /// Number of original line strings that contributed to this line string
    overlay_count: usize,
    /// Indices of original line strings that contributed to this line string
    /// It must be of size `overlay_count`
    overlay_ids: Vec<usize>,
}

/// Result of overlaying line strings
#[derive(Debug, Clone)]
struct OverlayResult {
    line_strings_with_metadata: Vec<LineString2DWithMetadata<f64>>,
    split_coords: Vec<Coordinate2D<f64>>,
}

/// Calculate the intersection between line strings
fn line_string_intersection_2d(
    line_strings: &[LineString2D<f64>],
    tolerance: f64,
) -> OverlayResult {
    let results = line_strings.par_iter().enumerate().map(|(i, line_string)| {
        let packed_line_string = LineStringWithTree2D::new(line_string.clone());

        struct IntersectionWithIndex {
            i_by: usize,
            i_other: usize,
            intersection: LineIntersection<f64, NoValue>,
        }

        let inters_with_index = (0..line_strings.len())
            .filter(|&j| j != i)
            .map(|j| (j, &line_strings[j]))
            .filter_map(|(j, other_line_string)| {
                let intersections = packed_line_string.intersection(other_line_string);
                if intersections.is_empty() {
                    return None;
                }

                let inters = intersections
                    .into_iter()
                    .map(|intersection| IntersectionWithIndex {
                        i_by: i,
                        i_other: j,
                        intersection,
                    })
                    .collect::<Vec<_>>();

                Some(inters)
            })
            .collect::<Vec<_>>();

        let split_coords_with_index = inters_with_index
            .iter()
            .flatten()
            .flat_map(|inter| {
                let coords = match inter.intersection {
                    LineIntersection::SinglePoint { intersection, .. } => vec![intersection],
                    LineIntersection::Collinear { intersection } => {
                        // treat collinear as points
                        vec![intersection.start, intersection.end]
                    }
                };

                coords
                    .into_iter()
                    .map(|coordinates| OverlayResultCoordinates {
                        coordinates,
                        i_by: inter.i_by,
                        i_other: inter.i_other,
                    })
            })
            .collect::<Vec<_>>();

        let split_coords = split_coords_with_index
            .iter()
            .map(|split| split.coordinates)
            .collect::<Vec<_>>();

        let LineStringSplitResult {
            split_line_strings,
            split_coords,
        } = packed_line_string.split(&split_coords, tolerance);

        let split_line_strings_with_indices = split_line_strings
            .into_iter()
            .map(|l| (i, l))
            .collect::<Vec<_>>();

        let split_coords_with_indices = split_coords
            .iter()
            .map(|&v| {
                let indices = split_coords_with_index
                    .iter()
                    .filter(|&o| (v - o.coordinates).norm() < tolerance)
                    .flat_map(|o| [o.i_by, o.i_other])
                    .collect::<Vec<_>>();
                (v, indices)
            })
            .collect::<Vec<_>>();

        (split_line_strings_with_indices, split_coords_with_indices)
    });

    let (result_line_strings, split_coords_with_indices): (Vec<_>, Vec<_>) = results.unzip();
    let line_strings_with_indices = result_line_strings
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let split_coords_with_indices = split_coords_with_indices
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // We have all the intersection points and line strings now.
    // What remains is to compute duplicates and overlay counts.
    let mut line_strings_with_metadata = Vec::new();
    let mut processed = vec![false; line_strings_with_indices.len()];
    for (i, (idx1, ls1)) in line_strings_with_indices.iter().enumerate() {
        if processed[i] {
            continue;
        }
        let mut overlay_count = 1; // itself
        let mut overlay_ids = vec![*idx1];
        for j in i + 1..line_strings_with_indices.len() {
            let ls2 = &line_strings_with_indices[j].1;
            let idx2 = &line_strings_with_indices[j].0;
            if ls1
                .0
                .iter()
                .zip(ls2.0.iter())
                .all(|(&c1, &c2)| (c1 - c2).norm() < tolerance)
                || ls1
                    .0
                    .iter()
                    .rev()
                    .zip(ls2.0.iter())
                    .all(|(&c1, &c2)| (c1 - c2).norm() < tolerance)
            {
                overlay_count += 1;
                overlay_ids.push(*idx2);
                processed[j] = true;
            }
        }
        let ls_with_metadata = LineString2DWithMetadata {
            line_string: ls1.clone(),
            overlay_count,
            overlay_ids,
        };
        line_strings_with_metadata.push(ls_with_metadata);
    }

    // Split coordinate duplicates
    let mut unique_split_coords = Vec::new();
    let mut processed_coords = vec![false; split_coords_with_indices.len()];
    for (i, coord1) in split_coords_with_indices.iter().enumerate() {
        if processed_coords[i] {
            continue;
        }
        unique_split_coords.push(coord1.clone());
        for (j, coord2) in split_coords_with_indices.iter().enumerate().skip(i + 1) {
            if (coord1.0 - coord2.0).norm() < tolerance {
                processed_coords[j] = true;
                // Merge the line string indices
                unique_split_coords.last_mut().unwrap().1.extend(&coord2.1);
            }
        }
    }

    OverlayResult {
        line_strings_with_metadata,
        split_coords: unique_split_coords.into_iter().map(|(c, _)| c).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay() {
        // Test the overlay functionality
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(5.0, 5.0),
        ]);
        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 5.0),
            Coordinate2D::new_(5.0, 0.0),
        ]);

        let overlay_result = line_string_intersection_2d(&[line_string1, line_string2], 0.1);

        // Assert the overlay result
        let OverlayResult {
            line_strings_with_metadata,
            split_coords,
        } = overlay_result;
        assert_eq!(line_strings_with_metadata.len(), 4);
        assert_eq!(split_coords.len(), 1);
        let split_coord = &split_coords[0];
        assert!((split_coord.x - 2.5).abs() < 1e-6);
        assert!((split_coord.y - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_overlay_duplicate_lines() {
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(4.0, 4.0),
        ]);

        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(1.0, 1.0),
            Coordinate2D::new_(4.0, 4.0),
        ]);

        let line_string3 = LineString2D::new(vec![
            Coordinate2D::new_(2.0, 2.0),
            Coordinate2D::new_(3.0, 3.0),
        ]);
        let overlay_result =
            line_string_intersection_2d(&[line_string1, line_string2, line_string3], 0.1);
        let OverlayResult {
            line_strings_with_metadata,
            split_coords,
        } = overlay_result;

        assert_eq!(line_strings_with_metadata.len(), 4);
        let mut overlay_counts = line_strings_with_metadata
            .iter()
            .map(|ls| ls.overlay_count)
            .collect::<Vec<_>>();
        overlay_counts.sort();
        assert_eq!(overlay_counts, vec![1, 2, 2, 3]);
        assert_eq!(split_coords.len(), 3);
    }

    #[test]
    fn test_overlay_two_squares() {
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(4.0, 4.0),
            Coordinate2D::new_(4.0, 0.0),
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(0.0, 4.0),
            Coordinate2D::new_(4.0, 4.0),
        ]);
        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(6.0, 6.0),
            Coordinate2D::new_(2.0, 6.0),
            Coordinate2D::new_(2.0, 2.0),
            Coordinate2D::new_(6.0, 2.0),
            Coordinate2D::new_(6.0, 6.0),
        ]);
        let overlay_result = line_string_intersection_2d(&[line_string1, line_string2], 0.1);
        let OverlayResult {
            line_strings_with_metadata,
            split_coords: _,
        } = overlay_result;

        assert_eq!(line_strings_with_metadata.len(), 6);
    }

    #[test]
    fn test_overlay_k_like_lines() {
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(0.0, 4.0),
        ]);
        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(2.0, 4.0),
            Coordinate2D::new_(0.0, 2.0),
            Coordinate2D::new_(2.0, 0.0),
        ]);

        let overlay_result = line_string_intersection_2d(&[line_string1, line_string2], 0.1);

        // Assert the overlay result
        let OverlayResult {
            line_strings_with_metadata,
            split_coords,
        } = overlay_result;
        assert_eq!(line_strings_with_metadata.len(), 4);
        assert!(line_strings_with_metadata
            .iter()
            .all(|ls| ls.overlay_count == 1));
        assert_eq!(split_coords.len(), 1);
    }

    #[test]
    fn test_overlay_adjacent_triangles() {
        let line_string1 = LineString2D::new(vec![
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(0.0, 1.0),
            Coordinate2D::new_(1.0, 0.0),
        ]);
        let line_string2 = LineString2D::new(vec![
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(1.0, 1.0),
            Coordinate2D::new_(0.0, 1.0),
            Coordinate2D::new_(1.0, 0.0),
        ]);
        let line_string3 = LineString2D::new(vec![
            Coordinate2D::new_(1.0, 0.0),
            Coordinate2D::new_(1.0, 1.0),
            Coordinate2D::new_(2.0, 1.0),
            Coordinate2D::new_(1.0, 0.0),
        ]);

        let overlay_result =
            line_string_intersection_2d(&[line_string1, line_string2, line_string3], 0.1);

        // Assert the overlay result
        let OverlayResult {
            line_strings_with_metadata,
            split_coords,
        } = overlay_result;
        assert_eq!(line_strings_with_metadata.len(), 5);
        let mut overlap_counts = line_strings_with_metadata
            .iter()
            .map(|ls| ls.overlay_count)
            .collect::<Vec<_>>();
        overlap_counts.sort();
        assert_eq!(overlap_counts, vec![1, 1, 1, 2, 2]);
        assert_eq!(split_coords.len(), 2);
    }
}
