use std::collections::HashMap;
#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

#[cfg(not(feature = "new-geometry"))]
use earcut::{utils3d::project3d_to_2d, Earcut};
use once_cell::sync::Lazy;
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_geometry::types::{
    coordinate::Coordinate3D,
    csg::{CSGChild, CSGOperation, CSG},
    face::Face,
    geometry::Geometry3D as FlowGeometry3D,
    polygon::Polygon3D,
    solid::Solid3D,
};
#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::{
    csg::{Csg, ThreeDimensional},
    Euclidean3DGeometry, Geometry,
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, REJECTED_PORT},
};
use reearth_flow_types::{
    Attribute, AttributeValue, Attributes, Code, CodeType, CompiledCode, Feature,
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Geometry, GeometryType, GeometryValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::super::errors::GeometryProcessorError;

static LEFT_PORT: Lazy<Port> = Lazy::new(|| Port::new("left"));
static RIGHT_PORT: Lazy<Port> = Lazy::new(|| Port::new("right"));
static INTERSECTION_PORT: Lazy<Port> = Lazy::new(|| Port::new("intersection"));
static UNION_PORT: Lazy<Port> = Lazy::new(|| Port::new("union"));
static DIFFERENCE_PORT: Lazy<Port> = Lazy::new(|| Port::new("difference"));

#[derive(Debug, Clone, Default)]
pub struct CSGBuilderFactory;

impl ProcessorFactory for CSGBuilderFactory {
    fn name(&self) -> &str {
        "CSG Builder"
    }

    fn description(&self) -> &str {
        "Pairs each left solid with the right solid that shares its pair value and emits the \
         union, the intersection and the difference of the pair as unevaluated Constructive \
         Solid Geometry trees. The trees describe the boolean without computing it, so a \
         CSG Evaluator downstream turns the branch you keep into a solid."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CSGBuilderParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["spatial", "3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![LEFT_PORT.clone(), RIGHT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            INTERSECTION_PORT.clone(),
            UNION_PORT.clone(),
            DIFFERENCE_PORT.clone(),
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
        let param: CSGBuilderParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::CSGBuilderFactory(format!(
                    "Failed to serialize 'with' parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::CSGBuilderFactory(format!(
                    "Failed to deserialize 'with' parameter: {e}"
                ))
            })?
        } else {
            return Err(GeometryProcessorError::CSGBuilderFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let pair_id = param.pair_id.compile().map_err(|e| {
            GeometryProcessorError::CSGBuilderFactory(format!(
                "Failed to compile pairId expression: {e:?}"
            ))
        })?;

        let processor = CSGBuilder {
            pair_id,
            left_buffer: HashMap::new(),
            right_buffer: HashMap::new(),
            list_attribute: param.list_attribute,
        };
        Ok(Box::new(processor))
    }
}

/// # CSG Builder Parameters
/// Sets how the two input streams are paired up and what the resulting trees
/// record about the solids they were built from.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct CSGBuilderParam {
    /// # Pair ID
    /// Expression evaluated on every feature to produce the value that pairs
    /// it up: a left feature and a right feature that evaluate to the same
    /// value are combined. A feature whose partner never arrives is rejected.
    pair_id: Code<{ CodeType::FlowExpr as u32 }>,

    /// # List Attribute
    /// Attribute that receives one entry for the left solid and one for the
    /// right, each holding that feature's own attributes. When omitted, no
    /// list is written and the resulting trees carry no attributes at all.
    list_attribute: Option<String>,
}

/// # CSG Builder
/// Builds the boolean trees of a paired left and right solid.
#[derive(Debug, Clone)]
pub struct CSGBuilder {
    pair_id: CompiledCode,
    left_buffer: HashMap<AttributeValue, Feature>,
    right_buffer: HashMap<AttributeValue, Feature>,
    list_attribute: Option<String>,
}

impl Processor for CSGBuilder {
    fn is_accumulating(&self) -> bool {
        false
    }

    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = ctx.feature.clone();
        let port = ctx.port.clone();

