use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use nusamai_citygml::GML31_NS;
use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_common::xml::XmlContext;
use reearth_flow_common::xml::{self, XmlRoNode};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_storage::storage::Storage;
use regex::Regex;
use serde::{Deserialize, Serialize};

use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use serde_json::{Number, Value};

use super::errors::PlateauProcessorError;

static VALID_SRS_NAME_6697: &str = "http://www.opengis.net/def/crs/EPSG/0/6697";
static VALID_SRS_NAME_6668: &str = "http://www.opengis.net/def/crs/EPSG/0/6668";

static GML_ID_GROUP_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z]+_[\da-f]{8}(-[\da-f]{4}){3}-[\da-f]{12}$").unwrap());
static GML_LINK_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.+\.gml)#(.+)").unwrap());

static VALID_SRS_NAME_FOR_UNF: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "http://www.opengis.net/def/crs/EPSG/0/10162",
        "http://www.opengis.net/def/crs/EPSG/0/10163",
        "http://www.opengis.net/def/crs/EPSG/0/10164",
        "http://www.opengis.net/def/crs/EPSG/0/10165",
        "http://www.opengis.net/def/crs/EPSG/0/10166",
        "http://www.opengis.net/def/crs/EPSG/0/10167",
        "http://www.opengis.net/def/crs/EPSG/0/10168",
        "http://www.opengis.net/def/crs/EPSG/0/10169",
        "http://www.opengis.net/def/crs/EPSG/0/10170",
        "http://www.opengis.net/def/crs/EPSG/0/10171",
        "http://www.opengis.net/def/crs/EPSG/0/10172",
        "http://www.opengis.net/def/crs/EPSG/0/10173",
        "http://www.opengis.net/def/crs/EPSG/0/10174",
    ]
});

static XML_NAMESPACES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("app", "http://www.opengis.net/citygml/appearance/2.0"),
        ("bldg", "http://www.opengis.net/citygml/building/2.0"),
        ("brid", "http://www.opengis.net/citygml/bridge/2.0"),
        ("core", "http://www.opengis.net/citygml/2.0"),
        ("dem", "http://www.opengis.net/citygml/relief/2.0"),
        ("frn", "http://www.opengis.net/citygml/cityfurniture/2.0"),
        ("gen", "http://www.opengis.net/citygml/generics/2.0"),
        ("gml", "http://www.opengis.net/gml"),
        ("grp", "http://www.opengis.net/citygml/cityobjectgroup/2.0"),
        ("luse", "http://www.opengis.net/citygml/landuse/2.0"),
        ("pbase", "http://www.opengis.net/citygml/profiles/base/2.0"),
        ("sch", "http://www.ascc.net/xml/schematron"),
        ("smil20", "http://www.w3.org/2001/SMIL20/"),
        ("smil20lang", "http://www.w3.org/2001/SMIL20/Language"),
        ("tex", "http://www.opengis.net/citygml/texturedsurface/2.0"),
        ("tran", "http://www.opengis.net/citygml/transportation/2.0"),
        ("tun", "http://www.opengis.net/citygml/tunnel/2.0"),
        ("veg", "http://www.opengis.net/citygml/vegetation/2.0"),
        ("wtr", "http://www.opengis.net/citygml/waterbody/2.0"),
        ("xAL", "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0"),
        ("xlink", "http://www.w3.org/1999/xlink"),
        ("xsi", "http://www.w3.org/2001/XMLSchema-instance"),
        ("uro", "https://www.geospatial.jp/iur/uro/3.0"),
        ("urf", "https://www.geospatial.jp/iur/urf/3.0"),
    ])
});

