//! L03(plateau6) application schema conformance (応用スキーマへの適合): checks
//! each feature type (地物型), attribute (主題属性・空間属性) and association
//! role (関連役割) a feature uses against the objectlist. A value the objectlist
//! does not break down (geometry, an address, ...) is left to the XSD
//! validation of L02.

use std::collections::{HashMap, HashSet};

use fastxml::schema::types::CompiledSchema;
use fastxml::transform::EditableNodeRef;

use super::PlateauProfile;
use crate::object_list::ObjectListDefinitions;

/// Why a use is an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Reason {
    /// The objectlist has no row for it.
    Undefined,
    /// Its row is not a creation target (作成対象).
    Unusable,
}

impl Reason {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Reason::Undefined => "未定義",
            Reason::Unusable => "利用不可",
        }
    }
}

/// One erroneous use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Violation {
    /// The innermost feature holding the use.
    pub(super) feature_type: String,
    pub(super) gml_id: String,
    /// `/`-separated element path from that feature. Empty when the feature
    /// type itself is the use.
    pub(super) path: String,
    pub(super) reason: Reason,
}

#[derive(Debug, Clone)]
struct FeatureTypeRules {
    usable: bool,
    /// Each path and whether its row is a creation target (作成対象).
    paths: HashMap<String, bool>,
    /// Every path some row lies below.
    parents: HashSet<String>,
}

/// The objectlist definitions, indexed for the check.
#[derive(Debug, Clone)]
pub(super) struct ConformanceRules {
    packages: HashMap<String, HashMap<String, FeatureTypeRules>>,
    /// Feature types defined in any package.
    feature_types: HashSet<String>,
    profile: &'static PlateauProfile,
}

impl ConformanceRules {
    pub(super) fn new(
        definitions: ObjectListDefinitions,
        profile: &'static PlateauProfile,
    ) -> Self {
        let mut feature_types = HashSet::new();
        let packages = definitions
            .into_inner()
            .into_iter()
            .map(|(package, types)| {
                let types = types
                    .into_iter()
                    .map(|(feature_type, definition)| {
                        feature_types.insert(feature_type.clone());
                        let parents = definition
                            .paths
                            .keys()
                            .flat_map(|path| {
                                path.match_indices('/').map(|(i, _)| path[..i].to_string())
                            })
                            .collect();
                        let rules = FeatureTypeRules {
                            usable: definition.usable,
                            paths: definition.paths,
                            parents,
                        };
                        (feature_type, rules)
                    })
                    .collect();
                (package, types)
            })
            .collect();
        warn_unknown_prefixes(&packages, profile);
        Self {
            packages,
            feature_types,
            profile,
        }
    }

    /// The rules of a package, or `None` when the objectlist has no sheet for
    /// it and its uses are not checked.
    pub(super) fn package<'a>(
        &'a self,
        package: &str,
        city_objects: Option<CityObjectTypes<'a>>,
    ) -> Option<PackageRules<'a>> {
        self.packages.get(package).map(|types| PackageRules {
            types,
            feature_types: &self.feature_types,
            city_objects,
            profile: self.profile,
        })
    }
}

/// The city object types (都市オブジェクト) of the application schema: the
/// elements substitutable for `core:AbstractCityObject`. The objectlist defines
/// each of them as a feature type of its own, unlike the other values of an
/// association role.
#[derive(Debug, Clone, Copy)]
pub(super) struct CityObjectTypes<'a> {
    schema: &'a CompiledSchema,
    core_ns: &'a str,
}

impl<'a> CityObjectTypes<'a> {
    pub(super) fn new(schema: &'a CompiledSchema, core_ns: &'a str) -> Self {
        Self { schema, core_ns }
    }

