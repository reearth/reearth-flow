# Action Processor - Geometry Processing Actions

## Overview

The Action Processor package contains a comprehensive suite of geometry processing actions for Re:Earth Flow workflows. These actions enable spatial transformations, analysis, and validation operations on geospatial features.

## HorizontalReprojector

### Description

The HorizontalReprojector transforms geometry coordinates between different coordinate reference systems (CRS) using EPSG codes. It performs horizontal (X/Y) coordinate transformations while preserving Z coordinates unchanged.

### Features

- **Global CRS Support**: Supports any valid EPSG code via the PROJ library (7000+ coordinate systems)
- **Auto-detection**: Automatically detects source EPSG from input geometry metadata
- **Standards-based**: Uses industry-standard PROJ library (same as QGIS, GDAL, PostGIS)
- **2D and 3D Support**: Handles both 2D and 3D geometries with horizontal-only transformation
- **Comprehensive Geometry Types**: Point, LineString, Polygon, MultiPoint, MultiLineString, MultiPolygon

### Configuration Parameters

```yaml
- name: Reprojector
  type: action
  action: HorizontalReprojector
  with:
    # Optional: Source EPSG code
    # If not specified, will use EPSG code from input geometry
    sourceEpsgCode: 2193

    # Required: Target EPSG code
    targetEpsgCode: 4326
```

**Parameters:**

- `sourceEpsgCode` (optional): Source coordinate system EPSG code. If not provided, the action will use the EPSG code embedded in the input geometry. This allows for automatic CRS detection from data sources like shapefiles.

- `targetEpsgCode` (required): Target coordinate system EPSG code for the reprojection. Supports any valid EPSG code.

**Common EPSG Codes:**

- `4326` - WGS 84 (GPS coordinates, latitude/longitude)
- `3857` - Web Mercator / Pseudo-Mercator (Google Maps, OpenStreetMap, web mapping services)
- `2193` - NZTM2000 (New Zealand Transverse Mercator)
- `32601-32660` - WGS 84 UTM zones (Northern Hemisphere)
- `32701-32760` - WGS 84 UTM zones (Southern Hemisphere)

For a complete list of EPSG codes, visit [epsg.io](https://epsg.io).

### Supported Geometry Types

#### Input/Output Support

| Geometry Type | Support Status |
|--------------|----------------|
| Point (2D/3D) | ✅ Supported |
| LineString (2D/3D) | ✅ Supported |
| Polygon (2D/3D) | ✅ Supported |
| MultiPoint (2D/3D) | ✅ Supported |
| MultiLineString (2D/3D) | ✅ Supported |
| MultiPolygon (2D/3D) | ✅ Supported |
| GeometryCollection | ⏳ Future |
| CityGML Geometry | ❌ Use VerticalReprojector |

### Usage Examples

#### Example 1: Automatic CRS Detection

When reading from data sources that include CRS information (like shapefiles with .prj files):

```yaml
workflow:
  - name: ReadShapefile
    type: action
    action: ShapefileReader
    with:
      path: "data/nz-roads.shp"

  - name: ConvertToWGS84
    type: action
    action: HorizontalReprojector
    with:
      targetEpsgCode: 4326  # Source CRS auto-detected from shapefile
```

#### Example 2: Explicit Source CRS

When input geometry lacks CRS information:

```yaml
workflow:
  - name: Transform
    type: action
    action: HorizontalReprojector
    with:
      sourceEpsgCode: 2193  # New Zealand Transverse Mercator
      targetEpsgCode: 4326  # WGS 84
```

#### Example 3: Web Mercator to Geographic

Convert web mapping coordinates to latitude/longitude:

```yaml
workflow:
  - name: WebToGeo
    type: action
    action: HorizontalReprojector
    with:
      sourceEpsgCode: 3857  # Web Mercator
      targetEpsgCode: 4326  # WGS 84
```

### Technical Details

#### Transformation Behavior

- **Horizontal Only**: Transforms X and Y coordinates using the PROJ library
- **Z Preservation**: Z coordinates (elevation/height) are passed through unchanged
- **Per-Feature**: Creates fresh projection transformation for each feature (thread-safe)
- **Error Handling**: Reports clear errors if EPSG codes are invalid or transformation fails

#### Integration with Data Sources

The HorizontalReprojector automatically integrates with data sources that provide CRS information:

- **ShapefileReader**: Extracts EPSG from .prj files (WKT format)
- **GeoJSON**: Reads CRS from standard GeoJSON CRS object (if present)
- **Other formats**: Any source that sets `geometry.epsg` field

#### Coordinate System Detection

When `sourceEpsgCode` is not specified:

1. Action checks input feature's `geometry.epsg` field
2. If EPSG is present, uses it as source CRS
3. If EPSG is missing, returns error with guidance:
   ```
   Source EPSG code not specified and geometry has no EPSG information.
   Either set sourceEpsgCode parameter or ensure input geometries have EPSG codes.
   ```

### Performance Considerations

- **Projection Creation**: New PROJ transformation created per feature (ensures thread safety)
- **Batch Processing**: Processes features independently for parallel execution
- **Memory**: Low memory overhead, transformations done in-place where possible

### Dependencies

The HorizontalReprojector requires the PROJ library:

**Build Dependencies:**
- `libproj-dev` (Debian/Ubuntu)
- `proj` (macOS via Homebrew)
- `proj` (Windows via vcpkg)

**Runtime Dependencies:**
- `libproj25` (or appropriate version for your distribution)

These dependencies are automatically included in Docker images.

### Related Actions

- **VerticalReprojector**: Transform Z coordinates between vertical datums (Japanese coordinate systems)
- **GeometryValidator**: Validate geometry structure before/after transformation
- **GeometryCoercer**: Convert between geometry types

### Troubleshooting

#### Error: Source EPSG code not specified

**Cause**: Input geometry has no EPSG information and `sourceEpsgCode` parameter not set

**Solution**:
1. Add `sourceEpsgCode` parameter to action configuration, OR
2. Ensure data source provides EPSG information (e.g., shapefile with .prj file)

#### Error: Failed to create PROJ transformation

**Cause**: Invalid EPSG code or unsupported transformation

**Solution**:
1. Verify EPSG codes are valid at [epsg.io](https://epsg.io)
2. Check that PROJ library supports the transformation
3. Ensure both CRS definitions are available in PROJ database

#### Incorrect Output Coordinates

**Cause**: Wrong source or target EPSG code

**Solution**:
1. Verify input data's actual coordinate system
2. Use QGIS or similar tool to confirm CRS of source data
3. Check that target CRS is appropriate for your use case

### Best Practices

1. **Explicit CRS**: When possible, specify `sourceEpsgCode` explicitly for clarity and maintainability

2. **Validate Input**: Use GeometryValidator before reprojection to catch geometry issues early

3. **CRS Documentation**: Document the coordinate systems used in your workflows

4. **Testing**: Verify transformations with known coordinates or reference datasets

5. **Regional CRS**: Use appropriate regional coordinate systems (e.g., UTM zones) for accurate local measurements

## Other Geometry Actions

This package includes many other geometry processing actions. For complete documentation of all actions, see the auto-generated documentation in `/docs/mdbook/src/action.md`.

## Development

### Adding New Actions

1. Create new `.rs` file in `src/geometry/` directory
2. Implement `ProcessorFactory` and `Processor` traits
3. Add action to mapping in `src/geometry/mapping.rs`
4. Add tests in the same file or separate test module
5. Run `cargo make doc-action` to update generated documentation

### Testing

```bash
cd engine/runtime/action-processor
cargo test
```

For more information on engine development, see `/engine/CLAUDE.md`.