// L03
// Package : Effective geo-feature type
static PACKAGE_TO_VALID_FEATURE_TYPES: Lazy<HashMap<&'static str, Vec<&'static str>>> =
    Lazy::new(|| {
        HashMap::from([
            ("app", vec!["http://www.opengis.net/citygml/appearance/2.0"]),
            ("bldg", vec!["Building", "CityObjectGroup"]),
            ("brid", vec!["Bridge"]),
            ("cons", vec!["OtherConstruction"]),
            ("dem", vec!["ReliefFeature"]),
            ("fld", vec!["WaterBody"]),
            ("frn", vec!["CityFurniture"]),
            ("gen", vec!["GenericCityObject"]),
            ("htd", vec!["WaterBody"]),
            ("ifld", vec!["WaterBody"]),
            ("lsld", vec!["SedimentDisasterProneArea"]),
            ("luse", vec!["LandUse"]),
            ("rwy", vec!["Railway"]),
            ("squr", vec!["Square"]),
            ("tnm", vec!["WaterBody"]),
            ("tran", vec!["Road"]),
            ("trk", vec!["Track"]),
            ("tun", vec!["Tunnel"]),
            ("ubld", vec!["UndergroundBuilding", "CityObjectGroup"]),
            (
                "unf",
                vec![
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
            (
                "urf",
                vec![
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
            ("veg", vec!["SolitaryVegetationObject", "PlantCover"]),
            ("wtr", vec!["WaterBody"]),
            ("wwy", vec!["Waterway"]),
        ])
    });

// Valid geometry object types for xlink references.
// Valid geometry objects shall be those listed in Standard Operating Procedure Annex B.3. v3. confirmed.
static VALID_GEOMETRY_TYPES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "gml:pos",
        "gml:posList",
        "gml:Point",
        "gml:MultiPoint",
        "gml:LineString",
        "gml:LinearRing",
        "gml:Polygon",
        "gml:OrientableSurface",
        "gml:MultiSurface",
        "gml:CompositeSurface",
        "gml:Solid",
        "gml:Triangle",
        "gml:TriangulatedSurface",
        "gml:Tin",
    ]
});

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct ValidateResponse {
    // L04
    correct_code_values: u32,
    code_value_errors: u32,
    code_space_errors: u32,
    // L06
    correct_extents: u32,
    incorrect_extents: u32,
    // L03({ feature type name : number })
    invalid_feature_types: HashMap<String, u32>,
    invalid_feature_types_num: u32,
    // error counter
    gml_id_not_well_formed_num: u32,
    xlink_has_no_reference_num: u32,
    xlink_invalid_object_type_num: u32,
    invalid_lod_x_geometry_num: u32,
    // envelope
    envelope: Envelope,
    // xlink
    external_file_to_gml_ids: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct Envelope {
    srs_name: String,
    lower_x: f64,
    lower_y: f64,
    lower_z: f64,
    upper_x: f64,
    upper_y: f64,
    upper_z: f64,
}

#[derive(Debug, Clone, Default)]
pub struct DomainOfDefinitionValidatorFactory;

impl ProcessorFactory for DomainOfDefinitionValidatorFactory {
    fn name(&self) -> &str {
        "PLATEAU3.DomainOfDefinitionValidator"
    }

    fn description(&self) -> &str {
        "Validates domain of definition of CityGML features"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let process = DomainOfDefinitionValidator {
            feature_buffer: vec![],
            codelists: None,
        };
        Ok(Box::new(process))
    }
}

type FeatureBuffer = Vec<(Vec<Feature>, HashMap<String, Vec<HashMap<String, String>>>)>;

#[derive(Debug, Clone)]
pub struct DomainOfDefinitionValidator {
    feature_buffer: FeatureBuffer,
    codelists: Option<HashMap<String, HashMap<String, String>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DomainOfDefinitionValidatorParam {
    city_gml_path: Expr,
    codelists_path: Option<String>,
    schemas_path: Option<String>,
}

impl Processor for DomainOfDefinitionValidator {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        if self.codelists.is_none() {
            let codelists =
                create_codelist(Arc::clone(&ctx.storage_resolver), feature).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e))
                })?;
            self.codelists = Some(codelists);
        }
        let codelists = self.codelists.as_ref().unwrap();
        let feature_results = process_feature(&ctx, fw, codelists, feature);
        self.feature_buffer.extend(feature_results);
        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        let mut gml_ids = HashMap::<String, Vec<HashMap<String, String>>>::new();
        for (_, gml_id) in self.feature_buffer.iter() {
            for (k, v) in gml_id.iter() {
                if let std::collections::hash_map::Entry::Vacant(e) = gml_ids.entry(k.to_string()) {
                    e.insert(v.clone());
                } else {
                    gml_ids.get_mut(k).unwrap().extend(v.clone());
                }
            }
        }
        let mut dup_id = 0;
        for (gml_id, attributes) in gml_ids {
            if attributes.len() <= 1 {
                continue;
            }
            for attribute in attributes.iter() {
                let mut result_feature = Feature::new();
                result_feature.insert(
                    "flag",
                    AttributeValue::String("GMLID_NotUnique".to_string()),
                );
                result_feature.insert("gmlId", AttributeValue::String(gml_id.clone()));
                result_feature.insert("duplicateId", AttributeValue::String(dup_id.to_string()));
                result_feature.insert(
                    "numDuplicates",
                    AttributeValue::Number(Number::from(attributes.len())),
                );
                attribute.iter().for_each(|(k, v)| {
                    result_feature.insert(k.clone(), AttributeValue::String(v.clone()));
                });
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    result_feature,
                    DEFAULT_PORT.clone(),
                ));
            }
            dup_id += 1;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "DomainOfDefinitionValidator"
    }
}