    fn contains(&self, element: &EditableNodeRef) -> bool {
        let Some(namespace) = element.namespace_uri() else {
            return false;
        };
        let mut current = self.schema.element_ns(&namespace, &element.name());
        // Bounds a cyclic substitution group in a broken schema.
        for _ in 0..32 {
            let Some(head) = current.and_then(|element| element.substitution_ns.as_ref()) else {
                return false;
            };
            if &*head.namespace_uri == self.core_ns && &*head.local_name == "AbstractCityObject" {
                return true;
            }
            current = self
                .schema
                .element_ns(&head.namespace_uri, &head.local_name);
        }
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PackageRules<'a> {
    types: &'a HashMap<String, FeatureTypeRules>,
    feature_types: &'a HashSet<String>,
    /// `None` when the schema could not be compiled, and a city object type
    /// the objectlist does not define passes.
    city_objects: Option<CityObjectTypes<'a>>,
    profile: &'a PlateauProfile,
}

impl PackageRules<'_> {
    /// The name of `element` as the objectlist writes it.
    fn name(&self, element: &EditableNodeRef) -> Name {
        let local = element.name();
        let key = match element.namespace_uri().filter(|ns| !ns.is_empty()) {
            Some(ns) => match self.profile.prefix_of(&ns) {
                Some(prefix) => format!("{prefix}:{local}"),
                // Matches no row, whatever prefix the document binds to `ns`.
                None => {
                    return Name {
                        key: format!("{{{ns}}}{local}"),
                        shown: element.qname(),
                    }
                }
            },
            None => local,
        };
        Name {
            shown: key.clone(),
            key,
        }
    }

    /// Checks a feature and every feature nested in it.
    pub(super) fn check_feature(
        &self,
        feature: &EditableNodeRef,
        gml_ns: &str,
        violations: &mut Vec<Violation>,
    ) {
        let feature_type = self.name(feature);
        let owner = Owner {
            feature_type: feature_type.shown,
            gml_id: feature.get_attribute_ns(gml_ns, "id").unwrap_or_default(),
        };
        let Some(rules) = self.types.get(&feature_type.key) else {
            violations.push(owner.violation(String::new(), Reason::Undefined));
            return;
        };
        // Each use under an unusable row is still checked on its own.
        if !rules.usable {
            violations.push(owner.violation(String::new(), Reason::Unusable));
        }
        self.check_properties(&owner, rules, "", feature, gml_ns, violations);
    }

    /// Checks the property elements of `object`, whose path from `owner` is
    /// `prefix`.
    fn check_properties(
        &self,
        owner: &Owner,
        rules: &FeatureTypeRules,
        prefix: &str,
        object: &EditableNodeRef,
        gml_ns: &str,
        violations: &mut Vec<Violation>,
    ) {
        // A use matching a row is in a known namespace, so `prefix` is both
        // the key and the shown path.
        for property in object.children() {
            let name = self.name(&property);
            let path = join(prefix, &name.key);
            match rules.paths.get(&path) {
                Some(false) => {
                    violations.push(owner.violation(path.clone(), Reason::Unusable));
                }
                // A row may name the property with its type
                // (`core:genericAttribute.gen:CodeAttribute`), leaving no row
                // for the property alone.
                None if !rules.parents.contains(&path) => {
                    violations.push(owner.violation(join(prefix, &name.shown), Reason::Undefined));
                    continue;
                }
                _ => {}
            }
            for value in property.children() {
                let value_type = self.name(&value);
                let value_path = join(&path, &value_type.key);
                let shown_path = || join(&path, &value_type.shown);
                if let Some(&usable) = rules.paths.get(&value_path) {
                    if !usable {
                        violations.push(owner.violation(value_path.clone(), Reason::Unusable));
                    }
                    self.check_properties(owner, rules, &value_path, &value, gml_ns, violations);
                } else if rules.parents.contains(&path) {
                    violations.push(owner.violation(shown_path(), Reason::Undefined));
                } else if self.types.contains_key(&value_type.key) {
                    self.check_feature(&value, gml_ns, violations);
                } else if self.feature_types.contains(&value_type.key)
                    || self
                        .city_objects
                        .is_some_and(|city_objects| city_objects.contains(&value))
                {
                    // A feature type the objectlist does not define for the
                    // package.
                    violations.push(owner.violation(shown_path(), Reason::Undefined));
                }
                // Otherwise the objectlist does not break the value down
                // (geometry, an address, ...), and L02 checks it.
            }
        }
    }
}

/// The feature whose rows a use is checked against.
struct Owner {
    feature_type: String,
    gml_id: String,
}

impl Owner {
    fn violation(&self, path: String, reason: Reason) -> Violation {
        Violation {
            feature_type: self.feature_type.clone(),
            gml_id: self.gml_id.clone(),
            path,
            reason,
        }
    }
}

/// An element name: `key` to match against the objectlist, `shown` to report.
struct Name {
    /// `prefix:local` with the standard prefix of the namespace,
    /// `{namespace}local` for a namespace without one, or `local` alone.
    key: String,
    /// `key`, or the document's own `prefix:local` for a namespace without a
    /// standard prefix.
    shown: String,
}

