use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async_with_config, tungstenite::http::Request};
use tracing::error;
use tracing::info;
use url::Url;
// Add these struct definitions at the top
#[derive(Serialize)]
struct Event<T> {
    event: EventData<T>,
    session_command: Option<String>,
}

#[derive(Serialize)]
struct EventData<T> {
    tag: &'static str,
    content: T,
}

#[derive(Serialize)]
struct JoinContent {
    room_id: String,
}

#[derive(Serialize)]
struct CreateContent {
    room_id: String,
}

#[derive(Serialize)]
struct EmitContent {
    data: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse(
        "ws://127.0.0.1:8080/ws?room_id=room123&user_id=test_user&project_id=test_project",
    )?;

    let request = Request::builder()
        .uri(url.as_str())
        .header("Host", url.host_str().unwrap())
        .header("Authorization", "Bearer your_auth_token_here")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_key())
        .body(())?;

    let (ws_stream, _) = connect_async_with_config(request, None, false).await?;
    info!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

    // Replace manual JSON construction with send_event
    send_event(
        &mut write,
        "Create",
        CreateContent {
            room_id: "room123".to_string(),
        },
    )
    .await?;
    info!("Create message sent");

    send_event(
        &mut write,
        "Join",
        JoinContent {
            room_id: "room123".to_string(),
        },
    )
    .await?;
    info!("Join message sent");

    send_event(
        &mut write,
        "Emit",
        EmitContent {
            data: "Hello, WebSocket!".to_string(),
        },
    )
    .await?;
    info!("Test data sent");

    // Receive and print server responses
    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => {
                info!("Received message: {:?}", msg);
                if msg.is_close() {
                    break;
                }
            }
            Err(e) => {
                error!("Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

// Add the send_event helper function
async fn send_event<T: Serialize>(
    writer: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    tag: &'static str,
    content: T,
) -> Result<(), Box<dyn std::error::Error>> {
    let event = Event {
        event: EventData { tag, content },
        session_command: None,
    };
    writer
        .send(Message::Text(serde_json::to_string(&event)?))
        .await?;
    Ok(())
}

fn generate_key() -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use rand::Rng;

    let mut key = [0u8; 16];
    rand::thread_rng().fill(&mut key);
    STANDARD.encode(key)
}
