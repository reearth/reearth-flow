//! Polygon slicing algorithm based on [geojson-vt](https://github.com/mapbox/geojson-vt).

use std::collections::HashMap;

use indexmap::IndexSet;
use reearth_flow_types::{Attribute, AttributeValue};
use serde::{Deserialize, Serialize};

use super::material::Material;

#[derive(Serialize, Deserialize)]
pub(crate) struct SlicedFeature {
    pub(crate) typename: String,
    // polygons [x, y, z, u, v]
    pub polygons: nusamai_geometry::MultiPolygon<'static, [f64; 5]>,
    // material ids for each polygon
    pub(crate) polygon_material_ids: Vec<u32>,
    // materials
    pub(crate) materials: IndexSet<Material>,
    // attribute values
    pub(crate) attributes: HashMap<Attribute, AttributeValue>,
}
