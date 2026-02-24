use std::collections::HashMap;

use nusamai_citygml::GeometryRef;
use nusamai_citygml::{object::ObjectStereotype, GeometryType, Value};
use nusamai_plateau::Entity;
use reearth_flow_geometry::types::coordinate::Coordinate3D;
use reearth_flow_geometry::types::line_string::LineString3D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon3D;

use crate::error::Error;
use crate::{AttributeValue, CityGmlGeometry, Geometry, GeometryValue, GmlGeometry};

// Convert nusamai geometry to reearth_flow_geometry Geometry
// Expect well-formed geometries parsed by nusamai_citygml:
// if the geometry contains NaN, the duplicate equality check fails and coordinate list will contain duplicates
impl TryFrom<Entity> for Geometry {
    type Error = Error;

    fn try_from(entity: Entity) -> Result<Self, Self::Error> {
        entity_to_geometry(entity, false)
    }
}

/// Convert a nusamai Entity into a Geometry.
///
/// When `reconstruct_unresolved_solid` is true and the entity has no geometry refs
/// but the geometry store contains polygons, a Solid is reconstructed from all
/// store polygons. This handles CityGML features (e.g. uro:OtherConstruction)
/// whose Solid geometry uses xlink:href forward references that the streaming
/// parser cannot resolve.
pub fn entity_to_geometry(
    entity: Entity,
    reconstruct_unresolved_solid: bool,
) -> Result<Geometry, Error> {
    let apperance = entity.appearance_store.read().unwrap();
    let theme = apperance
        .themes
        .get("rgbTexture")
        .or_else(|| apperance.themes.get("FMETheme"));
    let geoms = entity.geometry_store.read().unwrap();
    let epsg = geoms.epsg;
    // entity must be a Feature
    let Value::Object(obj) = &entity.root else {
        return Err(Error::unsupported_feature("no object found"));
    };
    let ObjectStereotype::Feature { id: _, geometries } = &obj.stereotype else {
        return Err(Error::unsupported_feature("no feature found"));
    };

    // Collect polygon ranges for this feature (global indices in geometry store)
    let mut polygon_ranges: Vec<(u32, u32, bool)> = geometries
        .iter()
        .filter(|g| {
            matches!(
                g.ty,
                GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle
            )
        })
        .flat_map(|g| {
            let inline = if g.len > 0 {
                vec![(g.pos, g.pos + g.len, false)]
            } else {
                vec![]
            };
            inline.into_iter().chain(g.resolved_ranges.iter().copied())
        })
        .collect();

    // Build gml_geometries with local pos/len (relative to this feature's polygon arrays)
    let mut gml_geometries = Vec::<GmlGeometry>::new();
    let mut local_pos: u32 = 0;
    let operation = |geometry: &GeometryRef| -> Option<GmlGeometry> {
        match geometry.ty {
            GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
                let mut polygons = Vec::<Polygon3D<f64>>::new();
                // Inline polygons
                for (local_idx, idx_poly) in geoms
                    .multipolygon
                    .iter_range(geometry.pos as usize..(geometry.pos + geometry.len) as usize).enumerate()
                {
                    let poly = idx_poly.transform(|c| geoms.vertices[*c as usize]);
                    let mut polygon: Polygon3D<f64> = poly.into();
                    // Get the global polygon index and lookup the ring ID (exterior ring's gml:id)
                    let global_poly_idx = geometry.pos as usize + local_idx;
                    if let Some(ring_ids) = geoms.ring_ids.get(global_poly_idx) {
                        if let Some(ring_id) = ring_ids {
                            // The ring_id already includes the "UUID_" prefix from nusamai
                            polygon.id = Some(format!("{}", ring_id.0));
                        }
                    }
                    polygons.push(polygon);
                }
                // Resolved xlink:href ranges
                // TODO: implement resolving for other geometry types (requires upstream support in nusamai_citygml parser)
                for &(start, end, flip) in &geometry.resolved_ranges {
                    for idx_poly in geoms.multipolygon.iter_range(start as usize..end as usize) {
                        let poly: Polygon3D<f64> =
                            idx_poly.transform(|c| geoms.vertices[*c as usize]).into();
                        if flip {
                            let (mut exterior, mut interiors) = poly.into_inner();
                            exterior.reverse_inplace();
                            for interior in interiors.iter_mut() {
                                interior.reverse_inplace();
                            }
                            polygons.push(Polygon3D::new(exterior, interiors));
                        } else {
                            polygons.push(poly);
                        }
                    }
                }
                let mut geometry_feature = GmlGeometry::from(geometry.clone());
                geometry_feature.len = polygons.len() as u32;
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
            GeometryType::Point => {
                let mut points = Vec::<Coordinate3D<f64>>::new();
                for idx_point in geoms
                    .multipoint
                    .iter_range(geometry.pos as usize..(geometry.pos + geometry.len) as usize)
                {
                    let coord = geoms.vertices[idx_point as usize];
                    points.push(Coordinate3D::new__(coord[0], coord[1], coord[2]));
                }
                let mut geometry_feature = GmlGeometry::from(geometry.clone());
                geometry_feature.points.extend(points);
                Some(geometry_feature)
            }
        }
    };
    for geometry in geometries.iter() {
        if let Some(mut gml_geo) = operation(geometry) {
            // Update pos to be local (relative to this feature's polygon arrays)
            // Only polygon geometries use pos to index into polygon_materials/textures/uvs
            let is_polygon_geometry = matches!(
                geometry.ty,
                GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle
            );

            if is_polygon_geometry {
                gml_geo.pos = local_pos;
                local_pos += gml_geo.len;
            } else {
                gml_geo.pos = 0; // unused for non-polygon geometries
            }

            gml_geometries.push(gml_geo);
        }
    }

    // When an entity has no geometry refs at all but the geometry store has polygons,
    // this indicates a Solid geometry whose xlink:href references weren't resolved
    // (forward references in a streaming parser). Reconstruct a Solid from all store polygons.
    // Note: entities with non-empty geometries (e.g. DmGeometricAttribute with Curve/Point refs)
    // that share a parent's geometry store must NOT trigger this reconstruction.
    let has_polygon_geometries = gml_geometries.iter().any(|g| !g.polygons.is_empty());
    if reconstruct_unresolved_solid
        && !has_polygon_geometries
        && geometries.is_empty()
        && !geoms.multipolygon.is_empty()
    {
        let mut polygons = Vec::<Polygon3D<f64>>::new();
        for idx_poly in geoms.multipolygon.iter_range(0..geoms.multipolygon.len()) {
            let poly = idx_poly.transform(|c| geoms.vertices[*c as usize]);
            polygons.push(poly.into());
        }
        if !polygons.is_empty() {
            let polygon_count = polygons.len();
            let solid_gml = GmlGeometry {
                id: entity.id.clone(),
                len: polygon_count as u32,
                polygons,
                feature_id: entity.id.clone(),
                feature_type: entity.typename.clone(),
                ..GmlGeometry::new(crate::geometry::GeometryType::Solid, None)
            };
            gml_geometries = vec![solid_gml];
            polygon_ranges = vec![(0, polygon_count as u32, false)];
        }
    }

    // Calculate total polygon count for this feature
    let total_polygons: usize = polygon_ranges
        .iter()
        .map(|(s, e, _)| (e - s) as usize)
        .sum();

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

    // Consistency check: total rings vs total polygon rings
    let total_rings: usize = geoms.ring_ids.len();
    let total_polygon_rings: usize = geoms.multipolygon.iter().map(|p| p.rings().count()).sum();
    if total_rings != total_polygon_rings {
        tracing::error!(
            "Inconsistent ring count: total_rings={} total_polygon_rings={}",
            total_rings,
            total_polygon_rings
        );
    }

    if let Some(theme) = theme {
        // find and apply materials (only for this feature's polygons)
        {
            let mut poly_materials = Vec::with_capacity(total_polygons);
            for &(start, end, _flip) in &polygon_ranges {
                for global_idx in start..end {
                    // Find material for this polygon via surface_spans
                    let mut mat_iter = geoms
                        .surface_spans
                        .iter()
                        .filter(|surface| global_idx >= surface.start && global_idx < surface.end)
                        .filter_map(|surface| theme.surface_id_to_material.get(&surface.id));
                    let mat = mat_iter.next().copied();
                    if mat_iter.next().is_some() {
                        tracing::warn!("Multiple materials found for polygon index {}", global_idx);
                    }
                    poly_materials.push(mat);
                }
            }
            geometry_entity.polygon_materials = poly_materials;
        }
        // find and apply textures (only for this feature's polygons)
        {
            let mut poly_textures = Vec::with_capacity(total_polygons);
            let mut poly_uvs = flatgeom::MultiPolygon::new();

            // Build a lookup table: polygon_index -> starting_ring_index
            // This pre-computes the cumulative ring count for efficient O(1) lookup
            let mut polygon_to_ring_start: Vec<usize> =
                Vec::with_capacity(geoms.multipolygon.len());
            let mut cumulative_rings = 0;
            for poly_idx in 0..geoms.multipolygon.len() {
                polygon_to_ring_start.push(cumulative_rings);
                cumulative_rings += geoms.multipolygon.get(poly_idx).rings().count();
            }

            for &(start, end, _flip) in &polygon_ranges {
                for global_idx in start..end {
                    let poly = geoms.multipolygon.get(global_idx as usize);
                    let global_ring_idx = polygon_to_ring_start[global_idx as usize];

                    for (i, ring) in poly.rings().enumerate() {
                        let ring_id = geoms.ring_ids[global_ring_idx + i].clone();
                        let tex = ring_id
                            .clone()
                            .and_then(|id| theme.ring_id_to_texture.get(&id));

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
                                tracing::error!(
                                    "Invalid texture: ring_id={:?}, polygon len={}, uv len={}",
                                    ring_id,
                                    ring.len(),
                                    uv.len(),
                                );
                                add_dummy_texture();
                            }
                            _ => {
                                // no texture found
                                add_dummy_texture();
                            }
                        };
                    }
                }
            }
            // apply textures to polygons
            geometry_entity.polygon_textures = poly_textures;
            geometry_entity.polygon_uvs = MultiPolygon2D::from(poly_uvs);
        }
    } else {
        // set 'null' appearance if no theme found
        geometry_entity.polygon_materials = vec![None; total_polygons];
        geometry_entity.polygon_textures = vec![None; total_polygons];
        let mut poly_uvs = flatgeom::MultiPolygon::new();
        for &(start, end, _flip) in &polygon_ranges {
            for global_idx in start..end {
                let poly = geoms.multipolygon.get(global_idx as usize);
                for (i, ring) in poly.rings().enumerate() {
                    let uv = [[0.0, 0.0]].into_iter().cycle().take(ring.len() + 1);
                    if i == 0 {
                        poly_uvs.add_exterior(uv);
                    } else {
                        poly_uvs.add_interior(uv);
                    }
                }
            }
        }
        geometry_entity.polygon_uvs = MultiPolygon2D::from(poly_uvs);
    }
    Ok(Geometry::new_with(
        epsg,
        GeometryValue::CityGmlGeometry(geometry_entity),
    ))
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
                if let Some(code) = v.code() {
                    result.insert(
                        format!("{key}_code"),
                        AttributeValue::String(code.to_owned()),
                    );
                }
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
                        let code = match code.code() {
                            Some(c) => AttributeValue::String(c.to_owned()),
                            None => AttributeValue::Null,
                        };
                        codes.push(code);
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
                result.insert(format!("{key}_code"), AttributeValue::Array(codes));
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
            // Append to existing array if key exists, otherwise create new
            Self::append_to_array(result, obj.typename.to_string(), generic_attrs);
        } else {
            // recursive process for other objects
            let attrs = Self::process_object_attributes(&obj.attributes);
            let new_value = AttributeValue::Map(attrs);
            // Append to existing array if key exists (e.g., multiple KeyValuePairAttribute)
            Self::append_to_array(result, obj.typename.to_string(), vec![new_value]);
        }
    }

    /// Helper to append values to an existing array or create a new one
    fn append_to_array(
        result: &mut HashMap<String, AttributeValue>,
        key: String,
        new_values: Vec<AttributeValue>,
    ) {
        match result.get_mut(&key) {
            Some(AttributeValue::Array(existing)) => {
                // Append to existing array
                existing.extend(new_values);
            }
            Some(_) => {
                // Key exists but is not an array - this shouldn't happen, but handle gracefully
                tracing::warn!(
                    "Expected array for key '{}' but found different type, replacing",
                    key
                );
                result.insert(key, AttributeValue::Array(new_values));
            }
            None => {
                // Create new array
                result.insert(key, AttributeValue::Array(new_values));
            }
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
