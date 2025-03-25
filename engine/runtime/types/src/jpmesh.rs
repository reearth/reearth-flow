use reearth_flow_geometry::types::{
    coordinate::Coordinate2D,
    rect::{Rect, Rect2D},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JPMeshType {
    /// 第1次地域区画
    Mesh80km,
    /// 第2次地域区画
    Mesh10km,
    /// 基準地域メッシュ
    Mesh1km,
    /// 2分の1地域メッシュ
    Mesh500m,
    /// 4分の1地域メッシュ
    Mesh250m,
    /// 8分の1地域メッシュ
    Mesh125m,
}

impl Ord for JPMeshType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.code_length().cmp(&other.code_length()).reverse()
    }
}

impl PartialOrd for JPMeshType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl JPMeshType {
    pub const fn code_length(&self) -> usize {
        match self {
            JPMeshType::Mesh80km => 4,
            JPMeshType::Mesh10km => 6,
            JPMeshType::Mesh1km => 8,
            JPMeshType::Mesh500m => 9,
            JPMeshType::Mesh250m => 10,
            JPMeshType::Mesh125m => 11,
        }
    }

    const fn lat_interval_seconds(&self) -> f64 {
        match self {
            JPMeshType::Mesh80km => 2400.0,
            JPMeshType::Mesh10km => 300.0,
            JPMeshType::Mesh1km => 30.0,
            JPMeshType::Mesh500m => 15.0,
            JPMeshType::Mesh250m => 7.5,
            JPMeshType::Mesh125m => 3.75,
        }
    }

    const fn lng_interval_seconds(&self) -> f64 {
        match self {
            JPMeshType::Mesh80km => 3600.0,
            JPMeshType::Mesh10km => 450.0,
            JPMeshType::Mesh1km => 45.0,
            JPMeshType::Mesh500m => 22.5,
            JPMeshType::Mesh250m => 11.25,
            JPMeshType::Mesh125m => 5.625,
        }
    }

    pub const fn lat_interval(&self) -> f64 {
        self.lat_interval_seconds() / 3600.0
    }

