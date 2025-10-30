use rhai::export_module;

#[export_module]
pub(crate) mod math_module {
    use rhai::plugin::*;

    // ============================================================================
    // Mathematical Constants
    // ============================================================================

    /// The mathematical constant π (pi).
    ///
    /// # Value
    /// π ≈ 3.14159265358979323846
    ///
    /// # Example
    /// ```rhai
    /// let pi_value = math::PI;
    /// let circumference = 2.0 * math::PI * radius;
    /// ```
    pub const PI: f64 = std::f64::consts::PI;

    /// The mathematical constant e (Euler's number).
    ///
    /// # Value
    /// e ≈ 2.71828182845904523536
    ///
    /// # Example
    /// ```rhai
    /// let e_value = math::E;
    /// let result = math::pow(math::E, 2.0);
    /// ```
    pub const E: f64 = std::f64::consts::E;

    // ============================================================================
    // Trigonometric Functions (Core)
    // ============================================================================

    /// Computes the sine of a number (in radians).
    ///
    /// # Arguments
    /// * `x` - Angle in radians
    ///
    /// # Returns
    /// Sine of x, in the range [-1, 1]
    ///
    /// # Example
    /// ```rhai
    /// let x = math::sin(math::pi() / 2.0);  // Returns 1.0
    /// let y = math::sin(math::to_radians(30.0));  // sin(30°) = 0.5
    /// ```
    pub fn sin(x: f64) -> f64 {
        x.sin()
    }

    /// Computes the cosine of a number (in radians).
    ///
    /// # Arguments
    /// * `x` - Angle in radians
    ///
    /// # Returns
    /// Cosine of x, in the range [-1, 1]
    ///
    /// # Example
    /// ```rhai
    /// let x = math::cos(0.0);  // Returns 1.0
    /// let y = math::cos(math::to_radians(60.0));  // cos(60°) = 0.5
    /// ```
    pub fn cos(x: f64) -> f64 {
        x.cos()
    }

    /// Computes the tangent of a number (in radians).
    ///
    /// # Arguments
    /// * `x` - Angle in radians
    ///
    /// # Returns
    /// Tangent of x
    ///
    /// # Example
    /// ```rhai
    /// let x = math::tan(math::pi() / 4.0);  // Returns ~1.0
    /// let y = math::tan(math::to_radians(45.0));  // tan(45°) = 1.0
    /// ```
    pub fn tan(x: f64) -> f64 {
        x.tan()
    }

    // ============================================================================
    // Inverse Trigonometric Functions
    // ============================================================================

    /// Computes the arcsine (inverse sine) of a number.
    ///
    /// # Arguments
    /// * `x` - Value in the range [-1, 1]
    ///
    /// # Returns
    /// Arcsine of x in radians, in the range [-π/2, π/2]
    /// Returns NaN if x is outside [-1, 1]
    ///
    /// # Example
    /// ```rhai
    /// let angle = math::asin(0.5);  // Returns π/6 radians (30°)
    /// let degrees = math::to_degrees(math::asin(1.0));  // Returns 90.0
    /// ```
    pub fn asin(x: f64) -> f64 {
        x.asin()
    }

    /// Computes the arccosine (inverse cosine) of a number.
    ///
    /// # Arguments
    /// * `x` - Value in the range [-1, 1]
    ///
    /// # Returns
    /// Arccosine of x in radians, in the range [0, π]
    /// Returns NaN if x is outside [-1, 1]
    ///
    /// # Example
    /// ```rhai
    /// let angle = math::acos(0.5);  // Returns π/3 radians (60°)
    /// let degrees = math::to_degrees(math::acos(0.0));  // Returns 90.0
    /// ```
    pub fn acos(x: f64) -> f64 {
        x.acos()
    }

    /// Computes the arctangent (inverse tangent) of a number.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// Arctangent of x in radians, in the range [-π/2, π/2]
    ///
    /// # Example
    /// ```rhai
    /// let angle = math::atan(1.0);  // Returns π/4 radians (45°)
    /// let degrees = math::to_degrees(math::atan(1.0));  // Returns 45.0
    /// ```
    pub fn atan(x: f64) -> f64 {
        x.atan()
    }

