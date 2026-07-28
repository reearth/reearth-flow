use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use serde_json::Value;

#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Feature, GeometryValue};

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::RemoveAppearance;
#[cfg(feature = "new-geometry")]
use reearth_flow_types::Feature;

#[derive(Debug, Clone, Default)]
pub struct AppearanceRemoverFactory;

impl ProcessorFactory for AppearanceRemoverFactory {
    fn name(&self) -> &str {
        "Appearance Remover"
    }

    fn description(&self) -> &str {
        "Discards the materials, textures and texture coordinates carried by a feature's geometry."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["citygml", "3d"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(AppearanceRemover))
    }
}

#[derive(Debug, Clone)]
pub struct AppearanceRemover;

#[cfg(feature = "new-geometry")]
impl AppearanceRemover {
    /// A copy of `feature` whose geometry carries no appearance.
    fn remove(&self, feature: &Feature) -> Feature {
        let mut feature = feature.clone();
        feature.geometry_mut().remove_appearance();
        feature
    }
}

impl Processor for AppearanceRemover {
    #[cfg(not(feature = "new-geometry"))]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let feature = match &feature.geometry.value {
            GeometryValue::CityGmlGeometry(gml) => {
                let mut gml = gml.clone();
                gml.materials.clear();
                gml.textures.clear();
                gml.polygon_materials.clear();
                gml.polygon_textures.clear();
                gml.polygon_uvs.0.clear();

                let mut geometry = (*feature.geometry).clone();
                geometry.value = GeometryValue::CityGmlGeometry(gml);
                Feature {
                    geometry: Arc::new(geometry),
                    attributes: feature.attributes.clone(),
                    id: feature.id,
                }
            }
            // For non-CityGML geometry, pass through unchanged
            _ => feature.clone(),
        };

        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = self.remove(&ctx.feature);
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Appearance Remover"
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use reearth_flow_geometry::appearance::{Material, PhongMaterial, ThemeId};
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::polygon::Polygon3D;
    use reearth_flow_geometry::{Euclidean3DGeometry, Geometry};
    use reearth_flow_types::{Attribute, AttributeValue};
    use std::sync::Arc;

    fn material() -> Material {
        Material::Phong(PhongMaterial {
            diffuse: [1.0, 1.0, 1.0],
            specular: [0.0; 3],
            emissive: [0.0; 3],
            ambient_intensity: 0.0,
            shininess: 0.0,
            transparency: 0.0,
            diffuse_map: None,
            emissive_map: None,
            normal_map: None,
        })
    }

    /// A face carrying a single-material appearance.
    fn painted_face() -> Polygon3D {
        let mut face = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        face.set_appearance(ThemeId(Arc::from("rgb")), material(), None)
            .unwrap();
        face
    }

    #[test]
    fn the_appearance_is_gone_and_the_rest_of_the_feature_remains() {
        let face = painted_face();
        let mut feature = Feature::from(Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(
            Box::new(face.clone()),
        )));
        feature.insert(Attribute::new("name"), AttributeValue::String("a".into()));

        let stripped = AppearanceRemover.remove(&feature);
        let Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(polygon)) = &*stripped.geometry
        else {
            panic!("expected a 3D polygon");
        };
        assert!(polygon.appearance().is_none());
        assert_eq!(polygon.exterior(), face.exterior());
        assert_eq!(
            stripped.attributes.get(&Attribute::new("name")),
            Some(&AttributeValue::String("a".into()))
        );
        assert_eq!(stripped.id, feature.id);
    }

    #[test]
    fn a_geometry_without_appearance_passes_through_unchanged() {
        let feature = Feature::from(Geometry::None);
        assert_eq!(*AppearanceRemover.remove(&feature).geometry, Geometry::None);
    }
}
