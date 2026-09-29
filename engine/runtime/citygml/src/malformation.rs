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
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, strum_macros::IntoStaticStr)]
pub enum MalformationKind {
    #[error("citygml geometry: invalid gml:posList content")]
    InvalidPosList,
    #[error("citygml geometry: gml:posList length not a multiple of 3")]
    PosListLengthNotMultipleOfThree,
    #[error("citygml geometry: invalid gml:pos content")]
    InvalidPos,
    #[error("citygml geometry: gml:pos ordinate count is not 3")]
    PosOrdinateCountNotThree,
    #[error("citygml: unsupported xlink:href format")]
    UnsupportedXlinkHref,
    #[error("citygml geometry: cyclic xlink:href")]
    CyclicXlinkHref,
    #[error("citygml geometry: non-curve ring member")]
    NonCurveRingMember,
    #[error("citygml geometry: expected a surface member")]
    ExpectedSurfaceMember,
    #[error("citygml geometry: solid with multiple exteriors")]
    SolidWithMultipleExteriors,
    #[error("citygml geometry: unexpected solid member role")]
    UnexpectedSolidMemberRole,
    /// Carries the weld error's message.
    #[error("citygml geometry: failed to weld mesh: {0}")]
    MeshWeldFailed(String),
    #[error("citygml geometry: expected a curve")]
    ExpectedCurve,
    #[error("citygml geometry: cannot use as a solid boundary")]
    InvalidSolidBoundary,
}

impl MalformationKind {
    /// The variant name, for output as an attribute value.
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}
