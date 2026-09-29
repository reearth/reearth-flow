//! Present-but-malformed input, collected (not just warned about) so a strict
//! caller can fail the read naming the offending location, per Action Standard
//! §4.3. See `pipeline::build_features_reporting`.

use std::fmt;

/// A place where present input was malformed. Collected rather than only warned
/// about, so a strict caller can fail the read naming the offending location,
/// per Action Standard §4.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Malformation {
    /// The source file it was found in, when known.
    pub file: String,
    /// The `gml:id` or element name locating it, when known. Empty when neither
    /// is available at the site.
    pub location: String,
    /// What was wrong.
    pub kind: MalformationKind,
    /// Where it was found, beyond `file`/`location`.
    pub detail: MalformationDetail,
}

impl Malformation {
    /// A malformation with `file`/`location` and `detail` left to be filled in.
    pub fn new(kind: MalformationKind) -> Self {
        Self {
            file: String::new(),
            location: String::new(),
            kind,
            detail: MalformationDetail::default(),
        }
    }
}

impl fmt::Display for Malformation {
    /// Formats as much of `file`/`location`/`kind` as is known. Either or both
    /// of `file`/`location` may be empty (some sites have no way to recover
    /// them), so this never prints a bare empty placeholder for them.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.file.is_empty(), self.location.is_empty()) {
            (false, false) => write!(f, "{} at {}: {}", self.file, self.location, self.kind),
            (false, true) => write!(f, "{}: {}", self.file, self.kind),
            (true, false) => write!(f, "at {}: {}", self.location, self.kind),
            (true, true) => write!(f, "{}", self.kind),
        }
    }
}

/// The context a malformation was found in. Each field is empty when unknown.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MalformationDetail {
    /// The `gml:id` of the top-level city object it was found in, or whose
    /// geometry was being resolved when it was found through an `xlink:href`.
    /// Empty for sites outside a city object.
    pub city_object_id: String,
    /// The qualified element name of that city object (e.g. `dem:ReliefFeature`).
    pub city_object_type: String,
}

/// Name the city object on every malformation in `found` that names none yet.
#[cfg(feature = "new-geometry")]
pub(crate) fn name_city_object(found: &mut [Malformation], id: Option<String>, ty: &str) {
    let id = id.unwrap_or_default();
    for m in found {
        if m.detail.city_object_id.is_empty() && m.detail.city_object_type.is_empty() {
            m.detail.city_object_id = id.clone();
            m.detail.city_object_type = ty.to_string();
        }
    }
}

/// What was malformed. One variant per detection site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MalformationKind {
    InvalidPosList,
    PosListLengthNotMultipleOfThree,
    InvalidPos,
    PosOrdinateCountNotThree,
    UnsupportedXlinkHref,
    CyclicXlinkHref,
    NonCurveRingMember,
    ExpectedSurfaceMember,
    SolidWithMultipleExteriors,
    UnexpectedSolidMemberRole,
    /// Carries the weld error's message.
    MeshWeldFailed(String),
    ExpectedCurve,
    InvalidSolidBoundary,
}

impl MalformationKind {
    /// The variant name, for output as an attribute value.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidPosList => "InvalidPosList",
            Self::PosListLengthNotMultipleOfThree => "PosListLengthNotMultipleOfThree",
            Self::InvalidPos => "InvalidPos",
            Self::PosOrdinateCountNotThree => "PosOrdinateCountNotThree",
            Self::UnsupportedXlinkHref => "UnsupportedXlinkHref",
            Self::CyclicXlinkHref => "CyclicXlinkHref",
            Self::NonCurveRingMember => "NonCurveRingMember",
            Self::ExpectedSurfaceMember => "ExpectedSurfaceMember",
            Self::SolidWithMultipleExteriors => "SolidWithMultipleExteriors",
            Self::UnexpectedSolidMemberRole => "UnexpectedSolidMemberRole",
            Self::MeshWeldFailed(_) => "MeshWeldFailed",
            Self::ExpectedCurve => "ExpectedCurve",
            Self::InvalidSolidBoundary => "InvalidSolidBoundary",
        }
    }
}

impl fmt::Display for MalformationKind {
    /// The same words as the site's warning.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPosList => {
                f.write_str("citygml geometry: invalid gml:posList content, skipped")
            }
            Self::PosListLengthNotMultipleOfThree => {
                f.write_str("citygml geometry: gml:posList length not a multiple of 3, skipped")
            }
            Self::InvalidPos => f.write_str("citygml geometry: invalid gml:pos content, skipped"),
            Self::PosOrdinateCountNotThree => {
                f.write_str("citygml geometry: gml:pos ordinate count is not 3, skipped")
            }
            Self::UnsupportedXlinkHref => {
                f.write_str("citygml: unsupported xlink:href format, skipped")
            }
            Self::CyclicXlinkHref => f.write_str("citygml geometry: cyclic xlink:href, skipped"),
            Self::NonCurveRingMember => {
                f.write_str("citygml geometry: non-curve ring member, skipped")
            }
            Self::ExpectedSurfaceMember => {
                f.write_str("citygml geometry: expected a surface member, skipped")
            }
            Self::SolidWithMultipleExteriors => {
                f.write_str("citygml geometry: solid with multiple exteriors, extra skipped")
            }
            Self::UnexpectedSolidMemberRole => {
                f.write_str("citygml geometry: unexpected solid member role, skipped")
            }
            Self::MeshWeldFailed(e) => write!(f, "citygml geometry: failed to weld mesh: {e}"),
            Self::ExpectedCurve => f.write_str("citygml geometry: expected a curve, skipped"),
            Self::InvalidSolidBoundary => {
                f.write_str("citygml geometry: cannot use as a solid boundary, skipped")
            }
        }
    }
}
