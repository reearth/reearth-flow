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
    /// 10分の1地域メッシュ
    Mesh100m,
}

impl Ord for JPMeshType {
    // Bigger cells sort later. Comparing code length would tie the 1/10 and
    // 1/4 meshes (both 10 digits) while they are not equal.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.lat_interval_seconds()
            .partial_cmp(&other.lat_interval_seconds())
            .expect("mesh intervals are finite constants")
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
            JPMeshType::Mesh100m => 10,
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
            JPMeshType::Mesh100m => 3.0,
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
            JPMeshType::Mesh100m => 4.5,
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

/// Whether a geographic coordinate has a JIS X 0410 mesh code: the 1st mesh
/// code's two-digit latitude and longitude fields only cover this range.
pub fn in_jis_range(lng: f64, lat: f64) -> bool {
    (0.0..100.0 / 1.5).contains(&lat) && (100.0..200.0).contains(&lng)
}

impl JPMeshCode {
    /// The mesh containing `coords` (`x` = longitude, `y` = latitude, JGD2011),
    /// or `None` outside the range the code can express.
    pub fn try_new(coords: Coordinate2D<f64>, mesh_type: JPMeshType) -> Option<Self> {
        in_jis_range(coords.x, coords.y).then(|| Self::new(coords, mesh_type))
    }

    pub fn mesh_type(&self) -> JPMeshType {
        self.mesh_type
    }

    pub fn new(coords: Coordinate2D<f64>, mesh_type: JPMeshType) -> Self {
        let seed = JPMeshCodeSeed::new(coords);
        JPMeshCode { mesh_type, seed }
    }

    pub fn from_number(mesh_code: u64, mesh_type: JPMeshType) -> Self {
        if mesh_type == JPMeshType::Mesh100m {
            let tenth = [((mesh_code / 10) % 10) as u8, (mesh_code % 10) as u8];
            let mut code = Self::from_number(mesh_code / 100, JPMeshType::Mesh1km);
            code.mesh_type = JPMeshType::Mesh100m;
            code.seed.tenth = tenth;
            return code;
        }
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
            seed: JPMeshCodeSeed {
                code_2,
                tenth: [0, 0],
            },
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
            JPMeshType::Mesh100m => Some(Self::from_number(
                self.to_number() / 100,
                JPMeshType::Mesh1km,
            )),
            _ => None,
        }
    }

    pub fn to_number(&self) -> u64 {
        self.digits().iter().fold(0, |acc, &d| acc * 10 + d as u64)
    }

