/// Per-member attribute keys describing the GML a `GeometryCollection` member was
/// read from. Its LOD, the local name of the property it filled, and of the GML type
/// that property held.
pub const MEMBER_LOD_KEY: &str = "lod";
pub const MEMBER_GML_PROPERTY_NAME_KEY: &str = "gmlPropertyName";
pub const MEMBER_GEOMETRY_NAME_KEY: &str = "geometryName";

/// Per-member attribute keys naming the element the geometry property hung from,
/// the object the geometry belongs to: one feature can hold the geometry of
/// several nested objects, so the feature's own `gml:id` and type do not identify
/// a member's owner. The type is always known, the id only when that element
/// carries a `gml:id`.
pub const MEMBER_GEOMETRY_GML_ID_KEY: &str = "__citygml_geometry_gml_id";
pub const MEMBER_GEOMETRY_FEATURE_TYPE_KEY: &str = "__citygml_geometry_feature_type";

/// Per-member attribute key naming the object that encloses the member's owner:
/// the `gml:id`, or synthetic owner key, of the nearest keyed ancestor of the
/// owner below the feature itself. Absent when the owner is the feature or no
/// keyed object sits between them. Written only for CityGML 2.0.
pub const MEMBER_GEOMETRY_PARENT_GML_ID_KEY: &str = "__citygml_geometry_parent_gml_id";

/// Per-member attribute key holding the synthetic owner key of an owner that has
/// no `gml:id`, in place of [`MEMBER_GEOMETRY_GML_ID_KEY`]. The key has the form
/// `"{document}#_synid{n}"`, where `document` is the last path segment of the
/// source URL and `n` counts the document's id-less owners from 1. Written only
/// for CityGML 2.0.
pub const MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY: &str = "__citygml_geometry_synthetic_owner_key";

/// The attribute-tree key carrying an id-less object's synthetic owner key,
/// beside the object's own data, so the tree entry and the members it owns can
/// be matched up. Written only when attributes are kept.
pub const SYNTHETIC_OWNER_TREE_KEY: &str = "__citygml_synthetic_owner_key";

#[cfg(not(feature = "new-geometry"))]
pub use build_legacy::{build_features, build_features_reporting};

/// Legacy feature building: resolves the parsed document (xlink + codespace) and returns one
/// feature per top-level city object, or — when `extract_tags` is non-empty — one feature per
/// matching flattened node. Kept in its own module (mirroring `build_next`) so its imports are
/// scoped to the code that uses them instead of going unused under the new-geometry flag.
#[cfg(not(feature = "new-geometry"))]
mod build_legacy {
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    use reearth_flow_geometry::types::line_string::LineString2D;
    use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
    use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
    use reearth_flow_types::{
        AttributeValue, Attributes, CityGmlGeometry, Feature, Geometry, GeometryType,
        GeometryValue, GmlGeometry, CITYGML_PARENT_GML_ID_KEY, CITYGML_ROOT_GML_ID_KEY,
    };

    use crate::{
        codespace, flatten, geometry,
        parser::{self, Parser},
        utils::{gml_id_attr, XmlNode},
        xlink,
    };