/// Warns of an objectlist row whose prefix has no namespace URI, as no
/// element can match it.
fn warn_unknown_prefixes(
    packages: &HashMap<String, HashMap<String, FeatureTypeRules>>,
    profile: &PlateauProfile,
) {
    let unknown: HashSet<&str> = packages
        .values()
        .flat_map(|types| types.iter())
        .flat_map(|(feature_type, rules)| {
            std::iter::once(feature_type.as_str())
                .chain(rules.paths.keys().flat_map(|path| path.split('/')))
        })
        .filter_map(|name| name.split_once(':').map(|(prefix, _)| prefix))
        .filter(|prefix| profile.namespace(prefix).is_none())
        .collect();
    if !unknown.is_empty() {
        tracing::warn!(
            ?unknown,
            "L03: objectlist prefixes without a namespace URI never match an element"
        );
    }
}

fn join(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}/{name}")
    }
}

#[cfg(test)]
mod tests {
    use fastxml::schema::fetcher::{CachingFetcher, NoopFetcher, SchemaFetcher};
    use fastxml::schema::xsd;
    use fastxml::transform::Transformer;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::object_list::FeatureTypeDefinition;

    const GML_NS: &str = "http://www.opengis.net/gml/3.2";
    const CORE_NS: &str = "http://www.opengis.net/citygml/3.0";

    static TEST_PROFILE: PlateauProfile = PlateauProfile {
        citygml: &crate::citygml::CITYGML3,
        uro_ns: "https://www.geospatial.jp/iur/uro/4.0",
        urf_ns: "https://www.geospatial.jp/iur/urf/4.0",
        urc_ns: Some("https://www.geospatial.jp/iur/urc/4.0"),
        action_prefix: "PLATEAU6",
    };

    fn definitions() -> ObjectListDefinitions {
        let definition = |usable: bool, paths: &[(&str, bool)]| FeatureTypeDefinition {
            usable,
            paths: paths
                .iter()
                .map(|(path, usable)| (path.to_string(), *usable))
                .collect(),
        };
        let building = HashMap::from([
            (
                "bldg:Building".to_string(),
                definition(
                    true,
                    &[
                        ("gml:name", true),
                        ("bldg:class", false),
                        ("bldg:buildingInstallation", false),
                        ("core:boundary", true),
                        ("core:lod1Solid", true),
                        ("bldg:buildingPart", true),
                        ("bldg:address", true),
                        ("bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute", true),
                        (
                            "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute/uro:buildingID",
                            true,
                        ),
                        (
                            "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute/uro:city",
                            false,
                        ),
                        (
                            "core:adeOfAbstractCityObject/uro:FacilityIdAttribute",
                            false,
                        ),
                        (
                            "core:adeOfAbstractCityObject/uro:FacilityIdAttribute/uro:facilityId",
                            false,
                        ),
                    ],
                ),
            ),
            (
                "con:WallSurface".to_string(),
                definition(true, &[("core:lod2MultiSurface", true)]),
            ),
            (
                "con:RoofSurface".to_string(),
                definition(
                    false,
                    &[("gml:name", true), ("core:lod2MultiSurface", false)],
                ),
            ),
        ]);
        let road = HashMap::from([("tran:TrafficArea".to_string(), definition(true, &[]))]);
        ObjectListDefinitions::new(HashMap::from([
            ("bldg".to_string(), building),
            ("tran".to_string(), road),
        ]))
    }

    /// `core:Address` is a feature but not a city object, as in CityGML 3.0.
    fn schema() -> CompiledSchema {
        let core = br#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
            xmlns:core="http://www.opengis.net/citygml/3.0"
            targetNamespace="http://www.opengis.net/citygml/3.0">
            <xs:element name="AbstractFeature" abstract="true"/>
            <xs:element name="AbstractCityObject" abstract="true" substitutionGroup="core:AbstractFeature"/>
            <xs:element name="Address" substitutionGroup="core:AbstractFeature"/>
        </xs:schema>"#;
        let building = br#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
            xmlns:core="http://www.opengis.net/citygml/3.0"
            xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
            targetNamespace="http://www.opengis.net/citygml/building/3.0">
            <xs:import namespace="http://www.opengis.net/citygml/3.0" schemaLocation="core.xsd"/>
            <xs:element name="AbstractBuilding" abstract="true" substitutionGroup="core:AbstractCityObject"/>
            <xs:element name="BuildingPart" substitutionGroup="bldg:AbstractBuilding"/>
        </xs:schema>"#;
        let fetcher = CachingFetcher::new(NoopFetcher);
        fetcher.seed("http://test/core.xsd", core.to_vec());
        let mut resolver = xsd::SchemaResolver::new(&fetcher);
        resolver
            .resolve_entry(building, "http://test/building.xsd")
            .unwrap();
        let mut compiled = xsd::compile_schemas(resolver.take_all_schemas()).unwrap();
        xsd::register_builtin_types(&mut compiled);
        compiled
    }

