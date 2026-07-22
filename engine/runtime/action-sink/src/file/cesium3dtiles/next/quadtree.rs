//! Containment-placement quadtree: assigns each feature to the deepest cell
//! that fully contains it, deriving every cell's region / geometric error
//! purely from the dataset's root extent plus `level` (3D Tiles 1.1 implicit
//! tiling re-derives both from `level` alone, rather than reading them per
//! tile). Rooted at the dataset's own extent, not the whole globe; regional
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

/// The deepest cell, bounded by `max_depth`, that fully contains `feature`
/// within `root`. `root` must already contain `feature` — the caller derives
/// `root` as the union of every placed feature's box.
pub(super) fn place(root: &GeoBox, feature: &GeoBox, max_depth: u32) -> Cell {
    let mut best = Cell::root();
    for level in 1..=max_depth {
        let n = 1u32 << level;
        let (Some((x_lo, x_hi)), Some((y_lo, y_hi))) = (
            span_indices(root.west, root.east, feature.west, feature.east, n),
            span_indices(root.south, root.north, feature.south, feature.north, n),
        ) else {
            break;
        };
        if x_lo != x_hi || y_lo != y_hi {
            break;
        }
        best = Cell {
            level,
            x: x_lo,
            y: y_lo,
        };
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

/// The region `cell` covers, derived purely from `root` and `cell`'s
/// `level`/`x`/`y` — the same halving a 1.1 implicit-tiling client performs
/// on its own, so this must stay exactly reproducible from `level` alone (no
/// per-tile tightening).
pub(super) fn cell_region(root: &GeoBox, cell: Cell) -> GeoBox {
    let n = 1u32 << cell.level;
    let lon_step = (root.east - root.west) / n as f64;
    let lat_step = (root.north - root.south) / n as f64;
    let west = root.west + cell.x as f64 * lon_step;
    let south = root.south + cell.y as f64 * lat_step;
    GeoBox {
        west,
        east: west + lon_step,
        south,
        north: south + lat_step,
        min_height: root.min_height,
        max_height: root.max_height,
    }
}

/// The root region's ground-diagonal size in metres, evaluated at its centre
/// latitude — the basis `geometric_error` halves from. Regional extents only
/// (no polar / antimeridian handling), matching `place`.
pub(super) fn root_ground_diagonal_m(root: &GeoBox) -> f64 {
    const METRES_PER_DEGREE_LAT: f64 = 111_320.0; // WGS84 mean meridional degree
    let centre_lat = (root.south + root.north) / 2.0;
    let lat_span_m = (root.north - root.south) * METRES_PER_DEGREE_LAT;
    let lon_span_m =
        (root.east - root.west) * METRES_PER_DEGREE_LAT * centre_lat.to_radians().cos();
    lat_span_m.hypot(lon_span_m)
}

/// Geometric error at `level`, halving from the root per level — the fixed
/// relationship 3D Tiles 1.1 implicit tiling requires (a client derives every
/// non-root tile's error this way, so the server has no freedom to pick
/// anything else).
pub(super) fn geometric_error(root_ground_diagonal_m: f64, level: u32) -> f64 {
    root_ground_diagonal_m / (1u64 << level) as f64
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
        assert_eq!(place(&root, &root, 10), Cell::root());
    }

    #[test]
    fn feature_within_one_quadrant_descends() {
        let root = geobox(0.0, 0.0, 10.0, 10.0);
        let feature = geobox(1.0, 1.0, 2.0, 2.0);
        let cell = place(&root, &feature, 10);
        assert_eq!(
            cell,
            Cell {
                level: 2,
                x: 0,
                y: 0
            }
        );
    }

    #[test]
    fn feature_straddling_a_boundary_stays_shallow() {
        let root = geobox(0.0, 0.0, 10.0, 10.0);
        // Straddles the x=5 boundary at level 1.
        let feature = geobox(4.0, 1.0, 6.0, 2.0);
        assert_eq!(place(&root, &feature, 10), Cell::root());
    }

    #[test]
    fn max_depth_caps_placement() {
        let root = geobox(0.0, 0.0, 10.0, 10.0);
        let point = geobox(1.0, 1.0, 1.0, 1.0);
        assert_eq!(place(&root, &point, 3).level, 3);
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

    #[test]
    fn cell_region_round_trips_through_level_and_index() {
        let root = geobox(0.0, 0.0, 10.0, 10.0);
        let region = cell_region(
            &root,
            Cell {
                level: 1,
                x: 1,
                y: 0,
            },
        );
        assert_eq!((region.west, region.east), (5.0, 10.0));
        assert_eq!((region.south, region.north), (0.0, 5.0));
    }

    #[test]
    fn geometric_error_halves_per_level() {
        let diag = 1000.0;
        assert_eq!(geometric_error(diag, 0), 1000.0);
        assert_eq!(geometric_error(diag, 1), 500.0);
        assert_eq!(geometric_error(diag, 2), 250.0);
    }
}
