use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio_tungstenite::{connect_async, tungstenite::Message};
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
struct EmitContent {
    data: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse(
        "ws://127.0.0.1:8080/ws/room123?token=nyaan&user_id=test_user&project_id=test_project",
    )?;

    // Connect to WebSocket server
    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

    // Replace manual JSON construction with send_event
    send_event(
        &mut write,
        "Join",
        JoinContent {
            room_id: "room123".to_string(),
        },
    )
    .await?;
    println!("Join message sent");

    send_event(
        &mut write,
        "Emit",
        EmitContent {
            data: "Hello, WebSocket!".to_string(),
        },
    )
    .await?;
    println!("Test data sent");

    // Receive and print server responses
    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => {
                println!("Received message: {:?}", msg);
                if msg.is_close() {
                    break;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
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