#[allow(clippy::type_complexity)]
fn process_feature(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    codelists: &HashMap<String, HashMap<String, String>>,
    feature: &Feature,
) -> super::errors::Result<(Vec<Feature>, HashMap<String, Vec<HashMap<String, String>>>)> {
    let mut gml_ids = HashMap::<String, Vec<HashMap<String, String>>>::new();
    let storage_resolver = Arc::clone(&ctx.storage_resolver);
    let mut result = Vec::<Feature>::new();
    let package = feature.attributes.get(&Attribute::new("package")).ok_or(
        PlateauProcessorError::DomainOfDefinitionValidator("package key empty".to_string()),
    )?;
    let AttributeValue::String(package) = package else {
        return Err(PlateauProcessorError::DomainOfDefinitionValidator(
            "package value not string".to_string(),
        ));
    };
    let mut pattern = "^".to_string();
    pattern.push_str(package);
    pattern.push_str(r"_[\da-f]{8}(-[\da-f]{4}){3}-[\da-f]{12}$");
    let gml_id_pattern = Regex::new(pattern.as_str()).unwrap();
    let valid_feature_types = PACKAGE_TO_VALID_FEATURE_TYPES
        .get(package.as_str())
        .map(|v| v.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        .unwrap_or_default();

    let city_gml_path = feature
        .attributes
        .get(&Attribute::new("cityGmlPath"))
        .ok_or(PlateauProcessorError::DomainOfDefinitionValidator(
            "cityGmlPath key empty".to_string(),
        ))?;

    let city_gml_uri = Uri::from_str(&city_gml_path.to_string())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let storage = storage_resolver
        .resolve(&city_gml_uri)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let xml_content = storage
        .get_sync(city_gml_uri.path().as_path())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let mut response = ValidateResponse::default();

    let xml_document = xml::parse(xml_content)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let root_node = xml::get_root_readonly_node(&xml_document)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let xml_ctx = xml::create_context(&xml_document)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let envelopes = xml::find_readonly_nodes_by_xpath(&xml_ctx, ".//gml:Envelope", &root_node)
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath with {:?}",
                e
            ))
        })?;
    response.envelope = parse_envelope(envelopes)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;

    let members =
        xml::find_readonly_nodes_by_xpath(&xml_ctx, ".//core:cityObjectMember/*", &root_node)
            .map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!(
                    "Failed to evaluate xpath with {:?}",
                    e
                ))
            })?;
    for member in members.iter() {
        let process_result = process_member_node(
            ctx,
            fw,
            &xml_ctx,
            codelists,
            feature,
            member,
            &valid_feature_types,
            &mut response,
            &mut gml_ids,
            &gml_id_pattern,
            Arc::clone(&storage_resolver),
        )?;
        result.extend(process_result);
    }
    // On the city object group model T03: Extracting unreferenced xlink:href

    let members = xml::find_readonly_nodes_by_xpath(
        &xml_ctx,
        ".//core:cityObjectMember/grp:CityObjectGroup",
        &root_node,
    )
    .map_err(|e| {
        PlateauProcessorError::DomainOfDefinitionValidator(format!(
            "Failed to evaluate xpath with {:?}",
            e
        ))
    })?;
    for member in members.iter() {
        let feture_type = member.get_name();
        let gml_id = member
            .get_attribute_ns("id", std::str::from_utf8(GML31_NS.into_inner()).unwrap())
            .unwrap_or_default();
        let xlinks = xml::find_readonly_nodes_by_xpath(&xml_ctx, ".//*[@xlink:href]", &root_node)
            .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath with {:?}",
                e
            ))
        })?;
        for xlink in xlinks {
            let xlink_href = xlink
                .get_attribute_ns("href", "http://www.w3.org/1999/xlink")
                .unwrap_or_default();
            if !gml_ids.contains_key(&xlink_href.chars().skip(1).collect::<String>()) {
                let mut result_feature = feature.clone();
                result_feature.insert(
                    "flag",
                    AttributeValue::String("XLink_NoReference".to_string()),
                );
                result_feature.insert(
                    "tag",
                    AttributeValue::String(xml::get_readonly_node_tag(&xlink)),
                );
                result_feature.insert(
                    "xpath",
                    AttributeValue::String(get_xpath(&xlink, Some(member), None)),
                );
                result_feature.insert("featureType", AttributeValue::String(feture_type.clone()));
                result_feature.insert("gmlId", AttributeValue::String(gml_id.clone()));
                fw.send(
                    ctx.new_with_feature_and_port(result_feature.clone(), DEFAULT_PORT.clone()),
                );
                result.push(result_feature);
                response.xlink_has_no_reference_num += 1;
            }
        }
    }
    let mut result_feature = feature.clone();
    let envelope = &response.envelope;
    result_feature.insert("flag", AttributeValue::String("Summary".to_string()));
    result_feature.insert("srsName", AttributeValue::String(envelope.srs_name.clone()));
    result_feature.insert(
        "invalidFeatureTypesNum",
        AttributeValue::Number(Number::from(response.invalid_feature_types_num)),
    );
    result_feature.insert(
        "invalidFeatureTypesDetail",
        AttributeValue::Map(
            response
                .invalid_feature_types
                .iter()
                .map(|(k, v)| (k.clone(), AttributeValue::Number(Number::from(*v))))
                .collect::<HashMap<_, _>>(),
        ),
    );
    result_feature.insert(
        "correctCodeValues",
        AttributeValue::Number(Number::from(response.correct_code_values)),
    );
    result_feature.insert(
        "inCorrectCodeValue",
        AttributeValue::Number(Number::from(response.code_value_errors)),
    );
    result_feature.insert(
        "inCorrectCodeSpace",
        AttributeValue::Number(Number::from(response.code_space_errors)),
    );
    result_feature.insert(
        "isCorrectSrsName",
        AttributeValue::Bool(
            envelope.srs_name == VALID_SRS_NAME_6697
                || envelope.srs_name == VALID_SRS_NAME_6668
                || (package == "unf"
                    && VALID_SRS_NAME_FOR_UNF.contains(&envelope.srs_name.as_str())),
        ),
    );
    result_feature.insert(
        "correctExtents",
        AttributeValue::Number(Number::from(response.correct_extents)),
    );
    result_feature.insert(
        "inCorrectExtents",
        AttributeValue::Number(Number::from(response.incorrect_extents)),
    );
    result_feature.insert(
        "lowerLatitude",
        AttributeValue::Number(Number::from_f64(envelope.lower_x).unwrap()),
    );
    result_feature.insert(
        "lowerLongitude",
        AttributeValue::Number(Number::from_f64(envelope.lower_y).unwrap()),
    );
    result_feature.insert(
        "lowerElevation",
        AttributeValue::Number(Number::from_f64(envelope.lower_z).unwrap()),
    );
    result_feature.insert(
        "upperLatitude",
        AttributeValue::Number(Number::from_f64(envelope.upper_x).unwrap()),
    );
    result_feature.insert(
        "upperLongitude",
        AttributeValue::Number(Number::from_f64(envelope.upper_y).unwrap()),
    );
    result_feature.insert(
        "upperElevation",
        AttributeValue::Number(Number::from_f64(envelope.upper_z).unwrap()),
    );
    result_feature.insert(
        "gmlIdNotWellformed",
        AttributeValue::Number(Number::from(response.gml_id_not_well_formed_num)),
    );
    result_feature.insert(
        "xlinkHasNoReference",
        AttributeValue::Number(Number::from(response.xlink_has_no_reference_num)),
    );
    result_feature.insert(
        "xlinkInvalidObjectType",
        AttributeValue::Number(Number::from(response.xlink_invalid_object_type_num)),
    );
    result_feature.insert(
        "invalidLodXGeometry",
        AttributeValue::Number(Number::from(response.invalid_lod_x_geometry_num)),
    );
    fw.send(ctx.new_with_feature_and_port(result_feature.clone(), DEFAULT_PORT.clone()));
    result.push(result_feature);
    Ok((result, gml_ids))
}

