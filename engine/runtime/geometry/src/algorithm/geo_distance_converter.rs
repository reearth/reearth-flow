const EARTH_RADIUS: f64 = 6_378_137.0;
const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;

pub fn coordinate_diff_to_meter(dlng: f64, dlat: f64, lat: f64) -> (f64, f64) {
    let lat_rad = lat * DEG_TO_RAD;
    let dlat_rad = dlat * DEG_TO_RAD;
    let dlng_rad = dlng * DEG_TO_RAD;

    let x = dlng_rad * EARTH_RADIUS * lat_rad.cos();

    let y = dlat_rad * EARTH_RADIUS;

    (x, y)
}

pub fn meter_to_coordinate_diff(meter_x: f64, meter_y: f64, lat: f64) -> (f64, f64) {
    let lat_rad = lat * DEG_TO_RAD;

    let dlat_rad = meter_y / EARTH_RADIUS;

    let dlng_rad = meter_x / (EARTH_RADIUS * lat_rad.cos());

    let dlng = dlng_rad * RAD_TO_DEG;
    let dlat = dlat_rad * RAD_TO_DEG;

    (dlng, dlat)
}
