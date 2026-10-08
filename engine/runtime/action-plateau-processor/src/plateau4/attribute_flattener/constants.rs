use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) struct AttributePath {
    pub(super) attribute: String,
    pub(super) data_type: String,
    pub(super) json_path: String,
}

pub(super) static FLATTEN_ATTRIBUTES: Lazy<HashMap<String, Vec<AttributePath>>> = Lazy::new(|| {
    let data = include_str!("flatten_attributes.json");
    serde_json::from_str(data).unwrap()
});

/// Attributes kept on the `flatSchema` port for feature types with a reduced flat attribute set.
pub(super) struct FlatSchemaAttributes {
    pub(super) attributes: &'static [&'static str],
    /// Risk package and the attribute name suffixes kept for it (empty keeps all).
    pub(super) risk_attributes: &'static [(&'static str, &'static [&'static str])],
}

const FLOOD_RANK_SUFFIXES: &[&str] = &["_浸水ランク", "_浸水ランクコード"];

pub(super) static FLAT_SCHEMA_ATTRIBUTES: Lazy<HashMap<&'static str, FlatSchemaAttributes>> =
    Lazy::new(|| {
        HashMap::from([(
            "bldg/bldg:Building",
            FlatSchemaAttributes {
                attributes: &[
                    "meshcode",
                    "feature_type",
                    "city_code",
                    "city_name",
                    "gml_id",
                    "_lod",
                    "_x",
                    "_y",
                    "gml:name",
                    "bldg:class",
                    "bldg:usage",
                    "bldg:yearOfConstruction",
                    "bldg:measuredHeight",
                    "bldg:storeysAboveGround",
                    "bldg:storeysBelowGround",
                    "bldg:address",
                    "uro:BuildingDetailAttribute_uro:buildingStructureType",
                    "uro:BuildingDetailAttribute_uro:fireproofStructureType",
                    "uro:BuildingDetailAttribute_uro:urbanPlanType",
                    "uro:BuildingDetailAttribute_uro:areaClassificationType",
                    "uro:BuildingDetailAttribute_uro:districtsAndZonesType",
                    "uro:BuildingDetailAttribute_uro:landUseType",
                    "uro:LargeCustomerFacilityAttribute_uro:name",
                    "uro:lod1HeightType",
                ],
                risk_attributes: &[
                    ("fld", FLOOD_RANK_SUFFIXES),
                    ("tnm", FLOOD_RANK_SUFFIXES),
                    ("htd", FLOOD_RANK_SUFFIXES),
                    ("ifld", FLOOD_RANK_SUFFIXES),
                    ("lsld", &[]),
                ],
            },
        )])
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_attributes() {
        let flatten_attributes = FLATTEN_ATTRIBUTES.clone();
        assert!(flatten_attributes.contains_key("bldg/bldg:Building"));
        assert!(flatten_attributes.contains_key("rwy/tran:AuxiliaryTrafficArea"));
        assert!(flatten_attributes.contains_key("rwy/tran:Railway"));
        assert!(flatten_attributes.contains_key("rwy/tran:TrafficArea"));
        assert!(flatten_attributes.contains_key("squr/tran:AuxiliaryTrafficArea"));
        assert!(flatten_attributes.contains_key("squr/tran:Square"));
        assert!(flatten_attributes.contains_key("squr/tran:TrafficArea"));
        assert!(flatten_attributes.contains_key("tran/tran:AuxiliaryTrafficArea"));
        assert!(flatten_attributes.contains_key("tran/tran:Road"));
        assert!(flatten_attributes.contains_key("tran/tran:TrafficArea"));
        assert!(flatten_attributes.contains_key("trk/tran:AuxiliaryTrafficArea"));
        assert!(flatten_attributes.contains_key("trk/tran:Track"));
        assert!(flatten_attributes.contains_key("trk/tran:TrafficArea"));
        assert!(flatten_attributes.contains_key("wwy/tran:TrafficArea"));
        assert!(flatten_attributes.contains_key("wwy/uro:Waterway"));
    }
}
