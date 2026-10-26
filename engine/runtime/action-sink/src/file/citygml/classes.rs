//! Resolving the value a feature claims as its class to a CityGML element.
//!
//! Exact qualified-name resolution against the loaded schemas, replacing a
//! substring match onto a fixed list of classes.

use reearth_flow_citygml::schema::{QName, SchemaSet};

use super::writer::CITYGML_2_NAMESPACES;

const CITYGML_CORE_NS: &str = "http://www.opengis.net/citygml/2.0";
const GENERICS_NS: &str = "http://www.opengis.net/citygml/generics/2.0";
/// Declared locally where used, never on the root, so it is not in
/// `CITYGML_2_NAMESPACES`.
const XAL_NS: &str = "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0";

/// Prefixes that bind infrastructure namespaces, not city-object modules.
const NON_CLASS_PREFIXES: &[&str] = &["gml", "app", "xlink", "xsi", "xAL"];

/// The gml:id prefix of each of the 17 classes the writer has always minted
/// ids for, as `(module prefix, local name, id prefix)`. Keeps existing output
/// byte-identical.
const CORE_ID_PREFIXES: &[(&str, &str, &str)] = &[
    ("bldg", "Building", "bldg"),
    ("bldg", "BuildingPart", "bldg_part"),
    ("tran", "Road", "road"),
    ("tran", "Railway", "rail"),
    ("tran", "Track", "track"),
    ("tran", "Square", "square"),
    ("brid", "Bridge", "brid"),
    ("brid", "BridgePart", "brid_part"),
    ("tun", "Tunnel", "tun"),
    ("tun", "TunnelPart", "tun_part"),
    ("wtr", "WaterBody", "wtr"),
    ("luse", "LandUse", "luse"),
    ("veg", "SolitaryVegetationObject", "veg_sol"),
    ("veg", "PlantCover", "veg_plant"),
    ("frn", "CityFurniture", "frn"),
    ("dem", "ReliefFeature", "dem"),
    ("gen", "GenericCityObject", "gen"),
];

/// Prefix to namespace, in declaration order: the CityGML 2.0 table, then any
/// ADE entries.
#[derive(Debug, Clone)]
pub struct Prefixes {
    entries: Vec<(String, String)>,
    core_len: usize,
}

impl Prefixes {
    /// The CityGML 2.0 prefixes the writer declares on its root, plus `xAL`.
    pub fn citygml2() -> Self {
        let mut entries: Vec<(String, String)> = CITYGML_2_NAMESPACES
            .iter()
            .map(|(attribute, uri)| {
                let prefix = attribute.strip_prefix("xmlns:").unwrap_or(attribute);
                (prefix.to_string(), uri.to_string())
            })
            .collect();
        entries.push(("xAL".to_string(), XAL_NS.to_string()));
        let core_len = entries.len();
        Self { entries, core_len }
    }

    /// `citygml2()` plus `(prefix, namespace)` pairs the caller has already
    /// checked against the built-in prefixes.
    pub fn with_ade(entries: &[(String, String)]) -> Self {
        let mut prefixes = Self::citygml2();
        prefixes.entries.extend(entries.iter().cloned());
        prefixes
    }

    pub fn namespace(&self, prefix: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(p, _)| p == prefix)
            .map(|(_, ns)| ns.as_str())
    }

    pub fn prefix_of(&self, namespace: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(_, ns)| ns == namespace)
            .map(|(p, _)| p.as_str())
    }

    /// Only the ADE entries, in the order given.
    pub fn ade(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries[self.core_len..]
            .iter()
            .map(|(p, ns)| (p.as_str(), ns.as_str()))
    }

    /// The namespaces of the CityGML 2.0 city-object modules.
    fn class_namespaces(&self) -> impl Iterator<Item = &str> {
        self.entries[..self.core_len]
            .iter()
            .filter(|(p, _)| !NON_CLASS_PREFIXES.contains(&p.as_str()))
            .map(|(_, ns)| ns.as_str())
    }
}

/// A city-object element the writer can emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CityClass {
    pub element: QName,
    /// The prefix the element is written with.
    pub prefix: String,
    /// The prefix of the gml:id minted for objects of this class.
    pub id_prefix: String,
}

impl CityClass {
    /// `gen:GenericCityObject`, where an unresolved value ends up.
    pub fn generic() -> Self {
        Self {
            element: QName::new(GENERICS_NS, "GenericCityObject"),
            prefix: "gen".to_string(),
            id_prefix: "gen".to_string(),
        }
    }

    /// The qualified name as written, `prefix:local`.
    pub fn name(&self) -> String {
        format!("{}:{}", self.prefix, self.element.local)
    }

    pub fn is_generic(&self) -> bool {
        self.element == QName::new(GENERICS_NS, "GenericCityObject")
    }
}

