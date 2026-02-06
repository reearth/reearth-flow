//! Solar position calculator using Spencer's (1971) algorithm.

use chrono::{DateTime, Datelike, Timelike, Utc};

/// Solar position result containing altitude and azimuth angles in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolarPosition {
    /// Solar altitude (elevation) angle in degrees. Negative = below horizon.
    pub altitude: f64,
    /// Solar azimuth angle in degrees. 0 = South, clockwise positive.
    pub azimuth: f64,
}

impl SolarPosition {
    #[allow(dead_code)]
    pub fn new(altitude: f64, azimuth: f64) -> Self {
        Self { altitude, azimuth }
    }
}

/// Calculate solar position for a given geographic location and UTC datetime.
pub fn calculate_solar_position(
    latitude: f64,
    longitude: f64,
    datetime: DateTime<Utc>,
    standard_meridian: f64,
) -> SolarPosition {
    let lat_rad = latitude.to_radians();
    let day_of_year = datetime.ordinal() as f64;
    let gamma = 2.0 * std::f64::consts::PI * (day_of_year - 1.0) / 365.0;

    let declination = calculate_declination(gamma);
    let equation_of_time = calculate_equation_of_time(gamma);
    let hour_angle = calculate_hour_angle(datetime, longitude, standard_meridian, equation_of_time);

    let sin_altitude =
        lat_rad.sin() * declination.sin() + lat_rad.cos() * declination.cos() * hour_angle.cos();
    let altitude_rad = sin_altitude.asin();
    let azimuth_rad = calculate_azimuth(lat_rad, declination, hour_angle, altitude_rad);

    SolarPosition {
        altitude: altitude_rad.to_degrees(),
        azimuth: azimuth_rad.to_degrees(),
    }
}

/// Spencer's (1971) Fourier series for solar declination. Returns radians.
fn calculate_declination(gamma: f64) -> f64 {
    0.006918 - 0.399912 * gamma.cos() + 0.070257 * gamma.sin() - 0.006758 * (2.0 * gamma).cos()
        + 0.000907 * (2.0 * gamma).sin()
        - 0.002697 * (3.0 * gamma).cos()
        + 0.00148 * (3.0 * gamma).sin()
}

/// Spencer's (1971) Fourier series for equation of time. Returns minutes.
fn calculate_equation_of_time(gamma: f64) -> f64 {
    229.18
        * (0.000075 + 0.001868 * gamma.cos()
            - 0.032077 * gamma.sin()
            - 0.014615 * (2.0 * gamma).cos()
            - 0.04089 * (2.0 * gamma).sin())
}

/// Calculate hour angle from local standard time. Returns radians.
///
/// Note: The datetime parameter is treated as local standard time (not UTC).
/// The user provides time in their local timezone (e.g., JST for Japan),
/// and this function calculates the solar time based on the difference
/// between the location's longitude and the standard meridian.
fn calculate_hour_angle(
    datetime: DateTime<Utc>, // Actually local time, just stored as UTC type
    longitude: f64,
    standard_meridian: f64,
    equation_of_time: f64,
) -> f64 {
    // Input time IS local standard time (not UTC)
    let local_time_minutes =
        datetime.hour() as f64 * 60.0 + datetime.minute() as f64 + datetime.second() as f64 / 60.0;

    // Longitude correction for difference from standard meridian
    // 4 minutes per degree of longitude
    let longitude_correction = 4.0 * (longitude - standard_meridian);
    let solar_time_minutes = local_time_minutes + longitude_correction + equation_of_time;

    let hour_angle_degrees = (solar_time_minutes / 60.0 - 12.0) * 15.0;
    hour_angle_degrees.to_radians()
}

