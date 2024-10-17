use nusamai_citygml::GeometryRef;
use nusamai_citygml::{object::ObjectStereotype, GeometryType, Value};
use nusamai_plateau::Entity;
use reearth_flow_geometry::types::polygon::Polygon3D;

use crate::error::Error;
use crate::{CityGmlGeometry, Geometry, GeometryValue, GmlGeometry};

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
                GeometryType::Curve => unimplemented!(),
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
                geometry_entity.polygon_uvs = poly_uvs.into();
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
            geometry_entity.polygon_uvs = poly_uvs.into();
        }
        Ok(Self::new(
            epsg,
            GeometryValue::CityGmlGeometry(geometry_entity),
        ))
    }
}
