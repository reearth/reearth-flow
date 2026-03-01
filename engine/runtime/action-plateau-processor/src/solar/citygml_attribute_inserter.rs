use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use bytes::Bytes;
use once_cell::sync::Lazy;
use proj::Proj;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::forwarder::ProcessorChannelForwarder;
use reearth_flow_runtime::node::{Port, Processor, ProcessorFactory, DEFAULT_PORT};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::CityGmlAttributeInserterError;

thread_local! {
    static PROJ_CACHE: RefCell<HashMap<(u32, u32), Proj>> = RefCell::new(HashMap::new());
}

static PATH_PORT: Lazy<Port> = Lazy::new(|| Port::new("path"));
static ELEMENT_PORT: Lazy<Port> = Lazy::new(|| Port::new("element"));
static BOUNDS_PORT: Lazy<Port> = Lazy::new(|| Port::new("textureBounds"));

#[derive(Debug, Clone, Default)]
pub struct CityGmlAttributeInserterFactory;

impl ProcessorFactory for CityGmlAttributeInserterFactory {
    fn name(&self) -> &str {
        "PLATEAU4.SolarCityGmlAttributeInserter"
    }

    fn description(&self) -> &str {
        "Inserts solar radiation measurement attributes into original CityGML files"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CityGmlAttributeInserterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![PATH_PORT.clone(), ELEMENT_PORT.clone(), BOUNDS_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: CityGmlAttributeInserterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                CityGmlAttributeInserterError::Factory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                CityGmlAttributeInserterError::Factory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(CityGmlAttributeInserterError::Factory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let output_dir_ast = expr_engine
            .compile(params.output_dir.as_ref())
            .map_err(|e| {
                CityGmlAttributeInserterError::Factory(format!(
                    "Failed to compile outputDir expression: {e}"
                ))
            })?;

        let texture_image_path_ast = params
            .texture_image_path
            .map(|expr| {
                expr_engine.compile(expr.as_ref()).map_err(|e| {
                    CityGmlAttributeInserterError::Factory(format!(
                        "Failed to compile textureImagePath expression: {e}"
                    ))
                })
            })
            .transpose()?;

        let source_epsg_ast = params
            .source_epsg
            .map(|expr| {
                expr_engine.compile(expr.as_ref()).map_err(|e| {
                    CityGmlAttributeInserterError::Factory(format!(
                        "Failed to compile sourceEpsg expression: {e}"
                    ))
                })
            })
            .transpose()?;

        let process = CityGmlAttributeInserter {
            output_dir_ast,
            gml_id_attribute: params
                .gml_id_attribute
                .unwrap_or_else(|| "gmlId".to_string()),
            path_attribute: params.path_attribute.unwrap_or_else(|| "path".to_string()),
            measurements: params.measurements,
            texture_image_path_ast,
            source_epsg_ast,
            paths: Vec::new(),
            elements: HashMap::new(),
            texture_bounds: None,
        };
        Ok(Box::new(process))
    }
}

/// Configuration for inserting measurement attributes into CityGML files.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct CityGmlAttributeInserterParam {
    /// Output directory expression for modified CityGML files
    output_dir: Expr,
    /// Attribute name on element features holding gml:id (default: "gmlId")
    #[serde(default)]
    gml_id_attribute: Option<String>,
    /// Attribute name on path features holding the file path (default: "path")
    #[serde(default)]
    path_attribute: Option<String>,
    /// Measurement definitions to insert as gen:measureAttribute elements
    measurements: Vec<MeasurementDef>,
    /// Path to the solar radiation texture PNG (texture insertion skipped if absent)
    #[serde(default)]
    texture_image_path: Option<Expr>,
    /// The projected CRS EPSG code used for rasterization (needed for UV computation)
    #[serde(default)]
    source_epsg: Option<Expr>,
}

/// A single measurement attribute definition.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct MeasurementDef {
    /// XML name attribute value (e.g. "年間予測日射量")
    name: String,
    /// Feature attribute key holding the numeric value (e.g. "totalSolarRadiation")
    attribute: String,
    /// Unit of measurement (e.g. "kWh")
    uom: String,
}

/// Collected roof ring metadata for texture UV computation.
#[derive(Debug, Clone)]
struct RoofRingData {
    polygon_id: String,
    ring_id: String,
    /// Vertex coordinates (lat, lon, height) from gml:posList in EPSG:6697
    coords: Vec<(f64, f64, f64)>,
}

#[derive(Debug, Clone)]
struct CityGmlAttributeInserter {
    output_dir_ast: rhai::AST,
    gml_id_attribute: String,
    path_attribute: String,
    measurements: Vec<MeasurementDef>,
    texture_image_path_ast: Option<rhai::AST>,
    source_epsg_ast: Option<rhai::AST>,
    paths: Vec<String>,
    /// gmlId → { attribute_key → value }
    elements: HashMap<String, HashMap<String, f64>>,
    /// Texture bounding box (min_x, min_y, max_x, max_y) in projected CRS,
    /// forwarded from ImageRasterizer via the textureBounds port.
    texture_bounds: Option<(f64, f64, f64, f64)>,
}