        // The value that pairs this feature with one from the other side.
        let Ok(pair_id) = self.pair_id.eval(&feature, ctx.variables.clone()) else {
            fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
            return Ok(());
        };

        // Check which port the feature came from and process accordingly
        if port == *LEFT_PORT {
            // Check if we already have a matching right feature
            if let Some(right_feature) = self.right_buffer.remove(&pair_id) {
                // We have a pair! Create CSG objects for all three operations
                self.create_and_send_csg(feature, right_feature, fw, &ctx)?;
            } else {
                // Store in left buffer waiting for its pair
                self.left_buffer.insert(pair_id, feature);
            }
        } else if port == *RIGHT_PORT {
            // Check if we already have a matching left feature
            if let Some(left_feature) = self.left_buffer.remove(&pair_id) {
                // We have a pair! Create CSG objects for all three operations
                self.create_and_send_csg(left_feature, feature, fw, &ctx)?;
            } else {
                // Store in right buffer waiting for its pair
                self.right_buffer.insert(pair_id, feature);
            }
        } else {
            // Unknown port, send to rejected
            fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
        }

        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Send all unpaired features to the rejected port
        for feature in self.left_buffer.values() {
            let exec_ctx = ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                REJECTED_PORT.clone(),
            );
            fw.send(exec_ctx);
        }

        for feature in self.right_buffer.values() {
            let exec_ctx = ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature.clone(),
                REJECTED_PORT.clone(),
            );
            fw.send(exec_ctx);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "CSG Builder"
    }
}

