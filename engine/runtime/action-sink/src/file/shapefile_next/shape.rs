//! What a feature writes to a shapefile, before the concrete shape type of the
//! file holding it is settled.
//!
//! A `.shp` holds records of one shape type, so features converting to different
//! kinds of positions are written to different files. Both geometry worlds convert
//! into the types here, leaving the pipeline one way to group and write features.

use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};

/// The file a written feature belongs in.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(super) enum Bucket {
    /// Features writing no shape at all, carrying only their attributes.
    Null,
    Point,
    PointZ,
    Curve,
    CurveZ,
    Area,
    AreaZ,
}

impl Bucket {
    /// The name distinguishing this bucket's file from a sibling bucket's, used
    /// only where one group of features fills more than one bucket.
    pub(super) fn suffix(&self) -> &'static str {
        match self {
            Bucket::Null => "null",
            Bucket::Point => "point",
            Bucket::PointZ => "pointz",
            Bucket::Curve => "polyline",
            Bucket::CurveZ => "polylinez",
            Bucket::Area => "polygon",
            Bucket::AreaZ => "polygonz",
        }
    }

    /// Whether this bucket's records carry an elevation.
    pub(super) fn elevated(&self) -> bool {
        matches!(self, Bucket::PointZ | Bucket::CurveZ | Bucket::AreaZ)
    }
}

/// One ring of an area, and whether it bounds the face or a hole in it.
pub(super) struct Ring {
    pub(super) outer: bool,
    pub(super) coords: Vec<[f64; 3]>,
}

/// The positions a feature writes, held as `[x, y, z]` whatever the geometry's
/// embedding. [`WrittenShape::elevated`] says whether the third component carries
/// an elevation the geometry stated or `0.0` stood in for it.
pub(super) enum Payload {
    /// One position per point.
    Points(Vec<[f64; 3]>),
    /// One chain of positions per part.
    Curve(Vec<Vec<[f64; 3]>>),
    /// Each face's exterior ring followed by its holes.
    Area(Vec<Ring>),
}

impl Payload {
    /// The bucket this payload belongs in at the given elevation.
    pub(super) fn bucket(&self, elevated: bool) -> Bucket {
        match (self, elevated) {
            (Payload::Points(_), false) => Bucket::Point,
            (Payload::Points(_), true) => Bucket::PointZ,
            (Payload::Curve(_), false) => Bucket::Curve,
            (Payload::Curve(_), true) => Bucket::CurveZ,
            (Payload::Area(_), false) => Bucket::Area,
            (Payload::Area(_), true) => Bucket::AreaZ,
        }
    }

    /// Whether two payloads hold the same kind of positions and can be written as
    /// one shape.
    #[cfg(feature = "new-geometry")]
    pub(super) fn same_kind(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Payload::Points(_), Payload::Points(_))
                | (Payload::Curve(_), Payload::Curve(_))
                | (Payload::Area(_), Payload::Area(_))
        )
    }

    /// Absorb `other`'s positions. The caller must have established that both hold
    /// the same kind via [`same_kind`](Self::same_kind).
    #[cfg(feature = "new-geometry")]
    pub(super) fn absorb(&mut self, other: Self) {
        match (self, other) {
            (Payload::Points(a), Payload::Points(b)) => a.extend(b),
            (Payload::Curve(a), Payload::Curve(b)) => a.extend(b),
            (Payload::Area(a), Payload::Area(b)) => a.extend(b),
            _ => unreachable!("payload kinds were checked before absorbing"),
        }
    }
}

/// What a feature's geometry writes to.
pub(super) struct WrittenShape {
    /// The positions to write, or `None` for a feature writing no shape.
    pub(super) payload: Option<Payload>,
    /// Whether the positions carry an elevation the geometry stated.
    pub(super) elevated: bool,
    /// The coordinate frames the positions came from.
    pub(super) frames: Frames,
}

impl WrittenShape {
    /// A feature writing no shape, carrying only its attributes.
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

/// The coordinate frames the positions written for a shapefile came from.
///
/// Decides whether one CRS covers a file, and so whether a `.prj` can describe it.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) enum Frames {
    /// Nothing carrying coordinates was written.
    #[default]
    Nothing,
    One(CoordinateFrame),
    /// More than one frame, which no single `.prj` describes.
    Mixed,
}

impl Frames {
    pub(super) fn of(frame: &CoordinateFrame) -> Self {
        Self::One(frame.clone())
    }

    /// The frames of two written parts, together.
    pub(super) fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Nothing, frames) | (frames, Self::Nothing) => frames,
            (Self::One(a), Self::One(b)) if a == b => Self::One(a),
            _ => Self::Mixed,
        }
    }

    /// The EPSG code covering every written position, if one does.
    pub(super) fn epsg(&self) -> Option<EpsgCode> {
        match self {
            Self::One(frame) => epsg_code(frame),
            Self::Nothing | Self::Mixed => None,
        }
    }
}

/// The EPSG code a frame names. `Euclidean` names none, and a `Tangent` plane's
/// in-plane coordinates are not its base CRS's.
pub(super) fn epsg_code(frame: &CoordinateFrame) -> Option<EpsgCode> {
    match frame {
        CoordinateFrame::Crs(code) => Some(*code),
        CoordinateFrame::Euclidean | CoordinateFrame::Tangent(_) => None,
    }
}