impl Processor for CityGmlAttributeInserter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let port = ctx.port.clone();
        if port == *PATH_PORT {
            let path = ctx
                .feature
                .get(&self.path_attribute)
                .and_then(|v| match v {
                    AttributeValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .ok_or_else(|| {
                    CityGmlAttributeInserterError::Process(format!(
                        "Missing '{}' attribute on path feature",
                        self.path_attribute
                    ))
                })?;
            self.paths.push(path);
        } else if port == *ELEMENT_PORT {
            let gml_id = ctx
                .feature
                .get(&self.gml_id_attribute)
                .and_then(|v| match v {
                    AttributeValue::String(s) => Some(s.clone()),
                    _ => None,
                });
            let Some(gml_id) = gml_id else {
                return Ok(());
            };
            let entry = self.elements.entry(gml_id).or_default();
            for m in &self.measurements {
                if let Some(val) = ctx.feature.get(&m.attribute) {
                    let num = match val {
                        AttributeValue::Number(n) => n.as_f64().unwrap_or(0.0),
                        _ => continue,
                    };
                    entry
                        .entry(m.attribute.clone())
                        .and_modify(|existing| *existing += num)
                        .or_insert(num);
                }
            }
        } else if port == *BOUNDS_PORT {
            // Receive texture bounding box from ImageRasterizer (projected CRS)
            let min_x = extract_f64_attr(&ctx.feature, "textureMinX");
            let min_y = extract_f64_attr(&ctx.feature, "textureMinY");
            let max_x = extract_f64_attr(&ctx.feature, "textureMaxX");
            let max_y = extract_f64_attr(&ctx.feature, "textureMaxY");
            if let (Some(min_x), Some(min_y), Some(max_x), Some(max_y)) =
                (min_x, min_y, max_x, max_y)
            {
                self.texture_bounds = Some((min_x, min_y, max_x, max_y));
            }
        } else {
            return Err(
                reearth_flow_runtime::errors::ExecutionError::InvalidPortHandle(port).into(),
            );
        }
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = expr_engine.new_scope();

        let output_dir = scope
            .eval_ast::<String>(&self.output_dir_ast)
            .map_err(|e| {
                CityGmlAttributeInserterError::Process(format!(
                    "Failed to evaluate outputDir expression: {e}"
                ))
            })?;

        // Evaluate optional texture parameters
        let texture_image_path = self
            .texture_image_path_ast
            .as_ref()
            .map(|ast| {
                scope.eval_ast::<String>(ast).map_err(|e| {
                    CityGmlAttributeInserterError::Process(format!(
                        "Failed to evaluate textureImagePath expression: {e}"
                    ))
                })
            })
            .transpose()?;

        let source_epsg: Option<u32> = self
            .source_epsg_ast
            .as_ref()
            .map(|ast| {
                let val = scope.eval_ast::<rhai::Dynamic>(ast).map_err(|e| {
                    CityGmlAttributeInserterError::Process(format!(
                        "Failed to evaluate sourceEpsg expression: {e}"
                    ))
                })?;
                // Accept both integer and string EPSG codes
                if let Ok(n) = val.as_int() {
                    u32::try_from(n).map_err(|_| {
                        CityGmlAttributeInserterError::Process(format!(
                            "sourceEpsg must be a positive integer, got: {n}"
                        ))
                    })
                } else if let Ok(s) = val.into_string() {
                    s.parse::<u32>().map_err(|e| {
                        CityGmlAttributeInserterError::Process(format!(
                            "Failed to parse sourceEpsg '{s}' as u32: {e}"
                        ))
                    })
                } else {
                    Err(CityGmlAttributeInserterError::Process(
                        "sourceEpsg must be an integer or string".to_string(),
                    ))
                }
            })
            .transpose()?;

        let collect_rings = texture_image_path.is_some() && source_epsg.is_some();

        // If texture is enabled, copy the texture PNG to the output directory
        let texture_filename = if let Some(ref tex_path) = texture_image_path {
            let tex_uri = Uri::from_str(tex_path).map_err(|e| {
                CityGmlAttributeInserterError::Process(format!(
                    "Invalid texture URI '{tex_path}': {e}"
                ))
            })?;
            let tex_path_buf = tex_uri.path().as_path().to_path_buf();
            let fname = tex_path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("texture.png")
                .to_string();

            let tex_storage = storage_resolver.resolve(&tex_uri).map_err(|e| {
                CityGmlAttributeInserterError::Process(format!(
                    "Failed to resolve storage for texture '{tex_path}': {e}"
                ))
            })?;
            let tex_bytes = tex_storage
                .get_sync(tex_uri.path().as_path())
                .map_err(|e| {
                    CityGmlAttributeInserterError::Process(format!(
                        "Failed to read texture '{tex_path}': {e}"
                    ))
                })?;

            let out_tex_path = format!("{}/{}", output_dir.trim_end_matches('/'), fname);
            let out_tex_uri = Uri::from_str(&out_tex_path).map_err(|e| {
                CityGmlAttributeInserterError::Process(format!(
                    "Invalid output texture URI '{out_tex_path}': {e}"
                ))
            })?;
            let out_tex_storage = storage_resolver.resolve(&out_tex_uri).map_err(|e| {
                CityGmlAttributeInserterError::Process(format!(
                    "Failed to resolve storage for output texture '{out_tex_path}': {e}"
                ))
            })?;
            out_tex_storage
                .put_sync(out_tex_uri.path().as_path(), tex_bytes)
                .map_err(|e| {
                    CityGmlAttributeInserterError::Process(format!(
                        "Failed to write texture '{out_tex_path}': {e}"
                    ))
                })?;

            // Emit the texture file as a feature so downstream zip writer includes it
            let tex_feature: Feature = indexmap::IndexMap::<Attribute, AttributeValue>::from([(
                Attribute::new("filePath".to_string()),
                AttributeValue::String(out_tex_path),
            )])
            .into();
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                tex_feature,
                DEFAULT_PORT.clone(),
            ));

            Some(fname)
        } else {
            None
        };

        let mut copied_textures: HashSet<String> = HashSet::new();

        for file_path in &self.paths {
            let input_uri = Uri::from_str(file_path).map_err(|e| {
                CityGmlAttributeInserterError::Process(format!(
                    "Invalid input URI '{file_path}': {e}"
                ))
            })?;
            let input_storage = storage_resolver.resolve(&input_uri).map_err(|e| {
                CityGmlAttributeInserterError::Process(format!(
                    "Failed to resolve storage for '{file_path}': {e}"
                ))
            })?;
            let input_bytes = input_storage
                .get_sync(input_uri.path().as_path())
                .map_err(|e| {
                    CityGmlAttributeInserterError::Process(format!(
                        "Failed to read '{file_path}': {e}"
                    ))
                })?;

            let (modified, roof_rings, image_uris) =
                self.insert_attributes(&input_bytes, collect_rings)?;

            // If we have roof rings and texture info, append appearance block
            let final_output = if let (false, Some(epsg), Some(ref tex_fname)) =
                (roof_rings.is_empty(), source_epsg, &texture_filename)
            {
                let appearance_xml =
                    self.build_appearance_xml(&roof_rings, epsg, tex_fname, self.texture_bounds)?;
                self.insert_appearance_block(&modified, &appearance_xml)?
            } else {
                modified
            };

            let path_buf = input_uri.path().as_path().to_path_buf();
            let filename = path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("output.gml");
            let output_path = format!("{}/{}", output_dir.trim_end_matches('/'), filename);
            let output_uri = Uri::from_str(&output_path).map_err(|e| {
                CityGmlAttributeInserterError::Process(format!(
                    "Invalid output URI '{output_path}': {e}"
                ))
            })?;
            let output_storage = storage_resolver.resolve(&output_uri).map_err(|e| {
                CityGmlAttributeInserterError::Process(format!(
                    "Failed to resolve storage for '{output_path}': {e}"
                ))
            })?;
            output_storage
                .put_sync(output_uri.path().as_path(), Bytes::from(final_output))
                .map_err(|e| {
                    CityGmlAttributeInserterError::Process(format!(
                        "Failed to write '{output_path}': {e}"
                    ))
                })?;

            let feature: Feature = indexmap::IndexMap::<Attribute, AttributeValue>::from([(
                Attribute::new("filePath".to_string()),
                AttributeValue::String(output_path),
            )])
            .into();
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                DEFAULT_PORT.clone(),
            ));

            // Copy original texture image files referenced by the GML.
            // The image URIs are relative to the GML's directory; we preserve that
            // relative path under output_dir so the output GML's references remain valid.
            let input_url_str = file_path.as_str();
            let input_dir = if let Some(pos) = input_url_str.rfind('/') {
                &input_url_str[..=pos]
            } else {
                input_url_str
            };

            for uri_str in &image_uris {
                if copied_textures.contains(uri_str) {
                    continue;
                }
                copied_textures.insert(uri_str.clone());

                let source_url = format!("{input_dir}{uri_str}");
                let source_uri = match Uri::from_str(&source_url) {
                    Ok(u) => u,
                    Err(e) => {
                        tracing::warn!("Skipping texture '{}': invalid URI: {e}", uri_str);
                        continue;
                    }
                };
                let src_storage = match storage_resolver.resolve(&source_uri) {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!(
                            "Skipping texture '{}': cannot resolve storage: {e}",
                            uri_str
                        );
                        continue;
                    }
                };
                let tex_bytes = match src_storage.get_sync(source_uri.path().as_path()) {
                    Ok(b) => b,
                    Err(e) => {
                        tracing::warn!("Skipping texture '{}': cannot read file: {e}", uri_str);
                        continue;
                    }
                };

                let out_tex_path = format!("{}/{}", output_dir.trim_end_matches('/'), uri_str);
                let out_tex_uri = match Uri::from_str(&out_tex_path) {
                    Ok(u) => u,
                    Err(e) => {
                        tracing::warn!("Skipping texture '{}': invalid output URI: {e}", uri_str);
                        continue;
                    }
                };
                let out_tex_storage = match storage_resolver.resolve(&out_tex_uri) {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!(
                            "Skipping texture '{}': cannot resolve output storage: {e}",
                            uri_str
                        );
                        continue;
                    }
                };
                if let Err(e) = out_tex_storage.put_sync(out_tex_uri.path().as_path(), tex_bytes) {
                    tracing::warn!("Skipping texture '{}': write failed: {e}", uri_str);
                    continue;
                }

                let tex_feature: Feature =
                    indexmap::IndexMap::<Attribute, AttributeValue>::from([(
                        Attribute::new("filePath".to_string()),
                        AttributeValue::String(out_tex_path),
                    )])
                    .into();
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    tex_feature,
                    DEFAULT_PORT.clone(),
                ));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "PLATEAU4.SolarCityGmlAttributeInserter"
    }

    fn is_accumulating(&self) -> bool {
        true
    }
}

