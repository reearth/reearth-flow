use rhai::export_module;

#[export_module]
pub(crate) mod math_module {
    use rhai::plugin::*;
    use std::f64::consts::{E, PI};

    // ============================================================================
    // Mathematical Constants
    // ============================================================================

    /// Returns the mathematical constant π (pi).
    ///
    /// # Returns
    /// The value of π ≈ 3.14159265358979323846
    ///
    /// # Example
    /// ```rhai
    /// let pi_value = math::pi();
    /// let circumference = 2.0 * math::pi() * radius;
    /// ```
    pub fn pi() -> f64 {
        PI
    }

    /// Returns the mathematical constant e (Euler's number).
    ///
    /// # Returns
    /// The value of e ≈ 2.71828182845904523536
    ///
    /// # Example
    /// ```rhai
    /// let e_value = math::e();
    /// let result = math::pow(math::e(), 2.0);
    /// ```
    pub fn e() -> f64 {
        E
    }

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
}

#[cfg(test)]
mod tests {
    use super::math_module::*;
    use std::f64::consts::{E, PI};

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
    fn test_pi() {
        assert_approx_eq(pi(), PI, "pi() should return correct value");
    }

    #[test]
    fn test_e() {
        assert_approx_eq(e(), E, "e() should return correct value");
    }

    // ========================================================================
    // Trigonometric Functions Tests
    // ========================================================================

    #[test]
    fn test_sin() {
        assert_approx_eq(sin(0.0), 0.0, "sin(0) should be 0");
        assert_approx_eq(sin(PI / 2.0), 1.0, "sin(π/2) should be 1");
        assert_approx_eq(sin(PI), 0.0, "sin(π) should be 0");
        assert_approx_eq(sin(3.0 * PI / 2.0), -1.0, "sin(3π/2) should be -1");
        assert_approx_eq(sin(PI / 6.0), 0.5, "sin(30°) should be 0.5");
    }

    #[test]
    fn test_cos() {
        assert_approx_eq(cos(0.0), 1.0, "cos(0) should be 1");
        assert_approx_eq(cos(PI / 2.0), 0.0, "cos(π/2) should be 0");
        assert_approx_eq(cos(PI), -1.0, "cos(π) should be -1");
        assert_approx_eq(cos(PI / 3.0), 0.5, "cos(60°) should be 0.5");
    }

    #[test]
    fn test_tan() {
        assert_approx_eq(tan(0.0), 0.0, "tan(0) should be 0");
        assert_approx_eq(tan(PI / 4.0), 1.0, "tan(π/4) should be 1");
        assert_approx_eq(tan(-PI / 4.0), -1.0, "tan(-π/4) should be -1");
    }

    // ========================================================================
    // Inverse Trigonometric Functions Tests
    // ========================================================================

    #[test]
    fn test_asin() {
        assert_approx_eq(asin(0.0), 0.0, "asin(0) should be 0");
        assert_approx_eq(asin(0.5), PI / 6.0, "asin(0.5) should be π/6");
        assert_approx_eq(asin(1.0), PI / 2.0, "asin(1) should be π/2");
        assert_approx_eq(asin(-1.0), -PI / 2.0, "asin(-1) should be -π/2");
    }

    #[test]
    fn test_acos() {
        assert_approx_eq(acos(1.0), 0.0, "acos(1) should be 0");
        assert_approx_eq(acos(0.5), PI / 3.0, "acos(0.5) should be π/3");
        assert_approx_eq(acos(0.0), PI / 2.0, "acos(0) should be π/2");
        assert_approx_eq(acos(-1.0), PI, "acos(-1) should be π");
    }

    #[test]
    fn test_atan() {
        assert_approx_eq(atan(0.0), 0.0, "atan(0) should be 0");
        assert_approx_eq(atan(1.0), PI / 4.0, "atan(1) should be π/4");
        assert_approx_eq(atan(-1.0), -PI / 4.0, "atan(-1) should be -π/4");
    }

    #[test]
    fn test_atan2() {
        // Quadrant I
        assert_approx_eq(atan2(1.0, 1.0), PI / 4.0, "atan2(1, 1) should be π/4");
        // Quadrant II
        assert_approx_eq(
            atan2(1.0, -1.0),
            3.0 * PI / 4.0,
            "atan2(1, -1) should be 3π/4",
        );
        // Quadrant III
        assert_approx_eq(
            atan2(-1.0, -1.0),
            -3.0 * PI / 4.0,
            "atan2(-1, -1) should be -3π/4",
        );
        // Quadrant IV
        assert_approx_eq(atan2(-1.0, 1.0), -PI / 4.0, "atan2(-1, 1) should be -π/4");
        // Special cases
        assert_approx_eq(atan2(0.0, 1.0), 0.0, "atan2(0, 1) should be 0");
        assert_approx_eq(atan2(1.0, 0.0), PI / 2.0, "atan2(1, 0) should be π/2");
    }

    // ========================================================================
    // Angle Conversion Tests
    // ========================================================================

    #[test]
    fn test_to_radians() {
        assert_approx_eq(to_radians(0.0), 0.0, "0° should be 0 radians");
        assert_approx_eq(to_radians(90.0), PI / 2.0, "90° should be π/2 radians");
        assert_approx_eq(to_radians(180.0), PI, "180° should be π radians");
        assert_approx_eq(to_radians(360.0), 2.0 * PI, "360° should be 2π radians");
        assert_approx_eq(to_radians(-90.0), -PI / 2.0, "-90° should be -π/2 radians");
    }

