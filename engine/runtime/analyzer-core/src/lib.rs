//! Re:Earth Flow DataSize trait for memory estimation.
//!
//! This crate provides the `DataSize` trait for estimating heap memory usage
//! of Rust types. It's designed for use with the Re:Earth Flow runtime analyzer.
//!
//! # Usage
//!
//! ```rust
//! use reearth_flow_analyzer_core::DataSize;
//!
//! #[derive(DataSize)]
//! struct MyProcessor {
//!     buffer: Vec<String>,
//!     count: usize,
//! }
//! ```

pub mod data_size;

pub use data_size::DataSize;
pub use reearth_flow_analyzer_core_derive::DataSize;
