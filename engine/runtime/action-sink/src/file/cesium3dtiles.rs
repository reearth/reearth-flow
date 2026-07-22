#[cfg(feature = "new-geometry")]
pub mod next;
#[cfg(not(feature = "new-geometry"))]
mod pipeline;
pub(crate) mod sink;
#[cfg(not(feature = "new-geometry"))]
mod slice;
mod tiling;
