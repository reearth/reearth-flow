mod point;
pub use point::SweepPoint;

mod events;
pub(crate) use events::{Event, EventType};

mod line_or_point;
pub use line_or_point::LineOrPoint;

mod cross;
pub use cross::Cross;

mod segment;
use segment::Segment;

mod active;
pub(super) use active::{Active, ActiveSet};

mod im_segment;
use im_segment::IMSegment;

mod vec_set;
pub(crate) use vec_set::VecSet;

mod proc;

mod iter;
pub(crate) use iter::Crossing;
pub use iter::Intersections;
