# AttributeMapper: Keep Existing Attributes

## Overview

The `AttributeMapper` action transforms feature attributes using expressions and mappings. As of this release, it supports a new parameter `keepExistingAttributes` that controls whether existing attributes are preserved or replaced.

## Parameter: `keepExistingAttributes`

**Type**: Boolean
**Default**: `false` (for backward compatibility)

### Behavior

- **`false` (default)**: Replaces ALL attributes with only the mapped attributes
  - **Use for**: Projection/selection workflows, extracting specific fields for reports, creating clean output datasets
  - **Equivalent to**: SQL `SELECT` - explicitly choose output columns

- **`true`**: Preserves ALL existing attributes and adds/overwrites only the mapped attributes
  - **Use for**: Adding calculated fields, chaining multiple mappers, pipeline-style processing
  - **Equivalent to**: SQL `UPDATE/ALTER TABLE ADD COLUMN` - augment existing data

## Examples

### Example 1: Projection Workflow (Default Behavior)

Extract only specific fields for CSV export:

```yaml
action: AttributeMapper
with:
  keepExistingAttributes: false  # or omit (default)
  mappers:
    - attribute: building_id
      valueAttribute: gmlId
    - attribute: height
      valueAttribute: measuredHeight
    - attribute: area_m2
      expr: |
        env.get("__value").geometry.area()
```

**Input Feature**:
```
{
  "gmlId": "BLD_123",
  "measuredHeight": 25.5,
  "geometry": {...},
  "cityCode": "13101",
  "buildingType": "residential",
  ... (20 other attributes)
}
```

**Output Feature** (only 3 attributes):
```
{
  "building_id": "BLD_123",
  "height": 25.5,
  "area_m2": 150.25
}
```

### Example 2: Augmentation Workflow (New Behavior)

Add calculated fields to existing feature:

```yaml
action: AttributeMapper
with:
  keepExistingAttributes: true
  mappers:
    - attribute: height_category
      expr: |
        let h = env.get("__value").measuredHeight;
        if h < 10.0 { "low" }
        else if h < 30.0 { "medium" }
        else { "high" }
```

**Input Feature**:
```
{
  "gmlId": "BLD_123",
  "measuredHeight": 25.5,
  "cityCode": "13101",
  ... (all other attributes)
}
```

**Output Feature** (all original + 1 new):
```
{
  "gmlId": "BLD_123",
  "measuredHeight": 25.5,
  "cityCode": "13101",
  ... (all other attributes)
  "height_category": "medium"  ← NEW
}
```

### Example 3: Chained Mappers (Pipeline Processing)

Build complex calculations step-by-step:

```yaml
# Step 1: Calculate base radiation
- action: AttributeMapper
  with:
    keepExistingAttributes: true
    mappers:
      - attribute: planar_radiation
        expr: |
          let hours = env.get("__value").daylight_hours;
          let altitude = env.get("__value").solar_altitude_deg;
          hours * math::sin(math::to_radians(altitude)) * (2.0 / math::pi())

# Step 2: Calculate surface radiation (using planar_radiation from step 1)
- action: AttributeMapper
  with:
    keepExistingAttributes: true
    mappers:
      - attribute: surface_radiation
        expr: |
          let planar = env.get("__value").planar_radiation;
          let area = env.get("__value").roof_area_m2;
          planar * area

# Step 3: Calculate power generation (using surface_radiation from step 2)
- action: AttributeMapper
  with:
    keepExistingAttributes: true
    mappers:
      - attribute: daily_power_kwh
        expr: |
          let radiation = env.get("__value").surface_radiation;
          radiation * 0.167 * 0.8 * 0.01
```

**Result**: Each step adds its calculated field while preserving all previous fields.

## Comparison with AttributeManager

| Feature | AttributeMapper | AttributeManager |
|---------|----------------|------------------|
| **Purpose** | Transform/project attributes using expressions | Manage attribute lifecycle |
| **Syntax** | Simple attribute → expression mapping | Explicit operations (Create/Convert/Rename/Remove) |
| **Best for** | Calculations, projections, pipelines | Complex management with renames/removes |
| **Preserve existing** | Optional (`keepExistingAttributes`) | Always (by default) |
| **When to use** | Most transformation workflows | When you need rename/remove operations |

**Rule of thumb**: Use `AttributeMapper` with `keepExistingAttributes: true` for 90% of workflows. Use `AttributeManager` when you specifically need `Rename` or `Remove` operations.

## See Also

- [AttributeManager Documentation](./attributemapper-guide.md)
- [Expression System Reference](./expression-math-functions.md)
- [FME to Flow Migration Guide](./fme-to-flow-expressions.md)
