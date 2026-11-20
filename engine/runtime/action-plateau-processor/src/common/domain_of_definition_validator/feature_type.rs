//! L03: the feature types of the package and, with an objectlist, the
//! conformance to the application schema.

use std::collections::BTreeMap;

use fastxml::transform::{EditableNode, EditableNodeRef};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature};
use serde_json::Number;

use super::report::Reporter;
use crate::common::PlateauProfile;
use crate::common::application_schema_conformance::{ElementName, PackageRules, Reason, Violation};

// L03
// Effective geo-feature types of the package, as `prefix:local` with the
// standard prefix.
pub(super) fn valid_feature_types(profile: &PlateauProfile, package: &str) -> Vec<String> {
    let qualified = |prefix: &str, local_names: &[&str]| -> Vec<String> {
        local_names
            .iter()
            .map(|local| format!("{prefix}:{local}"))
            .collect()
    };
    let with_group = |mut types: Vec<String>| {
        types.push("grp:CityObjectGroup".to_string());
        types
    };
    let citygml3 = profile.is_citygml3();
    match package {
        "app" => vec!["http://www.opengis.net/citygml/appearance/2.0".to_string()],
        "area" => qualified(
            "urf",
            &["Zone", "UnclassifiedBlankArea", "UnclassifiedUseDistrict"],
        ),
        "bldg" => with_group(qualified("bldg", &["Building"])),
        "brid" => qualified("brid", &["Bridge"]),
        // CityGML 3.0 adds `OtherConstruction`, which i-UR 3.x defined.
        "cons" if citygml3 => qualified("con", &["OtherConstruction"]),
        "cons" => qualified("uro", &["OtherConstruction"]),
        "dem" => qualified("dem", &["ReliefFeature"]),
        "fld" | "htd" | "ifld" | "rfld" | "tnm" | "wtr" => qualified("wtr", &["WaterBody"]),
        "frn" => qualified("frn", &["CityFurniture"]),
        "gen" => qualified("gen", profile.generic_city_object_types()),
        "lsld" => qualified("urf", &["SedimentDisasterProneArea"]),
        "luse" => qualified("luse", &["LandUse"]),
        "rwy" => qualified("tran", &["Railway"]),
        "squr" => qualified("tran", &["Square"]),
        "tran" => qualified("tran", &["Road"]),
        "trk" => qualified("tran", &["Track"]),
        "tun" => qualified("tun", &["Tunnel"]),
        "ubld" => with_group(qualified("uro", &["UndergroundBuilding"])),
        "veg" => qualified("veg", &["SolitaryVegetationObject", "PlantCover"]),
        // CityGML 3.0 adds `Waterway`, which i-UR 3.x defined.
        "wwy" if citygml3 => qualified("tran", &["Waterway"]),
        "wwy" => qualified("uro", &["Waterway"]),
        "unf" => qualified(
            "uro",
            &[
                "WaterPipe",
                "SewerPipe",
                "ThermalPipe",
                "OilGasChemicalsPipe",
                "Pipe",
                "TelecommunicationsCable",
                "ElectricityCable",
                "Cable",
                "Duct",
                "Appurtenance",
                "Manhole",
                "Handhole",
            ],
        ),
        "urf" => qualified(
            "urf",
            &[
                "UrbanPlanningArea",
                "QuasiUrbanPlanningArea",
                "AreaClassification",
                "DistrictsAndZones",
                "UseDistrict",
                "SpecialUseDistrict",
                "SpecialUseRestrictionDistrict",
                "ExceptionalFloorAreaRateDistrict",
                "HighRiseResidentialAttractionDistrict",
                "HeightControlDistrict",
                "HighLevelUseDistrict",
                "SpecifiedBlock",
                "SpecialUrbanRenaissanceDistrict",
                "HousingControlArea",
                "ResidentialEnvironmentImprovementDistrict",
                "SpecialUseAttractionDistrict",
                "FirePreventionDistrict",
                "SpecifiedDisasterPreventionBlockImprovementZone",
                "LandscapeZone",
                "ScenicDistrict",
                "ParkingPlaceDevelopmentZone",
                "PortZone",
                "SpecialZoneForPreservationOfHistoricalLandscape",
                "ZoneForPreservationOfHistoricalLandscape",
                "GreenSpaceConservationDistrict",
                "SpecialGreenSpaceConservationDistrict",
                "TreePlantingDistrict",
                "DistributionBusinessZone",
                "ProductiveGreenZone",
                "ConservationZoneForClustersOfTraditionalStructures",
                "AircraftNoiseControlZone",
                "ProjectPromotionArea",
                "UrbanRedevelopmentPromotionArea",
                "LandReadjustmentPromotionArea",
                "ResidentialBlockConstructionPromotionArea",
                "LandReadjustmentPromotionAreasForCoreBusinessUrbanDevelopment",
                "UnusedLandUsePromotionArea",
                "UrbanDisasterRecoveryPromotionArea",
                "UrbanFacility",
                "TrafficFacility",
                "OpenSpaceForPublicUse",
                "SupplyFacility",
                "TreatmentFacility",
                "Waterway",
                "EducationalAndCulturalFacility",
                "MedicalFacility",
                "SocialWelfareFacility",
                "MarketsSlaughterhousesCrematoria",
                "CollectiveHousingFacilities",
                "CollectiveGovernmentAndPublicOfficeFacilities",
                "DistributionBusinessPark",
                "CollectiveFacilitiesForTsunamiDisasterPrevention",
                "CollectiveFacilitiesForReconstructionAndRevitalization",
                "CollectiveFacilitiesForReconstruction",
                "CollectiveUrbanDisasterPreventionFacilities",
                "UrbanFacilityStipulatedByCabinetOrder",
                "TelecommunicationFacility",
                "WindProtectionFacility",
                "FireProtectionFacility",
                "TideFacility",
                "FloodPreventionFacility",
                "SnowProtectionFacility",
                "SandControlFacility",
                "UrbanDevelopmentProject",
                "LandReadjustmentProject",
                "NewHousingAndUrbanDevelopmentProject",
                "IndustrialParkDevelopmentProject",
                "UrbanRedevelopmentProject",
                "NewUrbanInfrastructureProject",
                "ResidentialBlockConstructionProject",
                "DisasterPreventionBlockImprovementProject",
                "UrbanRenewalProject",
                "ScheduledAreaForUrbanDevelopmentProject",
                "ScheduledAreaForNewHousingAndUrbanDevelopmentProjects",
                "ScheduledAreaForIndustrialParkDevelopmentProjects",
                "ScheduledAreaForNewUrbanInfrastructureProjects",
                "ScheduledAreaForCollectiveHousingFacilities",
                "ScheduledAreaForCollectiveGovernmentAndPublicOfficeFacilities",
                "ScheduledAreaForDistributionBusinessPark",
                "DistrictPlan",
                "RoadsideDistrictPlan",
                "RuralDistrictPlan",
                "HistoricSceneryMaintenanceAndImprovementDistrictPlan",
                "DisasterPreventionBlockImprovementZonePlan",
                "ResidenceAttractionArea",
                "UrbanFunctionAttractionArea",
            ],
        ),
        _ => Vec::new(),
    }
}

