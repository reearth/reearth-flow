#[inline]
fn cross((ax, ay, az): (f64, f64, f64), (bx, by, bz): (f64, f64, f64)) -> (f64, f64, f64) {
    (ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx)
}

pub fn calculate_normal(
    vertex_iter: impl IntoIterator<Item = [f64; 3]>,
) -> Option<(f64, f64, f64)> {
    let mut iter = vertex_iter.into_iter();
    let first = iter.next()?;
    let mut prev = first;

    let mut sum = (0., 0., 0.);

    for data in iter {
        // ..
        let (x, y, z) = (data[0], data[1], data[2]);
        let c = cross(
            (prev[0] - x, prev[1] - y, prev[2] - z),
            (prev[0] + x, prev[1] + y, prev[2] + z),
        );
        sum.0 += c.0;
        sum.1 += c.1;
        sum.2 += c.2;
        prev = [x, y, z];
    }

    {
        let (x, y, z) = (first[0], first[1], first[2]);
        let c = cross(
            (prev[0] - x, prev[1] - y, prev[2] - z),
            (prev[0] + x, prev[1] + y, prev[2] + z),
        );
        sum.0 += c.0;
        sum.1 += c.1;
        sum.2 += c.2;
    }

    match (sum.0 * sum.0 + sum.1 * sum.1 + sum.2 * sum.2).sqrt() {
        d if d < 1e-30 => None,
        d => Some((sum.0 / d, sum.1 / d, sum.2 / d)),
    }
}

pub fn y_slice_range(z: u8, y: u32) -> (f64, f64) {
    let (_, y_size) = size_for_z(z);
    let y = y as f64;
    let north = 90.0 - 180.0 * y / y_size as f64;
    let south = 90.0 - 180.0 * (y + 1.0) / y_size as f64;
    (south, north)
}

pub fn x_slice_range(z: u8, x: i32, xs: u32) -> (f64, f64) {
    let (x_size, _) = size_for_z(z);
    let west = -180.0 + 360.0 * x as f64 / x_size as f64;
    let east = -180.0 + 360.0 * (x + xs as i32) as f64 / x_size as f64;
    (west, east)
}

pub fn x_step(z: u8, y: u32) -> u32 {
    match z {
        0 | 1 => 1,
        _ => {
            let zz = 1 << z;
            if y < zz / 4 {
                u32::max(1, zz / (1 << msb(y))) / 4
            } else {
                u32::max(1, zz / (1 << msb(zz / 2 - y - 1))) / 4
            }
        }
    }
}

pub fn msb(d: u32) -> u32 {
    u32::BITS - d.leading_zeros()
}

pub fn size_for_z(z: u8) -> (u32, u32) {
    match z {
        0 => (1, 1),
        1 => (2, 2),
        _ => (1 << z, 1 << (z - 1)),
    }
}

pub fn geometric_error(z: u8, y: u32) -> f64 {
    let (_, y_size) = size_for_z(z);
    if y >= y_size {
        panic!("y out of range");
    }
    if z < 2 {
        return 1e+100;
    }
    use std::f64::consts::PI;
    const Q: f64 = 525957.5361033019;
    let zz = (1 << z) as f64;
    let error1 = Q / (1 << (z - 2)) as f64;
    let lat = (1.0 - (y as f64 + 0.5) * 4.0 / zz) * PI / 2.0;
    let error2 = lat.cos() * x_step(z, y) as f64 * error1;
    f64::max(error1, error2)
}

pub fn calc_parent_zxy(z: u8, x: u32, y: u32) -> (u8, u32, u32) {
    match z {
        0 => panic!("z=0 has no parent"),
        1 => (z - 1, 0, 0),
        2 => (z - 1, x / 2, y),
        _ => (z - 1, x / 2, y / 2),
    }
}

pub fn zxy_from_lng_lat(z: u8, lng: f64, lat: f64) -> (u8, u32, u32) {
    let (x_size, y_size) = size_for_z(z);
    let y = ((90.0 - lat) / 180.0 * y_size as f64).floor() as u32;
    let xs = x_step(z, y) as i32;
    let x = ((180.0 + lng) / 360.0 * x_size as f64).floor() as i32;
    (z, (x - x.rem_euclid(xs)) as u32, y)
}
