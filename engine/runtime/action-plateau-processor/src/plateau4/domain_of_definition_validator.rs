use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use fastxml::transform::{EditableNode, StreamTransformer};
use nusamai_citygml::GML31_NS;
use once_cell::sync::Lazy;
use quick_xml::{events::Event, Reader};
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

use reearth_flow_types::{Attribute, AttributeValue, Attributes, Expr, Feature};
use schemars::JsonSchema;
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

static DUPLICATE_GML_ID_STATS_PORT: Lazy<Port> = Lazy::new(|| Port::new("duplicateGmlIdStats"));

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

/// # DomainOfDefinitionValidator Parameters
///
/// Configuration for validating domain of definition of CityGML features.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct DomainOfDefinitionValidatorParam {
    /// Fallback codelists directory path expression. When codelists files are not found
    /// at the location relative to the GML file, this path will be used as the base
    /// directory for resolving codeSpace references.
    #[serde(skip_serializing_if = "Option::is_none")]
    codelists_path: Option<Expr>,
}

#[derive(Debug, Clone, Default)]
pub struct DomainOfDefinitionValidatorFactory;

impl ProcessorFactory for DomainOfDefinitionValidatorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.DomainOfDefinitionValidator"
    }

    fn description(&self) -> &str {
        "Validates domain of definition of CityGML features"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(DomainOfDefinitionValidatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            DEFAULT_PORT.clone(),
            REJECTED_PORT.clone(),
            DUPLICATE_GML_ID_STATS_PORT.clone(),
        ]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let global_params = with.clone();
        let params: DomainOfDefinitionValidatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            DomainOfDefinitionValidatorParam::default()
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let codelists_path_expr = if let Some(expr) = params.codelists_path {
            Some(expr_engine.compile(expr.as_ref()).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidatorFactory(format!(
                    "Failed to compile codelists_path expression: {e}"
                ))
            })?)
        } else {
            None
        };

        let process = DomainOfDefinitionValidator {
            feature_buffer: vec![],
            codelists: None,
            filenames: HashSet::new(),
            codelists_path_expr,
            global_params,
        };
        Ok(Box::new(process))
    }
}

type FeatureBuffer = Vec<(Vec<Feature>, HashMap<String, Vec<HashMap<String, String>>>)>;

#[derive(Debug, Clone)]
pub struct DomainOfDefinitionValidator {
    feature_buffer: FeatureBuffer,
    codelists: Option<HashMap<String, HashMap<String, String>>>,
    filenames: HashSet<String>,
    codelists_path_expr: Option<rhai::AST>,
    global_params: Option<HashMap<String, serde_json::Value>>,
}