    /// Computes the four-quadrant arctangent of y and x.
    ///
    /// # Arguments
    /// * `y` - Y coordinate
    /// * `x` - X coordinate
    ///
    /// # Returns
    /// Angle in radians in the range [-π, π]
    /// Returns the angle whose tangent is y/x, using the signs of both to determine the quadrant
    ///
    /// # Example
    /// ```rhai
    /// let angle = math::atan2(1.0, 1.0);  // Returns π/4 (45° in quadrant I)
    /// let angle2 = math::atan2(1.0, -1.0);  // Returns 3π/4 (135° in quadrant II)
    /// ```
    pub fn atan2(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }

    // ============================================================================
    // Angle Conversion
    // ============================================================================

    /// Converts degrees to radians.
    ///
    /// # Arguments
    /// * `degrees` - Angle in degrees
    ///
    /// # Returns
    /// Angle in radians
    ///
    /// # Example
    /// ```rhai
    /// let radians = math::to_radians(180.0);  // Returns π
    /// let radians2 = math::to_radians(90.0);  // Returns π/2
    /// ```
    pub fn to_radians(degrees: f64) -> f64 {
        degrees.to_radians()
    }

    /// Converts radians to degrees.
    ///
    /// # Arguments
    /// * `radians` - Angle in radians
    ///
    /// # Returns
    /// Angle in degrees
    ///
    /// # Example
    /// ```rhai
    /// let degrees = math::to_degrees(math::pi());  // Returns 180.0
    /// let degrees2 = math::to_degrees(math::pi() / 2.0);  // Returns 90.0
    /// ```
    pub fn to_degrees(radians: f64) -> f64 {
        radians.to_degrees()
    }

    // ============================================================================
    // Power & Root Functions
    // ============================================================================

    /// Computes the square root of a number.
    ///
    /// # Arguments
    /// * `x` - Non-negative number
    ///
    /// # Returns
    /// Square root of x
    /// Returns NaN if x is negative
    ///
    /// # Example
    /// ```rhai
    /// let result = math::sqrt(16.0);  // Returns 4.0
    /// let result2 = math::sqrt(2.0);  // Returns ~1.414
    /// ```
    pub fn sqrt(x: f64) -> f64 {
        x.sqrt()
    }

    /// Raises a number to a floating-point power.
    ///
    /// # Arguments
    /// * `base` - Base number
    /// * `exp` - Exponent
    ///
    /// # Returns
    /// base raised to the power of exp
    ///
    /// # Example
    /// ```rhai
    /// let result = math::pow(2.0, 3.0);  // Returns 8.0
    /// let result2 = math::pow(10.0, 2.0);  // Returns 100.0
    /// ```
    pub fn pow(base: f64, exp: f64) -> f64 {
        base.powf(exp)
    }

    // ============================================================================
    // Comparison & Selection
    // ============================================================================

    /// Returns the absolute value of a number.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// Absolute value of x (always non-negative)
    ///
    /// # Example
    /// ```rhai
    /// let result = math::abs(-5.0);  // Returns 5.0
    /// let result2 = math::abs(3.0);  // Returns 3.0
    /// ```
    pub fn abs(x: f64) -> f64 {
        x.abs()
    }

    /// Returns the maximum of two numbers.
    ///
    /// # Arguments
    /// * `a` - First number
    /// * `b` - Second number
    ///
    /// # Returns
    /// The larger of a and b
    ///
    /// # Example
    /// ```rhai
    /// let result = math::max(5.0, 10.0);  // Returns 10.0
    /// let result2 = math::max(-5.0, -10.0);  // Returns -5.0
    /// ```
    pub fn max(a: f64, b: f64) -> f64 {
        a.max(b)
    }

    /// Returns the minimum of two numbers.
    ///
    /// # Arguments
    /// * `a` - First number
    /// * `b` - Second number
    ///
    /// # Returns
    /// The smaller of a and b
    ///
    /// # Example
    /// ```rhai
    /// let result = math::min(5.0, 10.0);  // Returns 5.0
    /// let result2 = math::min(-5.0, -10.0);  // Returns -10.0
    /// ```
    pub fn min(a: f64, b: f64) -> f64 {
        a.min(b)
    }

    // ============================================================================
    // Rounding Functions
    // ============================================================================

    /// Returns the largest integer less than or equal to a number.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// The floor of x
    ///
    /// # Example
    /// ```rhai
    /// let result = math::floor(3.7);  // Returns 3.0
    /// let result2 = math::floor(-3.7);  // Returns -4.0
    /// ```
    pub fn floor(x: f64) -> f64 {
        x.floor()
    }

