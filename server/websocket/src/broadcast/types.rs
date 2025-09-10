#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    pub storage_enabled: bool,
    pub room_name: Option<String>,
    pub doc_name: Option<String>,
}
