# Step-by-Step Guide: Creating WASM Projects for Re:Earth Flow

## Overview
This guide provides a step-by-step approach to create a separate WASM project that can be used with the Re:Earth Flow framework. It covers creating the project, compiling it to WASM with the right configuration, and setting up default build behavior.

## Step 1: Create Separate WASM Project

### 1.1 Create the project structure
```bash
mkdir -p wasm/solar_radiation_calculator/src
cd wasm/solar_radiation_calculator
```

### 1.2 Create the Cargo.toml file
```toml
[package]
name = "solar_radiation_calculator"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "solar_radiation_calculator"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 1.3 Create the main.rs file (src/main.rs)
This file should implement a standard Rust binary that reads JSON from stdin and writes JSON to stdout:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Read};

#[derive(Serialize, Deserialize)]
struct InputData {
    // Define the input fields your WASM module expects
    #[serde(rename = "年月日")]
    date: Option<String>,
    #[serde(rename = "方位[°]")]
    azimuth: Option<f64>,
    #[serde(rename = "高度[°]")]
    altitude: Option<f64>,
    #[serde(rename = "出")]
    sunrise: Option<f64>,
    #[serde(rename = "入り")]
    sunset: Option<f64>,
    #[serde(rename = "南中")]
    noon: Option<f64>,

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

fn calculate_solar_radiation(_sunrise: f64, _sunset: f64, altitude_radians: f64) -> f64 {
    // Implement your calculation logic here
    if altitude_radians > 0.0 {
        altitude_radians.sin() * 5.0 // Example calculation
    } else {
        0.0
    }
}

fn main() {
    // Read input from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read from stdin");

    match serde_json::from_str::<InputData>(&input) {
        Ok(input_data) => {
            // Extract values with defaults
            let altitude = input_data.altitude.unwrap_or(0.0);
            let sunrise = input_data.sunrise.unwrap_or(0.0);
            let sunset = input_data.sunset.unwrap_or(0.0);

            // Perform calculation
            let altitude_radians = altitude.to_radians();
            let solar_radiation = calculate_solar_radiation(sunrise, sunset, altitude_radians);

            // Prepare output data
            let mut attributes = input_data.extra;
            if let Some(date) = input_data.date {
                attributes.insert("年月日".to_string(), serde_json::Value::String(date));
            }
            if let Some(azimuth) = input_data.azimuth {
                attributes.insert(
                    "方位[°]".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(azimuth)
                            .unwrap_or(serde_json::Number::from(0)),
                    ),
                );
            }
            if let Some(altitude_val) = input_data.altitude {
                attributes.insert(
                    "高度[°]".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(altitude_val)
                            .unwrap_or(serde_json::Number::from(0)),
                    ),
                );
            }
            // Add other fields as needed...

            // Add calculated result
            attributes.insert(
                "日射量[kWh/m2]".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(solar_radiation)
                        .unwrap_or(serde_json::Number::from(0)),
                ),
            );

            let output = OutputData {
                status: "success".to_string(),
                solar_radiation,
                attributes,
            };

            match serde_json::to_string(&output) {
                Ok(output_json) => println!("{}", output_json),
                Err(_) => {
                    println!("{{\"status\": \"error\", \"error\": \"Failed to serialize output\"}}")
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to parse input JSON: {}", e);
            println!(
                "{{\"status\": \"error\", \"error\": \"Failed to parse input JSON: {}\"}}",
                e
            );
        }
    }
}
```

### 1.4 Create configuration for default WASI target
Create `.cargo/config.toml` in the WASM project directory:

```toml
# Configuration for building WASM with correct target by default
[build]
target = "wasm32-wasip1"
```

## Step 2: Compile to WASM with Correct Target

### 2.1 Install the WASI target
```bash
rustup target add wasm32-wasip1
```

### 2.2 Compile the project
```bash
cd wasm/solar_radiation_calculator
cargo build --release
```
This will automatically use the `wasm32-wasip1` target due to the configuration in Step 1.4.

Alternatively, you can compile explicitly:
```bash
cargo build --target wasm32-wasip1 --release
```

### 2.3 Verify the WASM file
The compiled WASM file should be located at:
- Release: `target/wasm32-wasip1/release/solar_radiation_calculator.wasm`
- Debug: `target/wasm32-wasip1/debug/solar_radiation_calculator.wasm`

## Step 3: Configure Project for Automatic WASM Builds

### 3.1 The `.cargo/config.toml` setup (already done in Step 1.4)
The configuration file ensures that when you run:
```bash
cargo build
cargo build --release
```
from **within the WASM project directory**, it will automatically use the `wasm32-wasip1` target.

### 3.2 Expected JSON Input/Output Format
Make sure your WASM module handles the expected format:

**Input Format (from Re:Earth Flow):**
```json
{
  "年月日": "2023-01-01",
  "方位[°]": 45.0,
  "高度[°]": 30.0,
  "出": 28800.0,
  "入り": 64800.0,
  "南中": 46800.0
}
```

**Output Format (to Re:Earth Flow):**
```json
{
  "status": "success",
  "日射量[kWh/m2]": 2.5,
  "attributes": {
    "年月日": "2023-01-01",
    "方位[°]": 45.0,
    "高度[°]": 30.0,
    "出": 28800.0,
    "入り": 64800.0,
    "南中": 46800.0,
    "日射量[kWh/m2]": 2.5
  }
}
```

## Step 4: Integration with Re:Earth Flow

### 4.1 Configure the workflow file
In your workflow YAML file, specify the path to the generated WASM file:
```yaml
action: WasmRuntimeExecutor
with:
  processorType: "Attribute"
  programmingLanguage: "PrecompiledWasm"
  source: "file:///path/to/target/wasm32-wasip1/release/solar_radiation_calculator.wasm"
```

### 4.2 Build and run the workflow
```bash
# Build the WASM module
cd wasm/solar_radiation_calculator
cargo build --release

# Run the workflow
cargo run --package reearth-flow-cli run --workflow path/to/workflow.yml

# Example
cargo run --package reearth-flow-cli run --workflow runtime/examples/fixture/workflow/solar-radiation/time-to-time-value/solar-energy-workflow.yml
```

## Best Practices

1. **Always use WASI-compatible Rust code**: Use standard library functions that are available in WASI environment
2. **Handle JSON input/output properly**: Read from stdin and write to stdout in the expected format
3. **Include error handling**: Return proper error responses when JSON parsing fails
4. **Use the configuration file**: This ensures consistent builds across different environments
5. **Test your WASM module independently**: Before integrating with Re:Earth Flow

## Troubleshooting

- If the WASM file is not generated, make sure the `wasm32-wasip1` target is installed
- If the JSON format is incorrect, verify the input/output structure matches expectations
- If the module hangs, ensure all input is properly consumed and output is produced before the main function returns