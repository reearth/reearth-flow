//! What writing a `Feature` returns, in the form both geometry worlds share, so
//! a caller writes one code path whatever `Feature::geometry` is.
//!
//! [`WrittenFeature`] is GeoJSON's own. It is re-exported from
//! `conversion::geojson` in both worlds, and the module exists for it only while
//! there are two GeoJSON conversions, folding into the surviving one afterwards.
//!
//! [`CrsCoverage`] outgrew that scope: it describes what *any* write's
//! coordinates are expressed in, and the CityGML writer folds it to decide its
//! `srsName`. It is therefore also re-exported as `conversion::CrsCoverage`,
//! which is the path a non-GeoJSON caller should use, and it outlives the
//! GeoJSON split.

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
    /// whole. The rest are all in `alongside`, when any are in a CRS: the code
    /// is kept so a caller declaring one for the CRS-less coordinates can tell
    /// whether it agrees with the others.
    OutsideAnyCrs { alongside: Option<EpsgCode> },
}

impl CrsCoverage {
    /// Coordinates outside any CRS, beside others all in `alongside`, if any.
    pub fn outside(alongside: Option<EpsgCode>) -> Self {
        Self::OutsideAnyCrs { alongside }
    }

    /// The one code the coordinates in a CRS are in, if there is exactly one.
    fn code(self) -> Option<EpsgCode> {
        match self {
            Self::Single(code)
            | Self::OutsideAnyCrs {
                alongside: Some(code),
            } => Some(code),
            _ => None,
        }
    }

    /// The coverage of two writes, together. Associative, with
    /// [`NoCoordinates`](Self::NoCoordinates) as its identity, so a caller folds
    /// over features in any grouping and gets the same answer.
    ///
    /// A mixture outranks a CRS-less coordinate: both leave the whole
    /// uncovered, but only the mixture is something no declared code can fix.
    pub fn and(self, other: Self) -> Self {
        use CrsCoverage::*;
        match (self, other) {
            (NoCoordinates, coverage) | (coverage, NoCoordinates) => coverage,
            // Two codes already name the mixture; further ones add nothing.
            (mixed @ Mixed { .. }, _) => mixed,
            (this, Mixed { first: a, other: b }) => match this.code() {
                Some(first) => Mixed {
                    first,
                    other: if first == a { b } else { a },
                },
                None => Mixed { first: a, other: b },
            },
            (Single(first), Single(other)) if first != other => Mixed { first, other },
            (single @ Single(_), Single(_)) => single,
            // At least one side is outside any CRS from here on, so the result
            // is too, unless the codes on either side already disagree.
            (this, that) => match (this.code(), that.code()) {
                (Some(first), Some(other)) if first != other => Mixed { first, other },
                (code, other) => OutsideAnyCrs {
                    alongside: code.or(other),
                },
            },
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
            Self::OutsideAnyCrs { alongside: None } => {
                write!(f, "a written coordinate is outside any CRS")
            }
            Self::OutsideAnyCrs {
                alongside: Some(code),
            } => write!(
                f,
                "a written coordinate is outside any CRS, the others are in EPSG:{code}"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

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
        let mut coverage = CrsCoverage::Single(epsg(6675)).and(CrsCoverage::outside(None));
        assert!(matches!(coverage, CrsCoverage::OutsideAnyCrs { .. }));

        coverage = coverage.and(CrsCoverage::Single(epsg(6675)));
        assert!(matches!(coverage, CrsCoverage::OutsideAnyCrs { .. }));
    }

    // The code the other coordinates carry survives a CRS-less one, so a caller
    // can still tell it apart from a code it is asked to declare.
    #[test]
    fn coordinates_outside_any_crs_keep_the_code_written_alongside_them() {
        let coverage = CrsCoverage::Single(epsg(6697)).and(CrsCoverage::outside(None));
        assert_eq!(coverage, CrsCoverage::outside(Some(epsg(6697))));

        let coverage = CrsCoverage::outside(None).and(CrsCoverage::Single(epsg(6697)));
        assert_eq!(coverage, CrsCoverage::outside(Some(epsg(6697))));
    }

    // Coordinates outside any CRS do not hide that the rest are in two.
    #[test]
    fn coordinates_outside_any_crs_do_not_hide_a_mixture() {
        let mixed = CrsCoverage::Mixed {
            first: epsg(6697),
            other: epsg(6677),
        };

        let coverage = CrsCoverage::Single(epsg(6697))
            .and(CrsCoverage::outside(None))
            .and(CrsCoverage::Single(epsg(6677)));
        assert_eq!(coverage, mixed);

        assert_eq!(mixed.and(CrsCoverage::outside(None)), mixed);
        assert_eq!(CrsCoverage::outside(None).and(mixed), mixed);
    }

    // `and` is folded over features in whatever grouping the caller has, so every
    // grouping of every triple must agree.
    #[test]
    fn and_is_associative_with_no_coordinates_as_its_identity() {
        let (a, b, c) = (epsg(6697), epsg(6677), epsg(6668));
        let values = [
            CrsCoverage::NoCoordinates,
            CrsCoverage::Single(a),
            CrsCoverage::Single(b),
            CrsCoverage::Single(c),
            CrsCoverage::Mixed { first: a, other: b },
            CrsCoverage::Mixed { first: b, other: c },
            CrsCoverage::outside(None),
            CrsCoverage::outside(Some(a)),
            CrsCoverage::outside(Some(b)),
        ];

        for x in values {
            assert_eq!(CrsCoverage::NoCoordinates.and(x), x);
            assert_eq!(x.and(CrsCoverage::NoCoordinates), x);
            for y in values {
                for z in values {
                    assert_eq!(
                        x.and(y).and(z),
                        x.and(y.and(z)),
                        "({x:?} and {y:?}) and {z:?}"
                    );
                }
            }
        }
    }
}