impl CityGmlAttributeInserter {
    /// Stream through CityGML XML, inserting measure attributes and collecting roof ring data.
    /// Returns (modified_xml_bytes, collected_roof_rings, original_image_uris).
    #[allow(clippy::type_complexity)]
    fn insert_attributes(
        &self,
        input: &[u8],
        collect_rings: bool,
    ) -> Result<(Vec<u8>, Vec<RoofRingData>, Vec<String>), BoxedError> {
        let mut reader = Reader::from_reader(input);
        reader.config_mut().trim_text(false);

        let mut output = Vec::with_capacity(input.len() + 4096);
        let mut writer = Writer::new(&mut output);

        let mut depth: usize = 0;
        let mut building_depth: Option<usize> = None;
        let mut current_gml_id: Option<String> = None;
        let mut buf = Vec::new();

        // Roof ring collection state
        let mut roof_rings: Vec<RoofRingData> = Vec::new();
        let mut in_roof_surface = false;
        let mut roof_surface_depth: Option<usize> = None;
        let mut current_polygon_id: Option<String> = None;
        let mut current_ring_id: Option<String> = None;
        let mut in_pos_list = false;
        let mut pos_list_text = String::new();

        // Original texture image URI collection
        let mut image_uris: Vec<String> = Vec::new();
        let mut in_image_uri = false;
        let mut image_uri_text = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,
                Ok(Event::Start(ref e)) => {
                    depth += 1;
                    let name = e.name();
                    let local_name = name.as_ref();
                    let gml_id = extract_gml_id(e);

                    if is_building_tag(e) {
                        building_depth = Some(depth);
                        current_gml_id = gml_id.clone();
                    } else if gml_id.is_some()
                        && building_depth.is_some()
                        && current_gml_id.is_none()
                    {
                        current_gml_id.clone_from(&gml_id);
                    }

                    // Track app:imageURI for original texture file collection
                    if is_image_uri_tag(local_name) {
                        in_image_uri = true;
                        image_uri_text.clear();
                    }

                    // Track RoofSurface context for ring collection
                    if collect_rings {
                        if is_roof_surface_tag(local_name) && building_depth.is_some() {
                            in_roof_surface = true;
                            roof_surface_depth = Some(depth);
                        } else if in_roof_surface {
                            if is_polygon_tag(local_name) {
                                if let Some(id) = &gml_id {
                                    current_polygon_id = Some(id.clone());
                                }
                            } else if is_linear_ring_tag(local_name) {
                                if let Some(id) = &gml_id {
                                    current_ring_id = Some(id.clone());
                                }
                            } else if is_pos_list_tag(local_name) {
                                in_pos_list = true;
                                pos_list_text.clear();
                            }
                        }
                    }

                    writer.write_event(Event::Start(e.clone())).map_err(|e| {
                        CityGmlAttributeInserterError::Process(format!("XML write error: {e}"))
                    })?;
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let local_name = name.as_ref();

                    // Collect original texture image URIs
                    if is_image_uri_tag(local_name) && in_image_uri {
                        in_image_uri = false;
                        let uri_str = image_uri_text.trim().to_string();
                        if !uri_str.is_empty() && !uri_str.contains("://") {
                            image_uris.push(uri_str);
                        }
                        image_uri_text.clear();
                    }

                    // Collect ring data when closing LinearRing
                    if collect_rings && in_roof_surface {
                        if is_pos_list_tag(local_name) && in_pos_list {
                            in_pos_list = false;
                            if let (Some(poly_id), Some(ring_id)) =
                                (&current_polygon_id, &current_ring_id)
                            {
                                let coords = parse_pos_list(&pos_list_text);
                                if !coords.is_empty() {
                                    roof_rings.push(RoofRingData {
                                        polygon_id: poly_id.clone(),
                                        ring_id: ring_id.clone(),
                                        coords,
                                    });
                                }
                            }
                            pos_list_text.clear();
                        } else if is_linear_ring_tag(local_name) {
                            current_ring_id = None;
                        } else if is_polygon_tag(local_name) {
                            current_polygon_id = None;
                        } else if roof_surface_depth == Some(depth) {
                            in_roof_surface = false;
                            roof_surface_depth = None;
                        }
                    }

                    if building_depth == Some(depth) {
                        if let Some(ref gml_id) = current_gml_id {
                            if let Some(values) = self.elements.get(gml_id) {
                                self.write_measure_attributes(&mut writer, values)?;
                            }
                        }
                        building_depth = None;
                        current_gml_id = None;
                    }

                    depth = depth.checked_sub(1).ok_or_else(|| {
                        CityGmlAttributeInserterError::Process(
                            "Malformed XML: more closing tags than opening tags".to_string(),
                        )
                    })?;
                    writer.write_event(Event::End(e.clone())).map_err(|e| {
                        CityGmlAttributeInserterError::Process(format!("XML write error: {e}"))
                    })?;
                }
                Ok(Event::Text(ref e)) => {
                    if in_image_uri {
                        if let Ok(text) = e.unescape() {
                            image_uri_text.push_str(&text);
                        }
                    }
                    if collect_rings && in_pos_list {
                        if let Ok(text) = e.unescape() {
                            pos_list_text.push_str(&text);
                        }
                    }
                    writer.write_event(Event::Text(e.clone())).map_err(|e| {
                        CityGmlAttributeInserterError::Process(format!("XML write error: {e}"))
                    })?;
                }
                Ok(Event::Empty(ref e)) => {
                    writer.write_event(Event::Empty(e.clone())).map_err(|e| {
                        CityGmlAttributeInserterError::Process(format!("XML write error: {e}"))
                    })?;
                }
                Ok(event) => {
                    writer.write_event(event).map_err(|e| {
                        CityGmlAttributeInserterError::Process(format!("XML write error: {e}"))
                    })?;
                }
                Err(e) => {
                    return Err(CityGmlAttributeInserterError::Process(format!(
                        "XML read error: {e}"
                    ))
                    .into());
                }
            }
            buf.clear();
        }

        Ok((output, roof_rings, image_uris))
    }

    fn write_measure_attributes(
        &self,
        writer: &mut Writer<&mut Vec<u8>>,
        values: &HashMap<String, f64>,
    ) -> Result<(), BoxedError> {
        for m in &self.measurements {
            let Some(&val) = values.get(&m.attribute) else {
                continue;
            };
            // <gen:measureAttribute name="...">
            let mut attr_start = quick_xml::events::BytesStart::new("gen:measureAttribute");
            attr_start.push_attribute(("name", m.name.as_str()));
            writer.write_event(Event::Start(attr_start)).map_err(|e| {
                CityGmlAttributeInserterError::Process(format!("XML write error: {e}"))
            })?;

            // <gen:value uom="...">VALUE</gen:value>
            let mut value_start = quick_xml::events::BytesStart::new("gen:value");
            value_start.push_attribute(("uom", m.uom.as_str()));
            writer.write_event(Event::Start(value_start)).map_err(|e| {
                CityGmlAttributeInserterError::Process(format!("XML write error: {e}"))
            })?;
            writer
                .write_event(Event::Text(quick_xml::events::BytesText::new(&format!(
                    "{:.2}",
                    val
                ))))
                .map_err(|e| {
                    CityGmlAttributeInserterError::Process(format!("XML write error: {e}"))
                })?;
            writer
                .write_event(Event::End(quick_xml::events::BytesEnd::new("gen:value")))
                .map_err(|e| {
                    CityGmlAttributeInserterError::Process(format!("XML write error: {e}"))
                })?;

            // </gen:measureAttribute>
            writer
                .write_event(Event::End(quick_xml::events::BytesEnd::new(
                    "gen:measureAttribute",
                )))
                .map_err(|e| {
                    CityGmlAttributeInserterError::Process(format!("XML write error: {e}"))
                })?;
        }
        Ok(())
    }

    /// Reproject ring coordinates from EPSG:6697 to the projected CRS, compute UV coordinates,
    /// and generate the appearance XML block to insert before </core:CityModel>.
    fn build_appearance_xml(
        &self,
        roof_rings: &[RoofRingData],
        source_epsg: u32,
        texture_filename: &str,
        external_bounds: Option<(f64, f64, f64, f64)>,
    ) -> Result<String, BoxedError> {
        // Reproject all coordinates from EPSG:6697 (JGD2011 geographic) → projected CRS
        let geographic_epsg = 6697u32;
        get_or_create_proj(geographic_epsg, source_epsg)?;

        let mut reprojected_rings: Vec<(usize, Vec<(f64, f64)>)> = Vec::new();
        let mut computed_min_x = f64::MAX;
        let mut computed_min_y = f64::MAX;
        let mut computed_max_x = f64::MIN;
        let mut computed_max_y = f64::MIN;

        for (i, ring) in roof_rings.iter().enumerate() {
            let mut projected_coords = Vec::with_capacity(ring.coords.len());
            for &(lat, lon, _height) in &ring.coords {
                let (x, y) = reproject_coords(lat, lon, geographic_epsg, source_epsg)?;
                // Only track bounds if no external bounds were provided
                if external_bounds.is_none() {
                    if x < computed_min_x {
                        computed_min_x = x;
                    }
                    if x > computed_max_x {
                        computed_max_x = x;
                    }
                    if y < computed_min_y {
                        computed_min_y = y;
                    }
                    if y > computed_max_y {
                        computed_max_y = y;
                    }
                }
                projected_coords.push((x, y));
            }
            reprojected_rings.push((i, projected_coords));
        }

        // Use externally-provided bounds (from ImageRasterizer) when available, so that UV
        // coordinates are computed with the same reference frame as the rasterized texture.
        let (min_x, min_y, max_x, max_y) = external_bounds.unwrap_or((
            computed_min_x,
            computed_min_y,
            computed_max_x,
            computed_max_y,
        ));
        let width = max_x - min_x;
        let height = max_y - min_y;

        // Build the XML string
        let mut xml = String::new();
        xml.push_str("<app:appearanceMember>");
        xml.push_str("<app:Appearance>");
        xml.push_str("<app:theme>solarRadiation</app:theme>");
        xml.push_str("<app:surfaceDataMember>");
        xml.push_str("<app:ParameterizedTexture>");
        xml.push_str("<app:imageURI>");
        xml.push_str(texture_filename);
        xml.push_str("</app:imageURI>");
        xml.push_str("<app:mimeType>image/png</app:mimeType>");

        for (i, projected_coords) in &reprojected_rings {
            let ring = &roof_rings[*i];

            // Compute raw (unclamped) UV values to determine ring coverage.
            let raw_uvs: Vec<(f64, f64)> = projected_coords
                .iter()
                .map(|&(x, y)| {
                    let u = if width > 0.0 {
                        (x - min_x) / width
                    } else {
                        0.5
                    };
                    let v = if height > 0.0 {
                        (y - min_y) / height
                    } else {
                        0.5
                    };
                    (u, v)
                })
                .collect();

            // Use centroid-based check: a ring whose centroid UV falls within [0, 1]
            // is considered inside the solar texture area (even if a few edge vertices
            // stray slightly outside due to radiation-cell discretization).  Only rings
            // whose centroid is clearly outside the radiation bounds are skipped so that
            // they fall back to the original photo texture in entity_to_geometry.
            let n = raw_uvs.len() as f64;
            let (centroid_u, centroid_v) = if n > 0.0 {
                let sum = raw_uvs
                    .iter()
                    .fold((0.0f64, 0.0f64), |(su, sv), &(u, v)| (su + u, sv + v));
                (sum.0 / n, sum.1 / n)
            } else {
                (0.5, 0.5)
            };
            if !(0.0..=1.0).contains(&centroid_u) || !(0.0..=1.0).contains(&centroid_v) {
                // Ring centroid is outside the texture area; leave it out of the solar
                // appearance so it falls back to the original photo texture.
                continue;
            }

            // Clamp UV values to [0, 1] for rings that are mostly within coverage but
            // have a few edge vertices that stray slightly outside the radiation bounds.
            let clamped_uvs: Vec<(f64, f64)> = raw_uvs
                .iter()
                .map(|&(u, v)| (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0)))
                .collect();

            xml.push_str("<app:target uri=\"#");
            xml.push_str(&ring.polygon_id);
            xml.push_str("\">");
            xml.push_str("<app:TexCoordList>");
            xml.push_str("<app:textureCoordinates ring=\"#");
            xml.push_str(&ring.ring_id);
            xml.push_str("\">");

            let mut uv_strs: Vec<String> = clamped_uvs
                .iter()
                .map(|&(u, v)| format!("{:.6} {:.6}", u, v))
                .collect();
            // nusamai's parser requires the last UV pair to equal the first (ring closure).
            // Ensure this invariant even if the source GML ring was not closed.
            if uv_strs.len() >= 2 && uv_strs.last() != uv_strs.first() {
                let first = uv_strs[0].clone();
                uv_strs.push(first);
            }
            xml.push_str(&uv_strs.join(" "));

            xml.push_str("</app:textureCoordinates>");
            xml.push_str("</app:TexCoordList>");
            xml.push_str("</app:target>");
        }

        xml.push_str("</app:ParameterizedTexture>");
        xml.push_str("</app:surfaceDataMember>");
        xml.push_str("</app:Appearance>");
        xml.push_str("</app:appearanceMember>");

        Ok(xml)
    }

    /// Insert appearance XML block before the closing </core:CityModel> tag.
    fn insert_appearance_block(
        &self,
        xml_bytes: &[u8],
        appearance_xml: &str,
    ) -> Result<Vec<u8>, BoxedError> {
        let xml_str = std::str::from_utf8(xml_bytes).map_err(|e| {
            CityGmlAttributeInserterError::Process(format!("Invalid UTF-8 in XML: {e}"))
        })?;

        // Find the last </core:CityModel> closing tag
        let close_tag = "</core:CityModel>";
        let insert_pos = xml_str.rfind(close_tag).ok_or_else(|| {
            CityGmlAttributeInserterError::Process(
                "Could not find closing </core:CityModel> tag".to_string(),
            )
        })?;

        let mut result = Vec::with_capacity(xml_bytes.len() + appearance_xml.len());
        result.extend_from_slice(&xml_bytes[..insert_pos]);
        result.extend_from_slice(appearance_xml.as_bytes());
        result.extend_from_slice(&xml_bytes[insert_pos..]);
        Ok(result)
    }
}