    pub const fn lng_interval(&self) -> f64 {
        self.lng_interval_seconds() / 3600.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JPMeshCode {
    mesh_type: JPMeshType,
    seed: JPMeshCodeSeed,
}

impl JPMeshCode {
    pub fn new(coords: Coordinate2D<f64>, mesh_type: JPMeshType) -> Self {
        let seed = JPMeshCodeSeed::new(coords);
        JPMeshCode { mesh_type, seed }
    }

    pub fn from_number(mesh_code: u64, mesh_type: JPMeshType) -> Self {
        let mut code_2 = [0u8; 11];
        let mut mesh_code = mesh_code;
        let ifirst = 11 - mesh_type.code_length();
        for i in (0..11).rev() {
            let value = (mesh_code % 10) as u8;
            if i >= ifirst {
                code_2[i - ifirst] = value;
            }
            mesh_code /= 10;
        }

        JPMeshCode {
            mesh_type,
            seed: JPMeshCodeSeed { code_2 },
        }
    }

    pub fn from_inside_bounds(bounds: Rect2D<f64>, mesh_type: JPMeshType) -> Vec<Self> {
        let mut mesh_codes = vec![];
        let min = bounds.min();
        let max = bounds.max();
        let lat_len = ((max.y - min.y) / mesh_type.lat_interval()).ceil() as u64;
        let lng_len = ((max.x - min.x) / mesh_type.lng_interval()).ceil() as u64;

        let start_bounds = JPMeshCode::new(min, mesh_type).bounds();
        let start_lng = (start_bounds.min().x + start_bounds.max().x) / 2.0;
        let start_lat = (start_bounds.min().y + start_bounds.max().y) / 2.0;

        for i in 0..=lat_len {
            for j in 0..=lng_len {
                let coords = Coordinate2D::new_(
                    start_lng + j as f64 * mesh_type.lng_interval(),
                    start_lat + i as f64 * mesh_type.lat_interval(),
                );
                mesh_codes.push(JPMeshCode::new(coords, mesh_type));
            }
        }

        mesh_codes
    }

    pub fn downscale(&self) -> Vec<Self> {
        match self.mesh_type {
            JPMeshType::Mesh80km => (0..8)
                .flat_map(|i| {
                    (0..8)
                        .map(|j| {
                            Self::from_number(
                                self.to_number() * 100 + i * 10 + j,
                                JPMeshType::Mesh10km,
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
            JPMeshType::Mesh10km => (0..10)
                .flat_map(|i| {
                    (0..10)
                        .map(|j| {
                            Self::from_number(
                                self.to_number() * 100 + i * 10 + j,
                                JPMeshType::Mesh1km,
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
            JPMeshType::Mesh1km => (1..=4)
                .map(|i| Self::from_number(self.to_number() * 10 + i, JPMeshType::Mesh500m))
                .collect(),
            JPMeshType::Mesh500m => (1..=4)
                .map(|i| Self::from_number(self.to_number() * 10 + i, JPMeshType::Mesh250m))
                .collect(),
            JPMeshType::Mesh250m => (1..=4)
                .map(|i| Self::from_number(self.to_number() * 10 + i, JPMeshType::Mesh125m))
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn upscale(&self) -> Option<Self> {
        match self.mesh_type {
            JPMeshType::Mesh10km => Some(Self::from_number(
                self.to_number() / 100,
                JPMeshType::Mesh80km,
            )),
            JPMeshType::Mesh1km => Some(Self::from_number(
                self.to_number() / 100,
                JPMeshType::Mesh10km,
            )),
            JPMeshType::Mesh500m => Some(Self::from_number(
                self.to_number() / 10,
                JPMeshType::Mesh1km,
            )),
            JPMeshType::Mesh250m => Some(Self::from_number(
                self.to_number() / 10,
                JPMeshType::Mesh500m,
            )),
            JPMeshType::Mesh125m => Some(Self::from_number(
                self.to_number() / 10,
                JPMeshType::Mesh250m,
            )),
            _ => None,
        }
    }

    pub fn to_number(&self) -> u64 {
        let mut result = 0;
        for &digit in self.to_slice() {
            result = result * 10 + digit as u64;
        }
        result
    }

    pub fn to_slice(&self) -> &[u8] {
        match self.mesh_type {
            JPMeshType::Mesh80km => &self.seed.code_2[..JPMeshType::Mesh80km.code_length()],
            JPMeshType::Mesh10km => &self.seed.code_2[..JPMeshType::Mesh10km.code_length()],
            JPMeshType::Mesh1km => &self.seed.code_2[..JPMeshType::Mesh1km.code_length()],
            JPMeshType::Mesh500m => &self.seed.code_2[..JPMeshType::Mesh500m.code_length()],
            JPMeshType::Mesh250m => &self.seed.code_2[..JPMeshType::Mesh250m.code_length()],
            JPMeshType::Mesh125m => &self.seed.code_2[..JPMeshType::Mesh125m.code_length()],
        }
    }

    pub fn bounds(&self) -> Rect2D<f64> {
        self.seed.bounds(self.mesh_type)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JPMeshCodeSeed {
    // mesh code for 80km, 10km, 1km, 500m, 250m, 125m
    code_2: [u8; 11],
}

impl JPMeshCodeSeed {
    fn new(coords: Coordinate2D<f64>) -> Self {
        // latitude / interval (Mesh80km) = p % a
        let p = (coords.y / JPMeshType::Mesh80km.lat_interval()).floor() as u8;
        let a = coords.y % JPMeshType::Mesh80km.lat_interval();

        // a / lat_interval (Mesh10km) = q % b
        let q = (a / JPMeshType::Mesh10km.lat_interval()).floor() as u8;
        let b = a % JPMeshType::Mesh10km.lat_interval();

        // b / lat_interval (Mesh1km) = r % c
        let r = (b / JPMeshType::Mesh1km.lat_interval()).floor() as u8;
        let c = b % JPMeshType::Mesh1km.lat_interval();

        // c / lat_interval (Mesh500m) = s % d
        let s = (c / JPMeshType::Mesh500m.lat_interval()).floor() as u8;
        let d = c % JPMeshType::Mesh500m.lat_interval();

        // d / lat_interval (Mesh250m) = t % e
        let t = (d / JPMeshType::Mesh250m.lat_interval()).floor() as u8;
        let e = d % JPMeshType::Mesh250m.lat_interval();

        // e / lat_interval (Mesh125m) = tt
        let tt = (e / JPMeshType::Mesh125m.lat_interval()).floor() as u8;

        // longitude - 100 degrees = u % f
        let u = (coords.x - 100.0).floor() as u8;
        let f = coords.x - 100.0 - u as f64;

        // f / lng_interval (Mesh10km) = v % g
        let v = (f / JPMeshType::Mesh10km.lng_interval()).floor() as u8;
        let g = f % JPMeshType::Mesh10km.lng_interval();

        // g / lng_interval (Mesh1km) = w % h
        let w = (g / JPMeshType::Mesh1km.lng_interval()).floor() as u8;
        let h = g % JPMeshType::Mesh1km.lng_interval();

        // h / lng_interval (Mesh500m) = x % i
        let x = (h / JPMeshType::Mesh500m.lng_interval()).floor() as u8;
        let i = h % JPMeshType::Mesh500m.lng_interval();

        // i / lng_interval (Mesh250m) = y % j
        let y = (i / JPMeshType::Mesh250m.lng_interval()).floor() as u8;
        let j = i % JPMeshType::Mesh250m.lng_interval();

        // j / lng_interval (Mesh125m) = yy
        let yy = (j / JPMeshType::Mesh125m.lng_interval()).floor() as u8;

        // (s * 2)+(x + 1)= m
        let m = (s * 2) + (x + 1);

        // (t * 2)+(y + 1)= n
        let n = (t * 2) + (y + 1);

        // (tt * 2)+(yy + 1)= nn
        let nn = (tt * 2) + (yy + 1);

        // First 6 digits
        let head = {
            let p1 = (p / 10) % 10;
            let p2 = p % 10;
            let u1 = (u / 10) % 10;
            let u2 = u % 10;
            [p1, p2, u1, u2, q, v]
        };

        // Last 5 digits
        let tail_bin = { [r, w, m, n, nn] };

        let mut code_2 = [0u8; 11];
        code_2[..6].copy_from_slice(&head);
        code_2[6..11].copy_from_slice(&tail_bin);

        JPMeshCodeSeed { code_2 }
    }

    fn bounds(&self, mesh_type: JPMeshType) -> Rect2D<f64> {
        let mut code_2 = self.code_2;

        for (i, code) in code_2.iter_mut().enumerate().skip(8) {
            *code = if i >= mesh_type.code_length() { 1 } else { 0 };
        }

        let p = (code_2[0] * 10 + code_2[1]) as f64;
        let u = (code_2[2] * 10 + code_2[3]) as f64;
        let q = code_2[4] as f64;
        let v = code_2[5] as f64;
        let r = code_2[6] as f64;
        let w = code_2[7] as f64;
        let m = code_2[8] as f64;
        let n = code_2[9] as f64;
        let nn = code_2[10] as f64;

        // Calculate latitude (southwest corner)
        let lat_base = p * JPMeshType::Mesh80km.lat_interval();
        let lat_q = q * JPMeshType::Mesh10km.lat_interval();
        let lat_r = r * JPMeshType::Mesh1km.lat_interval();
        let lat_s = ((m - 1.0) / 2.0).floor() * JPMeshType::Mesh500m.lat_interval();
        let lat_t = ((n - 1.0) / 2.0).floor() * JPMeshType::Mesh250m.lat_interval();
        let lat_tt = ((nn - 1.0) / 2.0).floor() * JPMeshType::Mesh125m.lat_interval();

        // Calculate longitude (southwest corner)
        let lng_base = 100.0 + u;
        let lng_v = v * JPMeshType::Mesh10km.lng_interval();
        let lng_w = w * JPMeshType::Mesh1km.lng_interval();
        let lng_x = ((m - 1.0) % 2.0) * JPMeshType::Mesh500m.lng_interval();
        let lng_y = ((n - 1.0) % 2.0) * JPMeshType::Mesh250m.lng_interval();
        let lng_yy = ((nn - 1.0) % 2.0) * JPMeshType::Mesh125m.lng_interval();

        // Coordinates of southwest corner
        let min_lat = lat_base + lat_q + lat_r + lat_s + lat_t + lat_tt;
        let min_lng = lng_base + lng_v + lng_w + lng_x + lng_y + lng_yy;

        // Coordinates of northeast corner
        let max_lat = min_lat + mesh_type.lat_interval();
        let max_lng = min_lng + mesh_type.lng_interval();

        Rect::new(
            Coordinate2D::new_(min_lng, min_lat),
            Coordinate2D::new_(max_lng, max_lat),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-6;

    #[macro_export]
    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {
            assert!(
                ($a - $b).abs() < EPSILON,
                "assertion failed: `(left ≈ right)`\n  left: `{}`\n right: `{}`\n",
                $a,
                $b
            );
        };
    }

    #[macro_export]
    macro_rules! assert_mesh_size_correct {
        ($bounds:expr, $lng_interval_seconds:expr, $lat_interval_seconds:expr) => {
            let min_coord = $bounds.min();
            let max_coord = $bounds.max();
            assert_approx_eq!(max_coord.x - min_coord.x, $lng_interval_seconds / 3600.0);
            assert_approx_eq!(max_coord.y - min_coord.y, $lat_interval_seconds / 3600.0);
        };
    }

    #[macro_export]
    macro_rules! assert_rect_includes {
        ($rect:expr, $point:expr) => {
            assert!(
                $rect.min().x <= $point.x
                    && $rect.min().y <= $point.y
                    && $rect.max().x > $point.x
                    && $rect.max().y > $point.y
            );
        };
    }

    #[macro_export]
    macro_rules! assert_rect_not_includes {
        ($rect:expr, $point:expr) => {
            assert!(
                $rect.min().x > $point.x
                    || $rect.min().y > $point.y
                    || $rect.max().x <= $point.x
                    || $rect.max().y <= $point.y
            );
        };
    }

    // small offset for checking coordinate inside the mesh
    const INNER_OFFSET: f64 = 0.000003;

    #[derive(Debug)]
    struct TestCase {
        mesh_code_number: u64,
        mesh_type: JPMeshType,
        left_bottom: Coordinate2D<f64>,
    }

    impl TestCase {
        fn inner_coord(&self) -> Coordinate2D<f64> {
            Coordinate2D::new_(
                self.left_bottom.x + INNER_OFFSET,
                self.left_bottom.y + INNER_OFFSET,
            )
        }
    }

    fn get_test_cases() -> Vec<TestCase> {
        vec![
            TestCase {
                mesh_code_number: 64414277,
                mesh_type: JPMeshType::Mesh1km,
                left_bottom: Coordinate2D::new_(141.3375, 43.058333),
            },
            TestCase {
                mesh_code_number: 61401589,
                mesh_type: JPMeshType::Mesh1km,
                left_bottom: Coordinate2D::new_(140.7375, 40.816667),
            },
            TestCase {
                mesh_code_number: 59414142,
                mesh_type: JPMeshType::Mesh1km,
                left_bottom: Coordinate2D::new_(141.15, 39.7),
            },
            TestCase {
                mesh_code_number: 57403629,
                mesh_type: JPMeshType::Mesh1km,
                left_bottom: Coordinate2D::new_(140.8625, 38.266667),
            },
        ]
    }

    #[test]
    fn test_mesh_code_generation() {
        for test_case in get_test_cases() {
            let inner_coord = test_case.inner_coord();
            let mesh_code = JPMeshCode::new(inner_coord, test_case.mesh_type);

            let actual_number = mesh_code.to_number();
            assert_eq!(actual_number, test_case.mesh_code_number);
        }
    }

    #[test]
    fn test_mesh_code_bounds() {
        for test_case in get_test_cases() {
            let inner_coord = test_case.inner_coord();
            let mesh_code = JPMeshCode::new(inner_coord, test_case.mesh_type);

            let bounds = mesh_code.bounds();
            let min_coord = bounds.min();

            // check if the bottom left coordinate is correct
            assert_approx_eq!(min_coord.x, test_case.left_bottom.x);
            assert_approx_eq!(min_coord.y, test_case.left_bottom.y);

            // check if the size of the area is correct
            assert_mesh_size_correct!(bounds, 45.0, 30.0);
        }
    }

    #[test]
    fn test_mesh_code_from_number_to_number() {
        for test_case in get_test_cases() {
            let mesh_code =
                JPMeshCode::from_number(test_case.mesh_code_number, test_case.mesh_type);
            let number = mesh_code.to_number();
            assert_eq!(number, test_case.mesh_code_number);
        }
    }

    #[test]
    fn test_mesh_code_upscale() {
        // Create larger scale meshes by truncating digits from the dataset's mesh_code,
        // and verify that the dataset's inner coordinates are contained within these mesh boundaries
        for test_case in get_test_cases() {
            let mesh_code =
                JPMeshCode::from_number(test_case.mesh_code_number, test_case.mesh_type);

            // 1km -> 10km
            let mesh_code_10km = mesh_code.upscale().unwrap();
            let bounds_10km = mesh_code_10km.bounds();

            // verify that inner coordinates are contained within the mesh boundaries
            let inner_coord = test_case.inner_coord();
            assert_rect_includes!(bounds_10km, inner_coord);

            // verify that mesh size is correct
            assert_mesh_size_correct!(bounds_10km, 450.0, 300.0);

            // 1km -> 80km
            let mesh_code_80km = mesh_code_10km.upscale().unwrap();
            let bounds_80km = mesh_code_80km.bounds();

            // check if the inner coordinate is included in the mesh
            assert_rect_includes!(bounds_80km, inner_coord);

            // check if the size of the area is correct
            assert_mesh_size_correct!(bounds_80km, 3600.0, 2400.0);
        }
    }

    #[test]
    fn test_mesh_code_downscale() {
        // Create smaller scale meshes by adding digits to the dataset's mesh_code,
        // and verify that the dataset's inner coordinates are contained within these mesh boundaries
        for test_case in get_test_cases() {
            // the mesh code will be (test_case.mesh_code_number * 1000 + 111)
            let inner_coord = test_case.inner_coord();
            let mesh_codes =
                JPMeshCode::from_number(test_case.mesh_code_number, test_case.mesh_type);

            // 1km -> 500m
            let mesh_codes_500m = mesh_codes.downscale();
            for (i, mesh_code_500m) in mesh_codes_500m.iter().enumerate() {
                let bounds_500m = mesh_code_500m.bounds();

                assert_mesh_size_correct!(bounds_500m, 22.5, 15.0);

                if i == 0 {
                    assert_rect_includes!(bounds_500m, inner_coord);
                } else {
                    assert_rect_not_includes!(bounds_500m, inner_coord);
                }
            }

            // 1km -> 250m
            let mesh_codes_250m = mesh_codes_500m.first().unwrap().downscale();
            for (i, mesh_code_250m) in mesh_codes_250m.iter().enumerate() {
                let bounds_250m = mesh_code_250m.bounds();

                assert_mesh_size_correct!(bounds_250m, 11.25, 7.5);

                if i == 0 {
                    assert_rect_includes!(bounds_250m, inner_coord);
                } else {
                    assert_rect_not_includes!(bounds_250m, inner_coord);
                }
            }

            // 1km -> 125m
            let mesh_codes_125m = mesh_codes_250m.first().unwrap().downscale();
            for (i, mesh_code_125m) in mesh_codes_125m.iter().enumerate() {
                let bounds_125m = mesh_code_125m.bounds();

                assert_mesh_size_correct!(bounds_125m, 5.625, 3.75);

                if i == 0 {
                    assert_rect_includes!(bounds_125m, inner_coord);
                } else {
                    assert_rect_not_includes!(bounds_125m, inner_coord);
                }
            }
        }
    }

    #[test]
    fn test_mesh_type_order() {
        let mesh_types = [JPMeshType::Mesh80km,
            JPMeshType::Mesh10km,
            JPMeshType::Mesh1km,
            JPMeshType::Mesh500m,
            JPMeshType::Mesh250m,
            JPMeshType::Mesh125m];

        for i in 1..mesh_types.len() {
            assert!(mesh_types[i - 1] > mesh_types[i]);
        }
    }
}
