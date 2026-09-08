//! What writing a `Feature` as GeoJSON returns, in the form both geometry worlds
//! share, so a caller writes one code path whatever `Feature::geometry` is.
//!
//! Re-exported from `conversion::geojson` in both worlds; the module exists only
//! while there are two of them, and folds into the surviving one afterwards.

use std::fmt;

use reearth_flow_geometry::coordinate::EpsgCode;

/// What one feature writes to: the GeoJSON features it becomes, and what the
/// coordinates they carry are expressed in.
///
/// The coverage comes out of the same pass as the features, so a caller that has
/// to declare the CRS of what it wrote reads it off here instead of walking every
/// geometry a second time to recover it.
pub struct WrittenFeature {
    pub features: Vec<geojson::Feature>,
    pub crs: CrsCoverage,
}

/// How far one coordinate reference system covers the coordinates written so far.
///
/// A GeoJSON `FeatureCollection` can name one CRS for all of its coordinates or
/// none, so a caller writing that member needs to know whether a single code
/// covers them. Since the coverage is accumulated as the coordinates are written,
/// it describes exactly what reached the file: a geometry the writer dropped
/// contributes nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CrsCoverage {
    /// Nothing carrying coordinates has been written: the identity of
    /// [`and`](Self::and), and distinct from coordinates that name no CRS.
    #[default]
    NoCoordinates,
    /// Every written coordinate is expressed in this one CRS.
    Single(EpsgCode),
    /// Written coordinates are expressed in more than one CRS.
    Mixed { first: EpsgCode, other: EpsgCode },
    /// A written coordinate is expressed outside any CRS, so no code covers the
    /// rest either.
    OutsideAnyCrs,
}

impl CrsCoverage {
    /// The coverage of two writes, together. Associative, with
    /// [`NoCoordinates`](Self::NoCoordinates) as its identity, so a caller folds
    /// over features in any grouping and gets the same answer.
    pub fn and(self, other: Self) -> Self {
        use CrsCoverage::*;
        match (self, other) {
            // A coordinate outside every CRS is not covered by the code the
            // others carry, so nothing covers the whole.
            (OutsideAnyCrs, _) | (_, OutsideAnyCrs) => OutsideAnyCrs,
            (NoCoordinates, coverage) | (coverage, NoCoordinates) => coverage,
            // Two codes already name the mixture; further ones add nothing.
            (mixed @ Mixed { .. }, _) => mixed,
            (Single(first), Mixed { first: a, other: b }) => Mixed {
                first,
                other: if first == a { b } else { a },
            },
            (Single(first), Single(other)) if first != other => Mixed { first, other },
            (single @ Single(_), Single(_)) => single,
        }
    }
}

impl fmt::Display for CrsCoverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoCoordinates => write!(f, "no coordinate was written"),
            Self::Single(code) => write!(f, "every written coordinate is in EPSG:{code}"),
            Self::Mixed { first, other } => write!(
                f,
                "written coordinates are in both EPSG:{first} and EPSG:{other}"
            ),
            Self::OutsideAnyCrs => write!(f, "a written coordinate is outside any CRS"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn epsg(code: u16) -> EpsgCode {
        EpsgCode::new(code)
    }

    // Accumulating over features: nothing written, then one code, then the same
    // code again, then a second one, after which further codes add nothing.
    #[test]
    fn accumulating_narrows_to_one_code_and_then_to_a_mixture() {
        let mut coverage = CrsCoverage::default();
        assert_eq!(coverage, CrsCoverage::NoCoordinates);

        coverage = coverage.and(CrsCoverage::Single(epsg(6675)));
        assert_eq!(coverage, CrsCoverage::Single(epsg(6675)));

        coverage = coverage.and(CrsCoverage::Single(epsg(6675)));
        assert_eq!(coverage, CrsCoverage::Single(epsg(6675)));

        // A feature writing no coordinates says nothing about the CRS.
        coverage = coverage.and(CrsCoverage::NoCoordinates);
        assert_eq!(coverage, CrsCoverage::Single(epsg(6675)));

        let mixed = CrsCoverage::Mixed {
            first: epsg(6675),
            other: epsg(6669),
        };
        coverage = coverage.and(CrsCoverage::Single(epsg(6669)));
        assert_eq!(coverage, mixed);

        coverage = coverage.and(CrsCoverage::Single(epsg(3857)));
        assert_eq!(coverage, mixed);
    }

    // Coordinates outside any CRS are not covered by the code the others carry, so
    // they leave the whole uncovered whatever is folded in afterwards.
    #[test]
    fn coordinates_outside_any_crs_leave_nothing_covered() {
        let mut coverage = CrsCoverage::Single(epsg(6675)).and(CrsCoverage::OutsideAnyCrs);
        assert_eq!(coverage, CrsCoverage::OutsideAnyCrs);

        coverage = coverage.and(CrsCoverage::Single(epsg(6675)));
        assert_eq!(coverage, CrsCoverage::OutsideAnyCrs);
    }
}
