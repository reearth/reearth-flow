use std::collections::HashMap;

use nusamai_citygml::GeometryRef;
use nusamai_citygml::{object::ObjectStereotype, GeometryType, Value};
use nusamai_plateau::Entity;
use reearth_flow_geometry::types::coordinate::Coordinate3D;
use reearth_flow_geometry::types::line_string::LineString3D;
use reearth_flow_geometry::types::polygon::Polygon3D;

use crate::error::Error;
use crate::{AttributeValue, CityGmlGeometry, Geometry, GeometryValue, GmlGeometry};

impl TryFrom<Entity> for Geometry {
    type Error = Error;

    fn try_from(entity: Entity) -> Result<Self, Self::Error> {
        let apperance = entity.appearance_store.read().unwrap();
        let theme = {
            apperance
                .themes
                .get("rgbTexture")
                .or_else(|| apperance.themes.get("FMETheme"))
        };
        let geoms = entity.geometry_store.read().unwrap();
        let apperance = entity.appearance_store.read().unwrap();
        let epsg = geoms.epsg;
        // entity must be a Feature
        let Value::Object(obj) = &entity.root else {
            return Err(Error::unsupported_feature("no object found"));
        };
        let ObjectStereotype::Feature { id: _, geometries } = &obj.stereotype else {
            return Err(Error::unsupported_feature("no feature found"));
        };
        let mut gml_geometries = Vec::<GmlGeometry>::new();
        let operation = |geometry: &GeometryRef| -> Option<GmlGeometry> {
            match geometry.ty {
                GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
                    let mut polygons = Vec::<Polygon3D<f64>>::new();
                    for idx_poly in geoms
                        .multipolygon
                        .iter_range(geometry.pos as usize..(geometry.pos + geometry.len) as usize)
                    {
                        let poly = idx_poly.transform(|c| geoms.vertices[*c as usize]);
                        polygons.push(poly.into());
                    }
                    let mut geometry_feature = GmlGeometry::from(geometry.clone());
                    geometry_feature.polygons.extend(polygons);
                    Some(geometry_feature)
                }
                GeometryType::Curve => {
                    let mut linestrings = Vec::<LineString3D<f64>>::new();
                    for idx_linestring in geoms
                        .multilinestring
                        .iter_range(geometry.pos as usize..(geometry.pos + geometry.len) as usize)
                    {
                        let linestring = idx_linestring.transform(|c| geoms.vertices[*c as usize]);
                        // manually collect coordinates instead of using line_string.into()
                        // This avoids iter_closed() which would incorrectly close the linestring
                        linestrings.push(
                            linestring
                                .iter()
                                .map(|a| Coordinate3D::new__(a[0], a[1], a[2]))
                                .collect(),
                        );
                    }
                    let mut geometry_feature = GmlGeometry::from(geometry.clone());
                    geometry_feature.line_strings.extend(linestrings);
                    Some(geometry_feature)
                }
                GeometryType::Point => unimplemented!(),
            }
        };
        gml_geometries.extend(geometries.iter().flat_map(operation));

        let mut geometry_entity = CityGmlGeometry::new(
            gml_geometries,
            apperance
                .materials
                .iter()
                .cloned()
                .map(Into::into)
                .collect(),
            apperance.textures.iter().cloned().map(Into::into).collect(),
        );

        if let Some(theme) = theme {
            // find and apply materials
            {
                let mut poly_materials = vec![None; geoms.multipolygon.len()];
                for surface in &geoms.surface_spans {
                    if let Some(&mat) = theme.surface_id_to_material.get(&surface.id) {
                        for idx in surface.start..surface.end {
                            poly_materials[idx as usize] = Some(mat);
                        }
                    }
                }
                geometry_entity.polygon_materials = poly_materials;
            }
            // find and apply textures
            {
                let mut ring_id_iter = geoms.ring_ids.iter();
                let mut poly_textures = Vec::with_capacity(geoms.multipolygon.len());
                let mut poly_uvs = flatgeom::MultiPolygon::new();

                for poly in &geoms.multipolygon {
                    for (i, ring) in poly.rings().enumerate() {
                        let tex = ring_id_iter
                            .next()
                            .unwrap()
                            .clone()
                            .and_then(|ring_id| theme.ring_id_to_texture.get(&ring_id));

                        let mut add_dummy_texture = || {
                            let uv = [[0.0, 0.0]].into_iter().cycle().take(ring.len() + 1);
                            if i == 0 {
                                poly_textures.push(None);
                                poly_uvs.add_exterior(uv);
                            } else {
                                poly_uvs.add_interior(uv);
                            }
                        };

                        match tex {
                            Some((idx, uv)) if ring.len() == uv.len() => {
                                // texture found
                                if i == 0 {
                                    poly_textures.push(Some(*idx));
                                    poly_uvs.add_exterior(uv.iter_closed());
                                } else {
                                    poly_uvs.add_interior(uv.iter_closed());
                                }
                            }
                            Some((_, uv)) if uv.len() != ring.len() => {
                                // invalid texture found
                                add_dummy_texture();
                            }
                            _ => {
                                // no texture found
                                add_dummy_texture();
                            }
                        };
                    }
                }
                // apply textures to polygons
                geometry_entity.polygon_textures = poly_textures;
                geometry_entity.polygon_uvs = poly_uvs;
            }
        } else {
            // set 'null' appearance if no theme found
            geometry_entity.polygon_materials = vec![None; geoms.multipolygon.len()];
            geometry_entity.polygon_textures = vec![None; geoms.multipolygon.len()];
            let mut poly_uvs = flatgeom::MultiPolygon::new();
            for poly in &geoms.multipolygon {
                for (i, ring) in poly.rings().enumerate() {
                    let uv = [[0.0, 0.0]].into_iter().cycle().take(ring.len() + 1);
                    if i == 0 {
                        poly_uvs.add_exterior(uv);
                    } else {
                        poly_uvs.add_interior(uv);
                    }
                }
            }
            geometry_entity.polygon_uvs = poly_uvs;
        }
        Ok(Self::new_with(
            epsg,
            GeometryValue::CityGmlGeometry(geometry_entity),
        ))
    }
}