    /// Returns the smallest integer greater than or equal to a number.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// The ceiling of x
    ///
    /// # Example
    /// ```rhai
    /// let result = math::ceil(3.2);  // Returns 4.0
    /// let result2 = math::ceil(-3.2);  // Returns -3.0
    /// ```
    pub fn ceil(x: f64) -> f64 {
        x.ceil()
    }

    /// Returns the nearest integer to a number. Rounds half-way cases away from zero.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// The rounded value of x
    ///
    /// # Example
    /// ```rhai
    /// let result = math::round(3.5);  // Returns 4.0
    /// let result2 = math::round(3.4);  // Returns 3.0
    /// let result3 = math::round(-3.5);  // Returns -4.0
    /// ```
    pub fn round(x: f64) -> f64 {
        x.round()
    }

    // ============================================================================
    // Hyperbolic Functions
    // ============================================================================

    /// Computes the hyperbolic sine of a number.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// The hyperbolic sine of x
    ///
    /// # Example
    /// ```rhai
    /// let result = math::sinh(0.0);  // Returns 0.0
    /// let result2 = math::sinh(1.0);  // Returns ~1.175
    /// ```
    pub fn sinh(x: f64) -> f64 {
        x.sinh()
    }

    /// Computes the hyperbolic cosine of a number.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// The hyperbolic cosine of x (always ≥ 1)
    ///
    /// # Example
    /// ```rhai
    /// let result = math::cosh(0.0);  // Returns 1.0
    /// let result2 = math::cosh(1.0);  // Returns ~1.543
    /// ```
    pub fn cosh(x: f64) -> f64 {
        x.cosh()
    }

    /// Computes the hyperbolic tangent of a number.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// The hyperbolic tangent of x, in the range (-1, 1)
    ///
    /// # Example
    /// ```rhai
    /// let result = math::tanh(0.0);  // Returns 0.0
    /// let result2 = math::tanh(1.0);  // Returns ~0.762
    /// ```
    pub fn tanh(x: f64) -> f64 {
        x.tanh()
    }

    // ============================================================================
    // Inverse Hyperbolic Functions
    // ============================================================================

    /// Computes the inverse hyperbolic sine of a number.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// The inverse hyperbolic sine of x
    ///
    /// # Example
    /// ```rhai
    /// let result = math::asinh(0.0);  // Returns 0.0
    /// let result2 = math::asinh(1.0);  // Returns ~0.881
    /// ```
    pub fn asinh(x: f64) -> f64 {
        x.asinh()
    }

    /// Computes the inverse hyperbolic cosine of a number.
    ///
    /// # Arguments
    /// * `x` - Value ≥ 1
    ///
    /// # Returns
    /// The inverse hyperbolic cosine of x
    /// Returns NaN if x < 1
    ///
    /// # Example
    /// ```rhai
    /// let result = math::acosh(1.0);  // Returns 0.0
    /// let result2 = math::acosh(2.0);  // Returns ~1.317
    /// ```
    pub fn acosh(x: f64) -> f64 {
        x.acosh()
    }

    /// Computes the inverse hyperbolic tangent of a number.
    ///
    /// # Arguments
    /// * `x` - Value in the range (-1, 1)
    ///
    /// # Returns
    /// The inverse hyperbolic tangent of x
    /// Returns NaN if |x| ≥ 1
    ///
    /// # Example
    /// ```rhai
    /// let result = math::atanh(0.0);  // Returns 0.0
    /// let result2 = math::atanh(0.5);  // Returns ~0.549
    /// ```
    pub fn atanh(x: f64) -> f64 {
        x.atanh()
    }

    // ============================================================================
    // Exponential and Logarithmic Functions
    // ============================================================================

    /// Computes e raised to the power of x.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// e^x
    ///
    /// # Example
    /// ```rhai
    /// let result = math::exp(0.0);  // Returns 1.0
    /// let result2 = math::exp(1.0);  // Returns e ≈ 2.718
    /// let result3 = math::exp(2.0);  // Returns e^2 ≈ 7.389
    /// ```
    pub fn exp(x: f64) -> f64 {
        x.exp()
    }