/// The L03 check of a file.
pub(super) struct FeatureTypeCheck<'a> {
    valid_feature_types: Vec<String>,
    /// The objectlist conformance rules of the package; `None` when not
    /// checked.
    package_rules: Option<PackageRules<'a>>,
    /// Whether objectlist conformance runs, in which case each error is sent
    /// as its own feature.
    sends_violations: bool,
    /// { feature type outside the package : number }, filled only without
    /// objectlist conformance, ordered for a stable output.
    invalid_feature_types: BTreeMap<String, u32>,
    invalid_feature_types_num: u32,
    /// The errors of the city object and appearance members, sent after all
    /// members.
    violations: Vec<Violation>,
}

impl<'a> FeatureTypeCheck<'a> {
    pub(super) fn new(
        valid_feature_types: Vec<String>,
        package_rules: Option<PackageRules<'a>>,
        sends_violations: bool,
    ) -> Self {
        Self {
            valid_feature_types,
            package_rules,
            sends_violations,
            invalid_feature_types: BTreeMap::new(),
            invalid_feature_types_num: 0,
            violations: Vec::new(),
        }
    }

    /// Checks a city object member. A feature type outside the package is not
    /// checked further, and a city object group belongs to no package of the
    /// objectlist.
    pub(super) fn check_member(
        &mut self,
        member: &EditableNodeRef,
        member_name: &ElementName,
        is_city_object_group: bool,
        gml_ns: &str,
    ) {
        let out_of_package = !self.valid_feature_types.contains(&member_name.key);
        if !self.sends_violations {
            if out_of_package {
                self.invalid_feature_types_num += 1;
                *self.invalid_feature_types.entry(member.name()).or_insert(0) += 1;
            }
        } else if out_of_package {
            self.violations.push(Violation {
                feature_type: member_name.shown.clone(),
                gml_id: member.get_attribute_ns(gml_ns, "id").unwrap_or_default(),
                path: String::new(),
                reason: Reason::Undefined,
            });
        } else if let (Some(rules), false) = (self.package_rules, is_city_object_group) {
            rules.check_feature(member, gml_ns, &mut self.violations);
        }
    }

    /// Checks the children of an appearance member.
    pub(super) fn check_appearance_member(&mut self, node: &EditableNode, gml_ns: &str) {
        if let Some(rules) = self.package_rules {
            for appearance in node.children() {
                rules.check_feature(&appearance, gml_ns, &mut self.violations);
            }
        }
    }

    /// Counts the errors found and sends each as its own feature.
    pub(super) fn report_violations(&mut self, reporter: &Reporter) {
        for violation in std::mem::take(&mut self.violations) {
            self.invalid_feature_types_num += 1;
            let mut feature = reporter.file_feature(&["index", "udxDirs", "name"]);
            feature.insert(
                "flag",
                AttributeValue::String("ApplicationSchemaViolation".to_string()),
            );
            feature.insert(
                "featureType",
                AttributeValue::String(violation.feature_type),
            );
            feature.insert("gmlId", AttributeValue::String(violation.gml_id));
            feature.insert("path", AttributeValue::String(violation.path));
            feature.insert(
                "reason",
                AttributeValue::String(violation.reason.as_str().to_string()),
            );
            reporter.send(feature);
        }
    }

    pub(super) fn write_summary(&self, summary: &mut Feature) {
        summary.insert(
            "invalidFeatureTypesNum",
            AttributeValue::Number(Number::from(self.invalid_feature_types_num)),
        );
        summary.insert(
            "invalidFeatureTypesDetail",
            AttributeValue::Map(
                self.invalid_feature_types
                    .iter()
                    .map(|(k, v)| {
                        (
                            Attribute::new(k.as_str()),
                            AttributeValue::Number(Number::from(*v)),
                        )
                    })
                    .collect::<Attributes>(),
            ),
        );
    }
}
