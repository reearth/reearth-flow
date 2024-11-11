#[macro_export]
macro_rules! generate_id {
    ($prefix:expr) => {
        format!("{}{}", $prefix, uuid::Uuid::new_v4())
    };
}
