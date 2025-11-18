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
    pub fn from_nusamai_citygml_value(
        value: &nusamai_citygml::object::Value,
    ) -> HashMap<String, AttributeValue> {
        match value {
            nusamai_citygml::object::Value::Object(obj) => {
                Self::process_object_attributes(&obj.attributes)
            }
            nusamai_citygml::object::Value::Array(_arr) => {
                // Arrays at top level are not expected in typical CityGML
                HashMap::new()
            }
            _ => HashMap::new(),
        }
    }

    fn process_object_attributes(
        attributes: &nusamai_citygml::object::Map,
    ) -> HashMap<String, AttributeValue> {
        let mut result = HashMap::new();
        for (key, value) in attributes {
            let attrs = Self::process_attribute(key, value);
            result.extend(attrs);
        }
        result
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
                let value = serde_json::Number::from_string_unchecked(v.value().to_string());
                result.insert(key.to_string(), AttributeValue::Number(value));
                if let Some(uom) = v.uom() {
                    result.insert(format!("{key}_uom"), AttributeValue::String(uom.to_owned()));
                }
            }
            nusamai_citygml::Value::Date(v) => {
                // preserve plateau date format
                let string = v.format("%Y-%m-%d").to_string();
                result.insert(key.to_string(), AttributeValue::String(string));
            }
            nusamai_citygml::Value::Array(arr) => {
                Self::process_array_attribute(&mut result, key, arr);
            }
            nusamai_citygml::Value::Object(obj) => {
                Self::process_object_value(&mut result, obj);
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
        if arr.len() == 1 && !matches!(arr[0], nusamai_citygml::Value::Object(_)) {
            let nested = Self::process_attribute(key, &arr[0]);
            result.extend(nested);
        } else {
            // Process each element individually, accumulating values
            let mut values = Vec::new();
            let mut codes = Vec::new();
            let mut has_codes = false;

            for attr in arr.iter() {
                match attr {
                    nusamai_citygml::Value::Code(code) => {
                        // Unzip Code types: collect values and codes separately
                        values.push(AttributeValue::String(code.value().to_owned()));
                        codes.push(AttributeValue::String(code.code().to_owned()));
                        has_codes = true;
                    }
                    nusamai_citygml::Value::Object(obj) => {
                        Self::process_object_value(result, obj);
                    }
                    _ => {
                        // Skip non-object, non-code types in array
                        tracing::warn!("Skip non-object in array for key: {} {:?}", key, attr);
                    }
                }
            }

            // If we collected any Code values, add them to result
            if has_codes {
                result.insert(key.to_string(), AttributeValue::Array(values));
                result.insert(format!("{}_code", key), AttributeValue::Array(codes));
            }
        }
    }

    fn process_object_value(
        result: &mut HashMap<String, AttributeValue>,
        obj: &nusamai_citygml::object::Object,
    ) {
        // Special handling for gen:genericAttribute to match reference implementation format
        if obj.typename == "gen:genericAttribute" {
            let generic_attrs = Self::convert_generic_attributes(&obj.attributes);
            result.insert(
                obj.typename.to_string(),
                AttributeValue::Array(generic_attrs),
            );
        } else {
            // recursive process for other objects
            let attrs = Self::process_object_attributes(&obj.attributes);
            result.insert(
                obj.typename.to_string(),
                AttributeValue::Array(vec![AttributeValue::Map(attrs)]),
            );
        }
    }

    fn convert_generic_attributes(
        attributes: &nusamai_citygml::object::Map,
    ) -> Vec<AttributeValue> {
        let mut result = Vec::new();

        for (name, value) in attributes {
            let mut attr_map = HashMap::new();
            attr_map.insert("name".to_string(), AttributeValue::String(name.clone()));

            // Determine type and value based on the Value discriminant
            let (type_str, value_attr) = match value {
                nusamai_citygml::Value::String(v) => ("string", AttributeValue::String(v.clone())),
                nusamai_citygml::Value::Integer(v) => {
                    ("integer", AttributeValue::String(v.to_string()))
                }
                nusamai_citygml::Value::Double(v) => {
                    ("double", AttributeValue::String(v.to_string()))
                }
                nusamai_citygml::Value::Measure(m) => {
                    ("measure", AttributeValue::String(m.value().to_string()))
                }
                nusamai_citygml::Value::Date(d) => (
                    "date",
                    AttributeValue::String(d.format("%Y-%m-%d").to_string()),
                ),
                nusamai_citygml::Value::Uri(u) => {
                    ("uri", AttributeValue::String(u.value().to_string()))
                }
                nusamai_citygml::Value::Code(c) => {
                    ("code", AttributeValue::String(c.value().to_string()))
                }
                nusamai_citygml::Value::Object(obj) => {
                    // Nested generic attribute set
                    let nested_attrs = Self::convert_generic_attributes(&obj.attributes);
                    ("genericAttributeSet", AttributeValue::Array(nested_attrs))
                }
                _ => {
                    // Fallback for any other types
                    ("string", AttributeValue::String(format!("{value:?}")))
                }
            };

            attr_map.insert(
                "type".to_string(),
                AttributeValue::String(type_str.to_string()),
            );
            attr_map.insert("value".to_string(), value_attr);

            result.push(AttributeValue::Map(attr_map));
        }

        result
    }
}