/// Resolves the value read through `attributeKeys.featureType`.
///
/// `prefix:Local` is exact and case-sensitive. A bare `Local` matches the
/// non-abstract CityGML 2.0 city objects by local name, ignoring case, and
/// must match exactly one. Anything else is `None`.
pub fn resolve(value: &str, prefixes: &Prefixes, schema: &SchemaSet) -> Option<CityClass> {
    let city_object = QName::new(CITYGML_CORE_NS, "_CityObject");
    let is_city_object = |element: &QName| {
        !schema.is_abstract(element) && schema.substitutes_for(element, &city_object)
    };

    let (element, prefix) = match value.split_once(':') {
        Some((prefix, local)) => {
            let namespace = prefixes.namespace(prefix)?;
            let element = QName::new(namespace, local);
            if !schema.global_elements().any(|known| *known == element) {
                return None;
            }
            (element, prefix.to_string())
        }
        None => {
            let namespaces: Vec<&str> = prefixes.class_namespaces().collect();
            let mut matches = schema.global_elements().filter(|element| {
                namespaces.contains(&element.namespace.as_str())
                    && element.local.eq_ignore_ascii_case(value)
                    && is_city_object(element)
            });
            let element = matches.next()?;
            if matches.next().is_some() {
                return None;
            }
            let prefix = prefixes.prefix_of(&element.namespace)?;
            (element.clone(), prefix.to_string())
        }
    };
    if !is_city_object(&element) {
        return None;
    }
    let id_prefix = id_prefix_for(&element);
    Some(CityClass {
        element,
        prefix,
        id_prefix,
    })
}

/// The 17 original classes keep their table entry; any other class uses its
/// local name in lower case.
fn id_prefix_for(element: &QName) -> String {
    CORE_ID_PREFIXES
        .iter()
        .find(|(module, local, _)| {
            *local == element.local && core_namespace(module) == Some(element.namespace.as_str())
        })
        .map(|(_, _, id)| id.to_string())
        .unwrap_or_else(|| element.local.to_lowercase())
}

fn core_namespace(prefix: &str) -> Option<&'static str> {
    CITYGML_2_NAMESPACES
        .iter()
        .find(|(attribute, _)| attribute.strip_prefix("xmlns:") == Some(prefix))
        .map(|(_, uri)| *uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core() -> &'static SchemaSet {
        SchemaSet::core().unwrap()
    }

    fn iur() -> (SchemaSet, Prefixes) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../testing/data/fixtures/plateau-citymodel/15202_nagaoka-shi_city_2024_citygml_1_op/schemas/iur/uro/3.1/urbanObject.xsd");
        let set = SchemaSet::with_ade(&[path]).unwrap();
        let prefixes =
            Prefixes::with_ade(&[("uro".into(), "https://www.geospatial.jp/iur/uro/3.1".into())]);
        (set, prefixes)
    }

    fn names(value: &str, prefixes: &Prefixes, schema: &SchemaSet) -> Option<String> {
        resolve(value, prefixes, schema).map(|c| c.name())
    }

    #[test]
    fn core_classes_resolve_as_before() {
        let p = Prefixes::citygml2();
        for (value, expected) in [
            ("bldg:Building", "bldg:Building"),
            ("Building", "bldg:Building"),
            ("building", "bldg:Building"),
            ("bldg:BuildingPart", "bldg:BuildingPart"),
            ("tran:Road", "tran:Road"),
            ("gen:GenericCityObject", "gen:GenericCityObject"),
            ("bldg:WallSurface", "bldg:WallSurface"),
        ] {
            assert_eq!(
                names(value, &p, core()).as_deref(),
                Some(expected),
                "{value}"
            );
        }
        assert_eq!(
            resolve("bldg:Building", &p, core()).unwrap().id_prefix,
            "bldg"
        );
        assert_eq!(
            resolve("veg:PlantCover", &p, core()).unwrap().id_prefix,
            "veg_plant"
        );
    }

    #[test]
    fn substrings_case_and_abstract_elements_never_resolve() {
        let p = Prefixes::citygml2();
        for value in [
            "my_building_export_v2",
            "bldg:building",
            "bldg:_AbstractBuilding",
            "core:Address",
            "nope:Building",
            "",
        ] {
            assert_eq!(names(value, &p, core()), None, "{value}");
        }
    }

    #[test]
    fn an_unprefixed_name_in_several_modules_is_unresolved() {
        let p = Prefixes::citygml2();
        assert_eq!(names("Door", &p, core()), None);
        assert_eq!(names("WallSurface", &p, core()), None);
    }

    #[test]
    fn an_ade_class_is_unresolved_without_its_schema_and_itself_with_it() {
        assert_eq!(
            names("uro:UndergroundBuilding", &Prefixes::citygml2(), core()),
            None
        );
        let (set, prefixes) = iur();
        let class = resolve("uro:UndergroundBuilding", &prefixes, &set).unwrap();
        assert_eq!(class.name(), "uro:UndergroundBuilding");
        assert_eq!(class.id_prefix, "undergroundbuilding");
    }

    #[test]
    fn all_seventeen_core_id_prefixes_are_pinned_by_expanded_name() {
        let p = Prefixes::citygml2();
        for (module, local, id) in CORE_ID_PREFIXES {
            let class = resolve(&format!("{module}:{local}"), &p, core()).unwrap();
            assert_eq!(class.id_prefix, *id, "{module}:{local}");
        }
        assert_eq!(CORE_ID_PREFIXES.len(), 17);
    }

    #[test]
    fn prefixes_expose_only_ade_entries_and_look_up_both_ways() {
        let (_, p) = iur();
        assert_eq!(
            p.ade().collect::<Vec<_>>(),
            [("uro", "https://www.geospatial.jp/iur/uro/3.1")]
        );
        assert_eq!(Prefixes::citygml2().ade().count(), 0);
        assert_eq!(
            p.namespace("bldg"),
            Some("http://www.opengis.net/citygml/building/2.0")
        );
        assert_eq!(
            p.prefix_of("http://www.opengis.net/citygml/building/2.0"),
            Some("bldg")
        );
        assert!(CityClass::generic().is_generic());
        assert_eq!(CityClass::generic().name(), "gen:GenericCityObject");
    }
}