fn parse_envelope(envelopes: Vec<XmlRoNode>) -> super::errors::Result<Envelope> {
    let envelop_node =
        envelopes
            .first()
            .ok_or(PlateauProcessorError::DomainOfDefinitionValidator(
                "Failed to get envelop node".to_string(),
            ))?;
    let srs_name = envelop_node
        .get_attribute_node("srsName")
        .map(|n| n.get_content())
        .unwrap_or_default();
    let children = envelop_node.get_child_nodes();
    let lower_corner = children
        .iter()
        .find(|&n| xml::get_readonly_node_tag(n) == "gml:lowerCorner")
        .ok_or(PlateauProcessorError::DomainOfDefinitionValidator(
            "Failed to get lower corner node".to_string(),
        ))?;
    let upper_corner = children
        .iter()
        .find(|&n| xml::get_readonly_node_tag(n) == "gml:upperCorner")
        .ok_or(PlateauProcessorError::DomainOfDefinitionValidator(
            "Failed to get upper corner node".to_string(),
        ))?;
    let mut response = Envelope {
        srs_name,
        ..Default::default()
    };
    {
        let content = lower_corner.get_content();
        let lower_corder_value = content.split_whitespace().collect::<Vec<_>>();
        response.lower_x = lower_corder_value[0]
            .parse()
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
        response.lower_y = lower_corder_value[1]
            .parse()
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
        response.lower_z = lower_corder_value[2]
            .parse()
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    }
    {
        let content = upper_corner.get_content();
        let upper_corder_value = content.split_whitespace().collect::<Vec<_>>();
        response.upper_x = upper_corder_value[0]
            .parse()
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
        response.upper_y = upper_corder_value[1]
            .parse()
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
        response.upper_z = upper_corder_value[2]
            .parse()
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    }
    Ok(response)
}