fn is_building_tag(e: &quick_xml::events::BytesStart<'_>) -> bool {
    let name = e.name();
    let local = name.as_ref();
    // Match bldg:Building (with or without namespace prefix)
    local == b"bldg:Building" || local.ends_with(b":Building") || local == b"Building"
}

fn extract_gml_id(e: &quick_xml::events::BytesStart<'_>) -> Option<String> {
    for attr in e.attributes().flatten() {
        let key = attr.key.as_ref();
        if key == b"gml:id" || key == b"id" {
            return attr.unescape_value().ok().map(|v| v.to_string());
        }
    }
    None
}

fn is_roof_surface_tag(name: &[u8]) -> bool {
    name == b"bldg:RoofSurface" || name.ends_with(b":RoofSurface") || name == b"RoofSurface"
}

fn is_polygon_tag(name: &[u8]) -> bool {
    name == b"gml:Polygon" || name.ends_with(b":Polygon") || name == b"Polygon"
}

fn is_linear_ring_tag(name: &[u8]) -> bool {
    name == b"gml:LinearRing" || name.ends_with(b":LinearRing") || name == b"LinearRing"
}

fn is_image_uri_tag(name: &[u8]) -> bool {
    name == b"app:imageURI" || name.ends_with(b":imageURI") || name == b"imageURI"
}

