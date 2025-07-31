pub mod group;
pub mod group_ddd;
pub mod pool;
pub mod sub;
pub mod types;

// Re-export both the original and DDD versions
pub use group::BroadcastGroup;
pub use group_ddd::{BroadcastGroupDDD, BroadcastGroupDDDFactory};

// Re-export types for convenience
pub use types::BroadcastConfig as LegacyBroadcastConfig;
