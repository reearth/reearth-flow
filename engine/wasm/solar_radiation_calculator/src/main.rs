#![allow(unused)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io::{self, Read, Write};
use std::os::raw::c_char;

#[derive(Serialize, Deserialize)]
struct InputData {
    // Solar radiation related attributes
    #[serde(rename = "年月日")]
    date: Option<String>,
    #[serde(rename = "方位[°]")]
    azimuth: Option<f64>,
    #[serde(rename = "方位[°]00")]
    azimuth_00: Option<f64>,
    #[serde(rename = "高度[°]")]
    altitude: Option<f64>,
    #[serde(rename = "出")]
    sunrise: Option<f64>,
    #[serde(rename = "入り")]
    sunset: Option<f64>,
    #[serde(rename = "南中")]
    noon: Option<f64>,

    // Add any other attributes that might be passed
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct OutputData {
    status: String,
    #[serde(rename = "日射量[kWh/m2]")]
    solar_radiation: f64,
    #[serde(rename = "attributes")]
    attributes: HashMap<String, serde_json::Value>,
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let data: InputData = serde_json::from_str(&input).unwrap();

    // TODO: compute solar_radiation and attributes
    let output_data = OutputData {
        status: "ok".to_string(),
        solar_radiation: 123.45,
        attributes: data.extra, // or build new HashMap
    };

    let output_json = serde_json::to_string(&output_data).unwrap();
    io::stdout().write_all(output_json.as_bytes()).unwrap();
}
