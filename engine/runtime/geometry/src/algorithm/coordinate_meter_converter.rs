const EARTH_RADIUS: f64 = 6_378_137.0;
const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;

pub fn coordinate_diff_to_meter(dlng: f64, dlat: f64) -> (f64, f64) {
    let dlat = dlat * DEG_TO_RAD;
    let dlng = dlng * DEG_TO_RAD;
    (dlng * EARTH_RADIUS * dlat.cos(), dlat * EARTH_RADIUS)
}

pub fn meter_to_coordinate_diff(meter_x: f64, meter_y: f64) -> (f64, f64) {
    let dlat = meter_y / EARTH_RADIUS;
    let dlng = meter_x / (EARTH_RADIUS * dlat.cos());
    (dlng / DEG_TO_RAD, dlat / DEG_TO_RAD)
}