    fn check(xml: &str, city_objects: Option<CityObjectTypes>) -> Vec<Violation> {
        let rules = ConformanceRules::new(definitions(), &TEST_PROFILE);
        let package = rules.package("bldg", city_objects).unwrap();
        let mut violations = Vec::new();
        Transformer::from(xml)
            .with_root_namespaces()
            .unwrap()
            .on("//*[local-name()='cityObjectMember']", |node| {
                let feature = node.children().into_iter().next().unwrap();
                package.check_feature(&feature, GML_NS, &mut violations);
            })
            .for_each()
            .unwrap();
        violations
    }

    fn violation(feature_type: &str, gml_id: &str, path: &str, reason: Reason) -> Violation {
        Violation {
            feature_type: feature_type.to_string(),
            gml_id: gml_id.to_string(),
            path: path.to_string(),
            reason,
        }
    }

    fn city_model(members: &str) -> String {
        format!(
            r#"<core:CityModel xmlns:core="http://www.opengis.net/citygml/3.0"
                xmlns:gml="http://www.opengis.net/gml/3.2"
                xmlns:bldg="http://www.opengis.net/citygml/building/3.0"
                xmlns:con="http://www.opengis.net/citygml/construction/3.0"
                xmlns:tran="http://www.opengis.net/citygml/transportation/3.0"
                xmlns:veg="http://www.opengis.net/citygml/vegetation/3.0"
                xmlns:uro="https://www.geospatial.jp/iur/uro/4.0">{members}</core:CityModel>"#
        )
    }

    #[test]
    fn test_conforming_feature() {
        let xml = city_model(
            r#"<core:cityObjectMember><bldg:Building gml:id="b1">
                <gml:name>a</gml:name>
                <core:lod1Solid><gml:Solid/></core:lod1Solid>
                <bldg:adeOfAbstractBuilding><uro:BuildingIDAttribute>
                    <uro:buildingID>x</uro:buildingID>
                </uro:BuildingIDAttribute></bldg:adeOfAbstractBuilding>
                <core:boundary><con:WallSurface gml:id="w1">
                    <core:lod2MultiSurface><gml:MultiSurface/></core:lod2MultiSurface>
                </con:WallSurface></core:boundary>
            </bldg:Building></core:cityObjectMember>"#,
        );
        assert_eq!(check(&xml, None), vec![]);
    }

    #[test]
    fn test_violations() {
        let xml = city_model(
            r#"<core:cityObjectMember><veg:PlantCover gml:id="p1"/></core:cityObjectMember>
            <core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:class>1</bldg:class>
                <bldg:usage>1</bldg:usage>
                <bldg:adeOfAbstractBuilding><uro:BuildingIDAttribute>
                    <uro:city>x</uro:city>
                    <uro:branchID>1</uro:branchID>
                </uro:BuildingIDAttribute></bldg:adeOfAbstractBuilding>
                <bldg:adeOfAbstractBuilding><uro:BuildingDetailAttribute/></bldg:adeOfAbstractBuilding>
                <core:adeOfAbstractCityObject><uro:FacilityIdAttribute>
                    <uro:facilityId>x</uro:facilityId>
                    <uro:facilityType>1</uro:facilityType>
                </uro:FacilityIdAttribute></core:adeOfAbstractCityObject>
                <bldg:buildingInstallation><con:WallSurface gml:id="w2">
                    <gml:name>a</gml:name>
                </con:WallSurface></bldg:buildingInstallation>
                <core:boundary><tran:TrafficArea gml:id="t1"/></core:boundary>
                <core:boundary><con:RoofSurface gml:id="r1">
                    <gml:name>a</gml:name>
                    <core:lod2MultiSurface><gml:MultiSurface/></core:lod2MultiSurface>
                    <bldg:usage>1</bldg:usage>
                </con:RoofSurface></core:boundary>
                <core:boundary><con:WallSurface gml:id="w1">
                    <gml:name>a</gml:name>
                </con:WallSurface></core:boundary>
            </bldg:Building></core:cityObjectMember>"#,
        );
        let building = |path: &str, reason| violation("bldg:Building", "b1", path, reason);
        assert_eq!(
            check(&xml, None),
            vec![
                violation("veg:PlantCover", "p1", "", Reason::Undefined),
                building("bldg:class", Reason::Unusable),
                building("bldg:usage", Reason::Undefined),
                building(
                    "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute/uro:city",
                    Reason::Unusable
                ),
                building(
                    "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute/uro:branchID",
                    Reason::Undefined
                ),
                building(
                    "bldg:adeOfAbstractBuilding/uro:BuildingDetailAttribute",
                    Reason::Undefined
                ),
                building(
                    "core:adeOfAbstractCityObject/uro:FacilityIdAttribute",
                    Reason::Unusable
                ),
                building(
                    "core:adeOfAbstractCityObject/uro:FacilityIdAttribute/uro:facilityId",
                    Reason::Unusable
                ),
                building(
                    "core:adeOfAbstractCityObject/uro:FacilityIdAttribute/uro:facilityType",
                    Reason::Undefined
                ),
                building("bldg:buildingInstallation", Reason::Unusable),
                violation("con:WallSurface", "w2", "gml:name", Reason::Undefined),
                building("core:boundary/tran:TrafficArea", Reason::Undefined),
                violation("con:RoofSurface", "r1", "", Reason::Unusable),
                violation(
                    "con:RoofSurface",
                    "r1",
                    "core:lod2MultiSurface",
                    Reason::Unusable
                ),
                violation("con:RoofSurface", "r1", "bldg:usage", Reason::Undefined),
                violation("con:WallSurface", "w1", "gml:name", Reason::Undefined),
            ]
        );
    }

