use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use proj::Proj;
use reearth_flow_geometry::algorithm::centroid::Centroid;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{AttributeValue, Expr, Feature, GeometryValue};
use rhai::AST;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;

use super::calculator::calculate_solar_position;
use super::errors::SolarPositionError;

/// 3D centroid coordinates in source CRS
#[derive(Debug, Clone, Copy)]
struct Centroid3D {
    x: f64,
    y: f64,
    z: f64,
}

/// Target CRS for solar position calculation (JGD2011 geographic)
const TARGET_EPSG: u32 = 6697;

#[derive(Debug, Clone, Default)]
pub struct SolarPositionCalculatorFactory;

impl ProcessorFactory for SolarPositionCalculatorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.SolarPositionCalculator"
    }

    fn description(&self) -> &str {
        "Calculates solar position (altitude and azimuth) for geographic features using Spencer's algorithm"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(SolarPositionCalculatorParam))
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
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: SolarPositionCalculatorParam = if let Some(with) = with.as_ref() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SolarPositionError::Factory(format!("Failed to serialize parameters: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SolarPositionError::Factory(format!("Failed to deserialize parameters: {e}"))
            })?
        } else {
            return Err(
                SolarPositionError::Factory("Missing required parameters".to_string()).into(),
            );
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);

        // Compile source_epsg expression (required)
        let source_epsg_ast = expr_engine
            .compile(params.source_epsg().as_ref())
            .map_err(|e| {
                SolarPositionError::Factory(format!(
                    "Failed to compile source_epsg expression: {e}"
                ))
            })?;

        // Compile standard_meridian expression (optional)
        let standard_meridian_ast = if let Some(ref expr) = params.standard_meridian() {
            Some(expr_engine.compile(expr.as_ref()).map_err(|e| {
                SolarPositionError::Factory(format!(
                    "Failed to compile standard_meridian expression: {e}"
                ))
            })?)
        } else {
            None
        };

        let compiled_params = match &params {
            SolarPositionCalculatorParam::Time { time, .. } => {
                let time_ast = expr_engine.compile(time.as_ref()).map_err(|e| {
                    SolarPositionError::Factory(format!("Failed to compile time expression: {e}"))
                })?;
                CompiledParams::Time { time_ast }
            }
            SolarPositionCalculatorParam::Duration {
                start,
                end,
                step,
                step_unit,
                ..
            } => {
                let start_ast = expr_engine.compile(start.as_ref()).map_err(|e| {
                    SolarPositionError::Factory(format!("Failed to compile start expression: {e}"))
                })?;
                let end_ast = expr_engine.compile(end.as_ref()).map_err(|e| {
                    SolarPositionError::Factory(format!("Failed to compile end expression: {e}"))
                })?;
                let step_ast = expr_engine.compile(step.as_ref()).map_err(|e| {
                    SolarPositionError::Factory(format!("Failed to compile step expression: {e}"))
                })?;
                CompiledParams::Duration {
                    start_ast,
                    end_ast,
                    step_ast,
                    step_unit: step_unit.clone(),
                }
            }
        };

        Ok(Box::new(SolarPositionCalculator {
            global_params: with,
            compiled_params,
            source_epsg_ast,
            standard_meridian_ast,
            output_type: params.output_type(),
            source_epsg: u32::MAX, // Could be any number as long as it's invalid as prcs
            proj: None,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SolarPositionCalculatorParam {
    #[serde(rename_all = "camelCase")]
    Time {
        /// Time expression evaluating to "yyyy/mm/dd/hh/mm/ss" format (local time)
        time: Expr,
        /// Source EPSG code expression (required). Evaluates to int (e.g., 6677 for Japan Plane IX).
        source_epsg: Expr,
        /// Standard meridian in degrees (optional). If not provided, computed as round(longitude / 15) * 15.
        #[serde(default)]
        standard_meridian: Option<Expr>,
        /// Output type: unit normal vector or altitude/azimuth angles
        #[serde(default)]
        output_type: OutputType,
    },
    #[serde(rename_all = "camelCase")]
    Duration {
        /// Start time expression evaluating to "yyyy/mm/dd/hh/mm/ss" format (local time)
        start: Expr,
        /// End time expression evaluating to "yyyy/mm/dd/hh/mm/ss" format (local time)
        end: Expr,
        /// Step value expression evaluating to an integer
        step: Expr,
        /// Unit for the step value
        step_unit: StepUnit,
        /// Source EPSG code expression (required). Evaluates to int (e.g., 6677 for Japan Plane IX).
        source_epsg: Expr,
        /// Standard meridian in degrees (optional). If not provided, computed as round(longitude / 15) * 15.
        #[serde(default)]
        standard_meridian: Option<Expr>,
        /// Output type: unit normal vector or altitude/azimuth angles
        #[serde(default)]
        output_type: OutputType,
    },
}

impl SolarPositionCalculatorParam {
    fn source_epsg(&self) -> &Expr {
        match self {
            SolarPositionCalculatorParam::Time { source_epsg, .. } => source_epsg,
            SolarPositionCalculatorParam::Duration { source_epsg, .. } => source_epsg,
        }
    }

    fn standard_meridian(&self) -> Option<&Expr> {
        match self {
            SolarPositionCalculatorParam::Time {
                standard_meridian, ..
            } => standard_meridian.as_ref(),
            SolarPositionCalculatorParam::Duration {
                standard_meridian, ..
            } => standard_meridian.as_ref(),
        }
    }

    fn output_type(&self) -> OutputType {
        match self {
            SolarPositionCalculatorParam::Time { output_type, .. } => output_type.clone(),
            SolarPositionCalculatorParam::Duration { output_type, .. } => output_type.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StepUnit {
    Second,
    Minute,
    Hour,
    Day,
}

/// Output type for solar position calculation
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum OutputType {
    /// Output as unit normal vector (x, y, z) in ENU coordinate system
    #[default]
    UnitNormalVector,
    /// Output as altitude and azimuth angles in degrees
    AltitudeAndAzimuth,
    /// Output both unit normal vector and altitude/azimuth angles
    Both,
}

impl StepUnit {
    fn to_duration(&self, value: i64) -> Duration {
        match self {
            StepUnit::Second => Duration::seconds(value),
            StepUnit::Minute => Duration::minutes(value),
            StepUnit::Hour => Duration::hours(value),
            StepUnit::Day => Duration::days(value),
        }
    }
}

#[derive(Debug, Clone)]
enum CompiledParams {
    Time {
        time_ast: AST,
    },
    Duration {
        start_ast: AST,
        end_ast: AST,
        step_ast: AST,
        step_unit: StepUnit,
    },
}

#[derive(Debug)]
pub struct SolarPositionCalculator {
    global_params: Option<HashMap<String, Value>>,
    compiled_params: CompiledParams,
    source_epsg_ast: AST,
    standard_meridian_ast: Option<AST>,
    output_type: OutputType,
    source_epsg: u32,
    proj: Option<Mutex<proj::Proj>>,
}

impl Clone for SolarPositionCalculator {
    fn clone(&self) -> Self {
        Self {
            global_params: self.global_params.clone(),
            compiled_params: self.compiled_params.clone(),
            source_epsg_ast: self.source_epsg_ast.clone(),
            standard_meridian_ast: self.standard_meridian_ast.clone(),
            output_type: self.output_type.clone(),
            source_epsg: self.source_epsg,
            proj: None,
        }
    }
}

unsafe impl Send for SolarPositionCalculator {}
unsafe impl Sync for SolarPositionCalculator {}

impl Processor for SolarPositionCalculator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        // Evaluate source EPSG code
        let source_epsg = self.evaluate_epsg_expr(feature, &ctx, &self.source_epsg_ast.clone())?;

        // Extract 3D centroid in source CRS coordinates
        let centroid_3d = match Self::extract_centroid_3d(feature) {
            Ok(centroid) => centroid,
            Err(e) => {
                tracing::warn!("Failed to extract centroid: {}", e);
                fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                return Ok(());
            }
        };

        // Reproject to EPSG:6697 (lat/long) for solar position calculation
        let (latitude, longitude) =
            match self.reproject_to_6697(centroid_3d.x, centroid_3d.y, source_epsg) {
                Ok(coords) => coords,
                Err(e) => {
                    tracing::warn!("Failed to reproject coordinates: {}", e);
                    fw.send(ctx.new_with_feature_and_port(feature.clone(), REJECTED_PORT.clone()));
                    return Ok(());
                }
            };

        // Compute standard meridian from longitude if not provided
        let standard_meridian = match &self.standard_meridian_ast {
            Some(ast) => self.evaluate_float_expr(feature, &ctx, &ast.clone())?,
            None => {
                // Compute from longitude: round to nearest 15°
                (longitude / 15.0).round() * 15.0
            }
        };

        match &self.compiled_params {
            CompiledParams::Time { time_ast } => {
                let time_str = self.evaluate_string_expr(feature, &ctx, &time_ast.clone())?;
                let datetime = parse_time_string(&time_str)?;
                let position =
                    calculate_solar_position(latitude, longitude, datetime, standard_meridian);

                let mut new_feature = feature.clone();
                self.insert_solar_position_attributes(&mut new_feature, &position);
                Self::insert_centroid_attributes(&mut new_feature, &centroid_3d);
                new_feature.insert(
                    "solar_time",
                    AttributeValue::String(datetime.format("%Y/%m/%d/%H/%M/%S").to_string()),
                );

                fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
            }
            CompiledParams::Duration {
                start_ast,
                end_ast,
                step_ast,
                step_unit,
            } => {
                let start_str = self.evaluate_string_expr(feature, &ctx, &start_ast.clone())?;
                let end_str = self.evaluate_string_expr(feature, &ctx, &end_ast.clone())?;
                let step_value = self.evaluate_int_expr(feature, &ctx, &step_ast.clone())?;

                let start_datetime = parse_time_string(&start_str)?;
                let end_datetime = parse_time_string(&end_str)?;
                let step_duration = step_unit.to_duration(step_value);

                if step_duration.num_seconds() <= 0 {
                    return Err(SolarPositionError::Process(
                        "Step duration must be positive".to_string(),
                    )
                    .into());
                }

                let mut current = start_datetime;
                while current <= end_datetime {
                    let position =
                        calculate_solar_position(latitude, longitude, current, standard_meridian);

                    let mut new_feature = feature.clone();
                    new_feature.id = uuid::Uuid::new_v4();
                    self.insert_solar_position_attributes(&mut new_feature, &position);
                    Self::insert_centroid_attributes(&mut new_feature, &centroid_3d);
                    new_feature.insert(
                        "solar_time",
                        AttributeValue::String(current.format("%Y/%m/%d/%H/%M/%S").to_string()),
                    );

                    fw.send(ctx.new_with_feature_and_port(new_feature, DEFAULT_PORT.clone()));
                    current += step_duration;
                }
            }
        }

        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "SolarPositionCalculator"
    }
}

impl SolarPositionCalculator {
    /// Reproject coordinates from source EPSG to EPSG:6697 (JGD2011 geographic).
    /// Returns (latitude, longitude) in degrees.
    fn reproject_to_6697(
        &mut self,
        x: f64,
        y: f64,
        source_epsg: u32,
    ) -> Result<(f64, f64), SolarPositionError> {
        // If source is already geographic (6697), no reprojection needed
        if source_epsg == TARGET_EPSG {
            // For geographic CRS, input is typically (longitude, latitude)
            // but we need to return (latitude, longitude)
            return Ok((y, x));
        }

        if self.source_epsg != source_epsg {
            self.source_epsg = source_epsg;
            let from_crs = format!("EPSG:{}", source_epsg);
            let to_crs = format!("EPSG:{}", TARGET_EPSG);
            let proj = Proj::new_known_crs(&from_crs, &to_crs, None).map_err(|e| {
                SolarPositionError::Reprojection(format!(
                    "Failed to create the projection from {from_crs} to {to_crs}: {e}",
                ))
            })?;
            self.proj = Some(Mutex::new(proj))
        }

        let proj = self.proj.as_ref().unwrap().lock().unwrap();

        let (lon, lat) = proj.convert((x, y)).map_err(|e| {
            SolarPositionError::Reprojection(format!(
                "Failed to convert coordinates ({}, {}): {}",
                x, y, e
            ))
        })?;

        // Return (latitude, longitude) for solar position calculation
        Ok((lat, lon))
    }

    /// Extract 3D centroid from feature geometry (in source CRS coordinates).
    fn extract_centroid_3d(feature: &Feature) -> Result<Centroid3D, SolarPositionError> {
        let geometry = &feature.geometry;

        if geometry.is_empty() {
            return Err(SolarPositionError::InvalidGeometry(
                "Feature has no geometry".to_string(),
            ));
        }

        match &geometry.value {
            GeometryValue::None => Err(SolarPositionError::InvalidGeometry(
                "Geometry value is None".to_string(),
            )),
            GeometryValue::FlowGeometry2D(geom) => {
                let centroid = geom.centroid().ok_or_else(|| {
                    SolarPositionError::InvalidGeometry("Failed to compute 2D centroid".to_string())
                })?;
                Ok(Centroid3D {
                    x: centroid.x(),
                    y: centroid.y(),
                    z: 0.0,
                })
            }
            GeometryValue::FlowGeometry3D(geom) => {
                let centroid = geom.centroid().ok_or_else(|| {
                    SolarPositionError::InvalidGeometry("Failed to compute 3D centroid".to_string())
                })?;
                Ok(Centroid3D {
                    x: centroid.x(),
                    y: centroid.y(),
                    z: centroid.z(),
                })
            }
            GeometryValue::CityGmlGeometry(citygml) => {
                // Compute centroid from all polygons in CityGML geometry
                let mut sum_x = 0.0;
                let mut sum_y = 0.0;
                let mut sum_z = 0.0;
                let mut count = 0usize;

                for gml in &citygml.gml_geometries {
                    for poly in &gml.polygons {
                        if let Some(centroid) = poly.centroid() {
                            let cx = centroid.x();
                            let cy = centroid.y();
                            let cz = centroid.z();
                            // Skip polygons with non-finite centroids
                            if !cx.is_finite() || !cy.is_finite() || !cz.is_finite() {
                                continue;
                            }
                            sum_x += cx;
                            sum_y += cy;
                            sum_z += cz;
                            count += 1;
                        }
                    }
                }

                if count == 0 {
                    return Err(SolarPositionError::InvalidGeometry(
                        "CityGML geometry has no valid polygons for centroid computation"
                            .to_string(),
                    ));
                }

                Ok(Centroid3D {
                    x: sum_x / count as f64,
                    y: sum_y / count as f64,
                    z: sum_z / count as f64,
                })
            }
        }
    }

    /// Insert 3D centroid coordinates as attributes (in original source CRS).
    fn insert_centroid_attributes(feature: &mut Feature, centroid: &Centroid3D) {
        feature.insert(
            "ray_origin_x",
            AttributeValue::Number(
                serde_json::Number::from_f64(centroid.x).unwrap_or(serde_json::Number::from(0)),
            ),
        );
        feature.insert(
            "ray_origin_y",
            AttributeValue::Number(
                serde_json::Number::from_f64(centroid.y).unwrap_or(serde_json::Number::from(0)),
            ),
        );
        feature.insert(
            "ray_origin_z",
            AttributeValue::Number(
                serde_json::Number::from_f64(centroid.z).unwrap_or(serde_json::Number::from(0)),
            ),
        );
    }

    fn evaluate_string_expr(
        &self,
        feature: &Feature,
        ctx: &ExecutorContext,
        ast: &AST,
    ) -> Result<String, BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
        let result = scope.eval_ast::<rhai::Dynamic>(ast).map_err(|e| {
            SolarPositionError::Process(format!("Failed to evaluate expression: {e:?}"))
        })?;

        if let Some(s) = result.clone().try_cast::<String>() {
            Ok(s)
        } else {
            Err(
                SolarPositionError::Process("Expression did not evaluate to a string".to_string())
                    .into(),
            )
        }
    }

    fn evaluate_int_expr(
        &self,
        feature: &Feature,
        ctx: &ExecutorContext,
        ast: &AST,
    ) -> Result<i64, BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
        let result = scope.eval_ast::<rhai::Dynamic>(ast).map_err(|e| {
            SolarPositionError::Process(format!("Failed to evaluate expression: {e:?}"))
        })?;

        if let Some(i) = result.clone().try_cast::<i64>() {
            Ok(i)
        } else if let Some(f) = result.clone().try_cast::<f64>() {
            Ok(f as i64)
        } else {
            Err(SolarPositionError::Process(
                "Expression did not evaluate to an integer".to_string(),
            )
            .into())
        }
    }

    fn evaluate_float_expr(
        &self,
        feature: &Feature,
        ctx: &ExecutorContext,
        ast: &AST,
    ) -> Result<f64, BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
        let result = scope.eval_ast::<rhai::Dynamic>(ast).map_err(|e| {
            SolarPositionError::Process(format!("Failed to evaluate expression: {e:?}"))
        })?;

        if let Some(f) = result.clone().try_cast::<f64>() {
            Ok(f)
        } else if let Some(i) = result.clone().try_cast::<i64>() {
            Ok(i as f64)
        } else {
            Err(
                SolarPositionError::Process("Expression did not evaluate to a float".to_string())
                    .into(),
            )
        }
    }

    fn evaluate_epsg_expr(
        &self,
        feature: &Feature,
        ctx: &ExecutorContext,
        ast: &AST,
    ) -> Result<u32, BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = feature.new_scope(expr_engine.clone(), &self.global_params);
        let result = scope.eval_ast::<rhai::Dynamic>(ast).map_err(|e| {
            SolarPositionError::Process(format!("Failed to evaluate source_epsg expression: {e:?}"))
        })?;

        // Accept integer or string (e.g., "6677" or 6677)
        if let Some(i) = result.clone().try_cast::<i64>() {
            if i <= 0 {
                return Err(SolarPositionError::Process(
                    "EPSG code must be a positive integer".to_string(),
                )
                .into());
            }
            Ok(i as u32)
        } else if let Some(s) = result.clone().try_cast::<String>() {
            // Parse string, handling optional "EPSG:" prefix
            let epsg_str = s.trim().strip_prefix("EPSG:").unwrap_or(s.trim());
            epsg_str.parse::<u32>().map_err(|_| {
                SolarPositionError::Process(format!(
                    "Invalid EPSG code '{}': must be a positive integer",
                    s
                ))
                .into()
            })
        } else {
            Err(SolarPositionError::Process(
                "source_epsg expression must evaluate to an integer or string".to_string(),
            )
            .into())
        }
    }

    fn insert_solar_position_attributes(
        &self,
        feature: &mut Feature,
        position: &super::calculator::SolarPosition,
    ) {
        match &self.output_type {
            OutputType::UnitNormalVector | OutputType::Both => {
                // Convert altitude (elevation) and azimuth to unit normal vector in ENU coordinates
                // Azimuth convention: 0 = South, clockwise positive
                // ENU: x = East, y = North, z = Up
                let altitude_rad = position.altitude.to_radians();
                let azimuth_rad = position.azimuth.to_radians();

                let cos_alt = altitude_rad.cos();
                // Convert from "0 = South, clockwise" to ENU
                let x = -azimuth_rad.sin() * cos_alt; // East component
                let y = -azimuth_rad.cos() * cos_alt; // North component
                let z = altitude_rad.sin(); // Up component

                feature.insert(
                    "solar_direction_x",
                    AttributeValue::Number(
                        serde_json::Number::from_f64(x).unwrap_or(serde_json::Number::from(0)),
                    ),
                );
                feature.insert(
                    "solar_direction_y",
                    AttributeValue::Number(
                        serde_json::Number::from_f64(y).unwrap_or(serde_json::Number::from(0)),
                    ),
                );
                feature.insert(
                    "solar_direction_z",
                    AttributeValue::Number(
                        serde_json::Number::from_f64(z).unwrap_or(serde_json::Number::from(0)),
                    ),
                );

                if matches!(&self.output_type, OutputType::Both) {
                    feature.insert(
                        "solar_altitude",
                        AttributeValue::Number(
                            serde_json::Number::from_f64(position.altitude)
                                .unwrap_or(serde_json::Number::from(0)),
                        ),
                    );
                    feature.insert(
                        "solar_azimuth",
                        AttributeValue::Number(
                            serde_json::Number::from_f64(position.azimuth)
                                .unwrap_or(serde_json::Number::from(0)),
                        ),
                    );
                }
            }
            OutputType::AltitudeAndAzimuth => {
                feature.insert(
                    "solar_altitude",
                    AttributeValue::Number(
                        serde_json::Number::from_f64(position.altitude)
                            .unwrap_or(serde_json::Number::from(0)),
                    ),
                );
                feature.insert(
                    "solar_azimuth",
                    AttributeValue::Number(
                        serde_json::Number::from_f64(position.azimuth)
                            .unwrap_or(serde_json::Number::from(0)),
                    ),
                );
            }
        }
    }
}