impl Processor for DomainOfDefinitionValidator {
    fn num_threads(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        // Collect file name for statistics output
        if let Some(AttributeValue::String(name)) = feature.attributes.get(&Attribute::new("name"))
        {
            self.filenames.insert(name.clone());
        }

        // Evaluate fallback codelists path expression if provided
        let fallback_codelists_path = if let Some(expr) = &self.codelists_path_expr {
            let scope = feature.new_scope(Arc::clone(&ctx.expr_engine), &self.global_params);
            let path: String = scope.eval_ast(expr).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!(
                    "Failed to evaluate codelists_path expression: {e:?}"
                ))
            })?;
            Some(Uri::from_str(&path).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!(
                    "Invalid codelists_path URI: {e:?}"
                ))
            })?)
        } else {
            None
        };

        if self.codelists.is_none() {
            let codelists = create_codelist(
                Arc::clone(&ctx.storage_resolver),
                feature,
                fallback_codelists_path.as_ref(),
            )
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
            self.codelists = Some(codelists);
        }
        let codelists = self.codelists.as_ref().unwrap();
        let feature_results = process_feature(
            &ctx,
            fw,
            codelists,
            feature,
            fallback_codelists_path.as_ref(),
        )?;
        self.feature_buffer.push(feature_results);
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
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

        // Count duplicate gml:id occurrences per file
        let mut duplicate_gml_id_count = HashMap::<String, usize>::new();

        for attributes in gml_ids.values() {
            if attributes.len() > 1 {
                // Count each file containing duplicate gml:ids
                for attribute in attributes.iter() {
                    if let Some(name) = attribute.get("name") {
                        *duplicate_gml_id_count.entry(name.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Output statistics for all files (including those with 0 duplicates)
        for filename in &self.filenames {
            let duplicate_count = duplicate_gml_id_count.get(filename).unwrap_or(&0);

            let mut result_feature = Feature::new_with_attributes(Attributes::new());
            result_feature.insert("filename", AttributeValue::String(filename.clone()));
            result_feature.insert(
                "duplicateGmlIdCount",
                AttributeValue::Number(Number::from(*duplicate_count)),
            );

            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                result_feature,
                DUPLICATE_GML_ID_STATS_PORT.clone(),
            ));
        }

        // Output individual duplicate gml:id details
        let mut dup_id = 0;
        for (gml_id, attributes) in gml_ids {
            if attributes.len() <= 1 {
                continue;
            }
            for attribute in attributes.iter() {
                let mut result_feature = Feature::new_with_attributes(Attributes::new());
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

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn process_feature(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    codelists: &HashMap<String, HashMap<String, String>>,
    feature: &Feature,
    fallback_codelists_path: Option<&Uri>,
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

    let city_gml_path = feature.attributes.get(&Attribute::new("path")).ok_or(
        PlateauProcessorError::DomainOfDefinitionValidator("path key empty".to_string()),
    )?;

    let city_gml_uri = Uri::from_str(&city_gml_path.to_string())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let storage = storage_resolver
        .resolve(&city_gml_uri)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let xml_content = storage
        .get_sync(city_gml_uri.path().as_path())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let xml_str = String::from_utf8(xml_content.to_vec())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let root_namespaces = fastxml::namespace::extract_root_namespaces(&xml_str).map_err(|e| {
        PlateauProcessorError::DomainOfDefinitionValidator(format!(
            "Failed to parse root namespaces: {e:?}"
        ))
    })?;
    let xlink_prefixes = root_namespaces
        .iter()
        .filter(|(_, uri)| uri.as_str() == "http://www.w3.org/1999/xlink")
        .map(|(prefix, _)| prefix.to_string())
        .collect::<Vec<_>>();

    let mut response = ValidateResponse::default();

    let envelope_xpath =
        "//*[namespace-uri()='http://www.opengis.net/gml'][local-name()='Envelope']";
    response.envelope = stream_extract_envelope(&xml_str, envelope_xpath)?;

    let mut city_object_groups = Vec::<CityObjectGroupInfo>::new();
    let mut stream_error: Option<PlateauProcessorError> = None;
    let members_xpath =
        "//*[namespace-uri()='http://www.opengis.net/citygml/2.0'][local-name()='cityObjectMember']";

    let transformer = StreamTransformer::new(&xml_str)
        .with_root_namespaces()
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to parse root namespaces: {e:?}"
            ))
        })?;

    transformer
        .on(&members_xpath, |node| {
            if stream_error.is_some() {
                return;
            }

            let fragment = match editable_node_to_xml(node) {
                Ok(fragment) => fragment,
                Err(err) => {
                    stream_error = Some(err);
                    return;
                }
            };

            let xml_document = match xml::parse(fragment.as_bytes()) {
                Ok(doc) => doc,
                Err(e) => {
                    stream_error = Some(PlateauProcessorError::DomainOfDefinitionValidator(format!(
                        "{e:?}"
                    )));
                    return;
                }
            };
            let xml_ctx = match xml::create_context(&xml_document) {
                Ok(ctx) => ctx,
                Err(e) => {
                    stream_error = Some(PlateauProcessorError::DomainOfDefinitionValidator(format!(
                        "{e:?}"
                    )));
                    return;
                }
            };
            let root_node = match xml::get_root_readonly_node(&xml_document) {
                Ok(node) => node,
                Err(e) => {
                    stream_error = Some(PlateauProcessorError::DomainOfDefinitionValidator(format!(
                        "{e:?}"
                    )));
                    return;
                }
            };
            let member_nodes = match xml::find_readonly_nodes_by_xpath(&xml_ctx, "./*", &root_node)
            {
                Ok(nodes) => nodes,
                Err(e) => {
                    stream_error = Some(PlateauProcessorError::DomainOfDefinitionValidator(format!(
                        "{e:?}"
                    )));
                    return;
                }
            };
            let Some(member_node) = member_nodes.first() else {
                stream_error = Some(PlateauProcessorError::DomainOfDefinitionValidator(
                    "Failed to get cityObjectMember child".to_string(),
                ));
                return;
            };

            let is_city_object_group = member_node.get_name() == "CityObjectGroup"
                && member_node
                    .get_namespace()
                    .map(|ns| ns.get_href() == "http://www.opengis.net/citygml/cityobjectgroup/2.0")
                    .unwrap_or(false);
            if is_city_object_group {
                let gml_ns = std::str::from_utf8(GML31_NS.into_inner()).unwrap_or("");
                let gml_id = member_node
                    .get_attribute_ns("id", gml_ns)
                    .unwrap_or_default();
                city_object_groups.push(CityObjectGroupInfo {
                    feature_type: member_node.get_name(),
                    gml_id,
                });
            }

            match process_member_node(
                ctx,
                fw,
                &xml_ctx,
                codelists,
                feature,
                member_node,
                &valid_feature_types,
                &mut response,
                &mut gml_ids,
                &gml_id_pattern,
                Arc::clone(&storage_resolver),
                fallback_codelists_path,
            ) {
                Ok(process_result) => {
                    result.extend(process_result);
                }
                Err(e) => {
                    stream_error = Some(e);
                }
            }
        })
        .for_each()
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;

    if let Some(err) = stream_error {
        return Err(err);
    }

    // On the city object group model T03: Extracting unreferenced xlink:href
    let xlinks = collect_xlinks(&xml_str, &xlink_prefixes)?;
    for member in city_object_groups.iter() {
        for xlink in xlinks.iter() {
            let xlink_href = xlink.href.clone();
            if !gml_ids.contains_key(&xlink_href.chars().skip(1).collect::<String>()) {
                let mut result_feature = feature.clone();
                result_feature.insert(
                    "flag",
                    AttributeValue::String("XLink_NoReference".to_string()),
                );
                result_feature.insert("tag", AttributeValue::String(xlink.tag.clone()));
                result_feature.insert("xpath", AttributeValue::String(xlink.xpath.clone()));
                result_feature.insert(
                    "featureType",
                    AttributeValue::String(member.feature_type.clone()),
                );
                result_feature.insert("gmlId", AttributeValue::String(member.gml_id.clone()));
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

fn parse_envelope_from_node(node: &EditableNode) -> super::errors::Result<Envelope> {
    let mut response = Envelope {
        srs_name: node.get_attribute("srsName").unwrap_or_default(),
        ..Default::default()
    };

    let mut lower_corner_value = None;
    let mut upper_corner_value = None;
    for child in node.children() {
        match child.name().as_str() {
            "lowerCorner" => lower_corner_value = child.get_content(),
            "upperCorner" => upper_corner_value = child.get_content(),
            _ => {}
        }
    }

    let Some(lower_corner_value) = lower_corner_value else {
        return Err(PlateauProcessorError::DomainOfDefinitionValidator(
            "Failed to get lower corner node".to_string(),
        ));
    };
    let lower_corder_value = lower_corner_value.split_whitespace().collect::<Vec<_>>();
    if lower_corder_value.len() < 3 {
        return Err(PlateauProcessorError::DomainOfDefinitionValidator(
            "Failed to parse lower corner".to_string(),
        ));
    }
    response.lower_x = lower_corder_value[0]
        .parse()
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    response.lower_y = lower_corder_value[1]
        .parse()
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    response.lower_z = lower_corder_value[2]
        .parse()
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;

    let Some(upper_corner_value) = upper_corner_value else {
        return Err(PlateauProcessorError::DomainOfDefinitionValidator(
            "Failed to get upper corner node".to_string(),
        ));
    };
    let upper_corder_value = upper_corner_value.split_whitespace().collect::<Vec<_>>();
    if upper_corder_value.len() < 3 {
        return Err(PlateauProcessorError::DomainOfDefinitionValidator(
            "Failed to parse upper corner".to_string(),
        ));
    }
    response.upper_x = upper_corder_value[0]
        .parse()
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    response.upper_y = upper_corder_value[1]
        .parse()
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    response.upper_z = upper_corder_value[2]
        .parse()
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;

    Ok(response)
}

#[derive(Clone, Debug)]
struct CityObjectGroupInfo {
    feature_type: String,
    gml_id: String,
}

#[derive(Clone, Debug)]
struct XlinkInfo {
    href: String,
    tag: String,
    xpath: String,
}

fn stream_extract_envelope(
    raw_xml: &str,
    xpath: &str,
) -> super::errors::Result<Envelope> {
    let envelope = RefCell::new(None);
    let error = RefCell::new(None);

    let transformer = StreamTransformer::new(raw_xml)
        .with_root_namespaces()
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to parse root namespaces: {e:?}"
            ))
        })?;

    transformer
        .on(xpath, |node| {
            if envelope.borrow().is_some() || error.borrow().is_some() {
                return;
            }
            match parse_envelope_from_node(node) {
                Ok(value) => {
                    *envelope.borrow_mut() = Some(value);
                }
                Err(err) => {
                    *error.borrow_mut() = Some(err);
                }
            }
        })
        .for_each()
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;

    if let Some(err) = error.into_inner() {
        return Err(err);
    }

    envelope.into_inner().ok_or_else(|| {
        PlateauProcessorError::DomainOfDefinitionValidator(
            "Failed to get envelope element".to_string(),
        )
    })
}

fn editable_node_to_xml(node: &mut EditableNode) -> super::errors::Result<String> {
    node.to_xml_with_namespaces()
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))
}