impl AttributeValue {
    pub fn from_nusamai_cityml_value(
        value: &nusamai_citygml::object::Value,
    ) -> HashMap<String, AttributeValue> {
        Self::from_key_and_nusamai_cityml_value(None, value)
    }

    pub(crate) fn from_key_and_nusamai_cityml_value(
        key: Option<String>,
        value: &nusamai_citygml::object::Value,
    ) -> HashMap<String, AttributeValue> {
        match value {
            nusamai_citygml::object::Value::Object(obj) => Self::handle_object(obj),
            nusamai_citygml::object::Value::Array(arr) => Self::handle_array(arr),
            nusamai_citygml::object::Value::Code(code) => Self::handle_code(key, code),
            _ => Self::handle_simple_value(key, value),
        }
    }

    fn handle_object(obj: &nusamai_citygml::object::Object) -> HashMap<String, AttributeValue> {
        let mut result = HashMap::new();
        let value = Self::process_object_attributes(&obj.attributes);
        Self::merge_into_result(&mut result, &obj.typename, AttributeValue::Map(value));
        result
    }

    fn process_object_attributes(
        attributes: &nusamai_citygml::object::Map,
    ) -> HashMap<String, AttributeValue> {
        attributes
            .iter()
            .map(|(k, v)| Self::process_attribute(k, v))
            .fold(HashMap::new(), Self::merge_attribute_maps)
    }

    fn process_attribute(
        key: &str,
        value: &nusamai_citygml::Value,
    ) -> HashMap<String, AttributeValue> {
        let mut result = HashMap::new();
        match value {
            nusamai_citygml::Value::Code(v) => {
                result.insert(
                    key.to_string(),
                    AttributeValue::String(v.value().to_owned()),
                );
                result.insert(
                    format!("{key}_code"),
                    AttributeValue::String(v.code().to_owned()),
                );
            }
            nusamai_citygml::Value::Measure(v) => {
                // Convert the value losslessly
                let value = serde_json::Number::from_string_unchecked(v.value().to_string());
                let numeric_value = AttributeValue::Number(value);
                result.insert(key.to_string(), numeric_value);

                // If uom exists, create a companion _uom attribute (FME compatible)
                if let Some(uom) = v.uom() {
                    result.insert(format!("{key}_uom"), AttributeValue::String(uom.to_owned()));
                }
            }
            nusamai_citygml::Value::Array(arr) => {
                Self::process_array_attribute(&mut result, key, arr);
            }
            _ => {
                result.insert(key.to_string(), AttributeValue::from(value.clone()));
            }
        }
        result
    }

    fn process_array_attribute(
        result: &mut HashMap<String, AttributeValue>,
        key: &str,
        arr: &[nusamai_citygml::object::Value],
    ) {
        for value in arr {
            let target = Self::from_key_and_nusamai_cityml_value(Some(key.to_string()), value);
            for (key, value) in target {
                Self::merge_into_result(result, &key, value);
            }
        }
    }

    fn handle_array(arr: &[nusamai_citygml::object::Value]) -> HashMap<String, AttributeValue> {
        arr.iter()
            .map(Self::from_nusamai_cityml_value)
            .fold(HashMap::new(), Self::merge_attribute_maps)
    }

    fn handle_code(
        key: Option<String>,
        code: &nusamai_citygml::Code,
    ) -> HashMap<String, AttributeValue> {
        let mut result = HashMap::new();
        if let Some(key) = key {
            result.insert(key.clone(), AttributeValue::String(code.value().to_owned()));
            result.insert(
                format!("{key}_code"),
                AttributeValue::String(code.code().to_owned()),
            );
        }
        result
    }

    fn handle_simple_value(
        key: Option<String>,
        value: &nusamai_citygml::object::Value,
    ) -> HashMap<String, AttributeValue> {
        let mut result = HashMap::new();
        if let Some(key) = key {
            result.insert(key, AttributeValue::from(value.clone()));
        }
        result
    }

    fn merge_attribute_maps(
        mut result: HashMap<String, AttributeValue>,
        new_map: HashMap<String, AttributeValue>,
    ) -> HashMap<String, AttributeValue> {
        for (key, value) in new_map {
            Self::merge_into_result(&mut result, &key, value);
        }
        result
    }

    fn merge_into_result(
        result: &mut HashMap<String, AttributeValue>,
        key: &str,
        value: AttributeValue,
    ) {
        match result.get(key) {
            Some(AttributeValue::Array(existing_arr)) => {
                let mut new_arr = existing_arr.clone();
                match value {
                    AttributeValue::Array(arr) => new_arr.extend(arr),
                    _ => new_arr.push(value),
                }
                result.insert(key.to_string(), AttributeValue::Array(new_arr));
            }
            Some(AttributeValue::Map(existing_map)) => match value {
                AttributeValue::Map(_) => {
                    result.insert(
                        key.to_string(),
                        AttributeValue::Array(vec![
                            AttributeValue::Map(existing_map.clone()),
                            value,
                        ]),
                    );
                }
                _ => {
                    let mut new_map = existing_map.clone();
                    new_map.insert(key.to_string(), value);
                    result.insert(key.to_string(), AttributeValue::Map(new_map));
                }
            },
            _ => {
                result.insert(key.to_string(), value);
            }
        }
    }
}