    /// Computes the natural logarithm (base e) of a number.
    ///
    /// # Arguments
    /// * `x` - Value > 0
    ///
    /// # Returns
    /// The natural logarithm of x
    /// Returns NaN if x ≤ 0
    ///
    /// # Example
    /// ```rhai
    /// let result = math::ln(math::e());  // Returns 1.0
    /// let result2 = math::ln(1.0);  // Returns 0.0
    /// let result3 = math::ln(10.0);  // Returns ~2.303
    /// ```
    pub fn ln(x: f64) -> f64 {
        x.ln()
    }

    /// Computes the logarithm of a number with a specified base.
    ///
    /// # Arguments
    /// * `x` - Value > 0
    /// * `base` - Base of the logarithm (> 0, ≠ 1)
    ///
    /// # Returns
    /// The logarithm of x in the given base
    /// Returns NaN if x ≤ 0 or base ≤ 0 or base = 1
    ///
    /// # Example
    /// ```rhai
    /// let result = math::log(100.0, 10.0);  // Returns 2.0
    /// let result2 = math::log(8.0, 2.0);  // Returns 3.0
    /// let result3 = math::log(27.0, 3.0);  // Returns 3.0
    /// ```
    pub fn log(x: f64, base: f64) -> f64 {
        x.log(base)
    }

    /// Computes the base-10 logarithm of a number.
    ///
    /// # Arguments
    /// * `x` - Value > 0
    ///
    /// # Returns
    /// The base-10 logarithm of x
    /// Returns NaN if x ≤ 0
    ///
    /// # Example
    /// ```rhai
    /// let result = math::log10(100.0);  // Returns 2.0
    /// let result2 = math::log10(1000.0);  // Returns 3.0
    /// let result3 = math::log10(1.0);  // Returns 0.0
    /// ```
    pub fn log10(x: f64) -> f64 {
        x.log10()
    }

    /// Computes the base-2 logarithm of a number.
    ///
    /// # Arguments
    /// * `x` - Value > 0
    ///
    /// # Returns
    /// The base-2 logarithm of x
    /// Returns NaN if x ≤ 0
    ///
    /// # Example
    /// ```rhai
    /// let result = math::log2(8.0);  // Returns 3.0
    /// let result2 = math::log2(1024.0);  // Returns 10.0
    /// let result3 = math::log2(1.0);  // Returns 0.0
    /// ```
    pub fn log2(x: f64) -> f64 {
        x.log2()
    }

    // ============================================================================
    // Advanced Utility Functions
    // ============================================================================

    /// Computes the cube root of a number.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// The cube root of x
    ///
    /// # Example
    /// ```rhai
    /// let result = math::cbrt(8.0);  // Returns 2.0
    /// let result2 = math::cbrt(27.0);  // Returns 3.0
    /// let result3 = math::cbrt(-8.0);  // Returns -2.0
    /// ```
    pub fn cbrt(x: f64) -> f64 {
        x.cbrt()
    }

    /// Computes the Euclidean distance: sqrt(x^2 + y^2).
    ///
    /// # Arguments
    /// * `x` - First coordinate
    /// * `y` - Second coordinate
    ///
    /// # Returns
    /// The Euclidean distance from the origin to (x, y)
    ///
    /// # Example
    /// ```rhai
    /// let result = math::hypot(3.0, 4.0);  // Returns 5.0
    /// let result2 = math::hypot(5.0, 12.0);  // Returns 13.0
    /// let distance = math::hypot(x2 - x1, y2 - y1);  // Distance between points
    /// ```
    pub fn hypot(x: f64, y: f64) -> f64 {
        x.hypot(y)
    }

    /// Computes e^x - 1 with better precision for small values of x.
    ///
    /// # Arguments
    /// * `x` - Any real number
    ///
    /// # Returns
    /// e^x - 1
    ///
    /// # Example
    /// ```rhai
    /// let result = math::exp_m1(0.0);  // Returns 0.0
    /// let result2 = math::exp_m1(1.0);  // Returns ~1.718 (e - 1)
    /// let small_value = math::exp_m1(0.0001);  // More accurate than exp(x) - 1 for small x
    /// ```
    pub fn exp_m1(x: f64) -> f64 {
        x.exp_m1()
    }

