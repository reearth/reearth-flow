//! L03(plateau6) application schema conformance (応用スキーマへの適合): checks
//! each feature type (地物型), attribute (主題属性・空間属性) and association
//! role (関連役割) a feature uses against the objectlist. Uses the XSD does not
//! define are L02 errors and are not checked again.

use std::collections::{HashMap, HashSet};

use fastxml::transform::EditableNodeRef;

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
}

impl ConformanceRules {
    pub(super) fn new(definitions: ObjectListDefinitions) -> Self {
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
        Self {
            packages,
            feature_types,
        }
    }

    /// The rules of a package, or `None` when the objectlist has no sheet for
    /// it and its uses are not checked.
    pub(super) fn package(&self, package: &str) -> Option<PackageRules<'_>> {
        self.packages.get(package).map(|types| PackageRules {
            types,
            feature_types: &self.feature_types,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PackageRules<'a> {
    types: &'a HashMap<String, FeatureTypeRules>,
    feature_types: &'a HashSet<String>,
}

impl PackageRules<'_> {
    /// Checks a feature and every feature nested in it.
    pub(super) fn check_feature(
        &self,
        feature: &EditableNodeRef,
        gml_ns: &str,
        violations: &mut Vec<Violation>,
    ) {
        let owner = Owner {
            feature_type: feature.qname(),
            gml_id: feature.get_attribute_ns(gml_ns, "id").unwrap_or_default(),
        };
        let Some(rules) = self.types.get(&owner.feature_type) else {
            violations.push(owner.violation(String::new(), Reason::Undefined));
            return;
        };
        if !rules.usable {
            violations.push(owner.violation(String::new(), Reason::Unusable));
            return;
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
        for property in object.children() {
            let path = join(prefix, &property.qname());
            match rules.paths.get(&path) {
                Some(false) => {
                    violations.push(owner.violation(path, Reason::Unusable));
                    continue;
                }
                // A row may name the property with its type
                // (`core:genericAttribute.gen:CodeAttribute`), leaving no row
                // for the property alone.
                None if !rules.parents.contains(&path) => {
                    violations.push(owner.violation(path, Reason::Undefined));
                    continue;
                }
                _ => {}
            }
            for value in property.children() {
                let value_type = value.qname();
                let value_path = join(&path, &value_type);
                if let Some(&usable) = rules.paths.get(&value_path) {
                    if usable {
                        self.check_properties(
                            owner,
                            rules,
                            &value_path,
                            &value,
                            gml_ns,
                            violations,
                        );
                    } else {
                        violations.push(owner.violation(value_path, Reason::Unusable));
                    }
                } else if rules.parents.contains(&path) {
                    violations.push(owner.violation(value_path, Reason::Undefined));
                } else if self.types.contains_key(&value_type) {
                    self.check_feature(&value, gml_ns, violations);
                } else if self.feature_types.contains(&value_type) {
                    // A feature type of another package only.
                    violations.push(owner.violation(value_path, Reason::Undefined));
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

fn join(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}/{name}")
    }
}

#[cfg(test)]
mod tests {
    use fastxml::transform::Transformer;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::object_list::FeatureTypeDefinition;

    const GML_NS: &str = "http://www.opengis.net/gml/3.2";

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
                        ("core:boundary", true),
                        ("core:lod1Solid", true),
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
                    ],
                ),
            ),
            (
                "con:WallSurface".to_string(),
                definition(true, &[("core:lod2MultiSurface", true)]),
            ),
            ("con:RoofSurface".to_string(), definition(false, &[])),
        ]);
        let road = HashMap::from([("tran:TrafficArea".to_string(), definition(true, &[]))]);
        ObjectListDefinitions::new(HashMap::from([
            ("bldg".to_string(), building),
            ("tran".to_string(), road),
        ]))
    }

    fn check(xml: &str) -> Vec<Violation> {
        let rules = ConformanceRules::new(definitions());
        let package = rules.package("bldg").unwrap();
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
        assert_eq!(check(&xml), vec![]);
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
                <core:adeOfAbstractCityObject><uro:FacilityIdAttribute/></core:adeOfAbstractCityObject>
                <core:boundary><tran:TrafficArea gml:id="t1"/></core:boundary>
                <core:boundary><con:RoofSurface gml:id="r1"/></core:boundary>
                <core:boundary><con:WallSurface gml:id="w1">
                    <gml:name>a</gml:name>
                </con:WallSurface></core:boundary>
            </bldg:Building></core:cityObjectMember>"#,
        );
        let building = |path: &str, reason| violation("bldg:Building", "b1", path, reason);
        assert_eq!(
            check(&xml),
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
                building("core:boundary/tran:TrafficArea", Reason::Undefined),
                violation("con:RoofSurface", "r1", "", Reason::Unusable),
                violation("con:WallSurface", "w1", "gml:name", Reason::Undefined),
            ]
        );
    }

    #[test]
    fn test_package_without_sheet() {
        let rules = ConformanceRules::new(definitions());
        assert!(rules.package("veg").is_none());
    }
}