fn parse_time_string(time_str: &str) -> Result<DateTime<Utc>, SolarPositionError> {
    let parts: Vec<&str> = time_str.split('/').collect();

    if parts.len() != 6 {
        return Err(SolarPositionError::TimeParse(format!(
            "Invalid time format '{}'. Expected 'yyyy/mm/dd/hh/mm/ss'",
            time_str
        )));
    }

    let year: i32 = parts[0]
        .parse()
        .map_err(|_| SolarPositionError::TimeParse(format!("Invalid year in '{}'", time_str)))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| SolarPositionError::TimeParse(format!("Invalid month in '{}'", time_str)))?;
    let day: u32 = parts[2]
        .parse()
        .map_err(|_| SolarPositionError::TimeParse(format!("Invalid day in '{}'", time_str)))?;
    let hour: u32 = parts[3]
        .parse()
        .map_err(|_| SolarPositionError::TimeParse(format!("Invalid hour in '{}'", time_str)))?;
    let minute: u32 = parts[4]
        .parse()
        .map_err(|_| SolarPositionError::TimeParse(format!("Invalid minute in '{}'", time_str)))?;
    let second: u32 = parts[5]
        .parse()
        .map_err(|_| SolarPositionError::TimeParse(format!("Invalid second in '{}'", time_str)))?;

    let naive_dt = NaiveDateTime::parse_from_str(
        &format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        ),
        "%Y-%m-%d %H:%M:%S",
    )
    .map_err(|e| SolarPositionError::TimeParse(format!("Failed to parse datetime: {}", e)))?;

    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_parse_time_string_valid() {
        let result = parse_time_string("2024/06/21/12/00/00");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 21);
        assert_eq!(dt.hour(), 12);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn test_parse_time_string_invalid_format() {
        let result = parse_time_string("2024-06-21 12:00:00");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_time_string_missing_parts() {
        let result = parse_time_string("2024/06/21");
        assert!(result.is_err());
    }

    #[test]
    fn test_step_unit_to_duration() {
        assert_eq!(StepUnit::Second.to_duration(30).num_seconds(), 30);
        assert_eq!(StepUnit::Minute.to_duration(5).num_seconds(), 300);
        assert_eq!(StepUnit::Hour.to_duration(2).num_seconds(), 7200);
        assert_eq!(StepUnit::Day.to_duration(1).num_seconds(), 86400);
    }
}
