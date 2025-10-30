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
        assert_approx_eq(
            PI,
            std::f64::consts::PI,
            "PI constant should match std::f64::consts::PI",
        );
        assert_approx_eq(
            E,
            std::f64::consts::E,
            "E constant should match std::f64::consts::E",
        );
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
}
