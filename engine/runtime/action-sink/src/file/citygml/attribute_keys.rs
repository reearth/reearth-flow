//! Where the CityGML Writer finds its CityGML inputs: the attribute names it
//! reads, each defaulting to the key the CityGML readers write.

use reearth_flow_citygml::pipeline::{MEMBER_GML_PROPERTY_NAME_KEY, MEMBER_LOD_KEY};
use reearth_flow_types::{Attribute, CITYGML_FEATURE_TYPE_KEY, CITYGML_GML_ID_KEY};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # Attribute Keys
/// Names of the attributes the writer reads its CityGML inputs from. Any key left out uses the name the CityGML readers write.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeKeys {
    /// # Feature Type
    /// Feature attribute holding the CityGML class, such as `bldg:Building`. Matched case-insensitively by whether the value contains the class name, so `bldg:Building` and `Building` both work. Recognised classes: Building, BuildingPart, Road, Railway, Track, Square, Bridge, BridgePart, Tunnel, TunnelPart, WaterBody, LandUse, SolitaryVegetationObject, PlantCover, CityFurniture, ReliefFeature and GenericCityObject; anything else is written as `gen:GenericCityObject`. Defaults to `__citygml_feature_type`.
    #[serde(default)]
    pub feature_type: Option<Attribute>,
    /// # gml:id
    /// Feature attribute holding the `gml:id` to write. A value that is not a valid XML name is adjusted to one, and a missing, non-text or already-used value gets a generated id. Defaults to `__citygml_gml_id`.
    #[serde(default)]
    pub gml_id: Option<Attribute>,
    /// # LOD
    /// Attribute holding the level of detail, a whole number from 0 to 4 given as a number or as text. Read from each geometry member, and from the feature as well when this key is set. Defaults to `lod`.
    #[serde(default)]
    pub lod: Option<Attribute>,
    /// # Geometry Property Name
    /// Attribute holding the geometry property's own name, such as `lod0RoofEdge`. Must be a valid element name; a value that is not one is an error. Read from each geometry member, and from the feature as well when this key is set. Defaults to `gmlPropertyName`.
    #[serde(default)]
    pub gml_property_name: Option<Attribute>,
}

/// [`AttributeKeys`] with every default filled in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedKeys {
    pub feature_type: String,
    pub gml_id: String,
    pub lod: String,
    /// Whether the user named the LOD key, which is what lets the feature
    /// itself supply a LOD. A stray feature attribute under the default name
    /// must not change what an existing workflow writes.
    pub lod_on_feature: bool,
    pub gml_property_name: String,
    /// As [`Self::lod_on_feature`], for the property name.
    pub gml_property_name_on_feature: bool,
}

impl Default for ResolvedKeys {
    fn default() -> Self {
        AttributeKeys::default()
            .resolve()
            .expect("the default keys are never blank")
    }
}

impl AttributeKeys {
    /// Fill in defaults, rejecting a key that is empty or only whitespace.
    pub fn resolve(&self) -> Result<ResolvedKeys, String> {
        fn pick(set: &Option<Attribute>, field: &str, default: &str) -> Result<String, String> {
            match set {
                Some(key) if key.as_str().trim().is_empty() => Err(format!(
                    "`attributeKeys.{field}` is blank; give an attribute name or leave it out to use `{default}`"
                )),
                Some(key) => Ok(key.as_str().to_owned()),
                None => Ok(default.to_owned()),
            }
        }
        Ok(ResolvedKeys {
            feature_type: pick(&self.feature_type, "featureType", CITYGML_FEATURE_TYPE_KEY)?,
            gml_id: pick(&self.gml_id, "gmlId", CITYGML_GML_ID_KEY)?,
            lod: pick(&self.lod, "lod", MEMBER_LOD_KEY)?,
            lod_on_feature: self.lod.is_some(),
            gml_property_name: pick(
                &self.gml_property_name,
                "gmlPropertyName",
                MEMBER_GML_PROPERTY_NAME_KEY,
            )?,
            gml_property_name_on_feature: self.gml_property_name.is_some(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(json: serde_json::Value) -> AttributeKeys {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn omitted_keys_resolve_to_todays_names_and_never_read_the_feature() {
        let resolved = AttributeKeys::default().resolve().unwrap();

        assert_eq!(resolved.feature_type, "__citygml_feature_type");
        assert_eq!(resolved.gml_id, "__citygml_gml_id");
        assert_eq!(resolved.lod, "lod");
        assert_eq!(resolved.gml_property_name, "gmlPropertyName");
        assert!(!resolved.lod_on_feature);
        assert!(!resolved.gml_property_name_on_feature);
    }

    #[test]
    fn resolved_default_matches_resolving_the_default_parameter() {
        let resolved = ResolvedKeys::default();

        assert_eq!(resolved.lod, "lod");
        assert!(!resolved.lod_on_feature);
    }

    #[test]
    fn a_set_key_replaces_the_default_and_turns_on_the_feature_fallback() {
        let resolved = keys(serde_json::json!({ "lod": "level", "gmlPropertyName": "prop" }))
            .resolve()
            .unwrap();

        assert_eq!(resolved.lod, "level");
        assert!(resolved.lod_on_feature);
        assert_eq!(resolved.gml_property_name, "prop");
        assert!(resolved.gml_property_name_on_feature);
    }

    #[test]
    fn setting_a_key_to_its_default_name_still_counts_as_set() {
        let resolved = keys(serde_json::json!({ "lod": "lod" })).resolve().unwrap();

        assert!(resolved.lod_on_feature);
    }

    #[test]
    fn feature_type_and_gml_id_keys_are_taken_as_given() {
        let resolved = keys(serde_json::json!({
            "featureType": "kind",
            "gmlId": "id",
        }))
        .resolve()
        .unwrap();

        assert_eq!(resolved.feature_type, "kind");
        assert_eq!(resolved.gml_id, "id");
    }

    #[test]
    fn a_blank_key_is_rejected_naming_its_field() {
        for (field, json) in [
            ("featureType", serde_json::json!({ "featureType": "" })),
            ("gmlId", serde_json::json!({ "gmlId": "  " })),
            ("lod", serde_json::json!({ "lod": "" })),
            (
                "gmlPropertyName",
                serde_json::json!({ "gmlPropertyName": " " }),
            ),
        ] {
            let message = keys(json).resolve().unwrap_err();
            assert!(
                message.contains(&format!("attributeKeys.{field}")),
                "{message}"
            );
        }
    }
}