impl CSGBuilder {
    #[cfg(not(feature = "new-geometry"))]
    fn create_and_send_csg(
        &self,
        left_feature: Feature,
        right_feature: Feature,
        fw: &ProcessorChannelForwarder,
        ctx: &ExecutorContext,
    ) -> Result<(), BoxedError> {
        // Extract solid geometries from both features
        let left_solid = match &left_feature.geometry.value {
            GeometryValue::FlowGeometry3D(geom) => match geom {
                FlowGeometry3D::Solid(solid) => solid.clone(),
                _ => {
                    // Not a solid geometry, send both to rejected
                    fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                    fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                    return Ok(());
                }
            },
            GeometryValue::CityGmlGeometry(cg) => {
                let polygons: Vec<Polygon3D<f64>> = cg
                    .gml_geometries
                    .iter()
                    .filter(|gml_geometry| gml_geometry.ty == GeometryType::Solid)
                    .flat_map(|gml_geometry| gml_geometry.polygons.clone())
                    .collect();
                let faces = polygons_to_faces(&polygons);
                if faces.is_empty() {
                    // No solid faces found, send both to rejected
                    fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                    fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                    return Ok(());
                }
                Solid3D::new_with_faces(faces)
            }
            _ => {
                // Not a 3D geometry, send both to rejected
                fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        let right_solid = match &right_feature.geometry.value {
            GeometryValue::FlowGeometry3D(geom) => match geom {
                FlowGeometry3D::Solid(solid) => solid.clone(),
                _ => {
                    // Not a solid geometry, send both to rejected
                    fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                    fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                    return Ok(());
                }
            },
            GeometryValue::CityGmlGeometry(cg) => {
                let polygons: Vec<Polygon3D<f64>> = cg
                    .gml_geometries
                    .iter()
                    .filter(|gml_geometry| gml_geometry.ty == GeometryType::Solid)
                    .flat_map(|gml_geometry| gml_geometry.polygons.clone())
                    .collect();
                let faces = polygons_to_faces(&polygons);
                if faces.is_empty() {
                    // No solid faces found, send both to rejected
                    fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                    fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                    return Ok(());
                }
                Solid3D::new_with_faces(faces)
            }
            _ => {
                // Not a 3D geometry, send both to rejected
                fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
                fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        // Create CSGChild from solids
        let left_csg_child = CSGChild::Solid(left_solid);
        let right_csg_child = CSGChild::Solid(right_solid);

        let list_attribute = self.build_list_attribute(&left_feature, &right_feature);

        // Create and send intersection CSG
        let intersection_csg = CSG::new(
            left_csg_child.clone(),
            right_csg_child.clone(),
            CSGOperation::Intersection,
        );
        let mut intersection_feature = Feature::new_with_attributes(Attributes::new());
        intersection_feature.geometry = Arc::new(Geometry {
            epsg: left_feature.geometry.epsg,
            value: GeometryValue::FlowGeometry3D(FlowGeometry3D::CSG(Box::new(intersection_csg))),
        });

        // Add list attribute if created
        if let Some((attr_key, attr_value)) = &list_attribute {
            intersection_feature
                .attributes_mut()
                .insert(attr_key.clone(), attr_value.clone());
        }

        fw.send(ctx.new_with_feature_and_port(intersection_feature, INTERSECTION_PORT.clone()));

        // Create and send union CSG
        let union_csg = CSG::new(
            left_csg_child.clone(),
            right_csg_child.clone(),
            CSGOperation::Union,
        );
        let mut union_feature = Feature::new_with_attributes(Attributes::new());
        union_feature.geometry = Arc::new(Geometry {
            epsg: left_feature.geometry.epsg,
            value: GeometryValue::FlowGeometry3D(FlowGeometry3D::CSG(Box::new(union_csg))),
        });

        // Add list attribute if created
        if let Some((attr_key, attr_value)) = &list_attribute {
            union_feature
                .attributes_mut()
                .insert(attr_key.clone(), attr_value.clone());
        }

        fw.send(ctx.new_with_feature_and_port(union_feature, UNION_PORT.clone()));

        // Create and send difference CSG (left - right)
        let difference_csg = CSG::new(left_csg_child, right_csg_child, CSGOperation::Difference);
        let mut difference_feature = Feature::new_with_attributes(Attributes::new());
        difference_feature.geometry = Arc::new(Geometry {
            epsg: left_feature.geometry.epsg,
            value: GeometryValue::FlowGeometry3D(FlowGeometry3D::CSG(Box::new(difference_csg))),
        });

        // Add list attribute if created
        if let Some((attr_key, attr_value)) = &list_attribute {
            difference_feature
                .attributes_mut()
                .insert(attr_key.clone(), attr_value.clone());
        }

        fw.send(ctx.new_with_feature_and_port(difference_feature, DIFFERENCE_PORT.clone()));

        Ok(())
    }
}

#[cfg(feature = "new-geometry")]
impl CSGBuilder {
    /// Combine the paired features into the three boolean trees and send one
    /// feature per operation port. A pair whose members are not both solids or
    /// boolean trees is rejected.
    fn create_and_send_csg(
        &self,
        left_feature: Feature,
        right_feature: Feature,
        fw: &ProcessorChannelForwarder,
        ctx: &ExecutorContext,
    ) -> Result<(), BoxedError> {
        let (Some(left), Some(right)) = (
            csg_operand(left_feature.geometry.as_ref()),
            csg_operand(right_feature.geometry.as_ref()),
        ) else {
            fw.send(ctx.new_with_feature_and_port(left_feature, REJECTED_PORT.clone()));
            fw.send(ctx.new_with_feature_and_port(right_feature, REJECTED_PORT.clone()));
            return Ok(());
        };

        let list_attribute = self.build_list_attribute(&left_feature, &right_feature);

        let outputs = [
            (
                Csg::intersection(left.clone(), right.clone()),
                INTERSECTION_PORT.clone(),
            ),
            (Csg::union(left.clone(), right.clone()), UNION_PORT.clone()),
            (Csg::difference(left, right), DIFFERENCE_PORT.clone()),
        ];
        for (csg, port) in outputs {
            let mut feature = Feature::new_with_attributes(Attributes::new());
            *feature.geometry_mut() = Geometry::Euclidean3D(Euclidean3DGeometry::Csg(csg));
            if let Some((attr_key, attr_value)) = &list_attribute {
                feature
                    .attributes_mut()
                    .insert(attr_key.clone(), attr_value.clone());
            }
            fw.send(ctx.new_with_feature_and_port(feature, port));
        }

        Ok(())
    }
}

impl CSGBuilder {
    /// The list attribute holding both source features' attribute maps, when
    /// one was named.
    fn build_list_attribute(
        &self,
        left: &Feature,
        right: &Feature,
    ) -> Option<(Attribute, AttributeValue)> {
        let attr_name = self.list_attribute.as_ref()?;
        let attribute_objects = [left, right]
            .into_iter()
            .map(|feature| {
                let attrs: HashMap<String, AttributeValue> = feature
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect();
                AttributeValue::Map(attrs)
            })
            .collect();
        Some((
            Attribute::new(attr_name.clone()),
            AttributeValue::Array(attribute_objects),
        ))
    }
}

/// The geometry as a CSG operand: a solid, or an already-built boolean tree
/// (allowing trees to nest across builders).
#[cfg(feature = "new-geometry")]
fn csg_operand(geometry: &Geometry) -> Option<ThreeDimensional> {
    match geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Solid(solid)) => Some((**solid).clone().into()),
        Geometry::Euclidean3D(Euclidean3DGeometry::Csg(csg)) => Some(csg.clone().into()),
        _ => None,
    }
}

/// Convert CityGML polygons (which may have interior rings/holes) to Face objects.
/// Polygons without holes are converted directly from their exterior ring.
/// Polygons with holes are triangulated using earcut so the holes are respected.
#[cfg(not(feature = "new-geometry"))]
fn polygons_to_faces(polygons: &[Polygon3D<f64>]) -> Vec<Face> {
    let mut faces = Vec::new();
    let mut earcutter = Earcut::new();
    let mut buf3d: Vec<[f64; 3]> = Vec::new();
    let mut buf2d: Vec<[f64; 2]> = Vec::new();
    let mut index_buf: Vec<u32> = Vec::new();

    for polygon in polygons {
        if polygon.interiors().is_empty() {
            // No holes: use exterior ring directly as a face
            faces.push(polygon.exterior().clone().into());
        } else {
            // Has holes: triangulate with earcut to preserve the holes
            buf3d.clear();
            buf2d.clear();
            index_buf.clear();

            // Collect all coordinates: exterior ring first (without closing point),
            // then each interior ring (without closing points).
            // earcut expects implicitly-closed rings.
            let ext = polygon.exterior();
            let ext_count = if ext.is_closed() && ext.0.len() > 1 {
                ext.0.len() - 1
            } else {
                ext.0.len()
            };
            for c in &ext.0[..ext_count] {
                buf3d.push([c.x, c.y, c.z]);
            }
            let num_outer = ext_count;

            let mut hole_indices: Vec<u32> = Vec::new();
            for interior in polygon.interiors() {
                hole_indices.push(buf3d.len() as u32);
                let int_count = if interior.is_closed() && interior.0.len() > 1 {
                    interior.0.len() - 1
                } else {
                    interior.0.len()
                };
                for c in &interior.0[..int_count] {
                    buf3d.push([c.x, c.y, c.z]);
                }
            }

            // Project 3D coordinates to 2D for earcut
            if !project3d_to_2d(&buf3d, num_outer, &mut buf2d) {
                // Projection failed (degenerate polygon); fall back to exterior only
                faces.push(polygon.exterior().clone().into());
                continue;
            }

            earcutter.earcut(buf2d.iter().cloned(), &hole_indices, &mut index_buf);

            // Convert triangle indices back to Face objects
            let coords_3d: Vec<Coordinate3D<f64>> = buf3d
                .iter()
                .map(|&[x, y, z]| Coordinate3D::new__(x, y, z))
                .collect();

            for tri in index_buf.chunks_exact(3) {
                let i0 = tri[0] as usize;
                let i1 = tri[1] as usize;
                let i2 = tri[2] as usize;
                faces.push(Face::new(vec![
                    coords_3d[i0],
                    coords_3d[i1],
                    coords_3d[i2],
                    coords_3d[i0], // close the ring
                ]));
            }
        }
    }
    faces
}