fn is_pos_list_tag(name: &[u8]) -> bool {
    name == b"gml:posList" || name.ends_with(b":posList") || name == b"posList"
}

/// Parse a gml:posList string into a list of (lat, lon, height) triples.
/// CityGML posList is a flat list of coordinates: lat lon height lat lon height ...
fn parse_pos_list(text: &str) -> Vec<(f64, f64, f64)> {
    let values: Vec<f64> = text
        .split_whitespace()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();
    values.chunks_exact(3).map(|c| (c[0], c[1], c[2])).collect()
}

/// Get or create a cached Proj transformation.
fn get_or_create_proj(
    source_epsg: u32,
    target_epsg: u32,
) -> Result<(), CityGmlAttributeInserterError> {
    use std::collections::hash_map::Entry;
    PROJ_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let key = (source_epsg, target_epsg);
        if let Entry::Vacant(e) = cache.entry(key) {
            let from_crs = format!("EPSG:{}", source_epsg);
            let to_crs = format!("EPSG:{}", target_epsg);
            let proj = Proj::new_known_crs(&from_crs, &to_crs, None).map_err(|e| {
                CityGmlAttributeInserterError::Process(format!(
                    "Failed to create projection from {from_crs} to {to_crs}: {e}"
                ))
            })?;
            e.insert(proj);
        }
        Ok(())
    })
}

