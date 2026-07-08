use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use reearth_flow_geometry::types::line_string::LineString2D;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_types::{
    AttributeValue, Attributes, CityGmlGeometry, Feature, Geometry, GeometryType, GeometryValue,
    GmlGeometry, CITYGML_PARENT_GML_ID_KEY, CITYGML_ROOT_GML_ID_KEY,
};

use super::{
    codespace, flatten, geometry,
    parser::{self, Parser},
    utils::{gml_id_attr, XmlNode},
    xlink,
};

/// Resolves the parsed document (xlink + codespace) and returns one feature per top-level city
/// object, or — when `extract_tags` is non-empty — one feature per matching flattened node.
/// `base_attributes` maps a source file URL to the input feature's attributes (e.g. `package`),
/// merged into every feature parsed from that file.
#[cfg(not(feature = "new-geometry"))]
pub fn build_features(
    parser: Parser,
    extract_tags: &HashSet<String>,
    base_attributes: &HashMap<String, Attributes>,
    citygml_attribute_key: Option<&str>,
    keep_attributes: bool,
    flatten_single_child_objects: bool,
    flatten_measure_types: bool,
) -> Vec<Feature> {
    let (pending, raw_registry, ns_registry) = parser.finish();
    let mut codelist_resolver = codespace::CodelistResolver::new();
    let mut out = Vec::new();
    for feature_root in codespace::resolve(
        xlink::resolve(pending, &raw_registry),
        &mut codelist_resolver,
    ) {
        let base = base_attributes.get(feature_root.source_url.as_str());
        if extract_tags.is_empty() {
            let mut feature = build_feature(
                &feature_root,
                citygml_attribute_key,
                keep_attributes,
                flatten_single_child_objects,
                flatten_measure_types,
            );
            if let Some(base) = base {
                feature.extend(base.clone());
            }
            out.push(feature);
        } else {
            let root_gml_id = gml_id_attr(&feature_root.attrs);

            for (node, parent_id) in flatten::extract(&feature_root, extract_tags, &ns_registry) {
                let mut feature = build_feature(
                    &node,
                    citygml_attribute_key,
                    keep_attributes,
                    flatten_single_child_objects,
                    flatten_measure_types,
                );
                if let Some(id) = parent_id {
                    feature.insert(CITYGML_PARENT_GML_ID_KEY, AttributeValue::String(id));
                }
                if let Some(ref id) = root_gml_id {
                    feature.insert(CITYGML_ROOT_GML_ID_KEY, AttributeValue::String(id.clone()));
                }
                if let Some(base) = base {
                    feature.extend(base.clone());
                }
                out.push(feature);
            }
        }
    }
    out
}

#[cfg(not(feature = "new-geometry"))]
fn build_feature(
    node: &Arc<XmlNode>,
    citygml_attribute_key: Option<&str>,
    keep_attributes: bool,
    flatten_single_child_objects: bool,
    flatten_measure_types: bool,
) -> Feature {
    let (stripped, raw_geoms) = geometry::extract_geometries(node);
    let mut feature = parser::to_feature(
        &stripped,
        citygml_attribute_key,
        keep_attributes,
        flatten_single_child_objects,
        flatten_measure_types,
    );
    if !raw_geoms.is_empty() {
        *feature.geometry_mut() = Geometry::with_value(GeometryValue::CityGmlGeometry(
            build_citygml_geometry(raw_geoms),
        ));
    }
    feature
}

// pos is assigned here; neutral appearance arrays prevent out-of-bounds access in downstream consumers.
fn build_citygml_geometry(raw: Vec<GmlGeometry>) -> CityGmlGeometry {
    let mut polygon_materials: Vec<Option<u32>> = Vec::new();
    let mut polygon_textures: Vec<Option<u32>> = Vec::new();
    let mut polygon_uvs: Vec<Polygon2D<f64>> = Vec::new();
    let mut current_pos: u32 = 0;
    let mut gml_geometries: Vec<GmlGeometry> = Vec::with_capacity(raw.len());

    for mut g in raw {
        if matches!(
            g.ty,
            GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle
        ) {
            g.pos = current_pos;
            current_pos += g.len;
            for poly in &g.polygons {
                polygon_materials.push(None);
                polygon_textures.push(None);
                polygon_uvs.push(neutral_uv_polygon(poly));
            }
        }
        gml_geometries.push(g);
    }

    CityGmlGeometry {
        gml_geometries,
        materials: Vec::new(),
        textures: Vec::new(),
        polygon_materials,
        polygon_textures,
        polygon_uvs: MultiPolygon2D::new(polygon_uvs),
    }
}