    /// Resolves the parsed document (xlink + codespace) and returns one feature per top-level city
    /// object, or — when `extract_tags` is non-empty — one feature per matching flattened node.
    /// `base_attributes` maps a source file URL to the input feature's attributes (e.g. `package`),
    /// merged into every feature parsed from that file.
    #[allow(clippy::too_many_arguments)]
    pub fn build_features(
        parser: Parser,
        extract_tags: &HashSet<String>,
        base_attributes: &HashMap<String, Attributes>,
        citygml_attribute_key: Option<&str>,
        keep_attributes: bool,
        flatten_single_child_objects: bool,
        flatten_leaf_attributes: &[String],
        keep_code_space: bool,
    ) -> Vec<Feature> {
        let (pending, raw_registry, ns_registry) = parser.finish();
        let mut codelist_resolver = codespace::CodelistResolver::new();
        let mut out = Vec::new();
        for feature_root in codespace::resolve(
            xlink::resolve(pending, &raw_registry),
            &mut codelist_resolver,
            keep_code_space,
        ) {
            let base = base_attributes.get(feature_root.source_url.as_str());
            if extract_tags.is_empty() {
                let mut feature = build_feature(
                    &feature_root,
                    citygml_attribute_key,
                    keep_attributes,
                    flatten_single_child_objects,
                    flatten_leaf_attributes,
                );
                if let Some(base) = base {
                    feature.extend(base.clone());
                }
                out.push(feature);
            } else {
                let root_gml_id = gml_id_attr(&feature_root.attrs);

                for flatten::Extracted {
                    node,
                    parent_gml_id,
                    ..
                } in flatten::extract(&feature_root, extract_tags, &ns_registry)
                {
                    let mut feature = build_feature(
                        &node,
                        citygml_attribute_key,
                        keep_attributes,
                        flatten_single_child_objects,
                        flatten_leaf_attributes,
                    );
                    if let Some(id) = parent_gml_id {
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

    /// The legacy geometry path is not instrumented for Action Standard §4.3
    /// strictness (see `crate::malformation`) and is removed along with the
    /// `new-geometry` migration flag, so this always reports zero
    /// malformations rather than teaching the legacy parser to collect them.
    #[allow(clippy::too_many_arguments)]
    pub fn build_features_reporting(
        parser: Parser,
        extract_tags: &HashSet<String>,
        base_attributes: &HashMap<String, Attributes>,
        citygml_attribute_key: Option<&str>,
        keep_attributes: bool,
        flatten_single_child_objects: bool,
        flatten_leaf_attributes: &[String],
        keep_code_space: bool,
    ) -> (Vec<Feature>, Vec<crate::malformation::Malformation>) {
        let features = build_features(
            parser,
            extract_tags,
            base_attributes,
            citygml_attribute_key,
            keep_attributes,
            flatten_single_child_objects,
            flatten_leaf_attributes,
            keep_code_space,
        );
        (features, Vec::new())
    }

    fn build_feature(
        node: &Arc<XmlNode>,
        citygml_attribute_key: Option<&str>,
        keep_attributes: bool,
        flatten_single_child_objects: bool,
        flatten_leaf_attributes: &[String],
    ) -> Feature {
        let (stripped, raw_geoms) = geometry::extract_geometries(node);
        let mut feature = parser::to_feature(
            &stripped,
            citygml_attribute_key,
            keep_attributes,
            flatten_single_child_objects,
            flatten_leaf_attributes,
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
}

#[cfg(feature = "new-geometry")]
pub use build_next::{build_features, build_features_reporting};

/// New-geometry feature building: resolves each parsed city object's references
/// and attaches its geometry as a [`GeometryCollection`], one member per `lodN`
/// property. Kept in its own module so its geometry imports do not collide with
/// the legacy path's.
#[cfg(feature = "new-geometry")]
mod build_next {
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    use reearth_flow_geometry::coordinate::EpsgCode;
    use reearth_flow_geometry::{Geometry, GeometryCollection};
    use reearth_flow_types::{
        Attribute, AttributeValue, Attributes, Feature, CITYGML_PARENT_GML_ID_KEY,
        CITYGML_PARENT_OBJECT_GML_ID_KEY, CITYGML_ROOT_GML_ID_KEY,
    };

    use super::{
        MEMBER_GEOMETRY_FEATURE_TYPE_KEY, MEMBER_GEOMETRY_GML_ID_KEY, MEMBER_GEOMETRY_NAME_KEY,
        MEMBER_GEOMETRY_PARENT_GML_ID_KEY, MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY,
        MEMBER_GML_PROPERTY_NAME_KEY, MEMBER_LOD_KEY,
    };

    use crate::{
        appearance::{self, AppearanceIndex},
        codespace, flatten,
        malformation::{name_city_object, Malformation},
        parser::{self, CityGmlVersion, Parser, ParserOutput, RawRegistry},
        resolver::{self, GeomRegistry},
        utils::{gml_id_attr, NamespaceRegistry},
        xlink,
    };

    /// Resolves the parsed document and returns one feature per top-level city object, or — when
    /// `extract_tags` is non-empty — one feature per matching flattened node, each with its
    /// geometry attached. Signature mirrors the legacy `build_features` so the readers share one
    /// `finish` across geometry worlds. A thin wrapper over
    /// [`build_features_reporting`] that discards the malformations collected along the way, so
    /// this — and the processors that call it — stay lenient per Action Standard §4.3.
    // TODO: honor `keep_attributes` and `flatten_single_child_objects` in the new-geometry path.
    #[allow(clippy::too_many_arguments)]
    pub fn build_features(
        parser: Parser,
        extract_tags: &HashSet<String>,
        base_attributes: &HashMap<String, Attributes>,
        citygml_attribute_key: Option<&str>,
        keep_attributes: bool,
        flatten_single_child_objects: bool,
        flatten_leaf_attributes: &[String],
        keep_code_space: bool,
    ) -> Vec<Feature> {
        build_features_reporting(
            parser,
            extract_tags,
            base_attributes,
            citygml_attribute_key,
            keep_attributes,
            flatten_single_child_objects,
            flatten_leaf_attributes,
            keep_code_space,
        )
        .0
    }

    /// Same as [`build_features`], but also returns every present-but-malformed
    /// input site collected while parsing and resolving (Action Standard §4.3),
    /// so a strict caller can fail the read naming the offending location.
    #[allow(clippy::too_many_arguments)]
    pub fn build_features_reporting(
        parser: Parser,
        extract_tags: &HashSet<String>,
        base_attributes: &HashMap<String, Attributes>,
        citygml_attribute_key: Option<&str>,
        keep_attributes: bool,
        _flatten_single_child_objects: bool,
        flatten_leaf_attributes: &[String],
        keep_code_space: bool,
    ) -> (Vec<Feature>, Vec<Malformation>) {
        let ParserOutput {
            pending,
            raw_registry,
            geom_registry,
            appearance_members,
            srs_by_file,
            ns_registry,
            mut malformations,
            version,
        } = parser.finish();
        let appearance = appearance::build_index(&appearance_members, &raw_registry);
        let features = assemble_features(
            pending,
            &raw_registry,
            &geom_registry,
            &appearance,
            &srs_by_file,
            &ns_registry,
            version,
            extract_tags,
            base_attributes,
            citygml_attribute_key,
            keep_attributes,
            flatten_leaf_attributes,
            keep_code_space,
            &mut malformations,
        );
        (features, malformations)
    }

    /// Resolve every pending feature into emitted `Feature`s: one per top-level city object when
    /// `extract_tags` is empty, or the hoisted sub-features otherwise, each with its geometry
    /// attached. `base_attributes` maps a source file URL to the input feature's attributes,
    /// merged into every feature parsed from that file.
    #[allow(clippy::too_many_arguments)]
    fn assemble_features(
        pending: Vec<Arc<parser::RawNode>>,
        raw_registry: &RawRegistry,
        geom_registry: &GeomRegistry,
        appearance: &AppearanceIndex,
        srs_by_file: &HashMap<String, EpsgCode>,
        ns_registry: &NamespaceRegistry,
        version: CityGmlVersion,
        extract_tags: &HashSet<String>,
        base_attributes: &HashMap<String, Attributes>,
        citygml_attribute_key: Option<&str>,
        keep_attributes: bool,
        flatten_leaf_attributes: &[String],
        keep_code_space: bool,
        malformations: &mut Vec<Malformation>,
    ) -> Vec<Feature> {
        let mut out = Vec::new();
        let mut codelist_resolver = codespace::CodelistResolver::new();
        let mut xlink_cache = xlink::ResolveCache::new();
        // The object hierarchy keys are CityGML 2.0 only, so CityGML 3.0 output
        // stays as it was.
        let record_hierarchy = version == CityGmlVersion::V2;

        for root in pending {
            let before = malformations.len();
            let Some(resolved_root) = xlink::resolve_one(&root, raw_registry, &mut xlink_cache)
            else {
                continue;
            };
            let resolved =
                codespace::resolve(vec![resolved_root], &mut codelist_resolver, keep_code_space);
            let Some(feature_root) = resolved.into_iter().next() else {
                continue;
            };
            let base = base_attributes.get(feature_root.source_url.as_str());

            if extract_tags.is_empty() {
                let mut feature = parser::to_feature(
                    &feature_root,
                    citygml_attribute_key,
                    keep_attributes,
                    flatten_leaf_attributes,
                );
                attach_geometry(
                    &mut feature,
                    &flatten::collect_all_geometry(&feature_root),
                    geom_registry,
                    appearance,
                    srs_by_file,
                    record_hierarchy,
                    malformations,
                );
                if let Some(base) = base {
                    feature.extend(base.clone());
                }
                out.push(feature);
            } else {
                let root_gml_id = gml_id_attr(&feature_root.attrs);
                for flatten::Extracted {
                    node,
                    parent_gml_id,
                    parent_object_key,
                    geometry,
                } in flatten::extract(&feature_root, extract_tags, ns_registry)
                {
                    let mut feature = parser::to_feature(
                        &node,
                        citygml_attribute_key,
                        keep_attributes,
                        flatten_leaf_attributes,
                    );
                    // Only worth writing when the enclosing object is not the
                    // extracted parent already named.
                    let write_parent_object =
                        record_hierarchy && parent_object_key != parent_gml_id;
                    let parent_object_key = parent_object_key.filter(|_| write_parent_object);
                    if let Some(id) = parent_gml_id {
                        feature.insert(CITYGML_PARENT_GML_ID_KEY, AttributeValue::String(id));
                    }
                    if let Some(ref id) = root_gml_id {
                        feature.insert(CITYGML_ROOT_GML_ID_KEY, AttributeValue::String(id.clone()));
                    }
                    if let Some(key) = parent_object_key {
                        feature.insert(
                            CITYGML_PARENT_OBJECT_GML_ID_KEY,
                            AttributeValue::String(key),
                        );
                    }
                    attach_geometry(
                        &mut feature,
                        &geometry,
                        geom_registry,
                        appearance,
                        srs_by_file,
                        record_hierarchy,
                        malformations,
                    );
                    if let Some(base) = base {
                        feature.extend(base.clone());
                    }
                    out.push(feature);
                }
            }
            name_city_object(
                &mut malformations[before..],
                gml_id_attr(&root.attrs),
                &root.name.0,
            );
        }
        out
    }

    /// Resolve each found geometry and set the feature's geometry to a collection of the results,
    /// one member per geometry. Leaves the geometry unset when none resolve. `record_hierarchy`
    /// adds each member's synthetic owner key and owner parent key, for CityGML 2.0 only.
    fn attach_geometry<'a>(
        feature: &mut Feature,
        geoms: impl IntoIterator<Item = &'a flatten::FoundGeometry>,
        registry: &GeomRegistry,
        appearance: &AppearanceIndex,
        srs_by_file: &HashMap<String, EpsgCode>,
        record_hierarchy: bool,
        malformations: &mut Vec<Malformation>,
    ) {
        // Each member records its source LOD (a `tin` has none) so downstream
        // sinks can select a single LOD, and the object it belongs to so a
        // per-surface report can name it.
        let mut members: Vec<Geometry> = Vec::new();
        let mut attrs: Vec<Attributes> = Vec::new();
        for pending in geoms {
            let Some(member) = resolver::resolve_root(
                &pending.node,
                registry,
                appearance,
                srs_by_file,
                malformations,
            )
            .map(Geometry::Euclidean3D) else {
                continue;
            };
            let mut member_attrs = Attributes::new();
            if let Some(lod) = pending.lod {
                member_attrs.insert(
                    Attribute::new(MEMBER_LOD_KEY),
                    AttributeValue::Number(lod.into()),
                );
            }
            member_attrs.insert(
                Attribute::new(MEMBER_GML_PROPERTY_NAME_KEY),
                AttributeValue::String(pending.property.clone()),
            );
            if let Some(ref gml_type) = pending.gml_type {
                member_attrs.insert(
                    Attribute::new(MEMBER_GEOMETRY_NAME_KEY),
                    AttributeValue::String(gml_type.clone()),
                );
            }
            member_attrs.insert(
                Attribute::new(MEMBER_GEOMETRY_FEATURE_TYPE_KEY),
                AttributeValue::String(pending.owner_feature_type.clone()),
            );
            if let Some(owner_gml_id) = &pending.owner_gml_id {
                member_attrs.insert(
                    Attribute::new(MEMBER_GEOMETRY_GML_ID_KEY),
                    AttributeValue::String(owner_gml_id.clone()),
                );
            }
            if record_hierarchy {
                if let Some(key) = &pending.owner_synthetic_key {
                    member_attrs.insert(
                        Attribute::new(MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY),
                        AttributeValue::String(key.clone()),
                    );
                }
                if let Some(key) = &pending.owner_parent_key {
                    member_attrs.insert(
                        Attribute::new(MEMBER_GEOMETRY_PARENT_GML_ID_KEY),
                        AttributeValue::String(key.clone()),
                    );
                }
            }
            members.push(member);
            attrs.push(member_attrs);
        }
        if !members.is_empty() {
            let collection = GeometryCollection::with_attributes(members, attrs)
                .expect("attrs is built one-per-resolved-member");
            *feature.geometry_mut() = Geometry::GeometryCollection(collection);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::malformation::MalformationKind;
        use crate::parser::CityGmlVersion;
        use crate::pipeline::SYNTHETIC_OWNER_TREE_KEY;
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
                     xmlns:tran="http://www.opengis.net/citygml/transportation/3.0"
                     xmlns:gml="http://www.opengis.net/gml/3.2"
                     xmlns:xlink="http://www.w3.org/1999/xlink">{members}</core:CityModel>"#
            );
            let mut parser = Parser::new(CityGmlVersion::V3);
            parser
                .parse(xml.as_bytes(), &Url::parse("file:///test.gml").unwrap())
                .unwrap();
            let ParserOutput {
                pending,
                raw_registry,
                geom_registry,
                appearance_members,
                srs_by_file,
                ns_registry,
                version,
                ..
            } = parser.finish();
            let appearance = appearance::build_index(&appearance_members, &raw_registry);
            let tags: HashSet<String> = extract_tags.iter().map(|s| s.to_string()).collect();
            assemble_features(
                pending,
                &raw_registry,
                &geom_registry,
                &appearance,
                &srs_by_file,
                &ns_registry,
                version,
                &tags,
                &HashMap::new(),
                None,
                true,
                &[],
                false,
                &mut Vec::new(),
            )
        }

        /// Parse each `(file name, body)` as its own CityGML 2.0 document into one
        /// parser, then assemble features under `tags`.
        fn run_v2(files: &[(&str, &str)], tags: &[&str], keep_attributes: bool) -> Vec<Feature> {
            let tags: HashSet<String> = tags.iter().map(|s| s.to_string()).collect();
            let mut parser = Parser::with_extract_tags(CityGmlVersion::V2, tags.clone());
            for (name, body) in files {
                let xml = format!(
                    r#"<core:CityModel
                         xmlns:core="http://www.opengis.net/citygml/2.0"
                         xmlns:bldg="http://www.opengis.net/citygml/building/2.0"
                         xmlns:uro="https://www.geospatial.jp/iur/uro/3.1"
                         xmlns:gml="http://www.opengis.net/gml"
                         xmlns:xlink="http://www.w3.org/1999/xlink">{body}</core:CityModel>"#
                );
                parser
                    .parse(
                        xml.as_bytes(),
                        &Url::parse(&format!("file:///data/{name}")).unwrap(),
                    )
                    .unwrap();
            }
            let ParserOutput {
                pending,
                raw_registry,
                geom_registry,
                appearance_members,
                srs_by_file,
                ns_registry,
                version,
                ..
            } = parser.finish();
            let appearance = appearance::build_index(&appearance_members, &raw_registry);
            assemble_features(
                pending,
                &raw_registry,
                &geom_registry,
                &appearance,
                &srs_by_file,
                &ns_registry,
                version,
                &tags,
                &HashMap::new(),
                None,
                keep_attributes,
                &[],
                false,
                &mut Vec::new(),
            )
        }

        /// Each member's attributes, in member order.
        fn member_attrs(feature: &Feature) -> Vec<Attributes> {
            match &*feature.geometry {
                Geometry::GeometryCollection(gc) => gc.member_attributes().to_vec(),
                other => panic!("expected GeometryCollection, got {other:?}"),
            }
        }

        /// The attributes of the one member whose owner has type `owner_type`.
        fn member_owned_by(feature: &Feature, owner_type: &str) -> Attributes {
            let owned: Vec<Attributes> = member_attrs(feature)
                .into_iter()
                .filter(|a| a.get(MEMBER_GEOMETRY_FEATURE_TYPE_KEY) == Some(&string(owner_type)))
                .collect();
            assert_eq!(owned.len(), 1, "expected one member owned by {owner_type}");
            owned.into_iter().next().unwrap()
        }

        fn string(value: &str) -> AttributeValue {
            AttributeValue::String(value.to_string())
        }

        /// A CityGML 2.0 `bldg:lod<N>MultiSurface` property.
        fn v2_surface(lod: u8) -> String {
            format!(
                "<bldg:lod{lod}MultiSurface><gml:MultiSurface><gml:surfaceMember>{TA}</gml:surfaceMember></gml:MultiSurface></bldg:lod{lod}MultiSurface>"
            )
        }

        /// A `bldg:Building` city object member holding `inner`.
        fn v2_building(id: &str, inner: &str) -> String {
            format!(
                "<core:cityObjectMember><bldg:Building gml:id=\"{id}\">{inner}</bldg:Building></core:cityObjectMember>"
            )
        }

        /// A boundary wall with its own lod2 surface followed by `extra`. `id_attr`
        /// is the wall's `gml:id` attribute text, empty for an id-less wall.
        fn v2_wall(id_attr: &str, extra: &str) -> String {
            format!(
                "<bldg:boundedBy><bldg:WallSurface{id_attr}>{}{extra}</bldg:WallSurface></bldg:boundedBy>",
                v2_surface(2)
            )
        }

        /// A window opening with its own lod3 surface.
        fn v2_window(id: &str) -> String {
            format!(
                "<bldg:opening><bldg:Window gml:id=\"{id}\">{}</bldg:Window></bldg:opening>",
                v2_surface(3)
            )
        }

        /// The tree entry of the first wall under `feature`.
        fn wall_tree(feature: &Feature) -> Attributes {
            feature
                .get("bldg:boundedBy")
                .and_then(|v| v.as_map())
                .and_then(|m| m.get("bldg:WallSurface").cloned())
                .and_then(|v| v.as_map())
                .expect("the wall's tree entry")
        }

        /// Every attribute-tree key under `value`, at any depth.
        fn tree_keys(value: &AttributeValue, out: &mut Vec<String>) {
            match value {
                AttributeValue::Map(m) => {
                    for (k, v) in m {
                        out.push(k.to_string());
                        tree_keys(v, out);
                    }
                }
                AttributeValue::Array(a) => a.iter().for_each(|v| tree_keys(v, out)),
                _ => {}
            }
        }

        /// Every attribute-tree key of `feature`, at any depth.
        fn all_tree_keys(feature: &Feature) -> Vec<String> {
            let mut keys = Vec::new();
            for (k, v) in feature.attributes.iter() {
                keys.push(k.to_string());
                tree_keys(v, &mut keys);
            }
            keys
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

        #[test]
        fn case13_each_member_names_the_object_it_belongs_to() {
            // Without `extract_tags` the nested TrafficArea stays inside the Road
            // feature, so only the per-member attributes tell the two apart.
            let members = format!(
                "<core:cityObjectMember><tran:Road gml:id=\"road1\">\
                   <core:lod1MultiSurface><gml:MultiSurface><gml:surfaceMember>{TA}</gml:surfaceMember></gml:MultiSurface></core:lod1MultiSurface>\
                   <tran:trafficArea><tran:TrafficArea gml:id=\"ta1\">\
                     <core:lod3MultiSurface><gml:MultiSurface><gml:surfaceMember>{TB}</gml:surfaceMember></gml:MultiSurface></core:lod3MultiSurface>\
                   </tran:TrafficArea></tran:trafficArea>\
                 </tran:Road></core:cityObjectMember>"
            );
            let features = run(&members, &[]);
            assert_eq!(features.len(), 1);
            let Geometry::GeometryCollection(gc) = &*features[0].geometry else {
                panic!(
                    "expected GeometryCollection, got {:?}",
                    features[0].geometry
                );
            };
            let owners: Vec<(
                Option<&AttributeValue>,
                Option<&AttributeValue>,
                Option<&AttributeValue>,
            )> = gc
                .member_attributes()
                .iter()
                .map(|a| {
                    (
                        a.get(&Attribute::new(MEMBER_LOD_KEY)),
                        a.get(&Attribute::new(MEMBER_GEOMETRY_GML_ID_KEY)),
                        a.get(&Attribute::new(MEMBER_GEOMETRY_FEATURE_TYPE_KEY)),
                    )
                })
                .collect();
            let expected = |lod: u64, id: &str, ty: &str| {
                (
                    AttributeValue::Number(lod.into()),
                    AttributeValue::String(id.to_string()),
                    AttributeValue::String(ty.to_string()),
                )
            };
            let lod1 = expected(1, "road1", "tran:Road");
            let lod3 = expected(3, "ta1", "tran:TrafficArea");
            assert_eq!(
                owners,
                vec![
                    (Some(&lod1.0), Some(&lod1.1), Some(&lod1.2)),
                    (Some(&lod3.0), Some(&lod3.1), Some(&lod3.2)),
                ]
            );
        }

        #[test]
        fn a_window_records_its_wall_as_owner_parent() {
            let body = v2_building("b1", &v2_wall(" gml:id=\"w1\"", &v2_window("win1")));
            let features = run_v2(&[("a.gml", &body)], &[], true);
            assert_eq!(features.len(), 1);
            let window = member_owned_by(&features[0], "bldg:Window");
            assert_eq!(
                window.get(MEMBER_GEOMETRY_GML_ID_KEY),
                Some(&string("win1"))
            );
            assert_eq!(
                window.get(MEMBER_GEOMETRY_PARENT_GML_ID_KEY),
                Some(&string("w1"))
            );
            let wall = member_owned_by(&features[0], "bldg:WallSurface");
            assert_eq!(wall.get(MEMBER_GEOMETRY_PARENT_GML_ID_KEY), None);
        }

        #[test]
        fn an_id_less_nested_owner_gets_a_document_scoped_synthetic_key() {
            let body = v2_building("b1", &format!("{}{}", v2_surface(1), v2_wall("", "")));
            let features = run_v2(&[("a.gml", &body)], &[], true);
            let wall = member_owned_by(&features[0], "bldg:WallSurface");
            assert_eq!(
                wall.get(MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY),
                Some(&string("a.gml#_synid1"))
            );
            assert_eq!(wall.get(MEMBER_GEOMETRY_GML_ID_KEY), None);
            let building = member_owned_by(&features[0], "bldg:Building");
            assert_eq!(
                building.keys().map(Attribute::as_str).collect::<Vec<_>>(),
                vec![
                    MEMBER_LOD_KEY,
                    MEMBER_GML_PROPERTY_NAME_KEY,
                    MEMBER_GEOMETRY_NAME_KEY,
                    MEMBER_GEOMETRY_FEATURE_TYPE_KEY,
                    MEMBER_GEOMETRY_GML_ID_KEY,
                ]
            );
            assert_eq!(
                building.get(MEMBER_GEOMETRY_GML_ID_KEY),
                Some(&string("b1"))
            );
        }

        #[test]
        fn an_id_less_walls_key_is_its_windows_owner_parent() {
            let body = v2_building("b1", &v2_wall("", &v2_window("win1")));
            let features = run_v2(&[("a.gml", &body)], &[], true);
            let window = member_owned_by(&features[0], "bldg:Window");
            assert_eq!(
                window.get(MEMBER_GEOMETRY_PARENT_GML_ID_KEY),
                Some(&string("a.gml#_synid1"))
            );
        }

        #[test]
        fn synthetic_keys_restart_per_document() {
            let a = v2_building("b1", &v2_wall("", ""));
            let b = v2_building("b2", &v2_wall("", ""));
            let features = run_v2(&[("a.gml", &a), ("b.gml", &b)], &[], true);
            let keys: Vec<_> = features
                .iter()
                .map(|f| {
                    member_owned_by(f, "bldg:WallSurface")
                        .get(MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY)
                        .cloned()
                })
                .collect();
            assert_eq!(
                keys,
                vec![Some(string("a.gml#_synid1")), Some(string("b.gml#_synid1"))]
            );
        }

        #[test]
        fn an_id_less_object_without_geometry_gets_no_key() {
            // The id-less wall holds only a name, so it owns no member and its
            // tree entry keeps just its data.
            let body = v2_building(
                "b1",
                &format!(
                    "{}<bldg:boundedBy><bldg:WallSurface><gml:name>w</gml:name></bldg:WallSurface></bldg:boundedBy>",
                    v2_surface(1)
                ),
            );
            let features = run_v2(&[("a.gml", &body)], &[], true);
            assert_eq!(
                wall_tree(&features[0]),
                Attributes::from([(Attribute::new("gml:name"), string("w"))])
            );
            for member in member_attrs(&features[0]) {
                assert_eq!(member.get(MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY), None);
            }
        }

        #[test]
        fn the_synthetic_key_sits_in_the_attribute_tree_beside_the_objects_data() {
            let body = v2_building("b1", &v2_wall("", "<gml:name>w</gml:name>"));
            let features = run_v2(&[("a.gml", &body)], &[], true);
            let key = member_owned_by(&features[0], "bldg:WallSurface")
                .get(MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY)
                .cloned()
                .expect("the member's synthetic key");
            let wall = wall_tree(&features[0]);
            assert_eq!(wall.get(SYNTHETIC_OWNER_TREE_KEY), Some(&key));
            assert_eq!(wall.get("gml:name"), Some(&string("w")));
            assert_eq!(wall.get("@gml:id"), None);
        }

        #[test]
        fn with_keep_attributes_off_the_tree_has_no_synthetic_key() {
            let body = v2_building("b1", &v2_wall("", "<gml:name>w</gml:name>"));
            let features = run_v2(&[("a.gml", &body)], &[], false);
            assert_eq!(wall_tree(&features[0]).get(SYNTHETIC_OWNER_TREE_KEY), None);
            assert_eq!(
                member_owned_by(&features[0], "bldg:WallSurface")
                    .get(MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY),
                Some(&string("a.gml#_synid1"))
            );
        }

        #[test]
        fn an_id_less_ade_element_with_geometry_is_keyed_like_a_citygml_object() {
            // The key is what lets the writer put the geometry back inside the
            // DM attribute rather than on the wall.
            let dm = format!(
                "<uro:wallSurfaceDmAttribute><uro:DmGeometricAttribute>\
                   <uro:dmCode>1</uro:dmCode>\
                   <uro:lod0anyGeometry><gml:MultiSurface><gml:surfaceMember>{TA}</gml:surfaceMember></gml:MultiSurface></uro:lod0anyGeometry>\
                 </uro:DmGeometricAttribute></uro:wallSurfaceDmAttribute>"
            );
            let body = v2_building("b1", &v2_wall(" gml:id=\"w1\"", &dm));
            let features = run_v2(&[("a.gml", &body)], &[], true);
            let dm_member = member_owned_by(&features[0], "uro:DmGeometricAttribute");
            let key = string("a.gml#_synid1");
            assert_eq!(
                dm_member.get(MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY),
                Some(&key)
            );
            assert_eq!(dm_member.get(MEMBER_GEOMETRY_GML_ID_KEY), None);
            assert_eq!(
                dm_member.get(MEMBER_GEOMETRY_PARENT_GML_ID_KEY),
                Some(&string("w1"))
            );
            let dm_tree = wall_tree(&features[0])
                .get("uro:wallSurfaceDmAttribute")
                .and_then(|v| v.as_map())
                .and_then(|m| m.get("uro:DmGeometricAttribute").cloned())
                .expect("the ADE object's tree entry");
            assert_eq!(
                dm_tree,
                AttributeValue::Map(Attributes::from([
                    (Attribute::new("uro:dmCode"), string("1")),
                    (Attribute::new(SYNTHETIC_OWNER_TREE_KEY), key),
                ]))
            );
        }

        #[test]
        fn a_document_whose_nested_objects_all_have_ids_gains_no_new_keys() {
            let body = v2_building(
                "b1",
                &format!(
                    "{}{}{}",
                    v2_surface(1),
                    v2_wall(" gml:id=\"w1\"", ""),
                    v2_wall(" gml:id=\"w2\"", "")
                ),
            );
            for tags in [&[][..], &["Building"][..]] {
                let features = run_v2(&[("a.gml", &body)], tags, true);
                assert_eq!(features.len(), 1);
                let members = member_attrs(&features[0]);
                assert_eq!(members.len(), 3);
                for member in members {
                    assert_eq!(
                        member.keys().map(Attribute::as_str).collect::<Vec<_>>(),
                        vec![
                            MEMBER_LOD_KEY,
                            MEMBER_GML_PROPERTY_NAME_KEY,
                            MEMBER_GEOMETRY_NAME_KEY,
                            MEMBER_GEOMETRY_FEATURE_TYPE_KEY,
                            MEMBER_GEOMETRY_GML_ID_KEY,
                        ]
                    );
                }
                assert!(!all_tree_keys(&features[0])
                    .iter()
                    .any(|k| k == SYNTHETIC_OWNER_TREE_KEY));
                assert_eq!(features[0].get(CITYGML_PARENT_OBJECT_GML_ID_KEY), None);
            }
        }

        #[test]
        fn an_extracted_window_names_its_real_parent_object() {
            let body = v2_building("b1", &v2_wall(" gml:id=\"w1\"", &v2_window("win1")));
            let features = run_v2(&[("a.gml", &body)], &["Building", "Window"], true);
            let window = by_type(&features, "bldg:Window");
            assert_eq!(window.get(CITYGML_PARENT_GML_ID_KEY), Some(&string("b1")));
            assert_eq!(
                window.get(CITYGML_PARENT_OBJECT_GML_ID_KEY),
                Some(&string("w1"))
            );
            let building = by_type(&features, "bldg:Building");
            assert_eq!(building.get(CITYGML_PARENT_OBJECT_GML_ID_KEY), None);
        }

        #[test]
        fn an_extracted_window_under_an_id_less_wall_names_the_walls_synthetic_key() {
            let body = v2_building("b1", &v2_wall("", &v2_window("win1")));
            let features = run_v2(&[("a.gml", &body)], &["Building", "Window"], true);
            let wall_key = member_owned_by(by_type(&features, "bldg:Building"), "bldg:WallSurface")
                .get(MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY)
                .cloned()
                .expect("the wall's synthetic key");
            let window = by_type(&features, "bldg:Window");
            assert_eq!(window.get(CITYGML_PARENT_GML_ID_KEY), Some(&string("b1")));
            assert_eq!(
                window.get(CITYGML_PARENT_OBJECT_GML_ID_KEY),
                Some(&wall_key)
            );
        }

        #[test]
        fn a_citygml_3_document_gains_none_of_the_new_keys() {
            let surface = |lod: u8| {
                format!(
                    "<core:lod{lod}MultiSurface><gml:MultiSurface><gml:surfaceMember>{TA}</gml:surfaceMember></gml:MultiSurface></core:lod{lod}MultiSurface>"
                )
            };
            let window = |id: &str| {
                format!(
                    "<con:fillingSurface><con:WindowSurface gml:id=\"{id}\">{}</con:WindowSurface></con:fillingSurface>",
                    surface(3)
                )
            };
            let members = format!(
                "<core:cityObjectMember><bldg:Building gml:id=\"b1\">\
                   <core:boundary><con:WallSurface>{}{}</con:WallSurface></core:boundary>\
                   <core:boundary><con:WallSurface gml:id=\"w2\">{}{}</con:WallSurface></core:boundary>\
                 </bldg:Building></core:cityObjectMember>",
                surface(2),
                window("win1"),
                surface(2),
                window("win2")
            );
            for tags in [&[][..], &["Building", "WindowSurface"][..]] {
                let features = run(&members, tags);
                assert!(!features.is_empty());
                for feature in &features {
                    for member in member_attrs(feature) {
                        assert_eq!(member.get(MEMBER_GEOMETRY_PARENT_GML_ID_KEY), None);
                        assert_eq!(member.get(MEMBER_GEOMETRY_SYNTHETIC_OWNER_KEY), None);
                    }
                    assert!(!all_tree_keys(feature)
                        .iter()
                        .any(|k| k == SYNTHETIC_OWNER_TREE_KEY));
                    assert_eq!(feature.get(CITYGML_PARENT_OBJECT_GML_ID_KEY), None);
                }
            }
        }

        /// A malformation found while resolving a city object's geometry names
        /// that city object, like one found while parsing it.
        #[test]
        fn a_malformation_found_while_resolving_names_the_city_object() {
            let shell = format!(
                "<gml:Shell><gml:surfaceMember>{TA}</gml:surfaceMember><gml:surfaceMember>{TB}</gml:surfaceMember></gml:Shell>"
            );
            let xml = format!(
                r#"<core:CityModel
                     xmlns:core="http://www.opengis.net/citygml/3.0"
                     xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
                     xmlns:gml="http://www.opengis.net/gml/3.2">
                   <core:cityObjectMember><bldg:Building gml:id="b1"><core:lod1Solid><gml:Solid>
                     <gml:exterior>{shell}</gml:exterior><gml:exterior>{shell}</gml:exterior>
                   </gml:Solid></core:lod1Solid></bldg:Building></core:cityObjectMember>
                 </core:CityModel>"#
            );
            let mut parser = Parser::new(CityGmlVersion::V3);
            parser
                .parse(xml.as_bytes(), &Url::parse("file:///test.gml").unwrap())
                .unwrap();
            let (_, malformations) = build_features_reporting(
                parser,
                &HashSet::new(),
                &HashMap::new(),
                None,
                true,
                false,
                &[],
                false,
            );
            let found: Vec<_> = malformations
                .iter()
                .map(|m| {
                    (
                        m.kind.clone(),
                        m.detail.city_object_id.as_str(),
                        m.detail.city_object_type.as_str(),
                    )
                })
                .collect();
            assert_eq!(
                found,
                vec![(
                    MalformationKind::SolidWithMultipleExteriors,
                    "b1",
                    "bldg:Building"
                )]
            );
        }
    }
}
