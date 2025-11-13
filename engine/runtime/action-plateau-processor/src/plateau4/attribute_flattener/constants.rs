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
                    "attribute": "uro:BuildingIDAttribute_uro:buildingID",
                    "data_type": "string",
                    "json_path": "uro:BuildingIDAttribute uro:buildingID"
                },
                {
                    "attribute": "uro:BuildingIDAttribute_uro:branchID",
                    "data_type": "int",
                    "json_path": "uro:BuildingIDAttribute uro:branchID"
                },
                {
                    "attribute": "uro:BuildingIDAttribute_uro:partID",
                    "data_type": "int",
                    "json_path": "uro:BuildingIDAttribute uro:partID"
                },
                {
                    "attribute": "uro:BuildingIDAttribute_uro:prefecture",
                    "data_type": "string",
                    "json_path": "uro:BuildingIDAttribute uro:prefecture"
                },
                {
                    "attribute": "uro:BuildingIDAttribute_uro:prefecture_code",
                    "data_type": "string",
                    "json_path": "uro:BuildingIDAttribute uro:prefecture_code"
                },
                {
                    "attribute": "uro:BuildingIDAttribute_uro:city",
                    "data_type": "string",
                    "json_path": "uro:BuildingIDAttribute uro:city"
                },
                {
                    "attribute": "uro:BuildingIDAttribute_uro:city_code",
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
                    "attribute": "uro:buildingDataQualityAttribute_uro:geometrySrcDescLod0",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:geometrySrcDescLod0"
                },
                {
                    "attribute": "uro:buildingDataQualityAttribute_uro:geometrySrcDescLod0_code",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:geometrySrcDescLod0_code"
                },
                {
                    "attribute": "uro:buildingDataQualityAttribute_uro:geometrySrcDescLod1",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:geometrySrcDescLod1"
                },
                {
                    "attribute": "uro:buildingDataQualityAttribute_uro:lodType",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:lodType"
                },
                {
                    "attribute": "uro:buildingDataQualityAttribute_uro:lod1HeightType",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:lod1HeightType"
                },
                {
                    "attribute": "uro:buildingDataQualityAttribute_uro:lod1HeightType_code",
                    "data_type": "string",
                    "json_path": "uro:buildingDataQualityAttribute uro:lod1HeightType_code"
                }
            ],
            "squr/tran:Square": [
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
                    "attribute": "uro:TerminalAttribute_uro:prefecture",
                    "data_type": "string",
                    "json_path": "uro:TerminalAttribute uro:prefecture"
                },
                {
                    "attribute": "uro:TerminalAttribute_uro:city",
                    "data_type": "string",
                    "json_path": "uro:TerminalAttribute uro:city"
                },
                {
                    "attribute": "uro:TerminalAttribute_uro:urbanPlanningAreaName",
                    "data_type": "string",
                    "json_path": "uro:TerminalAttribute uro:urbanPlanningAreaName"
                },
                {
                    "attribute": "uro:TerminalAttribute_uro:areaPlanned",
                    "data_type": "string",
                    "json_path": "uro:TerminalAttribute uro:areaPlanned"
                },
                {
                    "attribute": "uro:TerminalAttribute_uro:status",
                    "data_type": "string",
                    "json_path": "uro:TerminalAttribute uro:status"
                },
                {
                    "attribute": "uro:TerminalAttribute_uro:terminalType",
                    "data_type": "string",
                    "json_path": "uro:TerminalAttribute uro:terminalType"
                },
                {
                    "attribute": "uro:TerminalAttribute_uro:dateOfRevision",
                    "data_type": "string",
                    "json_path": "uro:TerminalAttribute uro:dateOfRevision"
                },
                {
                    "attribute": "uro:StationSquareAttribute_uro:prefecture",
                    "data_type": "string",
                    "json_path": "uro:StationSquareAttribute uro:prefecture"
                },
                {
                    "attribute": "uro:StationSquareAttribute_uro:city",
                    "data_type": "string",
                    "json_path": "uro:StationSquareAttribute uro:city"
                },
                {
                    "attribute": "uro:StationSquareAttribute_uro:urbanPlanningAreaName",
                    "data_type": "string",
                    "json_path": "uro:StationSquareAttribute uro:urbanPlanningAreaName"
                },
                {
                    "attribute": "uro:StationSquareAttribute_uro:areaPlanned",
                    "data_type": "string",
                    "json_path": "uro:StationSquareAttribute uro:areaPlanned"
                },
                {
                    "attribute": "uro:StationSquareAttribute_uro:status",
                    "data_type": "string",
                    "json_path": "uro:StationSquareAttribute uro:status"
                },
                {
                    "attribute": "uro:StationSquareAttribute_uro:dateOfRevision",
                    "data_type": "string",
                    "json_path": "uro:StationSquareAttribute uro:dateOfRevision"
                },
                {
                    "attribute": "uro:StationSquareAttribute_uro:dateOfDecision",
                    "data_type": "string",
                    "json_path": "uro:StationSquareAttribute uro:dateOfDecision"
                },
                {
                    "attribute": "uro:geometrySrcDescLod1",
                    "data_type": "string",
                    "json_path": "uro:DataQualityAttribute uro:geometrySrcDescLod1"
                }
            ],
            "tran/tran:Road": [
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
                    "attribute": "uro:TransportationRoadIDAttribute_uro:prefecture",
                    "data_type": "string",
                    "json_path": "uro:TransportationRoadIDAttribute uro:prefecture"
                },
                {
                    "attribute": "uro:TransportationRoadIDAttribute_uro:city",
                    "data_type": "string",
                    "json_path": "uro:TransportationRoadIDAttribute uro:city"
                },
                {
                    "attribute": "uro:thematicSrcDesc",
                    "data_type": "string",
                    "json_path": "uro:DataQualityAttribute uro:thematicSrcDesc"
                },
                {
                    "attribute": "uro:geometrySrcDescLod1",
                    "data_type": "string",
                    "json_path": "uro:DataQualityAttribute uro:geometrySrcDescLod1"
                }
            ],
            "rwy/tran:Railway": [
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
                    "attribute": "tran_class",
                    "data_type": "string",
                    "json_path": "tran:class"
                },
                {
                    "attribute": "tran_function",
                    "data_type": "string",
                    "json_path": "tran:function"
                },
                {
                    "attribute": "uro:operatorType",
                    "data_type": "string",
                    "json_path": "uro:RailwayRouteAttribute uro:operatorType"
                },
                {
                    "attribute": "uro:operator",
                    "data_type": "string",
                    "json_path": "uro:RailwayRouteAttribute uro:operator"
                },
                {
                    "attribute": "uro:railwayType",
                    "data_type": "string",
                    "json_path": "uro:RailwayRouteAttribute uro:railwayType"
                },
                {
                    "attribute": "uro:startStation",
                    "data_type": "string",
                    "json_path": "uro:RailwayRouteAttribute uro:startStation"
                },
                {
                    "attribute": "uro:endStation",
                    "data_type": "string",
                    "json_path": "uro:RailwayRouteAttribute uro:endStation"
                },
                {
                    "attribute": "uro:TransportationRailwayIDAttribute_uro:prefecture",
                    "data_type": "string",
                    "json_path": "uro:TransportationRailwayIDAttribute uro:prefecture"
                },
                {
                    "attribute": "uro:TransportationRailwayIDAttribute_uro:city",
                    "data_type": "string",
                    "json_path": "uro:TransportationRailwayIDAttribute uro:city"
                },
                {
                    "attribute": "uro:geometrySrcDescLod1",
                    "data_type": "string",
                    "json_path": "uro:DataQualityAttribute uro:geometrySrcDescLod1"
                }
            ],
            "trk/tran:Track": [
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
                    "attribute": "uro:TransportationTrackIDAttribute_uro:prefecture",
                    "data_type": "string",
                    "json_path": "uro:TransportationTrackIDAttribute uro:prefecture"
                },
                {
                    "attribute": "uro:TransportationTrackIDAttribute_uro:city",
                    "data_type": "string",
                    "json_path": "uro:TransportationTrackIDAttribute uro:city"
                },
                {
                    "attribute": "uro:thematicSrcDesc",
                    "data_type": "string",
                    "json_path": "uro:DataQualityAttribute uro:thematicSrcDesc"
                },
                {
                    "attribute": "uro:geometrySrcDescLod1",
                    "data_type": "string",
                    "json_path": "uro:DataQualityAttribute uro:geometrySrcDescLod1"
                }
            ],
            "wwy/tran:Waterway": [
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
                    "attribute": "uro:TransportationWaterwayIDAttribute_uro:prefecture",
                    "data_type": "string",
                    "json_path": "uro:TransportationWaterwayIDAttribute uro:prefecture"
                },
                {
                    "attribute": "uro:TransportationWaterwayIDAttribute_uro:city",
                    "data_type": "string",
                    "json_path": "uro:TransportationWaterwayIDAttribute uro:city"
                },
                {
                    "attribute": "uro:thematicSrcDesc",
                    "data_type": "string",
                    "json_path": "uro:DataQualityAttribute uro:thematicSrcDesc"
                },
                {
                    "attribute": "uro:geometrySrcDescLod1",
                    "data_type": "string",
                    "json_path": "uro:DataQualityAttribute uro:geometrySrcDescLod1"
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
        assert!(flatten_attributes.len() >= 6); // At least bldg + 5 transportation types
        assert!(flatten_attributes.contains_key("bldg/bldg:Building"));
        assert!(flatten_attributes.contains_key("squr/tran:Square"));
        assert!(flatten_attributes.contains_key("tran/tran:Road"));
        assert!(flatten_attributes.contains_key("rwy/tran:Railway"));
        assert!(flatten_attributes.contains_key("trk/tran:Track"));
        assert!(flatten_attributes.contains_key("wwy/tran:Waterway"));
    }
}