/// Calculate solar azimuth. Returns radians (0 to 2π, 0 = South).
fn calculate_azimuth(lat_rad: f64, declination: f64, hour_angle: f64, altitude_rad: f64) -> f64 {
    let cos_altitude = altitude_rad.cos();
    if cos_altitude.abs() < 1e-10 {
        return 0.0;
    }

    let cos_azimuth =
        (altitude_rad.sin() * lat_rad.sin() - declination.sin()) / (cos_altitude * lat_rad.cos());
    let cos_azimuth = cos_azimuth.clamp(-1.0, 1.0);
    let mut azimuth = cos_azimuth.acos();

    if hour_angle < 0.0 {
        azimuth = -azimuth;
    }
    if azimuth < 0.0 {
        azimuth += 2.0 * std::f64::consts::PI;
    }

    azimuth
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // Note: All times in these tests are LOCAL time (e.g., JST for Tokyo),
    // stored as DateTime<Utc> for convenience (the Utc type is just a container).

    #[test]
    fn test_tokyo_summer_solstice_noon() {
        // 12:00 JST (local time) on summer solstice
        let datetime = Utc.with_ymd_and_hms(2024, 6, 21, 12, 0, 0).unwrap();
        let position = calculate_solar_position(35.6762, 139.6503, datetime, 135.0);

        assert!(
            position.altitude > 70.0 && position.altitude < 82.0,
            "Altitude {} should be around 75-80 degrees",
            position.altitude
        );
    }

    #[test]
    fn test_night_time() {
        // 00:00 JST (midnight local time)
        let datetime = Utc.with_ymd_and_hms(2024, 6, 21, 0, 0, 0).unwrap();
        let position = calculate_solar_position(35.6762, 139.6503, datetime, 135.0);

        assert!(
            position.altitude < 0.0,
            "Sun should be below horizon at midnight"
        );
    }

    #[test]
    fn test_tokyo_winter_solstice_noon() {
        // 12:00 JST (local time) on winter solstice
        let datetime = Utc.with_ymd_and_hms(2024, 12, 21, 12, 0, 0).unwrap();
        let position = calculate_solar_position(35.6762, 139.6503, datetime, 135.0);

        assert!(
            position.altitude > 25.0 && position.altitude < 40.0,
            "Altitude {} should be around 30-35 degrees",
            position.altitude
        );
    }

    #[test]
    fn test_equator_equinox_noon() {
        // 12:00 local time at equator on equinox
        let datetime = Utc.with_ymd_and_hms(2024, 3, 20, 12, 0, 0).unwrap();
        let position = calculate_solar_position(0.0, 0.0, datetime, 0.0);

        assert!(
            position.altitude > 85.0,
            "Altitude {} should be near 90 degrees",
            position.altitude
        );
    }

    #[test]
    fn test_declination_summer_solstice() {
        let gamma = 2.0 * std::f64::consts::PI * (172.0 - 1.0) / 365.0;
        let declination_degrees = calculate_declination(gamma).to_degrees();

        assert!(
            (declination_degrees - 23.44).abs() < 1.0,
            "Declination {} should be about 23.44 degrees",
            declination_degrees
        );
    }

    #[test]
    fn test_declination_winter_solstice() {
        let gamma = 2.0 * std::f64::consts::PI * (355.0 - 1.0) / 365.0;
        let declination_degrees = calculate_declination(gamma).to_degrees();

        assert!(
            (declination_degrees + 23.44).abs() < 1.0,
            "Declination {} should be about -23.44 degrees",
            declination_degrees
        );
    }

    #[test]
    fn test_azimuth_at_noon() {
        // 12:00 local time at equator
        let datetime = Utc.with_ymd_and_hms(2024, 4, 15, 12, 0, 0).unwrap();
        let position = calculate_solar_position(0.0, 0.0, datetime, 0.0);

        assert!(
            position.altitude > 70.0,
            "Altitude {} should be high at equator near solar noon",
            position.altitude
        );

        // 12:00 JST (local time) in Tokyo
        let tokyo_noon = Utc.with_ymd_and_hms(2024, 4, 15, 12, 0, 0).unwrap();
        let tokyo_pos = calculate_solar_position(35.6762, 139.6503, tokyo_noon, 135.0);

        assert!(
            tokyo_pos.azimuth < 30.0 || tokyo_pos.azimuth > 330.0,
            "Azimuth {} should be near South",
            tokyo_pos.azimuth
        );
    }

    #[test]
    fn test_azimuth_morning_afternoon() {
        // 9:00 JST (morning local time)
        let morning = Utc.with_ymd_and_hms(2024, 6, 21, 9, 0, 0).unwrap();
        // 15:00 JST (afternoon local time)
        let afternoon = Utc.with_ymd_and_hms(2024, 6, 21, 15, 0, 0).unwrap();

        let morning_pos = calculate_solar_position(35.6762, 139.6503, morning, 135.0);
        let afternoon_pos = calculate_solar_position(35.6762, 139.6503, afternoon, 135.0);

        assert!(
            (morning_pos.azimuth - afternoon_pos.azimuth).abs() > 90.0,
            "Morning {} and afternoon {} azimuths should differ significantly",
            morning_pos.azimuth,
            afternoon_pos.azimuth
        );
    }
}