    #[test]
    fn test_to_degrees() {
        assert_approx_eq(to_degrees(0.0), 0.0, "0 radians should be 0°");
        assert_approx_eq(to_degrees(PI / 2.0), 90.0, "π/2 radians should be 90°");
        assert_approx_eq(to_degrees(PI), 180.0, "π radians should be 180°");
        assert_approx_eq(to_degrees(2.0 * PI), 360.0, "2π radians should be 360°");
        assert_approx_eq(to_degrees(-PI / 2.0), -90.0, "-π/2 radians should be -90°");
    }

    #[test]
    fn test_angle_conversion_roundtrip() {
        let degrees = 45.0;
        let radians = to_radians(degrees);
        let back_to_degrees = to_degrees(radians);
        assert_approx_eq(
            back_to_degrees,
            degrees,
            "Conversion roundtrip should be exact",
        );
    }

    // ========================================================================
    // Power & Root Functions Tests
    // ========================================================================

    #[test]
    fn test_sqrt() {
        assert_approx_eq(sqrt(0.0), 0.0, "sqrt(0) should be 0");
        assert_approx_eq(sqrt(1.0), 1.0, "sqrt(1) should be 1");
        assert_approx_eq(sqrt(4.0), 2.0, "sqrt(4) should be 2");
        assert_approx_eq(sqrt(16.0), 4.0, "sqrt(16) should be 4");
        assert_approx_eq(
            sqrt(2.0),
            std::f64::consts::SQRT_2,
            "sqrt(2) should be ~1.414",
        );
    }

    #[test]
    fn test_pow() {
        assert_approx_eq(pow(2.0, 0.0), 1.0, "2^0 should be 1");
        assert_approx_eq(pow(2.0, 1.0), 2.0, "2^1 should be 2");
        assert_approx_eq(pow(2.0, 3.0), 8.0, "2^3 should be 8");
        assert_approx_eq(pow(10.0, 2.0), 100.0, "10^2 should be 100");
        assert_approx_eq(pow(3.0, 4.0), 81.0, "3^4 should be 81");
        assert_approx_eq(pow(4.0, 0.5), 2.0, "4^0.5 should be 2");
    }

    // ========================================================================
    // Comparison & Selection Tests
    // ========================================================================

    #[test]
    fn test_abs() {
        assert_approx_eq(abs(0.0), 0.0, "abs(0) should be 0");
        assert_approx_eq(abs(5.0), 5.0, "abs(5) should be 5");
        assert_approx_eq(abs(-5.0), 5.0, "abs(-5) should be 5");
        assert_approx_eq(abs(-7.5), 7.5, "abs(-7.5) should be 7.5");
    }

    #[test]
    fn test_max() {
        assert_approx_eq(max(5.0, 10.0), 10.0, "max(5, 10) should be 10");
        assert_approx_eq(max(10.0, 5.0), 10.0, "max(10, 5) should be 10");
        assert_approx_eq(max(-5.0, -10.0), -5.0, "max(-5, -10) should be -5");
        assert_approx_eq(max(0.0, 0.0), 0.0, "max(0, 0) should be 0");
    }

    #[test]
    fn test_min() {
        assert_approx_eq(min(5.0, 10.0), 5.0, "min(5, 10) should be 5");
        assert_approx_eq(min(10.0, 5.0), 5.0, "min(10, 5) should be 5");
        assert_approx_eq(min(-5.0, -10.0), -10.0, "min(-5, -10) should be -10");
        assert_approx_eq(min(0.0, 0.0), 0.0, "min(0, 0) should be 0");
    }

    // ========================================================================
    // Rounding Functions Tests
    // ========================================================================

    #[test]
    fn test_floor() {
        assert_approx_eq(floor(3.7), 3.0, "floor(3.7) should be 3");
        assert_approx_eq(floor(3.2), 3.0, "floor(3.2) should be 3");
        assert_approx_eq(floor(-3.2), -4.0, "floor(-3.2) should be -4");
        assert_approx_eq(floor(-3.7), -4.0, "floor(-3.7) should be -4");
        assert_approx_eq(floor(5.0), 5.0, "floor(5.0) should be 5");
    }

    #[test]
    fn test_ceil() {
        assert_approx_eq(ceil(3.2), 4.0, "ceil(3.2) should be 4");
        assert_approx_eq(ceil(3.7), 4.0, "ceil(3.7) should be 4");
        assert_approx_eq(ceil(-3.7), -3.0, "ceil(-3.7) should be -3");
        assert_approx_eq(ceil(-3.2), -3.0, "ceil(-3.2) should be -3");
        assert_approx_eq(ceil(5.0), 5.0, "ceil(5.0) should be 5");
    }

    #[test]
    fn test_round() {
        assert_approx_eq(round(3.5), 4.0, "round(3.5) should be 4");
        assert_approx_eq(round(3.4), 3.0, "round(3.4) should be 3");
        assert_approx_eq(round(3.6), 4.0, "round(3.6) should be 4");
        assert_approx_eq(round(-3.5), -4.0, "round(-3.5) should be -4");
        assert_approx_eq(round(-3.4), -3.0, "round(-3.4) should be -3");
        assert_approx_eq(round(5.0), 5.0, "round(5.0) should be 5");
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
            * (2.0 / pi());

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
}
