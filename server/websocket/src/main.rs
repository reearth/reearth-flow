use anyhow::Result;
use tracing::error;
use websocket::{start_server, ConfigService, WebSocketService};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .init();

    // 加载配置
    let config = match ConfigService::load() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    // 初始化应用状态
    let state = match WebSocketService::initialize_app_state(&config).await {
        Ok(state) => state,
        Err(e) => {
            error!("Failed to initialize application state: {}", e);
            std::process::exit(1);
        }
    };

    // 启动服务器
    if let Err(e) = start_server(state, &config.ws_port, &config).await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
