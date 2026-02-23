pub(crate) mod attribute;
pub(crate) mod echo;
pub(crate) mod feature;
pub(crate) mod file;
pub(crate) mod geometry;
pub(crate) mod http;
pub mod mapping;
pub(crate) mod noop;
pub(crate) mod utils;
pub(crate) mod xml;

#[cfg(test)]
pub(crate) mod tests;

/// Buffer size threshold for accumulating processors before flushing to disk.
/// When the in-memory buffer exceeds this size, data is written to disk files.
pub(crate) const ACCUMULATOR_BUFFER_BYTE_THRESHOLD: usize = 1_073_741_824; // 1 GB
