use tracing::error;
use websocket::interface::app;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .init();

    if let Err(err) = app::run().await {
        error!("Application error: {}", err);
        std::process::exit(1);
    }
}
