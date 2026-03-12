# CZML Timeseries Support

This document describes how to read and write time-dynamic CZML data in Re:Earth Flow, enabling workflows that process moving objects, satellite tracks, sensor trajectories, and other time-varying geospatial entities.

## Overview

[CZML](https://github.com/AnalyticalGraphicsInc/czml-writer/wiki/CZML-Guide) is the JSON-based format used by [Cesium](https://cesium.com/) for describing time-dynamic 3D scenes. A CZML file is a JSON array of *packets*, each representing an entity (vehicle, satellite, sensor, etc.) whose properties can change over time.

Re:Earth Flow supports:

- **Reading** CZML files with time-tagged position samples via `CzmlReader`
- **Writing** time-dynamic features back to CZML via `CzmlWriter`
- **Faithful round-trip** of all CZML properties (point, label, path, orientation, etc.)

## Table of Contents

- [Key Concepts](#key-concepts)
- [CzmlReader — Time Sampling Strategies](#czmlreader--time-sampling-strategies)
- [CzmlWriter — Output Modes](#czmlwriter--output-modes)
- [Feature Attributes](#feature-attributes)
- [Workflow Examples](#workflow-examples)
- [Supported CZML Patterns](#supported-czml-patterns)
- [Interpolation Algorithms](#interpolation-algorithms)
- [Tips and Best Practices](#tips-and-best-practices)

---

## Key Concepts

### CZML Time-Tagged Positions

A CZML packet can encode positions that change over time using `cartographicDegrees` with either:

- **Numeric offsets** — seconds since an `epoch`:
  ```json
  {
    "epoch": "2024-01-01T00:00:00Z",
    "cartographicDegrees": [0, 139.69, 35.69, 50, 120, 139.70, 35.69, 52]
  }
  ```
- **ISO 8601 timestamps** — absolute date-time strings:
  ```json
  {
    "cartographicDegrees": ["2024-01-01T00:00:00Z", 139.0, 36.0, 400000]
  }
  ```

### Option A vs Option B

There are two approaches for representing CZML timeseries as features:

- **Option A (`allSamples`)** — Expand each time sample into a separate Feature. A packet with 6 positions produces 6 features. Useful for per-sample processing.
- **Option B (`preserveRaw`)** — Keep one Feature per entity with all timeseries data and visual properties embedded as attributes. This is the **default and recommended approach** for faithful round-trips.

---

## CzmlReader — Time Sampling Strategies

The `timeSampling` parameter controls how time-dynamic positions are converted to features.

### `preserveRaw` (default)

Produces one feature per CZML entity. The geometry uses the first position sample. All timeseries data is stored in `czml.timeseries`, and all other CZML packet properties (point, label, path, orientation, ellipsoid, etc.) are preserved as `czml.<key>` attributes for faithful round-trip.

```yaml
action: "CzmlReader"
with:
  dataset: "path/to/data.czml"
  # timeSampling defaults to "preserveRaw"
```

**Output:** One feature per entity with embedded timeseries and visual properties.

**Feature attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `czml.timeseries` | String (JSON) | Array of `{time, timeOffset, lon, lat, height}` samples |
| `czml.epoch` | String | The epoch datetime from the position property |
| `czml.interpolationAlgorithm` | String | `"LAGRANGE"`, `"LINEAR"`, or `"HERMITE"` |
| `czml.interpolationDegree` | Number | Interpolation polynomial degree |
| `czml.point` | String (JSON) | Raw point visualization properties |
| `czml.label` | String (JSON) | Raw label properties |
| `czml.path` | String (JSON) | Raw path trail properties |
| `czml.<key>` | String (JSON) | Any other CZML packet property |

### `allSamples`

Expands every time-tagged sample into a separate feature. Each feature carries `czml.timestamp` and `czml.timeOffset` attributes.

```yaml
action: "CzmlReader"
with:
  dataset: "path/to/data.czml"
  timeSampling: "allSamples"
```

**Output:** One feature per time sample. A packet with 6 samples produces 6 features.

**Feature attributes added per sample:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `czml.timestamp` | String | ISO 8601 timestamp or epoch-relative string |
| `czml.timeOffset` | Number | Seconds since the epoch |
| `czml.epoch` | String | The epoch datetime |
| `czml.interpolationAlgorithm` | String | Interpolation algorithm |
| `czml.interpolationDegree` | Number | Interpolation degree |

### `firstSampleOnly`

Extracts only the first position sample as static geometry. All other CZML properties are preserved as `czml.<key>` attributes.

```yaml
action: "CzmlReader"
with:
  dataset: "path/to/data.czml"
  timeSampling: "firstSampleOnly"
```

**Output:** One feature per CZML packet with the first position as geometry.

---

## CzmlWriter — Output Modes

The writer automatically detects the feature format and chooses the appropriate output mode.

### Embedded Mode (Option B) — Recommended

When features have `czml.timeseries` attributes (from `preserveRaw` reading), the writer reconstructs full CZML packets:

- Position with time-tagged `cartographicDegrees` (numeric offsets or ISO strings)
- Interpolation metadata (`epoch`, `interpolationAlgorithm`, `interpolationDegree`)
- All `czml.*` properties merged back as raw JSON (point, label, path, etc.)
- Availability ranges preserved
- Static entities (without `czml.timeseries`) get simple `cartographicDegrees`

```yaml
action: "CzmlWriter"
with:
  output: "env.get(\"outputFilePath\")"
```

No additional parameters needed — the writer reads everything from `czml.*` attributes.

### Grouped Mode (Option A)

When both `timeField` and `groupTimeseriesBy` are set, features with the same group key are merged into a single CZML entity with time-tagged position samples.

```yaml
action: "CzmlWriter"
with:
  output: "env.get(\"outputFilePath\")"
  timeField: "czml.timestamp"
  epoch: "2024-01-01T00:00:00Z"
  interpolationAlgorithm: "LAGRANGE"
  interpolationDegree: 5
  groupTimeseriesBy: "id"
```

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `output` | Expr | Yes | — | Output file path (Rhai expression) |
| `timeField` | String | No | — | Attribute containing the timestamp (Option A) |
| `epoch` | String | No | auto | ISO 8601 epoch for numeric offsets (Option A) |
| `interpolationAlgorithm` | Enum | No | `LINEAR` | `LINEAR`, `LAGRANGE`, or `HERMITE` (Option A) |
| `interpolationDegree` | Number | No | `1` | Polynomial degree for interpolation (Option A) |
| `groupTimeseriesBy` | String | No | — | Attribute to group features into entities (Option A) |
| `groupBy` | Array | No | — | Attributes for splitting into separate CZML files |

---

## Feature Attributes

### Round-Trip with Embedded Mode (Recommended)

The `preserveRaw` reader + embedded writer path preserves all CZML properties:

```
CZML File → CzmlReader (preserveRaw) → Feature with czml.* attrs → CzmlWriter → CZML File
```

All packet properties (point, label, path, orientation, ellipsoid, billboard, model, polyline, polygon, etc.) are faithfully round-tripped as raw JSON without needing explicit support for each type.

### Standard Attributes

| Attribute | Source | Description |
|-----------|--------|-------------|
| `id` | CZML packet `id` | Entity identifier |
| `name` | CZML packet `name` | Entity display name |
| `description` | CZML packet `description` | Entity description |
| `availability` | CZML packet `availability` | Time range |
| `parent` | CZML packet `parent` | Parent entity reference |

---

## Workflow Examples

### Round-Trip (Recommended — Option B)

Read CZML and write it back with all properties preserved:

```yaml
graphs:
  - nodes:
      - name: "Read CZML"
        action: "CzmlReader"
        with:
          dataset: "vehicles.czml"
          skipDocumentPacket: true
          # preserveRaw is the default

      - name: "Write CZML"
        action: "CzmlWriter"
        with:
          output: "env.get(\"outputFilePath\")"
    edges:
      - from: "Read CZML"
        to: "Write CZML"
```

### Read and Inspect Timeseries

Expand all samples to JSON for inspection:

```yaml
graphs:
  - nodes:
      - name: "Read CZML"
        action: "CzmlReader"
        with:
          dataset: "vehicles.czml"
          skipDocumentPacket: true
          timeSampling: "allSamples"

      - name: "Write JSON"
        action: "FeatureWriter"
        with:
          output: "env.get(\"outputFilePath\")"
          format: "json"
    edges:
      - from: "Read CZML"
        to: "Write JSON"
```

### Round-Trip with Expanded Samples (Option A)

Read as individual samples, process, and re-group:

```yaml
graphs:
  - nodes:
      - name: "Read CZML"
        action: "CzmlReader"
        with:
          dataset: "vehicles.czml"
          skipDocumentPacket: true
          timeSampling: "allSamples"

      - name: "Filter"
        action: "FeatureFilter"
        with:
          conditions:
            - attribute: "name"
              operator: "equals"
              value: "Vehicle Alpha"

      - name: "Write CZML"
        action: "CzmlWriter"
        with:
          output: "env.get(\"outputFilePath\")"
          timeField: "czml.timestamp"
          epoch: "2024-01-01T00:00:00Z"
          interpolationAlgorithm: "LAGRANGE"
          interpolationDegree: 5
          groupTimeseriesBy: "id"
    edges:
      - from: "Read CZML"
        to: "Filter"
      - from: "Filter"
        to: "Write CZML"
```

---

## Supported CZML Patterns

The embedded mode (`preserveRaw` + writer) supports all CZML property patterns through raw JSON round-trip:

| Pattern | Example Property | Round-Trip |
|---------|-----------------|------------|
| Sampled position | `position.cartographicDegrees` with epoch | Structured via `czml.timeseries` |
| Point styling | `point` (pixelSize, color, outline) | `czml.point` raw JSON |
| Label | `label` (text, font, style, pixelOffset) | `czml.label` raw JSON |
| Path trail | `path` (material, width, leadTime, trailTime) | `czml.path` raw JSON |
| Orientation | `orientation` (quaternion) | `czml.orientation` raw JSON |
| Ellipsoid | `ellipsoid` (radii, material) | `czml.ellipsoid` raw JSON |
| Billboard | `billboard` (image, scale) | `czml.billboard` raw JSON |
| Model | `model` (gltf, scale) | `czml.model` raw JSON |
| Polyline | `polyline` (positions, material) | `czml.polyline` raw JSON |
| Any other | Any CZML packet property | `czml.<key>` raw JSON |

---

## Interpolation Algorithms

The `interpolationAlgorithm` parameter tells Cesium how to interpolate between time-tagged samples at render time.

| Algorithm | Degree | Best For |
|-----------|--------|----------|
| `LINEAR` | 1 | Straight-line movement between samples |
| `LAGRANGE` | 5 (typical) | Smooth curves for vehicles, aircraft, satellites |
| `HERMITE` | — | Spline interpolation when tangent data is available |

**Note:** The interpolation is performed by Cesium at visualization time, not by Re:Earth Flow. The engine writes the algorithm hint into the CZML output.

---

## Tips and Best Practices

1. **Use `preserveRaw` (default) for round-trips** — this preserves all CZML properties without needing to configure writer parameters for each property type.

2. **Always set `skipDocumentPacket: true`** when reading — the document packet contains clock metadata, not feature data.

3. **Use `allSamples` for per-sample processing** — this gives you full control over each time sample as a separate feature, allowing filtering, attribute manipulation, and transformation.

4. **The writer auto-detects the mode** — if features have `czml.timeseries`, it uses embedded mode. If `timeField` + `groupTimeseriesBy` are configured, it uses grouped mode. Otherwise it falls back to legacy CityGML polygon output.

5. **LAGRANGE with degree 5** produces smooth curves for most vehicle/satellite tracking use cases. Use `LINEAR` for data where straight-line interpolation between points is appropriate.

6. **Static entities pass through unchanged** — entities without time-tagged positions (like points of interest) are handled correctly by both reader and writer without any timeseries configuration.

7. **All CZML visual properties are preserved** — point colors, label fonts, path trails, orientations, and any other packet property survive the round-trip as `czml.<key>` raw JSON attributes.