fn neutral_uv_polygon(poly: &Polygon3D<f64>) -> Polygon2D<f64> {
    let ext = LineString2D::new(vec![[0.0f64, 0.0f64].into(); poly.exterior().0.len()]);
    let ints = poly
        .interiors()
        .iter()
        .map(|ring| LineString2D::new(vec![[0.0f64, 0.0f64].into(); ring.0.len()]))
        .collect();
    Polygon2D::new(ext, ints)
}

#[cfg(feature = "new-geometry")]
pub use build_next::build_features;

/// New-geometry feature building: resolves each parsed city object's references
/// and attaches its geometry as a [`GeometryCollection`], one member per `lodN`
/// property. Kept in its own module so its geometry imports do not collide with
/// the legacy path's.
#[cfg(feature = "new-geometry")]
mod build_next {
    use std::collections::{HashMap, HashSet};

    use reearth_flow_geometry::{Geometry, GeometryCollection};
    use reearth_flow_types::{
        AttributeValue, Attributes, Feature, CITYGML_PARENT_GML_ID_KEY, CITYGML_ROOT_GML_ID_KEY,
    };

    use crate::citygml_parser::{
        appearance::{self, AppearanceIndex},
        codespace, flatten, geometry,
        parser::{self, Parser, RawRegistry},
        resolver::{self, GeomRegistry},
        utils::{gml_id_attr, NamespaceRegistry},
        xlink,
    };

    /// Resolves the parsed document and returns one feature per top-level city object, or — when
    /// `extract_tags` is non-empty — one feature per matching flattened node, each with its
    /// geometry attached. Signature mirrors the legacy `build_features` so the readers share one
    /// `finish` across geometry worlds.
    // TODO: honor `base_attributes` and the keep/flatten options in the new-geometry path.
    pub fn build_features(
        parser: Parser,
        extract_tags: &HashSet<String>,
        _base_attributes: &HashMap<String, Attributes>,
        _citygml_attribute_key: Option<&str>,
        _keep_attributes: bool,
        _flatten_single_child_objects: bool,
        _flatten_measure_types: bool,
    ) -> Vec<Feature> {
        let (pending, raw_registry, geom_registry, appearance_members, ns_registry) =
            parser.finish();
        let appearance = appearance::build_index(&appearance_members, &raw_registry);
        assemble_features(
            pending,
            &raw_registry,
            &geom_registry,
            &appearance,
            &ns_registry,
            extract_tags,
        )
    }

    /// Resolve every pending feature into emitted `Feature`s: one per top-level city object when
    /// `extract_tags` is empty, or the hoisted sub-features otherwise, each with its geometry
    /// attached.
    fn assemble_features(
        pending: Vec<parser::PendingFeature>,
        raw_registry: &RawRegistry,
        geom_registry: &GeomRegistry,
        appearance: &AppearanceIndex,
        ns_registry: &NamespaceRegistry,
        extract_tags: &HashSet<String>,
    ) -> Vec<Feature> {
        let mut out = Vec::new();
        let mut codelist_resolver = codespace::CodelistResolver::new();
        let mut xlink_cache = xlink::ResolveCache::new();

        for parser::PendingFeature { root, geoms } in pending {
            let Some(resolved_root) = xlink::resolve_one(&root, raw_registry, &mut xlink_cache)
            else {
                continue;
            };
            let resolved = codespace::resolve(vec![resolved_root], &mut codelist_resolver);
            let Some(feature_root) = resolved.into_iter().next() else {
                continue;
            };

            if extract_tags.is_empty() {
                let mut feature = parser::to_feature(&feature_root);
                attach_geometry(&mut feature, &geoms, geom_registry, appearance);
                out.push(feature);
            } else {
                let root_gml_id = gml_id_attr(&feature_root.attrs);
                let extracted = flatten::extract(&feature_root, extract_tags, ns_registry);
                let emitted_ids: HashSet<String> = extracted
                    .iter()
                    .filter_map(|(n, _)| gml_id_attr(&n.attrs))
                    .collect();
                // Attach each carved geometry to its nearest emitted (hoisted) ancestor.
                let mut by_owner: HashMap<&str, Vec<&geometry::PendingGeom>> = HashMap::new();
                for g in &geoms {
                    if let Some(target) = g
                        .owner_ids
                        .iter()
                        .find(|id| emitted_ids.contains(id.as_str()))
                    {
                        by_owner.entry(target.as_str()).or_default().push(g);
                    }
                }
                for (node, parent_id) in &extracted {
                    let mut feature = parser::to_feature(node);
                    if let Some(id) = parent_id {
                        feature.insert(
                            CITYGML_PARENT_GML_ID_KEY,
                            AttributeValue::String(id.clone()),
                        );
                    }
                    if let Some(ref id) = root_gml_id {
                        feature.insert(CITYGML_ROOT_GML_ID_KEY, AttributeValue::String(id.clone()));
                    }
                    if let Some(gs) =
                        gml_id_attr(&node.attrs).and_then(|id| by_owner.get(id.as_str()))
                    {
                        attach_geometry(
                            &mut feature,
                            gs.iter().copied(),
                            geom_registry,
                            appearance,
                        );
                    }
                    out.push(feature);
                }
            }
        }
        out
    }

