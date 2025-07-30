pub mod broadcast;
pub mod kv;
pub mod redis;

pub use broadcast::BroadcastGroupService;

fn first_zero_bit(x: u32) -> u32 {
    (x + 1) & !x
}
