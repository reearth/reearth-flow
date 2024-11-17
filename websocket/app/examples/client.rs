use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse(
        "ws://127.0.0.1:8080/ws/room123?token=nyaan&user_id=test_user&project_id=test_project",
    )?;

    // Connect to WebSocket server
    let (ws_stream, _) = connect_async(url).await?;
    println!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

    // Send Join event
    let join_message = json!({
        "event": {
            "tag": "Join",
            "content": {
                "room_id": "room123"
            }
        },
        "session_command": null
    });
    write.send(Message::Text(join_message.to_string())).await?;
    println!("Join message sent");

    // Send some test data
    let test_data = json!({
        "event": {
            "tag": "Emit",
            "content": {
                "data": "Hello, WebSocket!"
            }
        },
        "session_command": null
    });
    write.send(Message::Text(test_data.to_string())).await?;
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
