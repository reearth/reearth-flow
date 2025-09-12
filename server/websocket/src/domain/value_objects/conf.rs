pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";
pub const DEFAULT_REDIS_TTL: u64 = 43200;
pub const DEFAULT_GCS_BUCKET: &str = "yrs-dev";
#[cfg(feature = "auth")]
pub const DEFAULT_AUTH_URL: &str = "http://localhost:8080";
pub const DEFAULT_APP_ENV: &str = "development";
pub const DEFAULT_ORIGINS: &[&str] = &[
    "http://localhost:3000",
    "https://api.flow.test.reearth.dev",
    "https://api.flow.reearth.dev",
    "http://localhost:8000",
    "http://localhost:8080",
];
pub const DEFAULT_WS_PORT: &str = "8000";