    #[test]
    fn test_undefined_city_object() {
        let xml = city_model(
            r#"<core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:buildingPart><bldg:BuildingPart gml:id="bp1"/></bldg:buildingPart>
                <bldg:address><core:Address><core:xalAddress/></core:Address></bldg:address>
            </bldg:Building></core:cityObjectMember>"#,
        );
        let schema = schema();
        assert_eq!(
            check(&xml, Some(CityObjectTypes::new(&schema, CORE_NS))),
            vec![violation(
                "bldg:Building",
                "b1",
                "bldg:buildingPart/bldg:BuildingPart",
                Reason::Undefined
            )]
        );
    }

    #[test]
    fn test_prefix_aliases() {
        let xml = r#"<c:CityModel xmlns:c="http://www.opengis.net/citygml/3.0"
            xmlns:g="http://www.opengis.net/gml/3.2"
            xmlns:b="http://www.opengis.net/citygml/building/3.0"
            xmlns:k="http://www.opengis.net/citygml/construction/3.0"
            xmlns:u="https://www.geospatial.jp/iur/uro/4.0">
            <c:cityObjectMember><b:Building g:id="b1">
                <g:name>a</g:name>
                <b:usage>1</b:usage>
                <b:adeOfAbstractBuilding><u:BuildingIDAttribute>
                    <u:buildingID>x</u:buildingID>
                </u:BuildingIDAttribute></b:adeOfAbstractBuilding>
                <c:boundary><k:WallSurface g:id="w1">
                    <c:lod2MultiSurface><g:MultiSurface/></c:lod2MultiSurface>
                </k:WallSurface></c:boundary>
            </b:Building></c:cityObjectMember>
        </c:CityModel>"#;
        assert_eq!(
            check(xml, None),
            vec![violation(
                "bldg:Building",
                "b1",
                "bldg:usage",
                Reason::Undefined
            )]
        );
    }

    #[test]
    fn test_standard_prefix_bound_to_other_namespace() {
        let xml = city_model(
            r#"<core:cityObjectMember><bldg:Building gml:id="b1">
                <bldg:adeOfAbstractBuilding><uro:BuildingIDAttribute>
                    <uro:buildingID>x</uro:buildingID>
                </uro:BuildingIDAttribute></bldg:adeOfAbstractBuilding>
            </bldg:Building></core:cityObjectMember>"#,
        )
        .replace("iur/uro/4.0", "iur/uro/3.1");
        assert_eq!(
            check(&xml, None),
            vec![violation(
                "bldg:Building",
                "b1",
                "bldg:adeOfAbstractBuilding/uro:BuildingIDAttribute",
                Reason::Undefined
            )]
        );
    }

    #[test]
    fn test_package_without_sheet() {
        let rules = ConformanceRules::new(definitions(), &TEST_PROFILE);
        assert!(rules.package("veg", None).is_none());
    }
}
