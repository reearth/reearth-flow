# Mathematical Functions Reference

This document describes all mathematical functions available in Re:Earth Flow's expression system through the `math::` module.

## Overview

The `math::` module provides comprehensive mathematical functions for use in actions like `AttributeMapper`, `FeatureFilter`, and anywhere Rhai expressions are supported.

## Table of Contents

- [Mathematical Constants](#mathematical-constants)
- [Trigonometric Functions](#trigonometric-functions)
- [Inverse Trigonometric Functions](#inverse-trigonometric-functions)
- [Hyperbolic Functions](#hyperbolic-functions)
- [Inverse Hyperbolic Functions](#inverse-hyperbolic-functions)
- [Angle Conversion](#angle-conversion)
- [Exponential & Logarithmic Functions](#exponential--logarithmic-functions)
- [Power & Root Functions](#power--root-functions)
- [Comparison & Selection](#comparison--selection)
- [Rounding Functions](#rounding-functions)
- [Sign Manipulation](#sign-manipulation)
- [Real-World Examples](#real-world-examples)

---

## Mathematical Constants

### `math::PI` → f64

The mathematical constant π (pi).

**Value:** 3.14159265358979323846

**Example:**

```rhai
let circumference = 2.0 * math::PI * radius;
let half_circle = math::PI * radius;
```

---

### `math::E` → f64

Euler's number (the base of natural logarithms).

**Value:** 2.71828182845904523536

**Example:**

```rhai
let natural_exp = math::pow(math::E, 2.0);
```

---

### `math::TAU` → f64

The mathematical constant τ (tau), equal to 2π.

**Value:** 6.28318530717958647692

**Why use tau?** Some mathematicians argue that τ is more natural than π because:

- One full circle = τ radians (instead of 2π radians)
- C = τr (circumference formula) instead of C = 2πr
- Many trigonometric formulas become simpler

**Example:**

```rhai
let full_circle = math::TAU;  // One complete rotation = τ radians
let half_circle = math::TAU / 2.0;  // Same as π
let quarter_circle = math::TAU / 4.0;  // Same as π/2

// Circumference using tau
let circumference = math::TAU * radius;  // Simpler than 2πr
```

---

## Trigonometric Functions

All trigonometric functions work with **radians**. Use [`math::to_radians()`](#mathto_radiansdegrees--f64) to convert from degrees.

### `math::sin(x)` → f64

Computes the sine of an angle (in radians).

**Parameters:**

- `x` (f64): Angle in radians

**Returns:** Sine of x, in the range [-1, 1]

**Example:**

```rhai
// Calculate sin(30°)
let result = math::sin(math::to_radians(30.0));  // Returns 0.5

// Calculate sin(π/2)
let result2 = math::sin(math::PI / 2.0);  // Returns 1.0
```

---

### `math::cos(x)` → f64

Computes the cosine of an angle (in radians).

**Parameters:**

- `x` (f64): Angle in radians

**Returns:** Cosine of x, in the range [-1, 1]

**Example:**

```rhai
// Calculate cos(60°)
let result = math::cos(math::to_radians(60.0));  // Returns 0.5

// Calculate cos(0)
let result2 = math::cos(0.0);  // Returns 1.0
```

---

### `math::tan(x)` → f64

Computes the tangent of an angle (in radians).

**Parameters:**

- `x` (f64): Angle in radians

**Returns:** Tangent of x

**Example:**

```rhai
// Calculate tan(45°)
let result = math::tan(math::to_radians(45.0));  // Returns 1.0

// Calculate tan(π/4)
let result2 = math::tan(math::PI / 4.0);  // Returns 1.0
```

---

## Inverse Trigonometric Functions

All inverse trigonometric functions return results in **radians**. Use [`math::to_degrees()`](#mathto_degreesradians--f64) to convert to degrees.

### `math::asin(x)` → f64

Computes the arcsine (inverse sine) of a number.

**Parameters:**

- `x` (f64): Value in the range [-1, 1]

**Returns:** Arcsine of x in radians, in the range [-π/2, π/2]

**Note:** Returns NaN if x is outside [-1, 1]

**Example:**

```rhai
// Find the angle whose sine is 0.5
let angle_rad = math::asin(0.5);  // Returns π/6 radians
let angle_deg = math::to_degrees(angle_rad);  // Returns 30°
```

---

### `math::acos(x)` → f64

Computes the arccosine (inverse cosine) of a number.

**Parameters:**

- `x` (f64): Value in the range [-1, 1]

**Returns:** Arccosine of x in radians, in the range [0, π]

**Note:** Returns NaN if x is outside [-1, 1]

**Example:**

```rhai
// Find the angle whose cosine is 0.5
let angle_rad = math::acos(0.5);  // Returns π/3 radians
let angle_deg = math::to_degrees(angle_rad);  // Returns 60°
```

---

### `math::atan(x)` → f64

Computes the arctangent (inverse tangent) of a number.

**Parameters:**

- `x` (f64): Any real number

**Returns:** Arctangent of x in radians, in the range [-π/2, π/2]

**Example:**

```rhai
// Find the angle whose tangent is 1
let angle_rad = math::atan(1.0);  // Returns π/4 radians
let angle_deg = math::to_degrees(angle_rad);  // Returns 45°
```

---

### `math::atan2(y, x)` → f64

Computes the four-quadrant arctangent of y and x.

**Parameters:**

- `y` (f64): Y coordinate
- `x` (f64): X coordinate

**Returns:** Angle in radians in the range [-π, π]

**Why use atan2?** Unlike `atan(y/x)`, `atan2` uses the signs of both arguments to determine the correct quadrant of the result.

**Example:**

```rhai
// Quadrant I (both positive)
let angle1 = math::atan2(1.0, 1.0);  // Returns π/4 (45°)

// Quadrant II (y positive, x negative)
let angle2 = math::atan2(1.0, -1.0);  // Returns 3π/4 (135°)

// Calculate direction from origin to point
let direction = math::atan2(building_y, building_x);
```

---

## Hyperbolic Functions

Hyperbolic functions are analogs of trigonometric functions but for hyperbolas instead of circles. They're used in physics, engineering, and modeling natural phenomena like hanging cables (catenary curves).

### `math::sinh(x)` → f64

Computes the hyperbolic sine of a number.

**Parameters:**

- `x` (f64): Any real number

**Returns:** Hyperbolic sine of x

**Formula:** `sinh(x) = (e^x - e^(-x)) / 2`

**Example:**

```rhai
let result1 = math::sinh(0.0);  // Returns 0.0
let result2 = math::sinh(1.0);  // Returns ~1.175
let result3 = math::sinh(-1.0); // Returns ~-1.175
```

---

### `math::cosh(x)` → f64

Computes the hyperbolic cosine of a number.

**Parameters:**

- `x` (f64): Any real number

**Returns:** Hyperbolic cosine of x (always ≥ 1)

**Formula:** `cosh(x) = (e^x + e^(-x)) / 2`

**Example:**

```rhai
let result1 = math::cosh(0.0);  // Returns 1.0
let result2 = math::cosh(1.0);  // Returns ~1.543
let result3 = math::cosh(-1.0); // Returns ~1.543 (even function)

// Catenary curve (hanging cable): y = a * cosh(x/a)
let a = 10.0;
let x = 5.0;
let y = a * math::cosh(x / a);
```

---

### `math::tanh(x)` → f64

Computes the hyperbolic tangent of a number.

**Parameters:**

- `x` (f64): Any real number

**Returns:** Hyperbolic tangent of x, in the range (-1, 1)

**Formula:** `tanh(x) = sinh(x) / cosh(x)`

**Example:**

```rhai
let result1 = math::tanh(0.0);  // Returns 0.0
let result2 = math::tanh(1.0);  // Returns ~0.762
let result3 = math::tanh(-1.0); // Returns ~-0.762

// Activation function in neural networks
let activation = math::tanh(weighted_sum);
```

---

## Inverse Hyperbolic Functions

### `math::asinh(x)` → f64

Computes the inverse hyperbolic sine of a number.

**Parameters:**

- `x` (f64): Any real number

**Returns:** Inverse hyperbolic sine of x

**Example:**

```rhai
let result1 = math::asinh(0.0);  // Returns 0.0
let result2 = math::asinh(1.0);  // Returns ~0.881

// asinh(sinh(x)) = x
let x = 1.5;
let round_trip = math::asinh(math::sinh(x));  // Returns 1.5
```

---

### `math::acosh(x)` → f64

Computes the inverse hyperbolic cosine of a number.

**Parameters:**

- `x` (f64): Value ≥ 1

**Returns:** Inverse hyperbolic cosine of x

**Note:** Returns NaN if x < 1

**Example:**

```rhai
let result1 = math::acosh(1.0);  // Returns 0.0
let result2 = math::acosh(2.0);  // Returns ~1.317
let result3 = math::acosh(5.0);  // Returns ~2.292
```

---

### `math::atanh(x)` → f64

Computes the inverse hyperbolic tangent of a number.

**Parameters:**

- `x` (f64): Value in the range (-1, 1)

**Returns:** Inverse hyperbolic tangent of x

**Note:** Returns ±infinity if x = ±1, NaN if |x| > 1

**Example:**

```rhai
let result1 = math::atanh(0.0);   // Returns 0.0
let result2 = math::atanh(0.5);   // Returns ~0.549
let result3 = math::atanh(-0.5);  // Returns ~-0.549
```

---

## Angle Conversion

### `math::to_radians(degrees)` → f64

Converts degrees to radians.

**Parameters:**

- `degrees` (f64): Angle in degrees

**Returns:** Angle in radians

**Formula:** `radians = degrees × π / 180`

**Example:**

```rhai
let rad1 = math::to_radians(0.0);    // Returns 0.0
let rad2 = math::to_radians(90.0);   // Returns π/2
let rad3 = math::to_radians(180.0);  // Returns π
let rad4 = math::to_radians(360.0);  // Returns 2π
```

---

### `math::to_degrees(radians)` → f64

Converts radians to degrees.

**Parameters:**

- `radians` (f64): Angle in radians

**Returns:** Angle in degrees

**Formula:** `degrees = radians × 180 / π`

**Example:**

```rhai
let deg1 = math::to_degrees(0.0);            // Returns 0.0
let deg2 = math::to_degrees(math::PI / 2.0);  // Returns 90.0
let deg3 = math::to_degrees(math::PI);        // Returns 180.0
let deg4 = math::to_degrees(2.0 * math::PI);  // Returns 360.0
```

---

## Exponential & Logarithmic Functions

### `math::exp(x)` → f64

Computes e raised to the power of x.

**Parameters:**

- `x` (f64): Any real number

**Returns:** e^x

**Example:**

```rhai
let result1 = math::exp(0.0);  // Returns 1.0
let result2 = math::exp(1.0);  // Returns e ≈ 2.718
let result3 = math::exp(2.0);  // Returns e² ≈ 7.389

// Exponential growth: P(t) = P₀ * e^(rt)
let population = initial_pop * math::exp(growth_rate * time);
```

---

### `math::ln(x)` → f64

Computes the natural logarithm (base e) of a number.

**Parameters:**

- `x` (f64): Value > 0

**Returns:** Natural logarithm of x

**Note:** Returns NaN if x ≤ 0

**Example:**

```rhai
let result1 = math::ln(1.0);      // Returns 0.0
let result2 = math::ln(math::E);  // Returns 1.0
let result3 = math::ln(10.0);     // Returns ~2.303

// Inverse of exp: ln(exp(x)) = x
let x = 2.5;
let round_trip = math::ln(math::exp(x));  // Returns 2.5
```

---

### `math::log(x, base)` → f64

Computes the logarithm of a number with a specified base.

**Parameters:**

- `x` (f64): Value > 0
- `base` (f64): Base of the logarithm (> 0, ≠ 1)

**Returns:** Logarithm of x in the given base

**Note:** Returns NaN if x ≤ 0 or base ≤ 0 or base = 1

**Example:**

```rhai
let result1 = math::log(100.0, 10.0);  // Returns 2.0 (log₁₀(100))
let result2 = math::log(8.0, 2.0);     // Returns 3.0 (log₂(8))
let result3 = math::log(27.0, 3.0);    // Returns 3.0 (log₃(27))

// Change of base: log_b(x) = ln(x) / ln(b)
let log_5_25 = math::log(25.0, 5.0);   // Returns 2.0
```

---

### `math::log10(x)` → f64

Computes the base-10 logarithm of a number.

**Parameters:**

- `x` (f64): Value > 0

**Returns:** Base-10 logarithm of x

**Note:** Returns NaN if x ≤ 0

**Example:**

```rhai
let result1 = math::log10(1.0);     // Returns 0.0
let result2 = math::log10(10.0);    // Returns 1.0
let result3 = math::log10(100.0);   // Returns 2.0
let result4 = math::log10(1000.0);  // Returns 3.0

// Richter scale (earthquakes), pH scale, decibels
let magnitude = math::log10(intensity);
```

---

### `math::log2(x)` → f64

Computes the base-2 logarithm of a number.

**Parameters:**

- `x` (f64): Value > 0

**Returns:** Base-2 logarithm of x

**Note:** Returns NaN if x ≤ 0

**Example:**

```rhai
let result1 = math::log2(1.0);     // Returns 0.0
let result2 = math::log2(2.0);     // Returns 1.0
let result3 = math::log2(8.0);     // Returns 3.0
let result4 = math::log2(1024.0);  // Returns 10.0

// Bits required to represent a number
let bits_needed = math::ceil(math::log2(value));
```

---

### `math::exp_m1(x)` → f64

Computes e^x - 1 with better precision for small values of x.

**Parameters:**

- `x` (f64): Any real number

**Returns:** e^x - 1

**Why use this?** For very small values of x, `exp_m1(x)` is more accurate than `exp(x) - 1` due to floating-point precision limits.

**Example:**

```rhai
let result1 = math::exp_m1(0.0);     // Returns 0.0
let result2 = math::exp_m1(1.0);     // Returns ~1.718 (e - 1)
let result3 = math::exp_m1(0.0001);  // High precision for small x
```

---

### `math::ln_1p(x)` → f64

Computes ln(1 + x) with better precision for small values of x.

**Parameters:**

- `x` (f64): Value > -1

**Returns:** ln(1 + x)

**Note:** Returns NaN if x ≤ -1

**Why use this?** For very small values of x, `ln_1p(x)` is more accurate than `ln(1 + x)` due to floating-point precision limits.

**Example:**

```rhai
let result1 = math::ln_1p(0.0);        // Returns 0.0
let result2 = math::ln_1p(math::E - 1.0);  // Returns 1.0
let result3 = math::ln_1p(0.0001);     // High precision for small x
```

---

## Power & Root Functions

### `math::sqrt(x)` → f64

Computes the square root of a number.

**Parameters:**

- `x` (f64): Non-negative number

**Returns:** Square root of x

**Note:** Returns NaN if x is negative

**Example:**

```rhai
let result1 = math::sqrt(0.0);   // Returns 0.0
let result2 = math::sqrt(4.0);   // Returns 2.0
let result3 = math::sqrt(16.0);  // Returns 4.0
let result4 = math::sqrt(2.0);   // Returns ~1.414

// Pythagorean theorem: c = √(a² + b²)
let c = math::sqrt(math::pow(a, 2.0) + math::pow(b, 2.0));
```

---

### `math::pow(base, exp)` → f64

Raises a number to a floating-point power.

**Parameters:**

- `base` (f64): Base number
- `exp` (f64): Exponent

**Returns:** base<sup>exp</sup>

**Example:**

```rhai
let result1 = math::pow(2.0, 3.0);   // Returns 8.0 (2³)
let result2 = math::pow(10.0, 2.0);  // Returns 100.0 (10²)
let result3 = math::pow(4.0, 0.5);   // Returns 2.0 (√4)
let result4 = math::pow(27.0, 1.0/3.0);  // Returns 3.0 (∛27)

// Calculate area of circle
let area = math::PI * math::pow(radius, 2.0);
```

---

### `math::cbrt(x)` → f64

Computes the cube root of a number.

**Parameters:**

- `x` (f64): Any real number

**Returns:** Cube root of x

**Note:** Unlike `sqrt`, `cbrt` works with negative numbers

**Example:**

```rhai
let result1 = math::cbrt(0.0);   // Returns 0.0
let result2 = math::cbrt(8.0);   // Returns 2.0
let result3 = math::cbrt(27.0);  // Returns 3.0
let result4 = math::cbrt(-8.0);  // Returns -2.0

// Cbrt is more accurate than pow(x, 1/3)
let cube_root = math::cbrt(value);
```

---

### `math::hypot(x, y)` → f64

Computes the Euclidean distance: √(x² + y²).

**Parameters:**

- `x` (f64): First coordinate
- `y` (f64): Second coordinate

**Returns:** Euclidean distance from origin to (x, y)

**Why use this?** `hypot` is more numerically stable than `sqrt(x² + y²)` for very large or very small values.

**Example:**

```rhai
let result1 = math::hypot(3.0, 4.0);   // Returns 5.0 (3-4-5 triangle)
let result2 = math::hypot(5.0, 12.0);  // Returns 13.0 (5-12-13 triangle)
let result3 = math::hypot(8.0, 15.0);  // Returns 17.0 (8-15-17 triangle)

// Calculate distance between two points
let dx = x2 - x1;
let dy = y2 - y1;
let distance = math::hypot(dx, dy);
```

---

## Comparison & Selection

### `math::abs(x)` → f64

Returns the absolute value of a number.

**Parameters:**

- `x` (f64): Any real number

**Returns:** |x| (always non-negative)

**Example:**

```rhai
let result1 = math::abs(0.0);    // Returns 0.0
let result2 = math::abs(5.0);    // Returns 5.0
let result3 = math::abs(-5.0);   // Returns 5.0
let result4 = math::abs(-3.14);  // Returns 3.14

// Calculate distance between points
let distance = math::abs(x2 - x1);
```

---

### `math::max(a, b)` → f64

Returns the maximum of two numbers.

**Parameters:**

- `a` (f64): First number
- `b` (f64): Second number

**Returns:** The larger of a and b

**Example:**

```rhai
let result1 = math::max(5.0, 10.0);    // Returns 10.0
let result2 = math::max(10.0, 5.0);    // Returns 10.0
let result3 = math::max(-5.0, -10.0);  // Returns -5.0

// Clamp negative values to zero
let positive = math::max(value, 0.0);
```

---

### `math::min(a, b)` → f64

Returns the minimum of two numbers.

**Parameters:**

- `a` (f64): First number
- `b` (f64): Second number

**Returns:** The smaller of a and b

**Example:**

```rhai
let result1 = math::min(5.0, 10.0);    // Returns 5.0
let result2 = math::min(10.0, 5.0);    // Returns 5.0
let result3 = math::min(-5.0, -10.0);  // Returns -10.0

// Clamp to maximum value
let clamped = math::min(value, 100.0);
```

---

## Rounding Functions

### `math::floor(x)` → f64

Returns the largest integer less than or equal to x.

**Parameters:**

- `x` (f64): Any real number

**Returns:** ⌊x⌋ (floor of x)

**Example:**

```rhai
let result1 = math::floor(3.7);   // Returns 3.0
let result2 = math::floor(3.2);   // Returns 3.0
let result3 = math::floor(-3.2);  // Returns -4.0
let result4 = math::floor(-3.7);  // Returns -4.0
let result5 = math::floor(5.0);   // Returns 5.0
```

---

### `math::ceil(x)` → f64

Returns the smallest integer greater than or equal to x.

**Parameters:**

- `x` (f64): Any real number

**Returns:** ⌈x⌉ (ceiling of x)

**Example:**

```rhai
let result1 = math::ceil(3.2);   // Returns 4.0
let result2 = math::ceil(3.7);   // Returns 4.0
let result3 = math::ceil(-3.7);  // Returns -3.0
let result4 = math::ceil(-3.2);  // Returns -3.0
let result5 = math::ceil(5.0);   // Returns 5.0
```

---

### `math::round(x)` → f64

Returns the nearest integer to x. Rounds half-way cases away from zero.

**Parameters:**

- `x` (f64): Any real number

**Returns:** Rounded value of x

**Example:**

```rhai
let result1 = math::round(3.5);   // Returns 4.0
let result2 = math::round(3.4);   // Returns 3.0
let result3 = math::round(3.6);   // Returns 4.0
let result4 = math::round(-3.5);  // Returns -4.0
let result5 = math::round(-3.4);  // Returns -3.0
```

---

## Sign Manipulation

### `math::copysign(x, y)` → f64

Returns a value with the magnitude of x and the sign of y.

**Parameters:**

- `x` (f64): Value providing the magnitude
- `y` (f64): Value providing the sign

**Returns:** |x| with the sign of y

**Example:**

```rhai
let result1 = math::copysign(5.0, 1.0);   // Returns 5.0 (positive)
let result2 = math::copysign(5.0, -1.0);  // Returns -5.0 (negative)
let result3 = math::copysign(-5.0, 1.0);  // Returns 5.0 (positive)
let result4 = math::copysign(-5.0, -1.0); // Returns -5.0 (negative)

// Useful for ensuring consistent signs in calculations
let values = [5.0, -5.0, 10.0, -10.0];
let reference_sign = -1.0;

// Make all values have the same sign as reference
for val in values {
    let adjusted = math::copysign(val, reference_sign);
    // All adjusted values will be negative
}
```

---

## Real-World Examples

### Example 1: Solar Radiation Calculation

Calculate planar solar radiation based on sunrise/sunset times and solar altitude.

```rhai
mappers:
  - attribute: planar_solar_radiation
    expr: |
      let sunrise = env.get("__value")["sunrise_seconds"];
      let sunset = env.get("__value")["sunset_seconds"];
      let altitude_deg = env.get("__value")["solar_altitude_degrees"];

      // Calculate daylight fraction
      let daylight_fraction = (sunset / 86400.0) - (sunrise / 86400.0);

      // Calculate radiation (simplified model)
      let radiation = daylight_fraction * 24.0
        * math::sin(math::to_radians(altitude_deg))
        * (2.0 / math::PI);

      radiation
```

---

### Example 2: Incidence Angle for Solar Panels

Calculate the cosine of the incidence angle between sunlight and a tilted surface.

```rhai
mappers:
  - attribute: cos_incidence_angle
    expr: |
      let solar_altitude = 54.1;  // Solar altitude in degrees
      let roof_slope = env.get("__value").roof_slope_degrees;
      let roof_azimuth = env.get("__value").roof_azimuth_degrees;

      // Convert to radians
      let solar_alt_rad = math::to_radians(solar_altitude);
      let slope_rad = math::to_radians(roof_slope);
      let azimuth_rad = math::to_radians(180.0 - roof_azimuth);

      // Calculate cosine of incidence angle
      let cos_angle = math::sin(solar_alt_rad) * math::cos(slope_rad)
        + math::cos(solar_alt_rad) * math::sin(slope_rad) * math::cos(azimuth_rad);

      cos_angle
```

---

### Example 3: Solar Panel Energy Calculation

Calculate potential solar energy generation for a roof surface.

```rhai
mappers:
  - attribute: daily_solar_energy_kwh
    expr: |
      let radiation = env.get("__value").daily_radiation_rate;
      let cos_angle = env.get("__value").cos_incidence_angle;
      let area_m2 = env.get("__value").roof_area_m2;

      // Only consider positive incidence angles (sunlit surfaces)
      let effective_cos = math::max(cos_angle, 0.0);

      // Calculate daily solar radiation (kWh/day)
      let solar_radiation = radiation * effective_cos * area_m2;

      // Calculate energy generation
      // 0.167 = panel capacity factor
      // 0.8 = panel efficiency
      // 0.01 = unit adjustment
      let energy = solar_radiation * 0.167 * 0.8 * 0.01;

      energy
```

---

### Example 4: Distance Calculation

Calculate the Euclidean distance between two points.

```rhai
mappers:
  - attribute: distance_to_center
    expr: |
      let center_x = 0.0;
      let center_y = 0.0;
      let building_x = env.get("__value").x_coordinate;
      let building_y = env.get("__value").y_coordinate;

      // Pythagorean theorem: d = √((x₂-x₁)² + (y₂-y₁)²)
      let dx = building_x - center_x;
      let dy = building_y - center_y;
      let distance = math::sqrt(math::pow(dx, 2.0) + math::pow(dy, 2.0));

      distance
```

---

### Example 5: Bearing/Direction Calculation

Calculate the bearing (compass direction) from one point to another.

```rhai
mappers:
  - attribute: bearing_degrees
    expr: |
      let from_x = env.get("__value").start_x;
      let from_y = env.get("__value").start_y;
      let to_x = env.get("__value").end_x;
      let to_y = env.get("__value").end_y;

      // Calculate direction using atan2
      let dx = to_x - from_x;
      let dy = to_y - from_y;
      let angle_rad = math::atan2(dy, dx);

      // Convert to degrees and normalize to 0-360
      let bearing = math::to_degrees(angle_rad);
      if bearing < 0.0 {
        bearing = bearing + 360.0;
      }

      bearing
```

---

### Example 6: Building Height Category

Categorize buildings by height using mathematical thresholds.

```rhai
mappers:
  - attribute: height_category
    expr: |
      let height = env.get("__value").building_height_m;

      // Ensure height is positive
      let abs_height = math::abs(height);

      // Categorize
      if abs_height > 100.0 {
        "high-rise"
      } else if abs_height > 30.0 {
        "mid-rise"
      } else {
        "low-rise"
      }
```

---

### Example 7: Normalize Values to 0-1 Range

```rhai
mappers:
  - attribute: normalized_value
    expr: |
      let value = env.get("__value").raw_value;
      let min_val = env.get("__value").min_value;
      let max_val = env.get("__value").max_value;

      // Clamp to range
      let clamped = math::max(math::min(value, max_val), min_val);

      // Normalize to [0, 1]
      let normalized = (clamped - min_val) / (max_val - min_val);

      normalized
```

---

## Function Summary Table

| Category                      | Functions                                                            |
| ----------------------------- | -------------------------------------------------------------------- |
| **Constants**                 | `PI`, `E`, `TAU`                                                     |
| **Trigonometry**              | `sin()`, `cos()`, `tan()`                                            |
| **Inverse Trig**              | `asin()`, `acos()`, `atan()`, `atan2()`                              |
| **Hyperbolic**                | `sinh()`, `cosh()`, `tanh()`                                         |
| **Inverse Hyperbolic**        | `asinh()`, `acosh()`, `atanh()`                                      |
| **Angle Conversion**          | `to_radians()`, `to_degrees()`                                       |
| **Exponential & Logarithmic** | `exp()`, `ln()`, `log()`, `log10()`, `log2()`, `exp_m1()`, `ln_1p()` |
| **Power & Roots**             | `sqrt()`, `pow()`, `cbrt()`, `hypot()`                               |
| **Comparison**                | `abs()`, `max()`, `min()`                                            |
| **Rounding**                  | `floor()`, `ceil()`, `round()`                                       |
| **Sign Manipulation**         | `copysign()`                                                         |

**Total:** 40 mathematical functions (19 Tier 1 + 19 Tier 2 + 2 Tier 3)

---

## Performance Notes

- All functions are inline and compiled at workflow initialization
- No runtime performance penalty compared to native Rust code
- Trigonometric functions use hardware-optimized implementations
- NaN (Not a Number) is returned for invalid operations (e.g., `sqrt(-1)`)

---

## See Also

- [AttributeMapper User Manual](./attributemapper-guide.md)
- [Expression System Overview](./expression-overview.md)
- [All Available Functions](./expression-functions.md)
