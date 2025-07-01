pub mod gcs;
pub mod kv;
pub mod redis;

fn first_zero_bit(x: u32) -> u32 {
    (x + 1) & !x
}
