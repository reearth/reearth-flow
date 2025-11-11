# CSV Writer Examples

This directory contains example workflows demonstrating how to use the CsvWriter action with geometry export support.

## Examples

### 1. WKT Geometry Export (`roundtrip-wkt.yml`)

Demonstrates reading CSV with WKT geometry and writing it back to CSV with geometry preserved.

**Input CSV:**
```csv
id,name,population,geometry
1,Tokyo,13960000,POINT(139.6917 35.6895)
2,Osaka,8839000,POINT(135.5022 34.6937)
3,Nagoya,2296000,POLYGON((136.88 35.13, 136.88 35.23, 136.98 35.23, 136.98 35.13, 136.88 35.13))
```

**Configuration:**
```yaml
action: CsvWriter
with:
  format: csv
  output: ./output-roundtrip-wkt.csv
  geometry:
    geometryMode: wkt
    column: geometry
```

**Output CSV:**
```csv
id,name,population,geometry
1,Tokyo,13960000,POINT(139.6917 35.6895)
2,Osaka,8839000,POINT(135.5022 34.6937)
3,Nagoya,2296000,POLYGON((136.88 35.13, 136.88 35.23, 136.98 35.23, 136.98 35.13, 136.88 35.13))
```

Supported WKT geometry types:
- `POINT(x y)` / `POINT(x y z)`
- `LINESTRING(x1 y1, x2 y2, ...)`
- `POLYGON((x1 y1, x2 y2, ...))`
- `MULTIPOINT((x1 y1), (x2 y2), ...)`
- `MULTILINESTRING(...)`
- `MULTIPOLYGON(...)`

### 2. Coordinate Columns Export (`roundtrip-coords.yml`)

Demonstrates reading CSV with coordinate columns and writing it back with coordinates preserved.

**Input CSV:**
```csv
id,name,temperature,longitude,latitude,elevation
1,Mount Fuji,12.5,138.7274,35.3606,3776.0
2,Tokyo Tower,18.2,139.7454,35.6586,332.9
```

**Configuration:**
```yaml
action: CsvWriter
with:
  format: csv
  output: ./output-roundtrip-coords.csv
  geometry:
    geometryMode: coordinates
    xColumn: longitude
    yColumn: latitude
    zColumn: elevation  # optional
```

**Output CSV:**
```csv
id,name,temperature,longitude,latitude,elevation
1,Mount Fuji,12.5,138.7274,35.3606,3776.0
2,Tokyo Tower,18.2,139.7454,35.6586,332.9
```

**Note**: Coordinate column mode only supports Point geometries (2D and 3D). If a feature contains a non-Point geometry (LineString, Polygon, etc.), a warning will be logged and empty strings will be written for the geometry columns.

## Parameters

### Common Parameters

- `format`: `csv` or `tsv` (tab-separated values)
- `output`: Path or expression for output CSV file

### Geometry Export Configuration

The `geometry` parameter is optional. If omitted, no geometry export is performed (geometry data is lost).

#### WKT Mode
```yaml
geometry:
  geometryMode: wkt
  column: <column_name>    # Name of column to write WKT
```

Works with all geometry types supported by CsvReader.

#### Coordinates Mode
```yaml
geometry:
  geometryMode: coordinates
  xColumn: <column_name>   # X coordinate (longitude)
  yColumn: <column_name>   # Y coordinate (latitude)
  zColumn: <column_name>   # Optional: Z coordinate (elevation)
```

**Limitations**: Only works with Point geometries. Non-point geometries will result in empty geometry columns with a warning logged.

## Running Examples

```bash
# From the engine directory, navigate to the examples directory
cd runtime/examples/fixture/workflow/examples/csv-writer

# Run WKT round-trip example
cargo run --package reearth-flow-cli -- run --workflow ./roundtrip-wkt.yml

# Run coordinates round-trip example
cargo run --package reearth-flow-cli -- run --workflow ./roundtrip-coords.yml
```

## Use Cases

### 1. Round-trip CSV Processing
Read CSV with geometry → Process features → Write back to CSV with geometry preserved

### 2. Format Conversion
- WKT to Coordinates: Read with WKT mode, write with coordinates mode (Point geometries only)
- Coordinates to WKT: Read with coordinates mode, write with WKT mode

### 3. Geometry Column Placement
Geometry columns are always placed at the END of the CSV (after all attribute columns), following GIS industry conventions.

## Backward Compatibility

If no `geometry` parameter is provided, CsvWriter behaves exactly as before - geometry data is not exported to the CSV file. This ensures existing workflows continue to work without modification.