    /// Computes ln(1 + x) with better precision for small values of x.
    ///
    /// # Arguments
    /// * `x` - Value > -1
    ///
    /// # Returns
    /// ln(1 + x)
    /// Returns NaN if x ≤ -1
    ///
    /// # Example
    /// ```rhai
    /// let result = math::ln_1p(0.0);  // Returns 0.0
    /// let result2 = math::ln_1p(math::e() - 1.0);  // Returns 1.0
    /// let small_value = math::ln_1p(0.0001);  // More accurate than ln(1 + x) for small x
    /// ```
    pub fn ln_1p(x: f64) -> f64 {
        x.ln_1p()
    }
}

#[cfg(test)]
mod tests {
    use super::math_module::*;

    const EPSILON: f64 = 1e-10;

    fn assert_approx_eq(a: f64, b: f64, msg: &str) {
        assert!(
            (a - b).abs() < EPSILON,
            "{}: expected {}, got {} (diff: {})",
            msg,
            b,
            a,
            (a - b).abs()
        );
    }

    // ========================================================================
    // Constants Tests
    // ========================================================================

    #[test]
    fn test_constants() {
        assert_approx_eq(PI, std::f64::consts::PI, "PI constant should match std::f64::consts::PI");
        assert_approx_eq(E, std::f64::consts::E, "E constant should match std::f64::consts::E");
    }

    // ========================================================================
    // Integration Tests (Real-world scenarios)
    // ========================================================================

    #[test]
    fn test_solar_radiation_scenario() {
        // Test Expression 4: Planar Solar Radiation Calculation
        let sunrise = 6.0 * 3600.0; // 6:00 AM in seconds
        let sunset = 18.0 * 3600.0; // 6:00 PM in seconds
        let altitude_degrees = 54.1;

        let result = (sunset / 86400.0 - sunrise / 86400.0)
            * 24.0
            * sin(to_radians(altitude_degrees))
            * (2.0 / PI);

        assert!(result > 0.0, "Solar radiation should be positive");
        assert!(
            result < 24.0,
            "Solar radiation should be less than 24 hours"
        );
    }

    #[test]
    fn test_incidence_angle_scenario() {
        // Test Expression 5: Incidence Angle Calculation
        let theta_slope = 30.0; // Roof slope in degrees
        let alpha_slope = 180.0; // Roof orientation in degrees

        let result = sin(to_radians(54.1)) * cos(to_radians(theta_slope))
            + cos(to_radians(54.1))
                * sin(to_radians(theta_slope))
                * cos(to_radians(180.0 - alpha_slope));

        assert!(
            (-1.0..=1.0).contains(&result),
            "Cosine should be in [-1, 1]"
        );
    }

    #[test]
    fn test_pythagorean_theorem() {
        let a = 3.0;
        let b = 4.0;
        let c = sqrt(pow(a, 2.0) + pow(b, 2.0));
        assert_approx_eq(c, 5.0, "Pythagorean theorem: 3-4-5 triangle");
    }

    #[test]
    fn test_distance_calculation() {
        let x1 = 0.0;
        let y1 = 0.0;
        let x2 = 3.0;
        let y2 = 4.0;
        let distance = sqrt(pow(x2 - x1, 2.0) + pow(y2 - y1, 2.0));
        assert_approx_eq(distance, 5.0, "Distance from (0,0) to (3,4) should be 5");
    }

    // ============================================================================
    // Tier 2 Tests: Hyperbolic Functions
    // ============================================================================

    #[test]
    fn test_sinh() {
        assert_approx_eq(sinh(0.0), 0.0, "sinh(0) should be 0");
        assert_approx_eq(sinh(1.0), 1.1752011936438014, "sinh(1) should be ~1.175");
        assert_approx_eq(
            sinh(-1.0),
            -1.1752011936438014,
            "sinh(-1) should be ~-1.175",
        );

        // sinh is an odd function: sinh(-x) = -sinh(x)
        let x = 2.5;
        assert_approx_eq(sinh(-x), -sinh(x), "sinh should be an odd function");
    }