    /// Resolve each carved geometry and set the feature's geometry to a collection of the results,
    /// one member per geometry. Leaves the geometry unset when none resolve.
    fn attach_geometry<'a>(
        feature: &mut Feature,
        geoms: impl IntoIterator<Item = &'a geometry::PendingGeom>,
        registry: &GeomRegistry,
        appearance: &AppearanceIndex,
    ) {
        // TODO: carry each geometry's LOD and gml:id in the collection's per-member attributes.
        let members: Vec<Geometry> = geoms
            .into_iter()
            .filter_map(|pending| {
                resolver::resolve_root(&pending.node, registry, appearance)
                    .map(Geometry::Euclidean3D)
            })
            .collect();
        if !members.is_empty() {
            *feature.geometry_mut() =
                Geometry::GeometryCollection(GeometryCollection::new(members));
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use reearth_flow_geometry::Euclidean3DGeometry;
        use reearth_flow_types::CitygmlFeatureExt;
        use url::Url;

        const TA: &str = "<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>35.6000 139.8000 10.0 35.6001 139.8000 10.0 35.6000 139.8001 12.0 35.6000 139.8000 10.0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>";
        const TB: &str = "<gml:Polygon><gml:exterior><gml:LinearRing><gml:posList>35.6001 139.8000 10.0 35.6001 139.8001 12.0 35.6000 139.8001 12.0 35.6001 139.8000 10.0</gml:posList></gml:LinearRing></gml:exterior></gml:Polygon>";

        /// Parse one CityModel wrapping `members`, then assemble features under
        /// `extract_tags` (the full pass-2 pipeline, minus forwarding).
        fn run(members: &str, extract_tags: &[&str]) -> Vec<Feature> {
            let xml = format!(
                r#"<core:CityModel
                     xmlns:core="http://www.opengis.net/citygml/3.0"
                     xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
                     xmlns:con="http://www.opengis.net/citygml/construction/3.0"
                     xmlns:gml="http://www.opengis.net/gml/3.2"
                     xmlns:xlink="http://www.w3.org/1999/xlink">{members}</core:CityModel>"#
            );
            let mut parser = Parser::new();
            parser
                .parse(xml.as_bytes(), &Url::parse("file:///test.gml").unwrap())
                .unwrap();
            let (pending, raw_registry, geom_registry, appearance_members, ns_registry) =
                parser.finish();
            let appearance = appearance::build_index(&appearance_members, &raw_registry);
            let tags: HashSet<String> = extract_tags.iter().map(|s| s.to_string()).collect();
            assemble_features(
                pending,
                &raw_registry,
                &geom_registry,
                &appearance,
                &ns_registry,
                &tags,
            )
        }

        /// The single `Euclidean3D` geometry of a feature whose `GeometryCollection`
        /// has exactly one member.
        fn only_e3d(feature: &Feature) -> &Euclidean3DGeometry {
            match &*feature.geometry {
                Geometry::GeometryCollection(gc) => {
                    assert_eq!(gc.len(), 1, "expected exactly one geometry");
                    match &gc.members()[0] {
                        Geometry::Euclidean3D(e) => e,
                        other => panic!("expected a Euclidean3D member, got {other:?}"),
                    }
                }
                other => panic!("expected GeometryCollection, got {other:?}"),
            }
        }

        fn by_type<'a>(features: &'a [Feature], ty: &str) -> &'a Feature {
            features
                .iter()
                .find(|f| f.feature_type().as_deref() == Some(ty))
                .unwrap_or_else(|| panic!("no feature of type {ty}"))
        }

        fn assert_single_polygon_surface(feature: &Feature) {
            match only_e3d(feature) {
                Euclidean3DGeometry::Collection(c) => {
                    assert_eq!(c.len(), 1);
                    assert!(matches!(c.members()[0], Euclidean3DGeometry::Polygon(_)));
                }
                other => panic!("expected Collection[Polygon], got {other:?}"),
            }
        }

        #[test]
        fn case10_feature_with_multiple_geometries() {
            let solid = format!("<gml:Solid><gml:exterior><gml:Shell><gml:surfaceMember>{TA}</gml:surfaceMember><gml:surfaceMember>{TB}</gml:surfaceMember></gml:Shell></gml:exterior></gml:Solid>");
            let features = run(
                &format!(
                    "<core:cityObjectMember><bldg:Building gml:id=\"b_multi\">\
                       <core:lod0MultiSurface><gml:MultiSurface><gml:surfaceMember>{TA}</gml:surfaceMember></gml:MultiSurface></core:lod0MultiSurface>\
                       <core:lod1Solid>{solid}</core:lod1Solid>\
                       <core:lod2Solid>{solid}</core:lod2Solid>\
                     </bldg:Building></core:cityObjectMember>"
                ),
                &[],
            );
            assert_eq!(features.len(), 1);
            assert_eq!(
                features[0].feature_type(),
                Some("bldg:Building".to_string())
            );
            // One member per lodN property, in document order: MultiSurface, Solid, Solid.
            match &*features[0].geometry {
                Geometry::GeometryCollection(gc) => {
                    let m = gc.members();
                    assert_eq!(m.len(), 3);
                    assert!(matches!(
                        m[0],
                        Geometry::Euclidean3D(Euclidean3DGeometry::Collection(_))
                    ));
                    assert!(matches!(
                        m[1],
                        Geometry::Euclidean3D(Euclidean3DGeometry::Solid(_))
                    ));
                    assert!(matches!(
                        m[2],
                        Geometry::Euclidean3D(Euclidean3DGeometry::Solid(_))
                    ));
                }
                other => panic!("expected GeometryCollection, got {other:?}"),
            }
        }

        #[test]
        fn case11_flatten_hoists_children() {
            let members = format!(
                "<core:cityObjectMember><bldg:Building gml:id=\"b_par\">\
                   <core:boundary><con:WallSurface gml:id=\"wall1\"><core:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>{TA}</gml:surfaceMember></gml:MultiSurface></core:lod2MultiSurface></con:WallSurface></core:boundary>\
                   <core:boundary><con:RoofSurface gml:id=\"roof1\"><core:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>{TB}</gml:surfaceMember></gml:MultiSurface></core:lod2MultiSurface></con:RoofSurface></core:boundary>\
                 </bldg:Building></core:cityObjectMember>"
            );
            let features = run(&members, &["WallSurface", "RoofSurface"]);
            // Two children hoisted; the Building itself is dropped (not in the tag set).
            assert_eq!(features.len(), 2);
            assert_single_polygon_surface(by_type(&features, "con:WallSurface"));
            assert_single_polygon_surface(by_type(&features, "con:RoofSurface"));
            assert_eq!(
                by_type(&features, "con:WallSurface").feature_id(),
                Some("wall1".to_string())
            );
        }

        #[test]
        fn case12_flatten_parent_and_child() {
            let members = format!(
                "<core:cityObjectMember><bldg:Building gml:id=\"b_par\">\
                   <core:lod1Solid><gml:Solid><gml:exterior><gml:Shell><gml:surfaceMember>{TA}</gml:surfaceMember><gml:surfaceMember>{TB}</gml:surfaceMember></gml:Shell></gml:exterior></gml:Solid></core:lod1Solid>\
                   <core:boundary><con:WallSurface gml:id=\"wall1\"><core:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>{TA}</gml:surfaceMember></gml:MultiSurface></core:lod2MultiSurface></con:WallSurface></core:boundary>\
                   <core:boundary><con:RoofSurface gml:id=\"roof1\"><core:lod2MultiSurface><gml:MultiSurface><gml:surfaceMember>{TB}</gml:surfaceMember></gml:MultiSurface></core:lod2MultiSurface></con:RoofSurface></core:boundary>\
                 </bldg:Building></core:cityObjectMember>"
            );
            let features = run(&members, &["Building", "WallSurface", "RoofSurface"]);
            assert_eq!(features.len(), 3);
            // The Building keeps only its own lod1Solid (its boundary surfaces are hoisted out).
            match only_e3d(by_type(&features, "bldg:Building")) {
                Euclidean3DGeometry::Solid(s) => assert_eq!(s.exterior().num_faces(), 2),
                other => panic!("expected Solid, got {other:?}"),
            }
            assert_single_polygon_surface(by_type(&features, "con:WallSurface"));
            assert_single_polygon_surface(by_type(&features, "con:RoofSurface"));
        }
    }
}