    pub fn digits(&self) -> Vec<u8> {
        match self.mesh_type {
            JPMeshType::Mesh100m => {
                let mut d = self.seed.code_2[..8].to_vec();
                d.extend_from_slice(&self.seed.tenth);
                d
            }
            t => self.seed.code_2[..t.code_length()].to_vec(),
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
    // 1/10 mesh digits [lat_index, lng_index] below the 3rd mesh, each 0-9
    tenth: [u8; 2],
}

impl JPMeshCodeSeed {
    fn new(coords: Coordinate2D<f64>) -> Self {
        // Snap each axis once to whole units, then derive every digit with
        // integer arithmetic so all levels stay consistent. Latitude units are
        // 0.75 arc-seconds and longitude units 1.125, the greatest common step
        // of every mesh size (the 1st mesh is 3200 units, the 1/10 mesh 4).
        // TOL is a tiny fraction of a unit (about 1e-10 degrees) so a point on
        // a cell edge, whose float division can land just below the integer,
        // goes north or east as the edge rule requires.
        const TOL: f64 = 1e-6;
        let n_lat = (coords.y * 3600.0 / 0.75 + TOL).floor() as u64;
        let n_lng = ((coords.x - 100.0) * 3600.0 / 1.125 + TOL).floor() as u64;

        let p = n_lat / 3200;
        let q = (n_lat % 3200) / 400;
        let r = (n_lat % 400) / 40;
        let s = (n_lat % 40) / 20;
        let t = (n_lat % 20) / 10;
        let tt = (n_lat % 10) / 5;
        let tenth_lat = (n_lat % 40) / 4;

        let u = n_lng / 3200;
        let v = (n_lng % 3200) / 400;
        let w = (n_lng % 400) / 40;
        let x = (n_lng % 40) / 20;
        let y = (n_lng % 20) / 10;
        let yy = (n_lng % 10) / 5;
        let tenth_lng = (n_lng % 40) / 4;

        let digit = |d: u64| d as u8;
        let code_2 = [
            digit((p / 10) % 10),
            digit(p % 10),
            digit((u / 10) % 10),
            digit(u % 10),
            digit(q),
            digit(v),
            digit(r),
            digit(w),
            digit(s * 2 + x + 1),
            digit(t * 2 + y + 1),
            digit(tt * 2 + yy + 1),
        ];

        JPMeshCodeSeed {
            code_2,
            tenth: [digit(tenth_lat), digit(tenth_lng)],
        }
    }

    fn bounds(&self, mesh_type: JPMeshType) -> Rect2D<f64> {
        if mesh_type == JPMeshType::Mesh100m {
            let parent = self.bounds(JPMeshType::Mesh1km);
            let min_lat =
                parent.min().y + self.tenth[0] as f64 * JPMeshType::Mesh100m.lat_interval();
            let min_lng =
                parent.min().x + self.tenth[1] as f64 * JPMeshType::Mesh100m.lng_interval();
            return Rect::new(
                Coordinate2D::new_(min_lng, min_lat),
                Coordinate2D::new_(
                    min_lng + JPMeshType::Mesh100m.lng_interval(),
                    min_lat + JPMeshType::Mesh100m.lat_interval(),
                ),
            );
        }

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
    fn test_mesh_type_order() {
        let mesh_types = [
            JPMeshType::Mesh80km,
            JPMeshType::Mesh10km,
            JPMeshType::Mesh1km,
            JPMeshType::Mesh500m,
            JPMeshType::Mesh250m,
            JPMeshType::Mesh125m,
        ];

        for i in 1..mesh_types.len() {
            assert!(mesh_types[i - 1] > mesh_types[i]);
        }
    }

    #[test]
    fn mesh_100m_round_trips_and_has_the_right_size() {
        // 3rd mesh 53394611 (Tokyo Station area), lat index 2, lng index 7.
        let sw = JPMeshCode::from_number(53394611, JPMeshType::Mesh1km)
            .bounds()
            .min();
        let inside = Coordinate2D::new_(
            sw.x + 7.0 * 4.5 / 3600.0 + INNER_OFFSET,
            sw.y + 2.0 * 3.0 / 3600.0 + INNER_OFFSET,
        );
        let code = JPMeshCode::new(inside, JPMeshType::Mesh100m);
        assert_eq!(code.to_number(), 5339461127);
        assert_eq!(code.digits().len(), 10);

        let back = JPMeshCode::from_number(5339461127, JPMeshType::Mesh100m);
        assert_eq!(back.to_number(), code.to_number());
        let bounds = back.bounds();
        assert_mesh_size_correct!(bounds, 4.5, 3.0);
        assert_rect_includes!(bounds, inside);
        assert_eq!(back.upscale().unwrap().to_number(), 53394611);
    }

    #[test]
    fn mesh_100m_cells_tile_their_parent_from_inside_bounds() {
        let parent = JPMeshCode::from_number(53394611, JPMeshType::Mesh1km).bounds();
        // Shrink slightly so the enumeration stays inside the parent.
        let eps = 1e-9;
        let inner = Rect2D::new(
            Coordinate2D::new_(parent.min().x + eps, parent.min().y + eps),
            Coordinate2D::new_(parent.max().x - eps, parent.max().y - eps),
        );
        let mut codes: Vec<u64> = JPMeshCode::from_inside_bounds(inner, JPMeshType::Mesh100m)
            .iter()
            .map(JPMeshCode::to_number)
            .filter(|n| n / 100 == 53394611)
            .collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), 100);
        assert_eq!(codes[0], 5339461100);
        assert_eq!(codes[99], 5339461199);
    }

    #[test]
    fn ordering_puts_bigger_cells_first_and_is_total() {
        let mut all = vec![
            JPMeshType::Mesh100m,
            JPMeshType::Mesh1km,
            JPMeshType::Mesh250m,
            JPMeshType::Mesh80km,
            JPMeshType::Mesh125m,
        ];
        all.sort();
        assert_eq!(
            all,
            vec![
                JPMeshType::Mesh100m,
                JPMeshType::Mesh125m,
                JPMeshType::Mesh250m,
                JPMeshType::Mesh1km,
                JPMeshType::Mesh80km,
            ]
        );
        assert_ne!(
            JPMeshType::Mesh100m.cmp(&JPMeshType::Mesh250m),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn coordinates_outside_the_jis_range_have_no_code() {
        for (lng, lat) in [(99.9, 35.0), (200.0, 35.0), (139.0, -0.1), (139.0, 66.7)] {
            assert!(
                JPMeshCode::try_new(Coordinate2D::new_(lng, lat), JPMeshType::Mesh1km).is_none(),
                "({lng}, {lat}) should be out of range"
            );
        }
        assert!(
            JPMeshCode::try_new(Coordinate2D::new_(139.7, 35.6), JPMeshType::Mesh1km).is_some()
        );
    }

    #[test]
    fn a_point_on_a_cell_corner_goes_north_east() {
        // Exactly the SW corner of 3rd mesh 53394611, as its own bounds give it.
        let cell = JPMeshCode::from_number(53394611, JPMeshType::Mesh1km);
        let sw = cell.bounds().min();
        let at_corner = JPMeshCode::try_new(sw, JPMeshType::Mesh1km).unwrap();
        assert_eq!(at_corner.to_number(), 53394611);
        // The NE corner belongs to the diagonal neighbour, not to this cell.
        let ne = cell.bounds().max();
        let beyond = JPMeshCode::try_new(ne, JPMeshType::Mesh1km).unwrap();
        assert_ne!(beyond.to_number(), 53394611);
    }

    #[test]
    fn a_point_just_below_a_first_mesh_boundary_stays_consistent() {
        // 36 degrees is a 1st-mesh lat boundary (p = 54).
        let lat = 36.0 - 1e-13;
        let point = Coordinate2D::new_(139.7, lat);
        let code = JPMeshCode::try_new(point, JPMeshType::Mesh1km).unwrap();
        let b = code.bounds();
        // Either the cell just south, or the boundary cell with finer digits 0.
        assert!(
            b.min().y <= lat + 1e-9 && b.max().y >= lat - 1e-9,
            "bounds {b:?} do not contain or touch the point"
        );
        let d = code.digits();
        assert!(
            (d[..2] == [5, 3] && d[4] == 7 && d[6] == 9)
                || (d[..2] == [5, 4] && d[4] == 0 && d[6] == 0),
            "inconsistent digits {d:?}"
        );
    }

    #[test]
    fn a_point_on_a_mesh_100m_sw_corner_maps_to_that_cell() {
        for n in [5339461100u64, 5339461127, 5339461199, 5339461109] {
            let cell = JPMeshCode::from_number(n, JPMeshType::Mesh100m);
            let sw = cell.bounds().min();
            let got = JPMeshCode::try_new(sw, JPMeshType::Mesh100m).unwrap();
            assert_eq!(got.to_number(), n);
        }
    }
}
