//! Where the CityGML Writer finds its CityGML inputs: the attribute names it
//! reads, each defaulting to the key the CityGML readers write.

use reearth_flow_citygml::pipeline::{
    MEMBER_GEOMETRY_FEATURE_TYPE_KEY, MEMBER_GEOMETRY_GML_ID_KEY, MEMBER_GML_PROPERTY_NAME_KEY,
    MEMBER_LOD_KEY,
};
use reearth_flow_types::{
    Attribute, CITYGML_FEATURE_TYPE_KEY, CITYGML_GML_ID_KEY, CITYGML_PARENT_OBJECT_GML_ID_KEY,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # Attribute Keys
/// Names of the attributes the writer reads its CityGML inputs from. Any key left out uses the name the CityGML readers write.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeKeys {
    /// # Feature Type
    /// Feature attribute holding the class to write, as a qualified name such as `bldg:Building` or `uro:UndergroundBuilding`, or a CityGML 2.0 class name alone, such as `Building`, matched ignoring case. An ADE class needs its ADE listed in `adeSchemas`. A value that names no class is written as `gen:GenericCityObject` and reported. Defaults to `__citygml_feature_type`.
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
    /// # CityGML Attributes
    /// Feature attribute holding all the CityGML properties as one map, for features read with the CityGML reader's `cityGmlAttributesKey` set to this name. Leave it out when the properties are the feature's own attributes, which is the reader's default.
    #[serde(default)]
    pub city_gml_attributes: Option<Attribute>,
    /// # Owner gml:id
    /// Geometry member attribute holding the gml:id of the object that owns that piece of geometry. The geometry is written on the nested object in the feature's attributes with that gml:id, or on the feature itself when the value is the feature's own gml:id. Geometry whose owner matches no object is written on the feature and reported; read CityGML with the reader's `recordHierarchy` on so every nested object can be matched. Defaults to `__citygml_geometry_gml_id`.
    #[serde(default)]
    pub owner_gml_id: Option<Attribute>,
    /// # Owner Feature Type
    /// Geometry member attribute holding the owner's class as a qualified name, such as `bldg:WallSurface`. Geometry with no owner gml:id whose class is not the feature's own is a nested object's that could not be matched: it is written on the feature and reported. Defaults to `__citygml_geometry_feature_type`.
    #[serde(default)]
    pub owner_feature_type: Option<Attribute>,
    /// # Parent gml:id
    /// Feature attribute holding the gml:id, or owner key, of the object the feature nests under: another feature, or a nested object inside one. A feature whose parent is not found or cannot hold it is written as a top-level object and reported. Defaults to `__citygml_parent_object_gml_id`.
    #[serde(default)]
    pub parent_gml_id: Option<Attribute>,
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
    /// The map attribute properties are read from, or `None` for the
    /// feature's own attributes.
    pub city_gml_attributes: Option<String>,
    /// The member attribute naming the object that owns the member's geometry.
    pub owner_gml_id: String,
    /// The member attribute holding the owner's class.
    pub owner_feature_type: String,
    /// The feature attribute holding the object the feature nests under.
    pub parent_gml_id: String,
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
            city_gml_attributes: match &self.city_gml_attributes {
                Some(key) if key.as_str().trim().is_empty() => {
                    return Err("`attributeKeys.cityGmlAttributes` is blank; give an attribute name or leave it out to read the feature's own attributes".to_owned())
                }
                Some(key) => Some(key.as_str().to_owned()),
                None => None,
            },
            owner_gml_id: pick(
                &self.owner_gml_id,
                "ownerGmlId",
                MEMBER_GEOMETRY_GML_ID_KEY,
            )?,
            owner_feature_type: pick(
                &self.owner_feature_type,
                "ownerFeatureType",
                MEMBER_GEOMETRY_FEATURE_TYPE_KEY,
            )?,
            parent_gml_id: pick(
                &self.parent_gml_id,
                "parentGmlId",
                CITYGML_PARENT_OBJECT_GML_ID_KEY,
            )?,
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
        assert!(resolved.city_gml_attributes.is_none());
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
            ("ownerGmlId", serde_json::json!({ "ownerGmlId": "" })),
            (
                "ownerFeatureType",
                serde_json::json!({ "ownerFeatureType": " " }),
            ),
            ("parentGmlId", serde_json::json!({ "parentGmlId": "  " })),
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

    #[test]
    fn the_hierarchy_keys_default_to_the_readers_names() {
        let resolved = AttributeKeys::default().resolve().unwrap();

        assert_eq!(resolved.owner_gml_id, "__citygml_geometry_gml_id");
        assert_eq!(
            resolved.owner_feature_type,
            "__citygml_geometry_feature_type"
        );
        assert_eq!(resolved.parent_gml_id, "__citygml_parent_object_gml_id");
    }

    #[test]
    fn the_hierarchy_keys_are_taken_as_given() {
        let resolved = keys(serde_json::json!({
            "ownerGmlId": "owner",
            "ownerFeatureType": "owner_type",
            "parentGmlId": "parent",
        }))
        .resolve()
        .unwrap();

        assert_eq!(resolved.owner_gml_id, "owner");
        assert_eq!(resolved.owner_feature_type, "owner_type");
        assert_eq!(resolved.parent_gml_id, "parent");
    }

    #[test]
    fn an_omitted_city_gml_attributes_key_reads_the_feature_itself() {
        let resolved = keys(serde_json::json!({})).resolve().unwrap();

        assert_eq!(resolved.city_gml_attributes, None);
    }

    #[test]
    fn a_set_city_gml_attributes_key_names_the_map_to_read() {
        let resolved = keys(serde_json::json!({ "cityGmlAttributes": "cga" }))
            .resolve()
            .unwrap();

        assert_eq!(resolved.city_gml_attributes.as_deref(), Some("cga"));
    }

    #[test]
    fn a_blank_city_gml_attributes_key_is_rejected_naming_its_field() {
        let message = keys(serde_json::json!({ "cityGmlAttributes": " " }))
            .resolve()
            .unwrap_err();

        assert!(
            message.contains("attributeKeys.cityGmlAttributes"),
            "{message}"
        );
    }
}
