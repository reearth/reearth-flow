//! Loose quadtree: assigns each feature to the deepest cell that holds its
//! centroid and that its box overhangs by at most a tolerance of the cell
//! side. Rooted at the dataset's own extent, not the whole globe; regional
//! extents only, no antimeridian handling.

/// A quadtree cell: level 0 is the dataset root; level `l` has `4^l` cells
/// indexed `x, y` in `0..2^l`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct Cell {
    pub(super) level: u32,
    pub(super) x: u32,
    pub(super) y: u32,
}

impl Cell {
    pub(super) fn root() -> Self {
        Cell {
            level: 0,
            x: 0,
            y: 0,
        }
    }

    pub(super) fn parent(self) -> Option<Self> {
        if self.level == 0 {
            None
        } else {
            Some(Cell {
                level: self.level - 1,
                x: self.x / 2,
                y: self.y / 2,
            })
        }
    }

    /// This cell's ancestor at `level`, or `None` if `level` is deeper than `self`.
    pub(super) fn ancestor_at(self, level: u32) -> Option<Self> {
        if level > self.level {
            return None;
        }
        let shift = self.level - level;
        Some(Cell {
            level,
            x: self.x >> shift,
            y: self.y >> shift,
        })
    }
}

/// A geographic extent: lon/lat in degrees, height in metres.
#[derive(Debug, Clone, Copy)]
pub(super) struct GeoBox {
    pub(super) west: f64,
    pub(super) south: f64,
    pub(super) east: f64,
    pub(super) north: f64,
    pub(super) min_height: f64,
    pub(super) max_height: f64,
}

impl GeoBox {
    /// The bounding box of `points` (lat, lon, height triples — WGS84/EPSG:4979's
    /// own axis order); `None` if empty.
    pub(super) fn of(points: &[[f64; 3]]) -> Option<Self> {
        points.iter().fold(None, |acc, &[lat, lon, height]| {
            Some(match acc {
                None => GeoBox {
                    west: lon,
                    east: lon,
                    south: lat,
                    north: lat,
                    min_height: height,
                    max_height: height,
                },
                Some(b) => GeoBox {
                    west: b.west.min(lon),
                    east: b.east.max(lon),
                    south: b.south.min(lat),
                    north: b.north.max(lat),
                    min_height: b.min_height.min(height),
                    max_height: b.max_height.max(height),
                },
            })
        })
    }

    pub(super) fn union(self, other: Self) -> Self {
        GeoBox {
            west: self.west.min(other.west),
            east: self.east.max(other.east),
            south: self.south.min(other.south),
            north: self.north.max(other.north),
            min_height: self.min_height.min(other.min_height),
            max_height: self.max_height.max(other.max_height),
        }
    }
}

pub(super) fn place_loose(root: &GeoBox, feature: &GeoBox, tolerance: f64, max_depth: u32) -> Cell {
    let lon = (feature.west + feature.east) / 2.0;
    let lat = (feature.south + feature.north) / 2.0;
    let mut best = Cell::root();
    for level in 1..=max_depth {
        let n = 1u32 << level;
        let (Some((x, _)), Some((y, _))) = (
            span_indices(root.west, root.east, lon, lon, n),
            span_indices(root.south, root.north, lat, lat, n),
        ) else {
            break;
        };
        let w = (root.east - root.west) / n as f64;
        let h = (root.north - root.south) / n as f64;
        let cell_west = root.west + x as f64 * w;
        let cell_south = root.south + y as f64 * h;
        if feature.west < cell_west - tolerance * w
            || feature.east > cell_west + w + tolerance * w
            || feature.south < cell_south - tolerance * h
            || feature.north > cell_south + h + tolerance * h
        {
            break;
        }
        best = Cell { level, x, y };
    }
    best
}

/// The `[lo, hi]` cell-index span `feature_lo..feature_hi` occupies when
/// `root_lo..root_hi` is divided into `n` equal, half-open `[.., ..)` cells.
/// `None` when `root_lo..root_hi` is degenerate (zero-width) and so can't be
/// subdivided.
fn span_indices(
    root_lo: f64,
    root_hi: f64,
    feature_lo: f64,
    feature_hi: f64,
    n: u32,
) -> Option<(u32, u32)> {
    let extent = root_hi - root_lo;
    if extent <= 0.0 {
        return None;
    }
    let frac = |v: f64| ((v - root_lo) / extent).clamp(0.0, 1.0);
    let lo_frac = frac(feature_lo);
    let hi_frac = frac(feature_hi);
    let n_minus_1 = (n - 1) as f64;
    let lo_idx = (lo_frac * n as f64).floor().min(n_minus_1) as u32;
    let hi_idx = if hi_frac <= lo_frac {
        lo_idx
    } else {
        (((hi_frac * n as f64).ceil() - 1.0).clamp(0.0, n_minus_1)) as u32
    };
    Some((lo_idx, hi_idx))
}

/// The region's ground-diagonal size in metres, evaluated at its centre
/// latitude. Regional extents only (no polar / antimeridian handling),
/// matching `place_loose`.
pub(super) fn ground_diagonal_m(region: &GeoBox) -> f64 {
    const METRES_PER_DEGREE_LAT: f64 = 111_320.0; // WGS84 mean meridional degree
    let centre_lat = (region.south + region.north) / 2.0;
    let lat_span_m = (region.north - region.south) * METRES_PER_DEGREE_LAT;
    let lon_span_m =
        (region.east - region.west) * METRES_PER_DEGREE_LAT * centre_lat.to_radians().cos();
    lat_span_m.hypot(lon_span_m)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geobox(west: f64, south: f64, east: f64, north: f64) -> GeoBox {
        GeoBox {
            west,
            south,
            east,
            north,
            min_height: 0.0,
            max_height: 0.0,
        }
    }

    #[test]
    fn root_sized_feature_stays_at_root() {
        let root = geobox(0.0, 0.0, 10.0, 10.0);
        assert_eq!(place_loose(&root, &root, 0.5, 10), Cell::root());
    }

    #[test]
    fn loose_placement_descends_across_a_boundary() {
        let root = geobox(0.0, 0.0, 16.0, 16.0);

        // Within tolerance: overhangs level-3 cell 4..6 but stays inside 3..7.
        let within = geobox(3.4, 1.0, 4.6, 2.0);
        assert_eq!(
            place_loose(&root, &within, 0.5, 10),
            Cell {
                level: 3,
                x: 2,
                y: 0
            }
        );

        // Beyond tolerance: west edge 2.9 is past level 3's loose bound 3.0,
        // so placement stops at level 2.
        let beyond = geobox(2.9, 1.0, 7.1, 2.0);
        assert_eq!(
            place_loose(&root, &beyond, 0.5, 10),
            Cell {
                level: 2,
                x: 1,
                y: 0
            }
        );
    }

    #[test]
    fn max_depth_caps_placement() {
        let root = geobox(0.0, 0.0, 10.0, 10.0);
        let point = geobox(1.0, 1.0, 1.0, 1.0);
        assert_eq!(place_loose(&root, &point, 0.5, 3).level, 3);
    }

    #[test]
    fn ancestor_at_matches_repeated_parent_calls() {
        let cell = Cell {
            level: 3,
            x: 5,
            y: 2,
        };
        assert_eq!(cell.ancestor_at(3), Some(cell));
        assert_eq!(cell.ancestor_at(2), cell.parent());
        assert_eq!(cell.ancestor_at(1), cell.parent().unwrap().parent());
        assert_eq!(cell.ancestor_at(4), None);
    }
}
