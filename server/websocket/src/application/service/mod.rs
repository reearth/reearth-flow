pub mod broadcast_group_service;
pub mod kv;
pub mod redis;

pub use broadcast_group_service::BroadcastGroupService;

fn first_zero_bit(x: u32) -> u32 {
    (x + 1) & !x
}
