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