#[allow(clippy::too_many_arguments)]
fn process_member_node(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    xml_ctx: &XmlContext,
    codelists: &HashMap<String, HashMap<String, String>>,
    feature: &Feature,
    member: &XmlRoNode,
    valid_feature_types: &[String],
    response: &mut ValidateResponse,
    all_gml_ids: &mut HashMap<String, Vec<HashMap<String, String>>>,
    gml_id_pattern: &Regex,
    storage_resolver: Arc<StorageResolver>,
) -> super::errors::Result<Vec<Feature>> {
    let mut base_feature = feature.clone();
    let mut result = Vec::<Feature>::new();
    let Some(gml_id) =
        member.get_attribute_ns("id", std::str::from_utf8(GML31_NS.into_inner()).unwrap())
    else {
        return Err(PlateauProcessorError::DomainOfDefinitionValidator(
            "Failed to get gml id".to_string(),
        ));
    };
    base_feature.insert("gmlId", AttributeValue::String(gml_id.clone()));
    let feature_type =
        if XML_NAMESPACES.contains_key(xml::get_readonly_node_prefix(member).as_str()) {
            let name = member.get_name();
            if !valid_feature_types.contains(&name) {
                response.invalid_feature_types_num += 1;
                if response.invalid_feature_types.contains_key(name.as_str()) {
                    *response
                        .invalid_feature_types
                        .get_mut(name.as_str())
                        .unwrap() += 1;
                } else {
                    response.invalid_feature_types.insert(name.clone(), 1);
                }
            }
            name
        } else {
            "".to_string()
        };
    base_feature.insert("featureType", AttributeValue::String(feature_type.clone()));
    let tag = xml::get_readonly_node_tag(member);

    // Verification of the format of gml:id of a geographical object instance
    // grp:CityObjectGroup pattern should be {any prefix}_{UUID}.
    if !gml_id_pattern.is_match(gml_id.as_str())
        || (tag == "grp:CityObjectGroup" && GML_ID_GROUP_PATTERN.is_match(gml_id.as_str()))
    {
        let mut result_feature = base_feature.clone();
        result_feature.insert(
            "flag",
            AttributeValue::String("GMLID_NotWellFormed".to_string()),
        );
        result_feature.insert("tag", AttributeValue::String(tag.clone().to_string()));
        result.push(result_feature.clone());
        fw.send(ctx.new_with_feature_and_port(result_feature, DEFAULT_PORT.clone()));
        response.gml_id_not_well_formed_num += 1;
    }
    // C03: gml:id collection
    // 1. gml:id record of geographic object instance
    let mut gml_ids = HashMap::<String, Vec<HashMap<String, String>>>::new();
    gml_ids.insert(
        gml_id.clone(),
        vec![HashMap::from([
            ("tag".to_string(), tag.clone()),
            ("xpath".to_string(), tag.clone()),
        ])],
    );
    // 2. gml:id collection of lower-level elements
    let gml_id_children = xml::find_readonly_nodes_by_xpath(xml_ctx, ".//*[@gml:id]", member)
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath with {:?}",
                e
            ))
        })?;
    for gml_id_child in gml_id_children {
        let gml_id = gml_id_child
            .get_attribute_ns("id", std::str::from_utf8(GML31_NS.into_inner()).unwrap())
            .unwrap_or_default();
        let tag = xml::get_readonly_node_tag(&gml_id_child);
        gml_ids.insert(
            gml_id.clone(),
            vec![HashMap::from([
                ("tag".to_string(), tag.clone()),
                (
                    "xpath".to_string(),
                    get_xpath(&gml_id_child, Some(member), None),
                ),
            ])],
        );
    }
    for (key, value) in gml_ids.iter() {
        if all_gml_ids.contains_key(key) {
            all_gml_ids.get_mut(key).unwrap().extend(value.clone());
        } else {
            all_gml_ids.insert(key.clone(), value.clone());
        }
    }
    // L04: code definition area verification
    let code_space_children =
        xml::find_readonly_nodes_by_xpath(xml_ctx, ".//*[@codeSpace]", member).map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath with {:?}",
                e
            ))
        })?;
    let city_gml_path = feature
        .attributes
        .get(&Attribute::new("cityGmlPath"))
        .ok_or(PlateauProcessorError::DomainOfDefinitionValidator(
            "cityGmlPath key empty".to_string(),
        ))?;
    let city_gml_uri = Uri::from_str(&city_gml_path.to_string())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let base_dir = city_gml_uri
        .dir()
        .ok_or(PlateauProcessorError::DomainOfDefinitionValidator(
            "illegal city gml path".to_string(),
        ))?;
    for code_space_member in code_space_children {
        let code_space = code_space_member
            .get_attribute_node("codeSpace")
            .map(|n| n.get_content())
            .unwrap_or_default();
        let code_space_path = base_dir
            .join(Path::new(code_space.as_str()))
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
        let code = codelists.get(&code_space_path.to_string());
        let code_value = code_space_member.get_content();
        let mut valid = false;
        let mut exists_code_list = false;
        if let Some(code) = code {
            exists_code_list = true;
            if code.contains_key(code_value.as_str()) {
                valid = true;
                response.correct_code_values += 1;
            } else {
                response.code_value_errors += 1;
            }
        } else {
            response.code_space_errors += 1;
        }
        if !valid {
            let mut result_feature = base_feature.clone();
            result_feature.insert("existsCodeList", AttributeValue::Bool(exists_code_list));
            result_feature.insert("flag", AttributeValue::String("CodeValidation".to_string()));
            result_feature.insert(
                "tag",
                AttributeValue::String(xml::get_readonly_node_tag(&code_space_member)),
            );
            result_feature.insert(
                "xpath",
                AttributeValue::String(get_xpath(&code_space_member, Some(member), None)),
            );
            result_feature.insert("codeSpace", AttributeValue::String(code_space));
            result_feature.insert("codeSpaceValue", AttributeValue::String(code_value));
            result.push(result_feature.clone());
            fw.send(ctx.new_with_feature_and_port(result_feature, DEFAULT_PORT.clone()));
        }
    }
    // L06: Geographical coverage verification
    let mut pos_children = xml::find_readonly_nodes_by_xpath(xml_ctx, ".//gml:pos", member)
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath with {:?}",
                e
            ))
        })?;
    let pos_list_children = xml::find_readonly_nodes_by_xpath(xml_ctx, ".//gml:posList", member)
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath with {:?}",
                e
            ))
        })?;
    let mut positions = Vec::<f64>::new();
    pos_children.extend(pos_list_children);
    for child in pos_children {
        let content = child.get_content();
        let values = content.split_whitespace().collect::<Vec<_>>();
        positions.extend(
            values
                .iter()
                .map(|v| {
                    v.parse().map_err(|e| {
                        PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e))
                    })
                })
                .collect::<super::errors::Result<Vec<f64>>>()?,
        );
    }
    if !positions.is_empty() {
        let envelope = &response.envelope;
        let min_x = *positions
            .iter()
            .step_by(3)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_x = *positions
            .iter()
            .step_by(3)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let min_y = *positions
            .iter()
            .skip(1)
            .step_by(3)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_y = *positions
            .iter()
            .skip(1)
            .step_by(3)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let min_z = *positions
            .iter()
            .skip(2)
            .step_by(3)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max_z = *positions
            .iter()
            .skip(2)
            .step_by(3)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        if envelope.lower_x <= min_x
            && max_x <= envelope.upper_x
            && envelope.lower_y <= min_y
            && max_y <= envelope.upper_y
            && envelope.lower_z <= min_z
            && max_z <= envelope.upper_z
        {
            response.correct_extents += 1;
        } else {
            response.incorrect_extents += 1;
            let mut result_feature = base_feature.clone();
            result_feature.insert(
                "flag",
                AttributeValue::String("ExtentsValidation".to_string()),
            );
            result_feature.insert("isExtentsCorrect", AttributeValue::Bool(false));
            result_feature.insert(
                "lowerLatitude",
                AttributeValue::Number(Number::from_f64(envelope.lower_x).unwrap()),
            );
            result_feature.insert(
                "lowerLongitude",
                AttributeValue::Number(Number::from_f64(envelope.lower_y).unwrap()),
            );
            result_feature.insert(
                "lowerElevation",
                AttributeValue::Number(Number::from_f64(envelope.lower_z).unwrap()),
            );
            result_feature.insert(
                "upperLatitude",
                AttributeValue::Number(Number::from_f64(envelope.upper_x).unwrap()),
            );
            result_feature.insert(
                "upperLongitude",
                AttributeValue::Number(Number::from_f64(envelope.upper_y).unwrap()),
            );
            result_feature.insert(
                "upperElevation",
                AttributeValue::Number(Number::from_f64(envelope.upper_z).unwrap()),
            );
            result_feature.insert(
                "minLatitude",
                AttributeValue::Number(Number::from_f64(min_x).unwrap()),
            );
            result_feature.insert(
                "minLongitude",
                AttributeValue::Number(Number::from_f64(min_y).unwrap()),
            );
            result_feature.insert(
                "minElevation",
                AttributeValue::Number(Number::from_f64(min_z).unwrap()),
            );
            result_feature.insert(
                "maxLatitude",
                AttributeValue::Number(Number::from_f64(max_x).unwrap()),
            );
            result_feature.insert(
                "maxLongitude",
                AttributeValue::Number(Number::from_f64(max_y).unwrap()),
            );
            result_feature.insert(
                "maxElevation",
                AttributeValue::Number(Number::from_f64(max_z).unwrap()),
            );
            result.push(result_feature.clone());
            fw.send(ctx.new_with_feature_and_port(result_feature, DEFAULT_PORT.clone()));
        }
    }
    // T03: Extraction of xlink:hrefs with no referent or whose referent is not a valid geometry object
    let xlink_children = xml::find_readonly_nodes_by_xpath(xml_ctx, ".//*[@xlink:href]", member)
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath with {:?}",
                e
            ))
        })?;
    for child in xlink_children
        .iter()
        .filter(|&child| xml::get_readonly_node_tag(child) != "core:CityObjectGroup")
    {
        let Some(xlink_href) = child.get_attribute_ns("href", "http://www.w3.org/1999/xlink")
        else {
            continue;
        };
        if let Some(caps) = GML_LINK_PATTERN.captures(xlink_href.as_str()) {
            let gml_path = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let gml_id = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
            if !response.external_file_to_gml_ids.contains_key(gml_path) {
                let gml_path_uri = Uri::from_str(gml_path).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e))
                })?;
                let storage = storage_resolver.resolve(&gml_path_uri).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e))
                })?;
                let xml_content = storage
                    .get_sync(gml_path_uri.path().as_path())
                    .map_err(|e| {
                        PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e))
                    })?;
                let xml_document = xml::parse(xml_content).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e))
                })?;
                let xml_ctx = xml::create_context(&xml_document).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e))
                })?;
                let root_node = xml::get_root_readonly_node(&xml_document).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e))
                })?;
                let gml_id_children =
                    xml::find_readonly_nodes_by_xpath(&xml_ctx, ".//*[@gml:id]", &root_node)
                        .map_err(|e| {
                            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                                "Failed to evaluate xpath with {:?}",
                                e
                            ))
                        })?;
                gml_id_children.iter().for_each(|gml_id_node| {
                    let Some(gml_id) = gml_id_node.get_attribute_ns(
                        "id",
                        std::str::from_utf8(GML31_NS.into_inner()).unwrap(),
                    ) else {
                        return;
                    };
                    if response.external_file_to_gml_ids.contains_key(gml_path) {
                        response
                            .external_file_to_gml_ids
                            .get_mut(gml_path)
                            .unwrap()
                            .push(gml_id.clone());
                    } else {
                        response
                            .external_file_to_gml_ids
                            .insert(gml_path.to_string(), vec![gml_id.clone()]);
                    }
                });
            }
            if !response.external_file_to_gml_ids.contains_key(gml_path)
                || !response
                    .external_file_to_gml_ids
                    .get(gml_path)
                    .unwrap()
                    .contains(&gml_id.to_string())
            {
                let mut result_feature = base_feature.clone();
                result_feature.insert(
                    "flag",
                    AttributeValue::String("XLink_NoReference".to_string()),
                );
                result_feature.insert(
                    "tag",
                    AttributeValue::String(xml::get_readonly_node_tag(child)),
                );
                result_feature.insert(
                    "xpath",
                    AttributeValue::String(get_xpath(child, Some(member), None)),
                );
                result.push(result_feature.clone());
                fw.send(ctx.new_with_feature_and_port(result_feature, DEFAULT_PORT.clone()));
                response.xlink_has_no_reference_num += 1;
            }
        } else if !gml_ids.contains_key(&xlink_href.chars().skip(1).collect::<String>()) {
            let mut result_feature = base_feature.clone();
            result_feature.insert(
                "flag",
                AttributeValue::String("XLink_NoReference".to_string()),
            );
            result_feature.insert(
                "tag",
                AttributeValue::String(xml::get_readonly_node_tag(child)),
            );
            result_feature.insert(
                "xpath",
                AttributeValue::String(get_xpath(child, Some(member), None)),
            );
            result_feature.insert("xlinkHref", AttributeValue::String(xlink_href.clone()));
            result.push(result_feature.clone());
            fw.send(ctx.new_with_feature_and_port(result_feature, DEFAULT_PORT.clone()));
            response.xlink_has_no_reference_num += 1;
        } else if let Some(gml_ids) = gml_ids.get(&xlink_href.chars().skip(1).collect::<String>()) {
            for item in gml_ids.iter().filter(|&item| {
                !VALID_GEOMETRY_TYPES
                    .contains(&item.get("tag").map(|t| t.as_str()).unwrap_or_default())
            }) {
                let mut result_feature = base_feature.clone();
                result_feature.insert(
                    "flag",
                    AttributeValue::String("XLink_NoReference".to_string()),
                );
                result_feature.insert(
                    "tag",
                    AttributeValue::String(xml::get_readonly_node_tag(child)),
                );
                result_feature.insert(
                    "xpath",
                    AttributeValue::String(get_xpath(child, Some(member), None)),
                );
                result_feature.insert("xlinkHref", AttributeValue::String(xlink_href.clone()));
                result_feature.insert(
                    "refGmlId",
                    AttributeValue::String(xlink_href.chars().skip(1).collect::<String>()),
                );
                result_feature.insert(
                    "refGmlTag",
                    AttributeValue::String(item.get("tag").cloned().unwrap_or_default()),
                );
                result_feature.insert(
                    "refGmlXpath",
                    AttributeValue::String(item.get("xpath").cloned().unwrap_or_default()),
                );
                result.push(result_feature.clone());
                fw.send(ctx.new_with_feature_and_port(result_feature, DEFAULT_PORT.clone()));
                response.xlink_invalid_object_type_num += 1;
            }
        }
    }
    // L-frn-01: Validation of geometric object types described as lod{0-4}Geometry.
    for lod in 0..4 {
        let mut xpath = ".//*[local-name()='lod".to_string();
        xpath.push_str(lod.to_string().as_str());
        xpath.push_str("Geometry']");

        let children = xml::find_readonly_nodes_by_xpath(xml_ctx, &xpath, member).map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath with {:?}",
                e
            ))
        })?;
        for child in children {
            let Some(parent) = child.get_parent() else {
                continue;
            };
            let parent_tag = xml::get_readonly_node_tag(&parent);
            let gml_tag = {
                let gml = xml::find_readonly_nodes_by_xpath(
                    xml_ctx,
                    "./*[namespace-uri()='http://www.opengis.net/gml']",
                    &child,
                )
                .map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!(
                        "Failed to evaluate xpath with {:?}",
                        e
                    ))
                })?;
                if gml.is_empty() {
                    "".to_string()
                } else {
                    xml::get_readonly_node_tag(gml.first().unwrap())
                }
            };
            let is_valid = {
                if parent_tag == "gen:GenericCityObject" {
                    if lod == 0 {
                        [
                            "gml:Point",
                            "gml:MultiPoint",
                            "gml:MultiCurve",
                            "gml:MultiSurface",
                        ]
                        .contains(&gml_tag.as_str())
                    } else {
                        ["gml:MultiSurface", "gml:Solid"].contains(&gml_tag.as_str())
                    }
                } else if parent_tag.contains(":DmG") {
                    [
                        "gml:Point",
                        "gml:MultiPoint",
                        "gml:MultiCurve",
                        "gml:MultiSurface",
                    ]
                    .contains(&gml_tag.as_str())
                } else if parent_tag.contains(":DmA") {
                    gml_tag == "gml:Point"
                } else {
                    ["gml:MultiSurface", "gml:Solid"].contains(&gml_tag.as_str())
                }
            };
            if !is_valid {
                let mut result_feature = base_feature.clone();
                result_feature.insert(
                    "flag",
                    AttributeValue::String("InvalidLodXGeometry".to_string()),
                );
                result_feature.insert("parentTag", AttributeValue::String(parent_tag));
                if let Some(gml_id) = parent
                    .get_attribute_ns("id", std::str::from_utf8(GML31_NS.into_inner()).unwrap())
                {
                    result_feature.insert("gmlId", AttributeValue::String(gml_id));
                } else {
                    result_feature.insert("gmlId", AttributeValue::String("".to_string()));
                }
                result_feature.insert("invalidGeometry", AttributeValue::String(gml_tag));
                result.push(result_feature.clone());
                fw.send(ctx.new_with_feature_and_port(result_feature, DEFAULT_PORT.clone()));
                response.invalid_lod_x_geometry_num += 1;
            }
        }
    }
    Ok(result)
}

