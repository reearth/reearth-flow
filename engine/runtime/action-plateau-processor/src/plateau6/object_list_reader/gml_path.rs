//! Attribute names in the objectlist follow the CityGML 3.0 conceptual model
//! and write every i-UR element as `uro:`; the GML uses other names for some of
//! them.

/// i-UR types declared in the `urc` namespace (urbanCore.xsd).
const URC_TYPES: &[&str] = &[
    "DataQualityAttribute",
    "DisasterRiskAttribute",
    "DmAnnotation",
    "DmAttribute",
    "DmElement",
    "DmGeometricAttribute",
    "ExteriorDataQualityAttribute",
    "FloodingRiskAttribute",
    "HighTideRiskAttribute",
    "InlandFloodingRiskAttribute",
    "InteriorDataQualityAttribute",
    "KeyValuePairAttribute",
    "LandSlideRiskAttribute",
    "PublicSurveyDataQualityAttribute",
    "ReservoirFloodingRiskAttribute",
    "RiverFloodingRiskAttribute",
    "TsunamiRiskAttribute",
];

/// `/`-separated element path of an attribute, from its C-G cells (属性名／関連役割名).
/// `None` when the attribute is not an element of the GML.
///
/// - A `property.Type` cell becomes `property/Type`, and parentheses are dropped.
/// - `core:name` / `core:description` / `core:identifier` are written as their
///   `gml:` elements.
/// - `core:featureID` is the `gml:id` attribute.
/// - A `uro:` type declared in `urc`, and everything below it, becomes `urc:`.
pub(super) fn gml_path(attribute_names: &[String]) -> Option<String> {
    let path = attribute_names
        .join("/")
        .replace(['(', ')'], "")
        .replace('.', "/");

    let mut in_urc = false;
    let mut segments = Vec::new();
    for segment in path.split('/') {
        let segment = match segment {
            "core:featureID" => return None,
            "core:name" => "gml:name",
            "core:description" => "gml:description",
            "core:identifier" => "gml:identifier",
            other => other,
        };
        match segment.strip_prefix("uro:") {
            Some(local) if in_urc || URC_TYPES.contains(&local) => {
                in_urc = true;
                segments.push(format!("urc:{local}"));
            }
            _ => segments.push(segment.to_string()),
        }
    }
    Some(segments.join("/"))
}