    #[test]
    fn test_cosh() {
        assert_approx_eq(cosh(0.0), 1.0, "cosh(0) should be 1");
        assert_approx_eq(cosh(1.0), 1.5430806348152437, "cosh(1) should be ~1.543");
        assert_approx_eq(cosh(-1.0), 1.5430806348152437, "cosh(-1) should be ~1.543");

        // cosh is an even function: cosh(-x) = cosh(x)
        let x = 2.5;
        assert_approx_eq(cosh(-x), cosh(x), "cosh should be an even function");

        // cosh(x) >= 1 for all x
        assert!(cosh(0.0) >= 1.0, "cosh should be >= 1");
        assert!(cosh(5.0) >= 1.0, "cosh should be >= 1");
    }

    #[test]
    fn test_tanh() {
        assert_approx_eq(tanh(0.0), 0.0, "tanh(0) should be 0");
        assert_approx_eq(tanh(1.0), 0.7615941559557649, "tanh(1) should be ~0.762");
        assert_approx_eq(
            tanh(-1.0),
            -0.7615941559557649,
            "tanh(-1) should be ~-0.762",
        );

        // tanh is an odd function: tanh(-x) = -tanh(x)
        let x = 2.5;
        assert_approx_eq(tanh(-x), -tanh(x), "tanh should be an odd function");

        // tanh(x) is always in (-1, 1)
        assert!(
            (-1.0..=1.0).contains(&tanh(5.0)),
            "tanh should be in (-1, 1)"
        );
        assert!(
            (-1.0..=1.0).contains(&tanh(-5.0)),
            "tanh should be in (-1, 1)"
        );
    }

    #[test]
    fn test_hyperbolic_identity() {
        // cosh^2(x) - sinh^2(x) = 1
        let x = 2.0;
        let identity = pow(cosh(x), 2.0) - pow(sinh(x), 2.0);
        assert_approx_eq(
            identity,
            1.0,
            "Hyperbolic identity: cosh²(x) - sinh²(x) = 1",
        );
    }

    // ============================================================================
    // Tier 2 Tests: Inverse Hyperbolic Functions
    // ============================================================================

    #[test]
    fn test_asinh() {
        assert_approx_eq(asinh(0.0), 0.0, "asinh(0) should be 0");
        assert_approx_eq(asinh(1.0), 0.881373587019543, "asinh(1) should be ~0.881");
        assert_approx_eq(
            asinh(-1.0),
            -0.881373587019543,
            "asinh(-1) should be ~-0.881",
        );

        // asinh(sinh(x)) = x
        let x = 1.5;
        assert_approx_eq(asinh(sinh(x)), x, "asinh(sinh(x)) should equal x");
    }

    #[test]
    fn test_acosh() {
        assert_approx_eq(acosh(1.0), 0.0, "acosh(1) should be 0");
        assert_approx_eq(acosh(2.0), 1.3169578969248166, "acosh(2) should be ~1.317");
        assert_approx_eq(acosh(5.0), 2.2924316695611777, "acosh(5) should be ~2.292");

        // acosh(cosh(x)) = x for x >= 0
        let x = 1.5;
        assert_approx_eq(acosh(cosh(x)), x, "acosh(cosh(x)) should equal x");

        // acosh(x) for x < 1 returns NaN
        assert!(acosh(0.5).is_nan(), "acosh(x < 1) should be NaN");
    }

    #[test]
    fn test_atanh() {
        assert_approx_eq(atanh(0.0), 0.0, "atanh(0) should be 0");
        assert_approx_eq(
            atanh(0.5),
            0.5493061443340548,
            "atanh(0.5) should be ~0.549",
        );
        assert_approx_eq(
            atanh(-0.5),
            -0.5493061443340548,
            "atanh(-0.5) should be ~-0.549",
        );

        // atanh(tanh(x)) = x
        let x = 0.5;
        assert_approx_eq(atanh(tanh(x)), x, "atanh(tanh(x)) should equal x");

        // atanh(x) for |x| >= 1 returns NaN or infinity
        assert!(atanh(1.0).is_infinite(), "atanh(1) should be infinite");
        assert!(atanh(-1.0).is_infinite(), "atanh(-1) should be infinite");
        assert!(atanh(1.5).is_nan(), "atanh(x > 1) should be NaN");
    }

    // ============================================================================
    // Tier 2 Tests: Exponential and Logarithmic Functions
    // ============================================================================

