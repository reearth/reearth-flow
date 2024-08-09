use uuid::Uuid;

pub fn generate_id(length: usize, prefix: &str) -> String {
    let _ = length;
    format!("{}{}", prefix, Uuid::new_v4())
}