/// Reproject a single coordinate using the cached Proj.
fn reproject_coords(
    lat: f64,
    lon: f64,
    source_epsg: u32,
    target_epsg: u32,
) -> Result<(f64, f64), CityGmlAttributeInserterError> {
    PROJ_CACHE.with(|cache| {
        let cache = cache.borrow();
        let key = (source_epsg, target_epsg);
        let proj = cache.get(&key).ok_or_else(|| {
            CityGmlAttributeInserterError::Process(
                "Proj not found in cache - call get_or_create_proj first".to_string(),
            )
        })?;
        // For geographic CRS (EPSG:6697), proj expects (lon, lat) order
        let (x, y) = proj.convert((lon, lat)).map_err(|e| {
            CityGmlAttributeInserterError::Process(format!("Coordinate reprojection failed: {e}"))
        })?;
        Ok((x, y))
    })
}

/// Extract an f64 value from a feature attribute (accepts Number type only).
fn extract_f64_attr(feature: &Feature, key: &str) -> Option<f64> {
    match feature.get(key)? {
        AttributeValue::Number(n) => n.as_f64(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_attributes_basic() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:gen="http://www.opengis.net/citygml/generics/2.0">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg_001">
      <bldg:measuredHeight>10.0</bldg:measuredHeight>
    </bldg:Building>
  </core:cityObjectMember>
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg_002">
      <bldg:measuredHeight>20.0</bldg:measuredHeight>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>"#;

        let mut elements = HashMap::new();
        let mut vals = HashMap::new();
        vals.insert("totalSolarRadiation".to_string(), 1234.56);
        vals.insert("totalSolarPower".to_string(), 789.01);
        elements.insert("bldg_001".to_string(), vals);

        let inserter = CityGmlAttributeInserter {
            output_dir_ast: rhai::AST::empty(),
            gml_id_attribute: "gmlId".to_string(),
            path_attribute: "path".to_string(),
            measurements: vec![
                MeasurementDef {
                    name: "年間予測日射量".to_string(),
                    attribute: "totalSolarRadiation".to_string(),
                    uom: "kWh".to_string(),
                },
                MeasurementDef {
                    name: "年間予測発電量".to_string(),
                    attribute: "totalSolarPower".to_string(),
                    uom: "kWh".to_string(),
                },
            ],
            texture_image_path_ast: None,
            source_epsg_ast: None,
            paths: Vec::new(),
            elements,
            texture_bounds: None,
        };

        let (result, _rings, _uris) = inserter.insert_attributes(xml.as_bytes(), false).unwrap();
        let output = String::from_utf8(result).unwrap();

        // bldg_001 should have measure attributes inserted
        assert!(
            output.contains("gen:measureAttribute"),
            "Output should contain gen:measureAttribute"
        );
        assert!(
            output.contains("年間予測日射量"),
            "Output should contain 年間予測日射量"
        );
        assert!(
            output.contains("1234.56"),
            "Output should contain radiation value"
        );
        assert!(
            output.contains("年間予測発電量"),
            "Output should contain 年間予測発電量"
        );
        assert!(
            output.contains("789.01"),
            "Output should contain power value"
        );

        // bldg_002 should NOT have measure attributes
        let bldg_002_pos = output.find("bldg_002").unwrap();
        let after_bldg_002 = &output[bldg_002_pos..];
        let end_building_pos = after_bldg_002.find("</bldg:Building>").unwrap();
        let bldg_002_content = &after_bldg_002[..end_building_pos];
        assert!(
            !bldg_002_content.contains("gen:measureAttribute"),
            "bldg_002 should not have measure attributes"
        );

        // Original structure should be preserved
        assert!(
            output.contains("bldg:measuredHeight"),
            "Original elements should be preserved"
        );
    }

    #[test]
    fn test_insert_attributes_no_matching_building() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:gen="http://www.opengis.net/citygml/generics/2.0">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg_999">
      <bldg:measuredHeight>10.0</bldg:measuredHeight>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>"#;

        let inserter = CityGmlAttributeInserter {
            output_dir_ast: rhai::AST::empty(),
            gml_id_attribute: "gmlId".to_string(),
            path_attribute: "path".to_string(),
            measurements: vec![MeasurementDef {
                name: "test".to_string(),
                attribute: "testAttr".to_string(),
                uom: "kWh".to_string(),
            }],
            texture_image_path_ast: None,
            source_epsg_ast: None,
            paths: Vec::new(),
            elements: HashMap::new(),
            texture_bounds: None,
        };

        let (result, _rings, _uris) = inserter.insert_attributes(xml.as_bytes(), false).unwrap();
        let output = String::from_utf8(result).unwrap();

        assert!(
            !output.contains("gen:measureAttribute"),
            "No measure attributes should be inserted when no matching gml:id"
        );
        assert!(
            output.contains("bldg:measuredHeight"),
            "Original content should be preserved"
        );
    }

    #[test]
    fn test_collect_roof_rings() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:gml="http://www.opengis.net/gml" xmlns:gen="http://www.opengis.net/citygml/generics/2.0" xmlns:app="http://www.opengis.net/citygml/appearance/2.0">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg_001">
      <bldg:boundedBy>
        <bldg:RoofSurface gml:id="roof_001">
          <bldg:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_001">
                  <gml:exterior>
                    <gml:LinearRing gml:id="ring_001">
                      <gml:posList>35.6 139.7 10.0 35.6 139.8 10.0 35.7 139.8 10.0 35.7 139.7 10.0 35.6 139.7 10.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </bldg:lod2MultiSurface>
        </bldg:RoofSurface>
      </bldg:boundedBy>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>"#;

        let inserter = CityGmlAttributeInserter {
            output_dir_ast: rhai::AST::empty(),
            gml_id_attribute: "gmlId".to_string(),
            path_attribute: "path".to_string(),
            measurements: vec![],
            texture_image_path_ast: None,
            source_epsg_ast: None,
            paths: Vec::new(),
            elements: HashMap::new(),
            texture_bounds: None,
        };

        let (_output, rings, _uris) = inserter.insert_attributes(xml.as_bytes(), true).unwrap();

        assert_eq!(rings.len(), 1, "Should collect one roof ring");
        assert_eq!(rings[0].polygon_id, "poly_001");
        assert_eq!(rings[0].ring_id, "ring_001");
        assert_eq!(rings[0].coords.len(), 5, "Should have 5 coordinate triples");
        assert!((rings[0].coords[0].0 - 35.6).abs() < 1e-10);
        assert!((rings[0].coords[0].1 - 139.7).abs() < 1e-10);
        assert!((rings[0].coords[0].2 - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_parse_pos_list() {
        let text = "35.6 139.7 10.0 35.6 139.8 10.0";
        let coords = parse_pos_list(text);
        assert_eq!(coords.len(), 2);
        assert!((coords[0].0 - 35.6).abs() < 1e-10);
        assert!((coords[1].1 - 139.8).abs() < 1e-10);

        // Partial coordinates (not a multiple of 3) should be ignored
        let text2 = "35.6 139.7";
        let coords2 = parse_pos_list(text2);
        assert!(coords2.is_empty());
    }

    #[test]
    fn test_insert_appearance_block() {
        let xml =
            b"<?xml version=\"1.0\"?><core:CityModel><core:cityObjectMember/></core:CityModel>";
        let appearance = "<app:appearanceMember><app:Appearance/></app:appearanceMember>";

        let inserter = CityGmlAttributeInserter {
            output_dir_ast: rhai::AST::empty(),
            gml_id_attribute: "gmlId".to_string(),
            path_attribute: "path".to_string(),
            measurements: vec![],
            texture_image_path_ast: None,
            source_epsg_ast: None,
            paths: Vec::new(),
            elements: HashMap::new(),
            texture_bounds: None,
        };

        let result = inserter.insert_appearance_block(xml, appearance).unwrap();
        let output = String::from_utf8(result).unwrap();

        assert!(output.contains("app:appearanceMember"));
        assert!(output.ends_with("</core:CityModel>"));
        // Appearance should be before the closing tag
        let app_pos = output.find("app:appearanceMember").unwrap();
        let close_pos = output.rfind("</core:CityModel>").unwrap();
        assert!(app_pos < close_pos);
    }

    #[test]
    fn test_no_ring_collection_when_disabled() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/2.0" xmlns:bldg="http://www.opengis.net/citygml/building/2.0" xmlns:gml="http://www.opengis.net/gml">
  <core:cityObjectMember>
    <bldg:Building gml:id="bldg_001">
      <bldg:boundedBy>
        <bldg:RoofSurface gml:id="roof_001">
          <bldg:lod2MultiSurface>
            <gml:MultiSurface>
              <gml:surfaceMember>
                <gml:Polygon gml:id="poly_001">
                  <gml:exterior>
                    <gml:LinearRing gml:id="ring_001">
                      <gml:posList>35.6 139.7 10.0 35.6 139.8 10.0 35.7 139.8 10.0</gml:posList>
                    </gml:LinearRing>
                  </gml:exterior>
                </gml:Polygon>
              </gml:surfaceMember>
            </gml:MultiSurface>
          </bldg:lod2MultiSurface>
        </bldg:RoofSurface>
      </bldg:boundedBy>
    </bldg:Building>
  </core:cityObjectMember>
</core:CityModel>"#;

        let inserter = CityGmlAttributeInserter {
            output_dir_ast: rhai::AST::empty(),
            gml_id_attribute: "gmlId".to_string(),
            path_attribute: "path".to_string(),
            measurements: vec![],
            texture_image_path_ast: None,
            source_epsg_ast: None,
            paths: Vec::new(),
            elements: HashMap::new(),
            texture_bounds: None,
        };

        let (_output, rings, _uris) = inserter.insert_attributes(xml.as_bytes(), false).unwrap();
        assert!(rings.is_empty(), "Should not collect rings when disabled");
    }
}