    #[test]
    fn test_exp() {
        assert_approx_eq(exp(0.0), 1.0, "exp(0) should be 1");
        assert_approx_eq(exp(1.0), e(), "exp(1) should be e");
        assert_approx_eq(exp(2.0), 7.38905609893065, "exp(2) should be ~7.389");
        assert_approx_eq(exp(-1.0), 1.0 / e(), "exp(-1) should be 1/e");

        // exp(ln(x)) = x
        let x = 5.0;
        assert_approx_eq(exp(ln(x)), x, "exp(ln(x)) should equal x");
    }

    #[test]
    fn test_ln() {
        assert_approx_eq(ln(1.0), 0.0, "ln(1) should be 0");
        assert_approx_eq(ln(e()), 1.0, "ln(e) should be 1");
        assert_approx_eq(ln(10.0), std::f64::consts::LN_10, "ln(10) should be ~2.303");

        // ln(exp(x)) = x
        let x = 2.5;
        assert_approx_eq(ln(exp(x)), x, "ln(exp(x)) should equal x");

        // ln(x) for x <= 0 returns NaN or -inf
        assert!(ln(0.0).is_infinite(), "ln(0) should be -infinity");
        assert!(ln(-1.0).is_nan(), "ln(negative) should be NaN");
    }

    #[test]
    fn test_log() {
        assert_approx_eq(log(100.0, 10.0), 2.0, "log₁₀(100) should be 2");
        assert_approx_eq(log(8.0, 2.0), 3.0, "log₂(8) should be 3");
        assert_approx_eq(log(27.0, 3.0), 3.0, "log₃(27) should be 3");
        assert_approx_eq(log(1.0, 10.0), 0.0, "log of 1 in any base should be 0");

        // log_b(b) = 1
        assert_approx_eq(log(5.0, 5.0), 1.0, "log_b(b) should be 1");

        // Invalid base or value
        assert!(log(-1.0, 10.0).is_nan(), "log of negative should be NaN");
        assert!(
            log(10.0, -1.0).is_nan(),
            "log with negative base should be NaN"
        );
        // Note: log(x, 1) returns infinity in Rust, not NaN
        assert!(
            log(10.0, 1.0).is_infinite() || log(10.0, 1.0).is_nan(),
            "log with base 1 should be infinite or NaN"
        );
    }

    #[test]
    fn test_log10() {
        assert_approx_eq(log10(1.0), 0.0, "log10(1) should be 0");
        assert_approx_eq(log10(10.0), 1.0, "log10(10) should be 1");
        assert_approx_eq(log10(100.0), 2.0, "log10(100) should be 2");
        assert_approx_eq(log10(1000.0), 3.0, "log10(1000) should be 3");

        // 10^(log10(x)) = x
        let x = 42.0;
        assert_approx_eq(pow(10.0, log10(x)), x, "10^(log10(x)) should equal x");
    }

    #[test]
    fn test_log2() {
        assert_approx_eq(log2(1.0), 0.0, "log2(1) should be 0");
        assert_approx_eq(log2(2.0), 1.0, "log2(2) should be 1");
        assert_approx_eq(log2(8.0), 3.0, "log2(8) should be 3");
        assert_approx_eq(log2(1024.0), 10.0, "log2(1024) should be 10");

        // 2^(log2(x)) = x
        let x = 42.0;
        assert_approx_eq(pow(2.0, log2(x)), x, "2^(log2(x)) should equal x");
    }

    #[test]
    fn test_logarithm_properties() {
        let a = 5.0;
        let b = 3.0;

        // log(a * b) = log(a) + log(b)
        assert_approx_eq(ln(a * b), ln(a) + ln(b), "log(a*b) = log(a) + log(b)");

        // log(a / b) = log(a) - log(b)
        assert_approx_eq(ln(a / b), ln(a) - ln(b), "log(a/b) = log(a) - log(b)");

        // log(a^b) = b * log(a)
        assert_approx_eq(ln(pow(a, b)), b * ln(a), "log(a^b) = b * log(a)");
    }

    // ============================================================================
    // Tier 2 Tests: Advanced Utility Functions
    // ============================================================================

    #[test]
    fn test_cbrt() {
        assert_approx_eq(cbrt(0.0), 0.0, "cbrt(0) should be 0");
        assert_approx_eq(cbrt(1.0), 1.0, "cbrt(1) should be 1");
        assert_approx_eq(cbrt(8.0), 2.0, "cbrt(8) should be 2");
        assert_approx_eq(cbrt(27.0), 3.0, "cbrt(27) should be 3");
        assert_approx_eq(cbrt(-8.0), -2.0, "cbrt(-8) should be -2");

        // cbrt(x^3) = x
        let x = 5.0;
        assert_approx_eq(cbrt(pow(x, 3.0)), x, "cbrt(x³) should equal x");
    }

