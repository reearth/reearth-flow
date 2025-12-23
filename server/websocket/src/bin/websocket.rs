use tracing::error;
use websocket::config::Config;
use websocket::infrastructure::tracing::{init_tracing, init_tracing_simple, shutdown_tracing};
use websocket::presentation::app;

#[tokio::main]
async fn main() {
    // Try to load config using the layered configuration system
    let config = match Config::load() {
        Ok(config) => config,
        Err(err) => {
            // Fall back to simple tracing if config fails
            init_tracing_simple();
            error!("Failed to load configuration: {}", err);
            std::process::exit(1);
        }
    };

    // Convert to infrastructure tracing config
    let tracing_config: websocket::infrastructure::tracing::TracingConfig =
        config.tracing.clone().into();

    // Initialize tracing with config
    if let Err(err) = init_tracing(&tracing_config).await {
        // Fall back to simple tracing if OpenTelemetry fails
        init_tracing_simple();
        error!("Failed to initialize tracing: {}", err);
        // Continue with simple tracing instead of exiting
    }

    if let Err(err) = app::run_with_config(config).await {
        error!("Application error: {}", err);
        shutdown_tracing();
        std::process::exit(1);
    }

    shutdown_tracing();
}
