//! What a feature writes to a shapefile, before the file's shape type is settled.

use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};

/// The file a written feature belongs in.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum Bucket {
    /// Features writing no shape.
    Null,
    Point,
    PointZ,
    Curve,
    CurveZ,
    Area,
    AreaZ,
    Multipatch,
}

impl Bucket {
    /// The suffix naming this bucket's file beside a sibling bucket's.
    pub(super) fn suffix(&self) -> &'static str {
        match self {
            Bucket::Null => "null",
            Bucket::Point => "point",
            Bucket::PointZ => "pointz",
            Bucket::Curve => "polyline",
            Bucket::CurveZ => "polylinez",
            Bucket::Area => "polygon",
            Bucket::AreaZ => "polygonz",
            Bucket::Multipatch => "multipatch",
        }
    }

    /// Whether this bucket's records carry an elevation.
    pub(super) fn elevated(&self) -> bool {
        matches!(
            self,
            Bucket::PointZ | Bucket::CurveZ | Bucket::AreaZ | Bucket::Multipatch
        )
    }
}

/// One ring of an area.
pub(super) struct Ring {
    pub(super) outer: bool,
    pub(super) coords: Vec<[f64; 3]>,
}

/// One patch of a surface.
pub(super) enum Patch {
    /// A ring of a face.
    Ring(Ring),
    /// One triangle.
    Triangle([[f64; 3]; 3]),
}

/// The positions a feature writes, as `[x, y, z]`; see [`WrittenShape::elevated`]
/// for whether `z` is an elevation.
pub(super) enum Payload {
    /// One position per point.
    Points(Vec<[f64; 3]>),
    /// One chain of positions per part.
    Curve(Vec<Vec<[f64; 3]>>),
    /// Each face's exterior ring followed by its holes.
    Area(Vec<Ring>),
    /// The patches of a surface in space, always elevated.
    Surface(Vec<Patch>),
}

impl Payload {
    /// The bucket this payload belongs in.
    pub(super) fn bucket(&self, elevated: bool) -> Bucket {
        match (self, elevated) {
            (Payload::Points(_), false) => Bucket::Point,
            (Payload::Points(_), true) => Bucket::PointZ,
            (Payload::Curve(_), false) => Bucket::Curve,
            (Payload::Curve(_), true) => Bucket::CurveZ,
            (Payload::Area(_), false) => Bucket::Area,
            (Payload::Area(_), true) => Bucket::AreaZ,
            (Payload::Surface(_), _) => Bucket::Multipatch,
        }
    }

    /// Whether two payloads can be written as one shape; a surface takes areas in.
    #[cfg(feature = "new-geometry")]
    pub(super) fn same_kind(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Payload::Points(_), Payload::Points(_))
                | (Payload::Curve(_), Payload::Curve(_))
                | (
                    Payload::Area(_) | Payload::Surface(_),
                    Payload::Area(_) | Payload::Surface(_)
                )
        )
    }

    /// Absorb `other`'s positions; both must be of the [`same_kind`](Self::same_kind).
    #[cfg(feature = "new-geometry")]
    pub(super) fn absorb(&mut self, other: Self) {
        match (self, other) {
            (Payload::Points(a), Payload::Points(b)) => a.extend(b),
            (Payload::Curve(a), Payload::Curve(b)) => a.extend(b),
            (Payload::Area(a), Payload::Area(b)) => a.extend(b),
            (Payload::Surface(a), Payload::Surface(b)) => a.extend(b),
            (Payload::Surface(a), Payload::Area(b)) => a.extend(b.into_iter().map(Patch::Ring)),
            (this @ Payload::Area(_), Payload::Surface(b)) => {
                let Payload::Area(a) = std::mem::replace(this, Payload::Surface(b)) else {
                    unreachable!("this is an area");
                };
                let Payload::Surface(patches) = this else {
                    unreachable!("this is now a surface");
                };
                patches.splice(0..0, a.into_iter().map(Patch::Ring));
            }
            _ => unreachable!("payload kinds were checked before absorbing"),
        }
    }
}

/// What a feature's geometry writes to.
pub(super) struct WrittenShape {
    /// The positions to write, or `None` for no shape.
    pub(super) payload: Option<Payload>,
    /// Whether the positions carry an elevation the geometry stated.
    pub(super) elevated: bool,
    /// The frames the positions came from.
    pub(super) frames: Frames,
}

impl WrittenShape {
    /// A feature writing no shape.
    pub(super) fn none() -> Self {
        Self {
            payload: None,
            elevated: false,
            frames: Frames::Nothing,
        }
    }

    /// The file this belongs in.
    pub(super) fn bucket(&self) -> Bucket {
        match &self.payload {
            Some(payload) => payload.bucket(self.elevated),
            None => Bucket::Null,
        }
    }
}

/// The frames a file's positions came from.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) enum Frames {
    /// Nothing carrying coordinates was written.
    #[default]
    Nothing,
    One(CoordinateFrame),
    /// More than one frame.
    Mixed,
}

impl Frames {
    /// One frame.
    pub(super) fn of(frame: &CoordinateFrame) -> Self {
        Self::One(frame.clone())
    }

    /// The frames of two parts together.
    pub(super) fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Nothing, frames) | (frames, Self::Nothing) => frames,
            (Self::One(a), Self::One(b)) if a == b => Self::One(a),
            _ => Self::Mixed,
        }
    }

    /// The EPSG code covering every position, if one does.
    pub(super) fn epsg(&self) -> Option<EpsgCode> {
        match self {
            Self::One(frame) => epsg_code(frame),
            Self::Nothing | Self::Mixed => None,
        }
    }
}

/// The EPSG code a frame names; `Euclidean` and `Tangent` name none.
pub(super) fn epsg_code(frame: &CoordinateFrame) -> Option<EpsgCode> {
    match frame {
        CoordinateFrame::Crs(code) => Some(*code),
        CoordinateFrame::Euclidean | CoordinateFrame::Tangent(_) => None,
    }
}
