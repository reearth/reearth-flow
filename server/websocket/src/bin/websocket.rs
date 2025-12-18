use tracing::error;
use websocket::conf::Config;
use websocket::infrastructure::tracing::{init_tracing, init_tracing_simple, shutdown_tracing};
use websocket::presentation::app;

#[tokio::main]
async fn main() {
    // Try to load config first (without tracing, since it's not initialized yet)
    let config = match Config::load() {
        Ok(config) => config,
        Err(err) => {
            // Fall back to simple tracing if config fails
            init_tracing_simple();
            error!("Failed to load configuration: {}", err);
            std::process::exit(1);
        }
    };

    // Initialize tracing with config
    if let Err(err) = init_tracing(&config.tracing).await {
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
