# Python Script Processor - Geospatial Processing Guide

This guide covers how to use the Python Script Processor for geospatial data manipulation in Re:Earth Flow.

## Overview

The Python Script Processor now supports full geospatial feature processing, allowing you to:
- Access and manipulate feature geometry alongside attributes
- Transform between different geometry types
- Perform spatial calculations and analysis
- Create new geometries programmatically

## Built-in Helper Functions

The processor provides several helper functions for working with geometries:

### Geometry Access Functions

```python
# Get the geometry type (Point, LineString, Polygon, etc.)
geom_type = get_geometry_type(geometry)

# Get coordinate arrays from geometry
coords = get_coordinates(geometry)
```

### Geometry Creation Functions

```python
# Create a Point geometry
point_geom = create_point(longitude, latitude)
# With elevation:
point_geom = create_point(longitude, latitude, elevation)

# Create a LineString geometry
line_geom = create_linestring([[x1, y1], [x2, y2], [x3, y3]])

# Create a Polygon geometry
polygon_geom = create_polygon([
    [[x1, y1], [x2, y2], [x3, y3], [x4, y4], [x1, y1]]  # exterior ring
    # Optional holes can be added as additional arrays
])

# Create a MultiPoint geometry
multipoint_geom = create_multipoint([[x1, y1], [x2, y2], [x3, y3]])

# Create a MultiLineString geometry
multiline_geom = create_multilinestring([
    [[x1, y1], [x2, y2]],  # first line
    [[x3, y3], [x4, y4]]   # second line
])

# Create a MultiPolygon geometry
multipoly_geom = create_multipolygon([
    [[[x1, y1], [x2, y2], [x3, y3], [x1, y1]]],  # first polygon
    [[[x4, y4], [x5, y5], [x6, y6], [x4, y4]]]   # second polygon
])
```

## Input and Output

### Input Data
Your Python script receives:
- `properties`: Dictionary of feature attributes
- `geometry`: GeoJSON geometry object
- `feature_id`: Unique identifier for the feature

### Output Options
You can output:
1. **Single Feature**: Modify `properties` and/or `geometry` in-place
2. **Multiple Features**: Return a list of GeoJSON Feature objects
3. **Feature Collection**: Return a GeoJSON FeatureCollection

## Example Scripts

### 1. Basic Geometry Transformation

```python
# Convert points to buffered polygons
if get_geometry_type(geometry) == "Point":
    coords = get_coordinates(geometry)
    x, y = coords[0], coords[1]
    
    # Create a square buffer
    buffer_size = 0.001  # degrees
    geometry = create_polygon([[
        [x - buffer_size, y - buffer_size],
        [x + buffer_size, y - buffer_size],
        [x + buffer_size, y + buffer_size],
        [x - buffer_size, y + buffer_size],
        [x - buffer_size, y - buffer_size]
    ]])
    
    properties['geometry_type'] = 'buffered_point'
```

### 2. Spatial Analysis

```python
import math

# Calculate area and perimeter for polygons
if get_geometry_type(geometry) == "Polygon":
    coords = get_coordinates(geometry)
    exterior_ring = coords[0]
    
    # Calculate area using shoelace formula
    area = 0
    perimeter = 0
    
    for i in range(len(exterior_ring) - 1):
        x1, y1 = exterior_ring[i]
        x2, y2 = exterior_ring[i + 1]
        
        # Area calculation
        area += x1 * y2 - x2 * y1
        
        # Perimeter calculation (approximate)
        dx = x2 - x1
        dy = y2 - y1
        perimeter += math.sqrt(dx*dx + dy*dy)
    
    area = abs(area) / 2
    
    properties['calculated_area'] = area
    properties['calculated_perimeter'] = perimeter
```

### 3. Feature Splitting

```python
# Split MultiPoint into individual Point features
if get_geometry_type(geometry) == "MultiPoint":
    coords = get_coordinates(geometry)
    
    # Create multiple features
    features = []
    for i, point_coords in enumerate(coords):
        new_feature = {
            "type": "Feature",
            "properties": {
                **properties,  # Copy existing properties
                "part_number": i + 1,
                "total_parts": len(coords)
            },
            "geometry": create_point(point_coords[0], point_coords[1])
        }
        features.append(new_feature)
    
    # Return multiple features
    return features
```

### 4. Coordinate Transformation

```python
# Simple coordinate transformation example
def transform_coordinates(x, y):
    # Example: Shift coordinates by offset
    return x + 0.001, y + 0.001

# Apply transformation based on geometry type
geom_type = get_geometry_type(geometry)

if geom_type == "Point":
    coords = get_coordinates(geometry)
    new_x, new_y = transform_coordinates(coords[0], coords[1])
    geometry = create_point(new_x, new_y)
    
elif geom_type == "LineString":
    coords = get_coordinates(geometry)
    new_coords = [transform_coordinates(x, y) for x, y in coords]
    geometry = create_linestring(new_coords)
    
elif geom_type == "Polygon":
    coords = get_coordinates(geometry)
    new_rings = []
    for ring in coords:
        new_ring = [transform_coordinates(x, y) for x, y in ring]
        new_rings.append(new_ring)
    geometry = create_polygon(new_rings)

properties['transformed'] = True
```

## Advanced Usage

### Working with External Libraries

While the processor doesn't include external geospatial libraries by default, you can install and use them in your Python environment:

```python
# Example using shapely (if installed in Python environment)
try:
    from shapely.geometry import shape, mapping
    from shapely.ops import transform
    
    # Convert to Shapely geometry
    shapely_geom = shape(geometry)
    
    # Perform operations
    buffered = shapely_geom.buffer(0.001)
    simplified = shapely_geom.simplify(0.0001)
    
    # Convert back to GeoJSON
    geometry = mapping(buffered)
    
except ImportError:
    # Fallback to built-in functions
    properties['warning'] = 'Shapely not available, using basic operations'
```

### Error Handling

```python
try:
    # Your geospatial processing code
    coords = get_coordinates(geometry)
    if not coords:
        raise ValueError("Invalid geometry coordinates")
        
    # Process coordinates...
    
except Exception as e:
    # Log error and set properties
    properties['processing_error'] = str(e)
    properties['error_type'] = type(e).__name__
    # Keep original geometry unchanged
```

## Best Practices

1. **Validate Input**: Always check geometry type and coordinates before processing
2. **Handle Edge Cases**: Consider empty geometries, invalid coordinates, etc.
3. **Preserve Data**: Copy existing properties when creating new features
4. **Use Helper Functions**: Prefer the built-in helper functions for geometry creation
5. **Error Handling**: Implement proper error handling to prevent workflow failures
6. **Performance**: For large datasets, prefer simple operations over complex calculations

## Integration with Re:Earth Flow

The Python Script Processor integrates seamlessly with other Re:Earth Flow actions:

- **Input**: Receives features from upstream actions (readers, transformers, etc.)
- **Output**: Sends processed features to downstream actions (writers, filters, etc.)
- **Chaining**: Can be chained with other processors for complex workflows
- **Parallel Processing**: Each feature is processed independently

## Limitations

- Only 2D and 3D geometries are currently supported
- CityGML geometries are passed through without modification
- External library support depends on your Python environment
- Processing is single-threaded per feature (but features are processed in parallel)

## Example Workflows

See the `tests/fixture/workflow/python/` directory for complete example workflows demonstrating:
- Basic geometry transformations
- Spatial analysis and calculations
- Feature splitting and merging
- Coordinate transformations