fn collect_xlinks(
    raw_xml: &str,
    xlink_prefixes: &[String],
) -> super::errors::Result<Vec<XlinkInfo>> {
    let mut reader = Reader::from_str(raw_xml);
    reader.config_mut().trim_text(false);
    reader.config_mut().expand_empty_elements = true;

    let mut buf = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut xlinks = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let qname = event_qname(&e)?;
                if let Some(href) = event_xlink_href(&e, xlink_prefixes)? {
                    let xpath = if stack.is_empty() {
                        qname.clone()
                    } else {
                        format!("{}/{}", stack.join("/"), qname)
                    };
                    xlinks.push(XlinkInfo {
                        href,
                        tag: qname.clone(),
                        xpath,
                    });
                }
                stack.push(qname);
            }
            Ok(Event::Empty(e)) => {
                let qname = event_qname(&e)?;
                if let Some(href) = event_xlink_href(&e, xlink_prefixes)? {
                    let xpath = if stack.is_empty() {
                        qname.clone()
                    } else {
                        format!("{}/{}", stack.join("/"), qname)
                    };
                    xlinks.push(XlinkInfo {
                        href,
                        tag: qname,
                        xpath,
                    });
                }
            }
            Ok(Event::End(_)) => {
                stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(err) => {
                return Err(PlateauProcessorError::DomainOfDefinitionValidator(format!(
                    "Failed to parse XML for xlink: {err:?}"
                )))
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(xlinks)
}

fn event_qname(e: &quick_xml::events::BytesStart<'_>) -> super::errors::Result<String> {
    let name = e.name();
    let qname = std::str::from_utf8(name.as_ref()).map_err(|e| {
        PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
    })?;
    Ok(qname.to_string())
}

fn event_xlink_href(
    e: &quick_xml::events::BytesStart<'_>,
    xlink_prefixes: &[String],
) -> super::errors::Result<Option<String>> {
    let mut expected_keys = Vec::new();
    for prefix in xlink_prefixes {
        if prefix.is_empty() {
            expected_keys.push("href".to_string());
        } else {
            expected_keys.push(format!("{prefix}:href"));
        }
    }
    if expected_keys.is_empty() {
        expected_keys.push("xlink:href".to_string());
    }

    for attr in e.attributes().filter_map(|a| a.ok()) {
        let key = std::str::from_utf8(attr.key.as_ref()).map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
        })?;
        if expected_keys.iter().any(|expected| expected == key) {
            let value = attr.unescape_value().map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
            })?;
            return Ok(Some(value.to_string()));
        }
    }

    Ok(None)
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
    fallback_codelists_path: Option<&Uri>,
) -> super::errors::Result<Vec<Feature>> {
    let mut base_feature = feature.clone();
    let mut result = Vec::<Feature>::new();
    let gml_ns = std::str::from_utf8(GML31_NS.into_inner()).unwrap_or("");
    let xlink_ns = "http://www.w3.org/1999/xlink";
    let Some(gml_id) = member.get_attribute_ns("id", gml_ns) else {
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
            (
                "index".to_string(),
                feature
                    .attributes
                    .get(&Attribute::new("index"))
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ),
            (
                "udxDirs".to_string(),
                feature
                    .attributes
                    .get(&Attribute::new("udxDirs"))
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ),
            (
                "name".to_string(),
                feature
                    .attributes
                    .get(&Attribute::new("name"))
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ),
        ])],
    );
    // 2. gml:id collection of lower-level elements
    let gml_id_children = xml::find_readonly_nodes_by_xpath(xml_ctx, ".//*[@gml:id]", member)
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath at {}:{}: {e:?}",
                file!(),
                line!()
            ))
        })?;
    for gml_id_child in gml_id_children {
        let gml_id = gml_id_child
            .get_attribute_ns("id", gml_ns)
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
                (
                    "index".to_string(),
                    feature
                        .attributes
                        .get(&Attribute::new("index"))
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "udxDirs".to_string(),
                    feature
                        .attributes
                        .get(&Attribute::new("udxDirs"))
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                ),
                (
                    "name".to_string(),
                    feature
                        .attributes
                        .get(&Attribute::new("name"))
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
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
                "Failed to evaluate xpath at {}:{}: {e:?}",
                file!(),
                line!()
            ))
        })?;
    let city_gml_path = feature.attributes.get(&Attribute::new("path")).ok_or(
        PlateauProcessorError::DomainOfDefinitionValidator("path key empty".to_string()),
    )?;
    let city_gml_uri = Uri::from_str(&city_gml_path.to_string())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let base_dir = city_gml_uri
        .dir()
        .ok_or(PlateauProcessorError::DomainOfDefinitionValidator(
            "illegal city gml path".to_string(),
        ))?;
    for code_space_member in code_space_children {
        let code_space = code_space_member
            .get_attribute("codeSpace")
            .unwrap_or_default();

        // First, try to resolve using dirCodelists if codeSpace contains "codelists/"
        // This handles cases where GML files are extracted to a temp directory but codelists are elsewhere
        let mut code = None;
        let mut dynamically_loaded_codelist;
        if let Some(codelists_relative) = extract_codelists_relative_path(&code_space) {
            if let Some(dir_codelists) = get_dir_codelists(feature) {
                // Build the expected path by finding matching entry in codelists dictionary
                // We search for entries ending with the relative path since URI formats may vary
                let suffix = format!("/{}", codelists_relative);
                code = codelists
                    .iter()
                    .find(|(k, _)| k.ends_with(&suffix))
                    .map(|(_, v)| v);

                // If not found by suffix, try direct join
                if code.is_none() {
                    if let Ok(resolved_path) = dir_codelists.join(&codelists_relative) {
                        code = codelists.get(&resolved_path.to_string());
                    }
                }

                // If still not found in pre-loaded dictionary, try to load the codelist file dynamically
                // This handles cases where the codelists dictionary was loaded from a different directory
                if code.is_none() {
                    if let Ok(resolved_path) = dir_codelists.join(&codelists_relative) {
                        if let Ok(loaded) =
                            try_load_codelist_file(Arc::clone(&storage_resolver), &resolved_path)
                        {
                            dynamically_loaded_codelist = loaded;
                            code = Some(&dynamically_loaded_codelist);
                        }
                    }
                }
            }
        }

        // Fallback: Try primary lookup (relative to GML file)
        if code.is_none() {
            let code_space_path = base_dir.join(Path::new(code_space.as_str())).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
            })?;
            code = codelists.get(&code_space_path.to_string());
        }

        // If still not found and fallback path is provided, try the fallback path
        if code.is_none() {
            if let Some(fallback_path) = fallback_codelists_path {
                // Extract the filename from codeSpace (e.g., "../../codelists/Building_class.xml" -> "Building_class.xml")
                if let Some(filename) = Path::new(code_space.as_str()).file_name() {
                    let fallback_code_space_path =
                        fallback_path.join(Path::new(filename)).map_err(|e| {
                            PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
                        })?;
                    // First try dictionary lookup
                    code = codelists.get(&fallback_code_space_path.to_string());

                    // If not in dictionary, try dynamic loading
                    if code.is_none() {
                        if let Ok(loaded) = try_load_codelist_file(
                            Arc::clone(&storage_resolver),
                            &fallback_code_space_path,
                        ) {
                            dynamically_loaded_codelist = loaded;
                            code = Some(&dynamically_loaded_codelist);
                        }
                    }
                }
            }
        }

        let code_value = code_space_member.get_content().unwrap_or_default();

        if let Some(code) = code {
            if code.contains_key(code_value.as_str()) {
                response.correct_code_values += 1;
                continue;
            }

            response.code_value_errors += 1;
            handle_code_validation_failure(
                ctx,
                fw,
                "CodeValidation",
                &base_feature,
                &code_space_member,
                member,
                &code_space,
                &code_value,
                &mut result,
            );
        } else {
            response.code_space_errors += 1;
            handle_code_validation_failure(
                ctx,
                fw,
                "inCorrectCodeSpace",
                &base_feature,
                &code_space_member,
                member,
                &code_space,
                &code_value,
                &mut result,
            );
        }
    }
    // L06: Geographical coverage verification
    let mut pos_children = xml::find_readonly_nodes_by_xpath(xml_ctx, ".//gml:pos", member)
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath at {}:{}: {e:?}",
                file!(),
                line!()
            ))
        })?;
    let pos_list_children = xml::find_readonly_nodes_by_xpath(xml_ctx, ".//gml:posList", member)
        .map_err(|e| {
            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                "Failed to evaluate xpath at {}:{}: {e:?}",
                file!(),
                line!()
            ))
        })?;
    let mut positions = Vec::<f64>::new();
    pos_children.extend(pos_list_children);
    for child in pos_children {
        let content = child.get_content().unwrap_or_default();
        let values = content.split_whitespace().collect::<Vec<_>>();
        positions.extend(
            values
                .iter()
                .map(|v| {
                    v.parse().map_err(|e| {
                        PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
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
                "Failed to evaluate xpath at {}:{}: {e:?}",
                file!(),
                line!()
            ))
        })?;
    for child in xlink_children
        .iter()
        .filter(|&child| xml::get_readonly_node_tag(child) != "core:CityObjectGroup")
    {
        let Some(xlink_href) = child.get_attribute_ns("href", xlink_ns) else {
            continue;
        };
        if let Some(caps) = GML_LINK_PATTERN.captures(xlink_href.as_str()) {
            let gml_path = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let gml_id = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
            if !response.external_file_to_gml_ids.contains_key(gml_path) {
                let gml_path_uri = Uri::from_str(gml_path).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
                })?;
                let storage = storage_resolver.resolve(&gml_path_uri).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
                })?;
                let xml_content = storage
                    .get_sync(gml_path_uri.path().as_path())
                    .map_err(|e| {
                        PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
                    })?;
                let xml_document = xml::parse(xml_content).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
                })?;
                let xml_ctx = xml::create_context(&xml_document).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
                })?;
                let root_node = xml::get_root_readonly_node(&xml_document).map_err(|e| {
                    PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
                })?;
                let gml_id_children =
                    xml::find_readonly_nodes_by_xpath(&xml_ctx, ".//*[@gml:id]", &root_node)
                        .map_err(|e| {
                            PlateauProcessorError::DomainOfDefinitionValidator(format!(
                                "Failed to evaluate xpath at {}:{}: {e:?}",
                                file!(),
                                line!()
                            ))
                        })?;
                gml_id_children.iter().for_each(|gml_id_node| {
                    let Some(gml_id) = gml_id_node.get_attribute_ns("id", gml_ns) else {
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
                    AttributeValue::String("XLink_InvalidObjectType".to_string()),
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
                "Failed to evaluate xpath at {}:{}: {e:?}",
                file!(),
                line!()
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
                        "Failed to evaluate xpath at {}:{}: {e:?}",
                        file!(),
                        line!()
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
                    // DmGeometricAttribute
                    [
                        "gml:Point",
                        "gml:MultiPoint",
                        "gml:MultiCurve",
                        "gml:MultiSurface",
                    ]
                    .contains(&gml_tag.as_str())
                } else if parent_tag.contains(":DmA") {
                    // DmAnnoation
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
                if let Some(gml_id) = parent.get_attribute_ns("id", gml_ns) {
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

/// Extract the relative path after "codelists/" from a codeSpace path
/// e.g., "../../codelists/Building_class.xml" -> "Building_class.xml"
fn extract_codelists_relative_path(code_space: &str) -> Option<String> {
    const CODELISTS_DIR: &str = "codelists/";
    code_space
        .find(CODELISTS_DIR)
        .map(|idx| code_space[idx + CODELISTS_DIR.len()..].to_string())
}

/// Get the dirCodelists attribute from a feature if it exists
fn get_dir_codelists(feature: &Feature) -> Option<Uri> {
    feature
        .attributes
        .get(&Attribute::new("dirCodelists"))
        .and_then(|v| Uri::from_str(&v.to_string()).ok())
}

/// Try to load a single codelist file dynamically from storage
/// Returns the codelist as a HashMap if successful, or an error if the file cannot be loaded
fn try_load_codelist_file(
    storage_resolver: Arc<StorageResolver>,
    codelist_path: &Uri,
) -> super::errors::Result<HashMap<String, String>> {
    let storage = storage_resolver.resolve(codelist_path).map_err(|e| {
        PlateauProcessorError::DomainOfDefinitionValidator(format!(
            "Failed to resolve codelist path: {e:?}"
        ))
    })?;

    if !storage
        .exists_sync(codelist_path.path().as_path())
        .unwrap_or(false)
    {
        return Err(PlateauProcessorError::DomainOfDefinitionValidator(format!(
            "Codelist file not found: {}",
            codelist_path
        )));
    }

    create_detail_codelist(storage, codelist_path)
}

fn create_codelist(
    storage_resolver: Arc<StorageResolver>,
    first: &Feature,
    fallback_codelists_path: Option<&Uri>,
) -> super::errors::Result<HashMap<String, HashMap<String, String>>> {
    // Try to get dirCodelists from feature, fall back to provided path
    let dir = first
        .attributes
        .get(&Attribute::new("dirCodelists"))
        .and_then(|v| Uri::from_str(&v.to_string()).ok());

    let dir = match dir {
        Some(d) => {
            let storage = storage_resolver.resolve(&d).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
            })?;
            let exist = storage.exists_sync(d.path().as_path()).map_err(|e| {
                PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}"))
            })?;
            if exist {
                d
            } else if let Some(fallback) = fallback_codelists_path {
                fallback.clone()
            } else {
                return Err(PlateauProcessorError::DomainOfDefinitionValidator(
                    "dirCodelists not found and no fallback provided".to_string(),
                ));
            }
        }
        None => {
            if let Some(fallback) = fallback_codelists_path {
                fallback.clone()
            } else {
                return Err(PlateauProcessorError::DomainOfDefinitionValidator(
                    "dirCodelists key empty and no fallback provided".to_string(),
                ));
            }
        }
    };

    let storage = storage_resolver
        .resolve(&dir)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let exist = storage
        .exists_sync(dir.path().as_path())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    if !exist {
        return Err(PlateauProcessorError::DomainOfDefinitionValidator(
            "codelists directory not found".to_string(),
        ));
    }
    let mut codelist = HashMap::new();
    let lists = storage
        .list_sync(Some(dir.path().as_path()), true)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    for file in lists
        .iter()
        .filter(|f| f.is_file() && f.path().extension() == Some("xml".as_ref()))
    {
        let result = create_detail_codelist(Arc::clone(&storage), file)
            .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
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
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let xml_content = String::from_utf8(xml_content.to_vec())
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let xml_document = xml::parse(xml_content)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let ctx = xml::create_context(&xml_document)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let root = xml::get_root_readonly_node(&xml_document)
        .map_err(|e| PlateauProcessorError::DomainOfDefinitionValidator(format!("{e:?}")))?;
    let definitions = xml::find_readonly_nodes_by_xpath(
        &ctx,
        ".//*[namespace-uri()='http://www.opengis.net/gml' and local-name()='Definition']",
        &root,
    )
    .map_err(|e| {
        PlateauProcessorError::DomainOfDefinitionValidator(format!(
            "Failed to evaluate xpath at {}:{}: {e:?}",
            file!(),
            line!()
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
            Some((name.get_content()?, description.get_content()?))
        })
        .collect::<HashMap<String, String>>();
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn handle_code_validation_failure(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    flag: &str,
    base_feature: &Feature,
    code_space_member: &XmlRoNode,
    member: &XmlRoNode,
    code_space: &str,
    code_value: &str,
    result: &mut Vec<Feature>,
) {
    let mut result_feature = base_feature.clone();
    result_feature.insert("flag", AttributeValue::String(flag.to_string()));
    result_feature.insert(
        "tag",
        AttributeValue::String(xml::get_readonly_node_tag(code_space_member)),
    );
    result_feature.insert(
        "xpath",
        AttributeValue::String(get_xpath(code_space_member, Some(member), None)),
    );
    result_feature.insert("codeSpace", AttributeValue::String(code_space.to_string()));
    result_feature.insert(
        "codeSpaceValue",
        AttributeValue::String(code_value.to_string()),
    );
    result.push(result_feature.clone());
    fw.send(ctx.new_with_feature_and_port(result_feature, DEFAULT_PORT.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::{create_default_execute_context, create_default_node_context};
    use indexmap::IndexMap;
    use reearth_flow_runtime::{
        event::EventHub,
        forwarder::{NoopChannelForwarder, ProcessorChannelForwarder},
        node::ProcessorFactory,
    };
    use reearth_flow_types::Feature;
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_gml_file(file_path: &str, gml_id: &str) -> std::io::Result<()> {
        let gml_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:brid="http://www.opengis.net/citygml/bridge/2.0" xmlns:tran="http://www.opengis.net/citygml/transportation/2.0" xmlns:frn="http://www.opengis.net/citygml/cityfurniture/2.0" xmlns:wtr="http://www.opengis.net/citygml/waterbody/2.0" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:veg="http://www.opengis.net/citygml/vegetation/2.0" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:tun="http://www.opengis.net/citygml/tunnel/2.0" xmlns:tex="http://www.opengis.net/citygml/texturedsurface/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:app="http://www.opengis.net/citygml/appearance/2.0" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:dem="http://www.opengis.net/citygml/relief/2.0" xmlns:luse="http://www.opengis.net/citygml/landuse/2.0" xmlns:uro="https://www.geospatial.jp/iur/uro/3.1" xmlns:xAL="urn:oasis:names:tc:ciq:xsdschema:xAL:2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:smil20="http://www.w3.org/2001/SMIL20/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:smil20lang="http://www.w3.org/2001/SMIL20/Language" xmlns:pbase="http://www.opengis.net/citygml/profiles/base/2.0" xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:grp="http://www.opengis.net/citygml/cityobjectgroup/2.0" xsi:schemaLocation="http://www.opengis.net/citygml/2.0 http://schemas.opengis.net/citygml/2.0/cityGMLBase.xsd http://www.opengis.net/citygml/building/2.0 http://schemas.opengis.net/citygml/building/2.0/building.xsd http://www.opengis.net/gml http://schemas.opengis.net/gml/3.1.1/base/gml.xsd">
    <gml:boundedBy>
        <gml:Envelope srsName="http://www.opengis.net/def/crs/EPSG/0/6697" srsDimension="3">
            <gml:lowerCorner>36.6470041354812 137.05268308385453 0</gml:lowerCorner>
            <gml:upperCorner>36.647798243275254 137.0537094956814 105.03314</gml:upperCorner>
        </gml:Envelope>
    </gml:boundedBy>
    <core:cityObjectMember>
        <bldg:Building gml:id="{gml_id}">
            <core:creationDate>2025-03-21</core:creationDate>
            <bldg:class codeSpace="../../codelists/Building_class.xml">3003</bldg:class>
            <bldg:usage codeSpace="../../codelists/Building_usage.xml">411</bldg:usage>
            <bldg:yearOfConstruction>0001</bldg:yearOfConstruction>
            <bldg:measuredHeight uom="m">8.6</bldg:measuredHeight>
            <bldg:storeysAboveGround>9999</bldg:storeysAboveGround>
            <bldg:storeysBelowGround>9999</bldg:storeysBelowGround>
            <bldg:lod0RoofEdge>
                <gml:MultiSurface>
                    <gml:surfaceMember>
                        <gml:Polygon>
                            <gml:exterior>
                                <gml:LinearRing>
                                    <gml:posList>36.64773967207627 137.0537094956814 0 36.647798243275254 137.05370057460766 0 36.64778832538864 137.053600937653 0 36.64772975430324 137.053609970643 0 36.64773967207627 137.0537094956814 0</gml:posList>
                                </gml:LinearRing>
                            </gml:exterior>
                        </gml:Polygon>
                    </gml:surfaceMember>
                </gml:MultiSurface>
            </bldg:lod0RoofEdge>
        </bldg:Building>
    </core:cityObjectMember>
</core:CityModel>"#
        );

        if let Some(parent) = std::path::Path::new(file_path).parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(file_path)?;
        file.write_all(gml_content.as_bytes())?;
        Ok(())
    }

    fn create_test_feature(
        name: &str,
        file_path: &str,
        codelists_dir: &std::path::Path,
    ) -> Feature {
        let mut attributes = IndexMap::new();
        attributes.insert(
            Attribute::new("name"),
            AttributeValue::String(name.to_string()),
        );
        attributes.insert(
            Attribute::new("package"),
            AttributeValue::String("bldg".to_string()),
        );
        attributes.insert(
            Attribute::new("dirCodelists"),
            AttributeValue::String(format!("file://{}", codelists_dir.display())),
        );
        attributes.insert(
            Attribute::new("path"),
            AttributeValue::String(file_path.to_string()),
        );

        Feature::new_with_attributes(attributes)
    }

    // Helper function to extract file_stats outputs from ProcessorChannelForwarder
    fn extract_file_stats_outputs(
        fw: &ProcessorChannelForwarder,
    ) -> Result<HashMap<String, u64>, BoxedError> {
        match fw {
            ProcessorChannelForwarder::Noop(noop_fw) => {
                let send_ports = noop_fw.send_ports.lock().unwrap();
                let send_features = noop_fw.send_features.lock().unwrap();

                let file_stats_outputs: HashMap<String, u64> = send_ports
                    .iter()
                    .enumerate()
                    .filter(|(_, port)| port.as_ref() == "duplicateGmlIdStats")
                    .map(|(i, _)| &send_features[i])
                    .filter_map(|feature| {
                        if let (
                            Some(AttributeValue::String(filename)),
                            Some(AttributeValue::Number(count)),
                        ) = (feature.get("filename"), feature.get("duplicateGmlIdCount"))
                        {
                            Some((filename.clone(), count.as_u64().unwrap_or(0)))
                        } else {
                            None
                        }
                    })
                    .collect();

                Ok(file_stats_outputs)
            }
            ProcessorChannelForwarder::ChannelManager(_) => {
                Err("Expected Noop forwarder for testing".into())
            }
        }
    }

    // Helper function to setup and run processor with given GML file configs
    fn run_processor_test(
        gml_configs: Vec<(&str, &str)>, // (filename, gml_id) pairs
    ) -> Result<HashMap<String, u64>, BoxedError> {
        let temp_dir = TempDir::with_prefix("domain_of_definition_validator_test_")?;

        let codelists_dir = temp_dir.path().join("codelists");
        fs::create_dir_all(&codelists_dir)?;

        // Create GML files and collect features
        let mut features = Vec::new();
        for (filename, gml_id) in gml_configs {
            let file_path = temp_dir.path().join(filename);
            create_gml_file(&file_path.to_string_lossy(), gml_id)?;
            features.push(create_test_feature(
                filename,
                &format!("file://{}", file_path.display()),
                &codelists_dir,
            ));
        }

        // Create processor
        let factory = DomainOfDefinitionValidatorFactory {};
        let ctx = create_default_node_context();
        let mut processor: Box<dyn Processor> =
            factory.build(ctx, EventHub::new(1024), "test".to_string(), None)?;

        // Process features
        let fw = ProcessorChannelForwarder::Noop(NoopChannelForwarder::default());
        for feature in features {
            let ctx = create_default_execute_context(feature);
            processor.process(ctx, &fw)?;
        }

        // Call finish to trigger file_stats output
        let ctx = create_default_node_context();
        processor.finish(ctx, &fw)?;

        // Extract results
        let file_stats_outputs = extract_file_stats_outputs(&fw)?;

        Ok(file_stats_outputs)
    }

    #[test]
    fn test_gml_id_duplicate_detection() -> Result<(), BoxedError> {
        // Create test GML files:
        let gml_id_1 = format!("bldg_{}", uuid::Uuid::new_v4());
        let gml_id_2 = format!("bldg_{}", uuid::Uuid::new_v4());
        let gml_configs = vec![
            ("file1.gml", gml_id_1.as_str()),
            ("file2.gml", gml_id_1.as_str()),
            ("file3.gml", gml_id_2.as_str()),
        ];

        let file_stats_outputs = run_processor_test(gml_configs)?;

        // Assert the expected duplicate counts:
        assert_eq!(file_stats_outputs.get("file1.gml"), Some(&1));
        assert_eq!(file_stats_outputs.get("file2.gml"), Some(&1));
        assert_eq!(file_stats_outputs.get("file3.gml"), Some(&0));
        Ok(())
    }

    #[test]
    fn test_gml_id_no_duplicates() -> Result<(), BoxedError> {
        // Create test GML files with all unique gml:ids:
        let gml_id_1 = format!("bldg_{}", uuid::Uuid::new_v4());
        let gml_id_2 = format!("bldg_{}", uuid::Uuid::new_v4());
        let gml_id_3 = format!("bldg_{}", uuid::Uuid::new_v4());
        let gml_configs = vec![
            ("file1.gml", gml_id_1.as_str()),
            ("file2.gml", gml_id_2.as_str()),
            ("file3.gml", gml_id_3.as_str()),
        ];

        let file_stats_outputs = run_processor_test(gml_configs)?;

        // Assert the expected duplicate counts:
        assert_eq!(file_stats_outputs.get("file1.gml"), Some(&0));
        assert_eq!(file_stats_outputs.get("file2.gml"), Some(&0));
        assert_eq!(file_stats_outputs.get("file3.gml"), Some(&0));
        Ok(())
    }
}
