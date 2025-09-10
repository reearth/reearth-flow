# Python Script Processor Action - Enhanced with Geospatial Support

## Overview

The Python Script Processor action has been enhanced to support full geospatial data processing. This allows users to write Python scripts that can access and manipulate both feature attributes and geometry, enabling powerful geospatial transformations within Re:Earth Flow workflows.

## Key Features

### ✅ Enhanced Capabilities

- **Full Geospatial Feature Access**: Scripts receive complete feature data including properties, geometry (as GeoJSON), and feature ID
- **Geometry Manipulation**: Built-in helper functions for creating and modifying geometries
- **Multiple Output Modes**: Support for one-to-one, one-to-many, and many-to-one feature operations
- **GeoJSON Interchange**: Seamless conversion between Re:Earth Flow geometry types and GeoJSON
- **Backward Compatibility**: Existing scripts continue to work unchanged

### 🔧 Architecture

The processor works by:
1. Converting incoming Re:Earth Flow features to GeoJSON format
2. Providing a Python wrapper with helper functions and convenient data access
3. Executing user scripts with full access to feature geometry and attributes
4. Parsing script output (single Feature or FeatureCollection) back to Re:Earth Flow format
5. Forwarding processed features through the standard output channel

## Usage

### Basic Example

```python
# Access feature data
print(f"Processing feature: {properties.get('name', 'Unknown')}")
print(f"Geometry type: {get_geometry_type(geometry)}")

# Modify properties
properties['processed'] = True
properties['area'] = 42.5

# Transform geometry (convert point to buffered polygon)
if get_geometry_type(geometry) == "Point":
    coords = get_coordinates(geometry)
    x, y = coords[0], coords[1]
    
    # Create a buffer around the point
    buffer_size = 0.001
    geometry = create_polygon([[
        [x - buffer_size, y - buffer_size],
        [x + buffer_size, y - buffer_size],
        [x + buffer_size, y + buffer_size],
        [x - buffer_size, y + buffer_size],
        [x - buffer_size, y - buffer_size]
    ]])
```

### Multiple Feature Output

```python
# Split a MultiPoint into individual Point features
if get_geometry_type(geometry) == "MultiPoint":
    coords = get_coordinates(geometry)
    
    output_features = []
    for i, point_coords in enumerate(coords):
        feature = {
            "type": "Feature",
            "properties": {
                **properties,  # Copy existing properties
                "part_id": i + 1
            },
            "geometry": create_point(point_coords[0], point_coords[1])
        }
        output_features.append(feature)
    
    # Return multiple features - processor handles this automatically
```

## Available Helper Functions

### Geometry Access
- `get_geometry_type(geometry)`: Returns the geometry type string (e.g., "Point", "Polygon")
- `get_coordinates(geometry)`: Extracts coordinate arrays from geometry

### Geometry Creation
- `create_point(x, y)`: Creates a Point geometry
- `create_polygon(coordinates)`: Creates a Polygon geometry
- Additional helpers can be added as needed

### Data Access
- `properties`: Dictionary of feature attributes (read/write)
- `geometry`: GeoJSON geometry object (read/write)
- `feature_id`: Unique feature identifier (read-only)
- `attributes`: Alias for `properties` (backward compatibility)

## Configuration Parameters

```yaml
- name: PythonProcessor
  type: action
  action: PythonScriptProcessor
  with:
    script: |
      # Your Python code here
      properties['enhanced'] = True
    
    # Optional parameters:
    pythonPath: "python3"     # Path to Python interpreter
    timeoutSeconds: 30        # Script execution timeout
```

## Supported Geometry Types

### Input (Re:Earth Flow → GeoJSON)
- ✅ Point → Point
- ✅ LineString → LineString  
- ✅ Polygon → Polygon
- ✅ MultiPoint → MultiPoint
- ✅ MultiLineString → MultiLineString
- ✅ MultiPolygon → MultiPolygon
- ✅ Triangle → Polygon (converted)
- ✅ Rect → Polygon (converted)
- ⏳ 3D Geometries (future enhancement)
- ⏳ CityGML Geometries (future enhancement)

### Output (GeoJSON → Re:Earth Flow)
- ✅ Point → Point
- ⏳ Additional types (incremental implementation)

## Example Workflows

### 1. Basic Geospatial Processing
Location: `tests/fixture/workflow/python/geospatial_processor.yaml`

Demonstrates:
- Point-to-polygon buffering
- Polygon centroid calculation
- Distance calculations
- Area estimation

### 2. Feature Creation & Splitting  
Location: `tests/fixture/workflow/python/feature_creation.yaml`

Demonstrates:
- MultiPoint splitting into individual features
- Creating derived features (service areas, connections)
- Bounding box generation
- Complex one-to-many transformations

## Integration with Re:Earth Flow

The enhanced Python Script Processor integrates seamlessly with the Re:Earth Flow engine:

- **Input**: Receives features from upstream actions (readers, transformers, etc.)
- **Processing**: Applies Python-based geospatial transformations
- **Output**: Sends processed features to downstream actions (writers, filters, etc.)
- **Data Flow**: Maintains feature IDs and metadata appropriately
- **Performance**: Processes features independently for parallel execution

## Best Practices

1. **Validate Input**: Always check geometry type and coordinates before processing
2. **Handle Edge Cases**: Consider null geometries, empty coordinates, etc.
3. **Preserve Data**: Copy existing properties when creating new features
4. **Error Handling**: Use try/catch blocks for robust script execution
5. **Performance**: For large datasets, prefer simple operations over complex calculations

## Documentation

- **User Guide**: `/docs/geospatial_guide.md` - Comprehensive usage documentation
- **Examples**: `/tests/fixture/workflow/python/` - Sample workflows
- **API Reference**: This README and inline code documentation

## Future Enhancements

- Extended geometry type support (3D, CityGML)
- Built-in geospatial library integration (Shapely, GeoPandas)
- Performance optimizations for large datasets  
- Advanced spatial operations and analysis functions
- Coordinate system transformation support

## Development Notes

The implementation follows Re:Earth Flow patterns:
- Uses standard `ProcessorFactory` and `Processor` traits
- Integrates with the existing feature forwarding system
- Maintains compatibility with expression evaluation engine
- Follows error handling conventions
- Includes comprehensive parameter documentation via JSON Schema

This enhancement positions the Python Script Processor as a powerful tool for geospatial data processing within Re:Earth Flow, enabling users to implement complex spatial transformations with familiar Python syntax while maintaining full integration with the workflow engine.