fn get_xpath(node: &XmlRoNode, top: Option<&XmlRoNode>, tags: Option<Vec<String>>) -> String {
    let xpath = node.get_name();
    let tag = xml::get_readonly_node_tag(node);
    let mut tags = tags.unwrap_or_default();
    tags.push(tag.clone());
    let parent = node.get_parent();
    if (top.is_none() && parent.is_none()) || &parent.unwrap() == top.unwrap() {
        if let Some(top) = top {
            tags.push(xml::get_readonly_node_tag(top));
        }
        tags.reverse();
        tags.join("/")
    } else {
        let parent = node.get_parent();
        if parent.is_none() {
            return xpath;
        }
        get_xpath(&parent.unwrap(), top, Some(tags))
    }
}

fn create_codelist(
    storage_resolver: Arc<StorageResolver>,
    first: &Feature,
) -> super::errors::Result<HashMap<String, HashMap<String, String>>> {
    let dir = first
        .attributes
        .get(&Attribute::new("dirCodelists"))
        .ok_or(PlateauProcessorError::DomainOfDefinitionValidator(
            "dirCodelists key empty".to_string(),
        ))?;
    let dir = Uri::from_str(&dir.to_string())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let storage = storage_resolver
        .resolve(&dir)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let exist = storage
        .exists_sync(dir.path().as_path())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    if !exist {
        return Err(PlateauProcessorError::DomainOfDefinitionValidator(
            "dirCodelists not found".to_string(),
        ));
    }
    let mut codelist = HashMap::new();
    let lists = storage
        .list_sync(Some(dir.path().as_path()), true)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    for file in lists
        .iter()
        .filter(|f| f.is_file() && f.path().extension() == Some("xml".as_ref()))
    {
        let result = create_detail_codelist(Arc::clone(&storage), file)
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
        codelist.insert(file.to_string(), result);
    }
    Ok(codelist)
}

fn create_detail_codelist(
    storage: Arc<Storage>,
    xml_path: &Uri,
) -> super::errors::Result<HashMap<String, String>> {
    let xml_content = storage
        .get_sync(xml_path.path().as_path())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let xml_content = String::from_utf8(xml_content.to_vec())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let xml_document = xml::parse(xml_content)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let ctx = xml::create_context(&xml_document)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let root = xml::get_root_readonly_node(&xml_document)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{:?}", e)))?;
    let definitions =
        xml::find_readonly_nodes_by_xpath(&ctx, ".//gml:Definition", &root).map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath with {:?}",
                e
            ))
        })?;
    let result = definitions
        .iter()
        .flat_map(|node| {
            let nodes = node.get_child_nodes();
            let name = nodes
                .iter()
                .find(|n| xml::get_readonly_node_tag(n) == "gml:name")?;
            let description = nodes
                .iter()
                .find(|n| xml::get_readonly_node_tag(n) == "gml:description")?;
            Some((name.get_content(), description.get_content()))
        })
        .collect::<HashMap<String, String>>();
    Ok(result)
}