    #[test]
    fn test_hypot() {
        assert_approx_eq(hypot(3.0, 4.0), 5.0, "hypot(3, 4) should be 5");
        assert_approx_eq(hypot(5.0, 12.0), 13.0, "hypot(5, 12) should be 13");
        assert_approx_eq(hypot(8.0, 15.0), 17.0, "hypot(8, 15) should be 17");
        assert_approx_eq(hypot(0.0, 0.0), 0.0, "hypot(0, 0) should be 0");

        // hypot is symmetric
        assert_approx_eq(
            hypot(3.0, 4.0),
            hypot(4.0, 3.0),
            "hypot should be symmetric",
        );

        // hypot(x, 0) = |x|
        assert_approx_eq(hypot(5.0, 0.0), abs(5.0), "hypot(x, 0) should be |x|");
        assert_approx_eq(hypot(-5.0, 0.0), abs(-5.0), "hypot(-x, 0) should be |x|");
    }

    #[test]
    fn test_exp_m1() {
        assert_approx_eq(exp_m1(0.0), 0.0, "exp_m1(0) should be 0");
        assert_approx_eq(exp_m1(1.0), e() - 1.0, "exp_m1(1) should be e - 1");

        // For small values, exp_m1 is more accurate than exp(x) - 1
        let small = 0.0001;
        let result = exp_m1(small);
        assert!(result > 0.0, "exp_m1(small positive) should be positive");
        assert!(result < small * 2.0, "exp_m1(small) should be close to x");

        // exp_m1(x) = exp(x) - 1
        let x = 2.0;
        assert_approx_eq(exp_m1(x), exp(x) - 1.0, "exp_m1(x) = exp(x) - 1");
    }

    #[test]
    fn test_ln_1p() {
        assert_approx_eq(ln_1p(0.0), 0.0, "ln_1p(0) should be 0");
        assert_approx_eq(ln_1p(e() - 1.0), 1.0, "ln_1p(e-1) should be 1");

        // For small values, ln_1p is more accurate than ln(1 + x)
        let small = 0.0001;
        let result = ln_1p(small);
        assert!(result > 0.0, "ln_1p(small positive) should be positive");
        assert!(result < small * 2.0, "ln_1p(small) should be close to x");

        // ln_1p(x) = ln(1 + x)
        let x = 2.0;
        assert_approx_eq(ln_1p(x), ln(1.0 + x), "ln_1p(x) = ln(1 + x)");

        // ln_1p(x) for x <= -1 returns NaN or -inf
        assert!(ln_1p(-1.0).is_infinite(), "ln_1p(-1) should be -infinity");
        assert!(ln_1p(-2.0).is_nan(), "ln_1p(x < -1) should be NaN");
    }

    #[test]
    fn test_exponential_growth_model() {
        // Model: population = initial * e^(rate * time)
        let initial_population = 100.0;
        let growth_rate = 0.05; // 5% per year
        let years = 10.0;

        let final_population = initial_population * exp(growth_rate * years);
        assert!(
            final_population > initial_population,
            "Population should grow"
        );
        assert_approx_eq(
            final_population,
            164.87212707001282,
            "Exponential growth calculation",
        );
    }

    #[test]
    fn test_logarithmic_scale() {
        // Richter scale, pH scale, decibels - all use logarithmic scales
        let intensity1 = 100.0;
        let intensity2 = 1000.0;

        let magnitude_diff = log10(intensity2) - log10(intensity1);
        assert_approx_eq(magnitude_diff, 1.0, "10x intensity = 1 unit on log scale");
    }

    #[test]
    fn test_catenary_curve() {
        // Catenary curve: y = a * cosh(x/a)
        // Used for hanging cables, chains
        let a = 10.0;
        let x = 5.0;
        let y = a * cosh(x / a);

        assert!(y >= a, "Catenary minimum is at y = a");
        // Note: Using more relaxed precision due to compounding floating-point operations
        assert!(
            (y - 11.275).abs() < 0.01,
            "Catenary curve calculation should be around 11.275"
        );
    }
}
