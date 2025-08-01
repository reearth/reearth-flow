pub mod kv;
pub fn first_zero_bit(x: u32) -> u32 {
    (x + 1) & !x
}
