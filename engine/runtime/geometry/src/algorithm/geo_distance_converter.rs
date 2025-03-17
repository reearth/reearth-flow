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

#[cfg(test)]
mod tests {
    use crate::types::coordinate::Coordinate2D;

    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_coordinate_diff_to_meter() {
        // calculated using "Online geodesic calculations using the GeodSolve utility (https://geographiclib.sourceforge.io/cgi-bin/GeodSolve)"
        let coords_a = Coordinate2D::new_(139.6917, 35.6895);
        let coords_b = Coordinate2D::new_(139.69280478, 35.69040128);

        let (dx_meter, dy_meter) =
            coordinate_diff_to_meter(coords_b.x - coords_a.x, coords_b.y - coords_a.y, coords_a.y);

        assert_relative_eq!(dx_meter, 100.0, epsilon = 1.0);
        assert_relative_eq!(dy_meter, 100.0, epsilon = 1.0);
    }

    #[test]
    fn test_meter_to_coordinate_diff() {
        let coords_a = Coordinate2D::new_(139.6917, 35.6895);
        let coords_b = Coordinate2D::new_(139.69280478, 35.69040128);

        let (dx_meter, dy_meter) = (100.0, 100.0);

        let (dx, dy) = meter_to_coordinate_diff(dx_meter, dy_meter, coords_a.y);

        assert_relative_eq!(dx, coords_b.x - coords_a.x, epsilon = 1e-5);
        assert_relative_eq!(dy, coords_b.y - coords_a.y, epsilon = 1e-5);
    }
}
