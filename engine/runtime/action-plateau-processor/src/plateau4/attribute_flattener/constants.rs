use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) struct AttributePath {
    pub(super) attribute: String,
    pub(super) data_type: String,
    pub(super) json_path: String,
}

pub(super) static FLATTEN_ATTRIBUTES: Lazy<HashMap<String, Vec<AttributePath>>> = Lazy::new(|| {
    let value = json!(
        {
            "bldg/bldg:Building": [
                {
                    "attribute": "gml:name",
                    "data_type": "string",
                    "json_path": "gml:name"
                },
                {
                    "attribute": "core:creationDate",
                    "data_type": "string",
                    "json_path": "core:creationDate"
                },
                {
                    "attribute": "bldg:class",
                    "data_type": "string",
                    "json_path": "bldg:class"
                },
                {
                    "attribute": "bldg:usage",
                    "data_type": "string",
                    "json_path": "bldg:usage"
                },
                {
                    "attribute": "bldg:yearOfConstruction",
                    "data_type": "int",
                    "json_path": "bldg:yearOfConstruction"
                },
                {
                    "attribute": "bldg:measuredHeight",
                    "data_type": "int",
                    "json_path": "bldg:measuredHeight"
                },
                {
                    "attribute": "bldg:storeysAboveGround",
                    "data_type": "int",
                    "json_path": "bldg:storeysAboveGround"
                },
                {
                    "attribute": "bldg:storeysBelowGround",
                    "data_type": "int",
                    "json_path": "bldg:storeysBelowGround"
                },
                {
                    "attribute": "bldg:address",
                    "data_type": "string",
                    "json_path": "bldg:address"
                },
                {
                    "attribute": "uro:buildingID",
                    "data_type": "string",
                    "json_path": "uro:BuildingIDAttribute uro:buildingID"
                },
                {
                    "attribute": "uro:branchID",
                    "data_type": "int",
                    "json_path": "uro:BuildingIDAttribute uro:branchID"
                },
                {
                    "attribute": "uro:partID",
                    "data_type": "int",
                    "json_path": "uro:BuildingIDAttribute uro:partID"
                },
                {
                    "attribute": "uro:prefecture",
                    "data_type": "string",
                    "json_path": "uro:BuildingIDAttribute uro:prefecture"
                },
                {
                    "attribute": "uro:prefecture_code",
                    "data_type": "string",
                    "json_path": "uro:BuildingIDAttribute uro:prefecture_code"
                },
                {
                    "attribute": "uro:city",
                    "data_type": "string",
                    "json_path": "uro:BuildingIDAttribute uro:city"
                },
                {
                    "attribute": "uro:city_code",
                    "data_type": "string",
                    "json_path": "uro:BuildingIDAttribute uro:city_code"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:serialNumberOfBuildingCertification",
                    "data_type": "string",
                    "json_path": "uro:BuildingDetailAttribute uro:serialNumberOfBuildingCertification"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:siteArea",
                    "data_type": "int",
                    "json_path": "uro:BuildingDetailAttribute uro:siteArea"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:totalFloorArea",
                    "data_type": "int",
                    "json_path": "uro:BuildingDetailAttribute uro:totalFloorArea"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:buildingFootprintArea",
                    "data_type": "int",
                    "json_path": "uro:BuildingDetailAttribute uro:buildingFootprintArea"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:buildingRoofEdgeArea",
                    "data_type": "int",
                    "json_path": "uro:BuildingDetailAttribute uro:buildingRoofEdgeArea"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:buildingStructureType",
                    "data_type": "string",
                    "json_path": "uro:BuildingDetailAttribute uro:buildingStructureType"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:buildingStructureType_code",
                    "data_type": "string",
                    "json_path": "uro:BuildingDetailAttribute uro:buildingStructureType_code"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:fireproofStructureType",
                    "data_type": "string",
                    "json_path": "uro:BuildingDetailAttribute uro:fireproofStructureType"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:urbanPlanType",
                    "data_type": "string",
                    "json_path": "uro:BuildingDetailAttribute uro:urbanPlanType"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:areaClassificationType",
                    "data_type": "string",
                    "json_path": "uro:BuildingDetailAttribute uro:areaClassificationType"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:districtsAndZonesType",
                    "data_type": "string",
                    "json_path": "uro:BuildingDetailAttribute uro:districtsAndZonesType"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:landUseType",
                    "data_type": "string",
                    "json_path": "uro:BuildingDetailAttribute uro:landUseType"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:vacancy",
                    "data_type": "string",
                    "json_path": "uro:BuildingDetailAttribute uro:vacancy"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:buildingCoverageRate",
                    "data_type": "int",
                    "json_path": "uro:BuildingDetailAttribute uro:buildingCoverageRate"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:floorAreaRate",
                    "data_type": "int",
                    "json_path": "uro:BuildingDetailAttribute uro:floorAreaRate"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:specifiedBuildingCoverageRate",
                    "data_type": "int",
                    "json_path": "uro:BuildingDetailAttribute uro:specifiedBuildingCoverageRate"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:specifiedFloorAreaRate",
                    "data_type": "int",
                    "json_path": "uro:BuildingDetailAttribute uro:specifiedFloorAreaRate"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:standardFloorAreaRate",
                    "data_type": "int",
                    "json_path": "uro:BuildingDetailAttribute uro:standardFloorAreaRate"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:buildingHeight",
                    "data_type": "int",
                    "json_path": "uro:BuildingDetailAttribute uro:buildingHeight"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:eaveHeight",
                    "data_type": "int",
                    "json_path": "uro:BuildingDetailAttribute uro:eaveHeight"
                },
                {
                    "attribute": "uro:BuildingDetailAttribute_uro:surveyYear",
                    "data_type": "string",
                    "json_path": "uro:BuildingDetailAttribute uro:surveyYear"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:class",
                    "data_type": "string",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:class"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:name",
                    "data_type": "string",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:name"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:capacity",
                    "data_type": "int",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:capacity"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:totalFloorArea",
                    "data_type": "int",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:totalFloorArea"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:inauguralDate",
                    "data_type": "string",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:inauguralDate"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:yearOpened",
                    "data_type": "int",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:yearOpened"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:yearClosed",
                    "data_type": "int",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:yearClosed"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:urbanPlanType",
                    "data_type": "string",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:urbanPlanType"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:areaClassificationType",
                    "data_type": "string",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:areaClassificationType"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:districtsAndZonesType",
                    "data_type": "string",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:districtsAndZonesType"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:landUseType",
                    "data_type": "string",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:landUseType"
                },
                {
                    "attribute": "uro:LargeCustomerFacilityAttribute_uro:surveyYear",
                    "data_type": "int",
                    "json_path": "uro:LargeCustomerFacilityAttribute uro:surveyYear"
                },
                {
                    "attribute": "uro:RealEstateIDAttribute_uro:realEstateIDOfBuilding",
                    "data_type": "string",
                    "json_path": "uro:RealEstateIDAttribute uro:realEstateIDOfBuilding"
                },
                {
                    "attribute": "uro:RealEstateIDAttribute_uro:matchingScore",
                    "data_type": "int",
                    "json_path": "uro:RealEstateIDAttribute uro:matchingScore"
                },
                {
                    "attribute": "uro:geometrySrcDescLod0",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:geometrySrcDescLod0"
                },
                {
                    "attribute": "uro:geometrySrcDescLod0_code",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:geometrySrcDescLod0_code"
                },
                {
                    "attribute": "uro:geometrySrcDescLod1",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:geometrySrcDescLod1"
                },
                {
                    "attribute": "uro:lodType",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:lodType"
                },
                {
                    "attribute": "uro:lod1HeightType",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:lod1HeightType"
                },
                {
                    "attribute": "uro:lod1HeightType_code",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:lod1HeightType_code"
                }
            ]
        }
    );
    serde_json::from_value(value).unwrap()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_attributes() {
        let flatten_attributes = FLATTEN_ATTRIBUTES.clone();
        assert_eq!(flatten_attributes.len(), 1);
        assert!(flatten_attributes.contains_key("bldg/bldg:Building"));
    }
}
