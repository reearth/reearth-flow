# Solar Radiation and Power Generation Workflow

This workflow demonstrates the complete calculation of solar radiation and power generation potential for building roof surfaces using Re:Earth Flow's `math::` module.

## Overview

The workflow implements **all 8 FME ExpressionEvaluator expressions** used in solar panel analysis:

1. ✅ Sunrise time to TIMEVALUE
2. ✅ Sunset time to TIMEVALUE
3. ✅ Solar noon time to TIMEVALUE
4. ✅ Planar solar radiation calculation
5. ✅ Incidence angle calculation
6. ✅ Surface solar radiation
7. ✅ Power generation potential
8. ✅ Adjusted solar potential (with shadow/reflection)

## Workflow Structure

```
FeatureCreator (Test Data)
  ↓
TimeConversion (Expressions 1-3)
  ↓
PlanarRadiation (Expression 4)
  ↓
IncidenceAngle (Expression 5)
  ↓
SurfaceRadiation (Expression 6)
  ↓
PowerGeneration (Expression 7)
  ↓
FinalPotential (Expression 8)
  ↓
FileWriter (CSV Output)
```

## Running the Workflow

```bash
# From the engine directory
cargo run --package reearth-flow-cli -- run \
  --workflow runtime/examples/fixture/workflow/solar-radiation/workflow.yml

# Output will be created in the current directory:
# ./solar-radiation-results.csv
```

**Alternative**: Run from project root:
```bash
cd engine
cargo run --package reearth-flow-cli -- run \
  --workflow ./runtime/examples/fixture/workflow/solar-radiation/workflow.yml
```

## Expected Output

The workflow creates `solar-radiation-results.csv` with these calculated attributes:

| Attribute | Description | Unit |
|-----------|-------------|------|
| `sunrise_seconds` | Sunrise time | seconds since midnight |
| `sunset_seconds` | Sunset time | seconds since midnight |
| `solar_noon_seconds` | Solar noon time | seconds since midnight |
| `planar_radiation_rate` | Daily planar radiation | kWh/m²/day |
| `cos_incidence_angle` | Cosine of incidence angle | dimensionless (0-1) |
| `surface_solar_radiation` | Total surface radiation | kWh/day |
| `daily_power_generation_kwh` | Daily power generation | kWh/day |
| `adjusted_solar_potential_kwh` | Adjusted potential | kWh/day |
| `annual_potential_kwh` | Annual potential | kWh/year |
| `is_viable_for_panels` | Viability flag | boolean |
| `panel_priority` | Priority rating | high/medium/low/not-viable |

## Sample Results

### Building 1 (South-facing, 30° slope, 100m²)
```
sunrise_seconds: 22500 (6:15 AM)
sunset_seconds: 67500 (6:45 PM)
planar_radiation_rate: 6.45 kWh/m²/day
cos_incidence_angle: 0.9947
surface_solar_radiation: 641.20 kWh/day
daily_power_generation_kwh: 0.8566 kWh/day
adjusted_solar_potential_kwh: 0.7538 kWh/day
annual_potential_kwh: 275.15 kWh/year
panel_priority: not-viable
```

### Building 2 (SSE-facing, 20° slope, 150m²)
```
sunrise_seconds: 22800 (6:20 AM)
sunset_seconds: 67200 (6:40 PM)
planar_radiation_rate: 6.36 kWh/m²/day
cos_incidence_angle: 0.9549
surface_solar_radiation: 911.00 kWh/day
daily_power_generation_kwh: 1.2171 kWh/day
adjusted_solar_potential_kwh: 1.0863 kWh/day
annual_potential_kwh: 396.49 kWh/year
panel_priority: low
```

## Input Parameters

### Per-Feature Attributes
- `sunrise_hour`, `sunrise_minute` - Sunrise time
- `sunset_hour`, `sunset_minute` - Sunset time
- `solar_noon_hour`, `solar_noon_minute`, `solar_noon_second` - Solar noon time
- `solar_altitude_deg` - Solar altitude angle (degrees)
- `roof_slope_degrees` - Roof inclination (0° = flat, 90° = vertical)
- `roof_azimuth_degrees` - Roof orientation (0° = North, 180° = South)
- `roof_area_m2` - Roof surface area (square meters)
- `shadow_impact_score` - Shadow loss factor (0.0-1.0)
- `reflection_intensity` - Reflection gain factor (0.0-1.0)

### Global Parameters (workflow-level)
- `solar_altitude_degrees: 54.1` - Solar altitude at noon
- `panel_capacity_factor: 0.167` - Panel capacity rating
- `panel_efficiency: 0.8` - System efficiency
- `unit_adjustment: 0.01` - Unit conversion factor

## Mathematical Functions Used

This workflow demonstrates the use of:
- `math::sin()`, `math::cos()` - Trigonometry
- `math::to_radians()` - Angle conversion
- `math::pi()` - Mathematical constant
- `math::max()` - Value comparison

## See Also

- [FME to Flow Expression Conversion Guide](../../../../docs/fme-to-flow-expressions.md)
- [Math Functions Reference](../../../../docs/expression-math-functions.md)
- [AttributeMapper User Manual](../../../../docs/attributemapper-guide.md)
