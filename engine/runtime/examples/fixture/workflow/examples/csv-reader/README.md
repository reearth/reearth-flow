# CSV Reader Examples

This directory contains example workflows demonstrating how to use the CsvReader action with geometry support.

## Examples

### 1. WKT Geometry Parsing (`wkt-example.yml`)

Reads CSV files with geometry stored as Well-Known Text (WKT) in a single column.

**Sample CSV:**
```csv
id,name,population,geometry
1,Tokyo,13960000,POINT(139.6917 35.6895)
2,Osaka,8839000,POINT(135.5022 34.6937)
```

**Configuration:**
```yaml
action: CsvReader
with:
  format: csv
  dataset: ./sample-with-wkt.csv
  geometry:
    geometryMode: wkt
    column: geometry
    epsg: 4326
```

Supported WKT geometry types:
- `POINT(x y)`
- `LINESTRING(x1 y1, x2 y2, ...)`
- `POLYGON((x1 y1, x2 y2, ...))`
- `MULTIPOINT((x1 y1), (x2 y2), ...)`
- `MULTILINESTRING(...)`
- `MULTIPOLYGON(...)`

### 2. Coordinate Columns (`coordinates-example.yml`)

Reads CSV files with coordinates in separate X, Y, and optional Z columns.

**Sample CSV:**
```csv
id,name,temperature,longitude,latitude,elevation
1,Mount Fuji,12.5,138.7274,35.3606,3776.0
2,Tokyo Tower,18.2,139.7454,35.6586,332.9
```

**Configuration:**
```yaml
action: CsvReader
with:
  format: csv
  dataset: ./sample-with-coords.csv
  geometry:
    geometryMode: coordinates
    xColumn: longitude
    yColumn: latitude
    zColumn: elevation  # optional
    epsg: 4326
```

## Parameters

### Common Parameters

- `format`: `csv` or `tsv` (tab-separated values)
- `dataset`: Path to CSV file (can use expressions)
- `inline`: Inline CSV content (alternative to `dataset`)
- `offset`: Number of rows to skip before header (default: 0)

### Geometry Configuration

The `geometry` parameter is optional. If omitted, no geometry parsing is performed.

#### WKT Mode
```yaml
geometry:
  geometryMode: wkt
  column: <column_name>    # Name of column containing WKT
  epsg: <epsg_code>        # Optional: CRS code (e.g., 4326 for WGS84)
```

#### Coordinates Mode
```yaml
geometry:
  geometryMode: coordinates
  xColumn: <column_name>   # X coordinate (longitude)
  yColumn: <column_name>   # Y coordinate (latitude)
  zColumn: <column_name>   # Optional: Z coordinate (elevation)
  epsg: <epsg_code>        # Optional: CRS code
```

## Running Examples

```bash
# From the engine directory, navigate to the examples directory
cd runtime/examples/fixture/workflow/examples/csv-reader

# Run WKT example
cargo run --package reearth-flow-cli -- run --workflow ./wkt-example.yml

# Run coordinates example
cargo run --package reearth-flow-cli -- run --workflow ./coordinates-example.yml
```

## Output

All examples output features that can be:
- Written to GeoJSON with `GeoJsonWriter`
- Written back to CSV with `FeatureWriter`
- Processed with other Flow actions (filters, transformers, etc.)

Features will have:
- `id`: UUID
- `attributes`: CSV columns as string attributes (excluding geometry columns)
- `geometry.epsg`: EPSG code if specified
- `geometry.value`: Parsed geometry (if geometry config provided)

**Note**: When geometry parsing is enabled, the columns used for geometry (WKT column, or X/Y/Z columns) are automatically excluded from the feature attributes to avoid redundancy. This follows GIS industry standards (similar to QGIS, PostGIS, ArcGIS).
