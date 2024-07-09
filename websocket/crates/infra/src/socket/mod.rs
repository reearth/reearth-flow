pub trait Socket: Send + Sync {
    async fn on_disconnect(&self);
    async fn join(&self, room_id: &str);
    async fn leave(&self, room_id: &str);
    async fn emit(&self, event: &str, data: &str);
    async fn timeout<T>(&self, duration: Duration) -> Result<T, String>;
}
