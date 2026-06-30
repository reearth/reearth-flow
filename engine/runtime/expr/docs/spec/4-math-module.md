# (DRAFT) Math Module

The `math` module provides mathematical functions and constants.

## math.pi

The mathematical constant pi.

## math.e

The mathematical constant e.

## math.abs

`math.abs(x)` returns the absolute value of `x`.

## math.floor

`math.floor(x)` returns the largest integer value less than or equal to `x`.

## math.ceil

`math.ceil(x)` returns the smallest integer value greater than or equal to `x`.

## math.round

`math.round(x)` returns `x` rounded to the nearest integer.

> Note: round behavior (away-from-zero or banker's round) is not stabilized by this spec.

## math.sqrt

`math.sqrt(x)` returns the square root of `x`.

## math.exp

`math.exp(x)` returns e raised to the power `x`.

## math.log

`math.log(x)` returns the natural logarithm of `x`.

`math.log(x, base)` returns the logarithm of `x` in the given `base`.

## math.log2

`math.log2(x)` returns the base-2 logarithm of `x`.

## math.log10

`math.log10(x)` returns the base-10 logarithm of `x`.

## math.sin

`math.sin(x)` returns the sine of `x` in radians.

## math.cos

`math.cos(x)` returns the cosine of `x` in radians.

## math.tan

`math.tan(x)` returns the tangent of `x` in radians.

## math.asin

`math.asin(x)` returns the arcsine of `x` in radians.

## math.acos

`math.acos(x)` returns the arccosine of `x` in radians.

## math.atan

`math.atan(x)` returns the arctangent of `x` in radians.

## math.atan2

`math.atan2(y, x)` returns the angle in radians between the positive x-axis and the point `(x, y)`.
The return value is in the range `[-pi, pi]`.

## math.radians

`math.radians(x)` converts `x` from degrees to radians.

## math.degrees

`math.degrees(x)` converts `x` from radians to degrees.

## math.inf

The floating-point positive infinity.

## math.nan

The floating-point not-a-number value.

## math.is_inf

`math.is_inf(x)` returns `true` if `x` is positive or negative infinity, `false` otherwise.

## math.is_nan

`math.is_nan(x)` returns `true` if `x` is not-a-number, `false` otherwise.

## math.is_finite

`math.is_finite(x)` returns `true` if `x` is neither infinite nor not-a-number, `false` otherwise.
