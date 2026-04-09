use std::sync::Arc;

use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};
use testcontainers_modules::redis::Redis;
use websocket::infrastructure::gcs::GcsStore;
use websocket::{RedisConfig, RedisStore};

/// A test harness that holds running fake-GCS and Redis containers
/// along with pre-configured store instances.
pub struct TestInfra {
    pub gcs_store: Arc<GcsStore>,
    pub redis_store: Arc<RedisStore>,
    #[allow(dead_code)]
    pub bucket: String,
    #[allow(dead_code)]
    pub gcs_endpoint: String,
    #[allow(dead_code)]
    pub redis_url: String,
    // Hold containers to keep them alive for the test duration.
    // Fields are read in drop order (top to bottom), so stores drop before containers.
    _gcs_container: ContainerAsync<GenericImage>,
    _redis_container: ContainerAsync<Redis>,
}

impl TestInfra {
    /// Spin up fake-gcs-server and Redis containers, create the test bucket,
    /// and return configured store instances.
    pub async fn start() -> Self {
        Self::start_with_bucket("test-bucket").await
    }

    pub async fn start_with_bucket(bucket_name: &str) -> Self {
        // Start both containers in parallel
        let (gcs_container, redis_container) = tokio::join!(start_fake_gcs(), start_redis());

        let gcs_port = gcs_container.get_host_port_ipv4(4443).await.unwrap();
        let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();

        let gcs_endpoint = format!("http://127.0.0.1:{}", gcs_port);
        let redis_url = format!("redis://127.0.0.1:{}", redis_port);

        // Create the test bucket via HTTP
        let http = reqwest::Client::new();
        let resp = http
            .post(format!("{}/storage/v1/b", gcs_endpoint))
            .query(&[("project", "test")])
            .json(&serde_json::json!({ "name": bucket_name }))
            .send()
            .await
            .expect("failed to create bucket");
        assert!(
            resp.status().is_success(),
            "bucket creation failed: {}",
            resp.status()
        );

        let gcs_config = websocket::infrastructure::gcs::GcsConfig {
            bucket_name: bucket_name.to_string(),
            endpoint: Some(gcs_endpoint.clone()),
        };
        let gcs_store = Arc::new(
            GcsStore::new_with_config(gcs_config)
                .await
                .expect("failed to create GcsStore"),
        );

        let redis_config = RedisConfig {
            url: redis_url.clone(),
            ttl: 3600,
            stream_trim_interval: 60,
            stream_max_message_age: 3_600_000,
            stream_max_length: 1000,
        };
        let redis_store = Arc::new(
            RedisStore::new(redis_config)
                .await
                .expect("failed to create RedisStore"),
        );

        Self {
            gcs_store,
            redis_store,
            bucket: bucket_name.to_string(),
            gcs_endpoint,
            redis_url,
            _gcs_container: gcs_container,
            _redis_container: redis_container,
        }
    }
}

async fn start_fake_gcs() -> ContainerAsync<GenericImage> {
    GenericImage::new("fsouza/fake-gcs-server", "latest")
        .with_exposed_port(4443.tcp())
        .with_wait_for(WaitFor::message_on_stderr("server started at"))
        .with_cmd(vec![
            "-scheme".to_string(),
            "http".to_string(),
            "-port".to_string(),
            "4443".to_string(),
        ])
        .start()
        .await
        .expect("failed to start fake-gcs-server")
}

async fn start_redis() -> ContainerAsync<Redis> {
    Redis::default()
        .start()
        .await
        .expect("failed to start Redis")
}
