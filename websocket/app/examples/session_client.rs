use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::{connect_async_with_config, tungstenite::Message};
use tracing::{error, info};
use url::Url;
use yrs::{Doc, Text, Transact};

#[derive(Serialize)]
struct Event<T> {
    event: EventData<T>,
    session_command: Option<SessionCommand>,
}

#[derive(Serialize)]
struct EventData<T> {
    tag: &'static str,
    content: T,
}

#[derive(Serialize)]
struct SessionCommand {
    command_type: &'static str,
    project_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<User>,
}

#[derive(Serialize, Clone)]
struct User {
    id: String,
    email: Option<String>,
    name: Option<String>,
}

#[derive(Serialize)]
struct JoinContent {
    room_id: String,
}

#[derive(Serialize)]
struct CreateContent {
    room_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let project_id = "test_project";
    let user_id = "test_user";
    let room_id = "room123";

    let url = Url::parse(&format!(
        "ws://127.0.0.1:8080/ws/{}/ws?user_id={}&project_id={}",
        room_id, user_id, project_id
    ))?;

    let auth_token = "your_auth_token_here";

    let request = Request::builder()
        .uri(url.as_str())
        .header("Host", url.host_str().unwrap())
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_key())
        .header("Authorization", format!("Bearer {}", auth_token))
        .body(())?;

    let (ws_stream, _) = connect_async_with_config(request, None, false).await?;
    info!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();
    let room_id = "room123".to_string();

    send_event(
        &mut write,
        "Create",
        CreateContent {
            room_id: room_id.clone(),
        },
        None,
    )
    .await?;
    info!("Room created");

    send_event(
        &mut write,
        "Join",
        JoinContent {
            room_id: room_id.clone(),
        },
        None,
    )
    .await?;
    info!("Joined room");

    send_event(
        &mut write,
        "SessionCommand",
        (),
        Some(SessionCommand {
            command_type: "AddTask",
            project_id: project_id.to_string(),
            user: None,
        }),
    )
    .await?;
    info!("AddTask command sent");

    let test_user = User {
        id: user_id.to_string(),
        email: Some("test.user@example.com".to_string()),
        name: Some("Test User".to_string()),
    };

    send_event(
        &mut write,
        "SessionCommand",
        (),
        Some(SessionCommand {
            command_type: "Start",
            project_id: project_id.to_string(),
            user: Some(test_user.clone()),
        }),
    )
    .await?;
    info!("Start command sent");

    let doc = Doc::new();
    let text = doc.get_or_insert_text("test");

    let update1 = {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, "Hello, YJS!");
        txn.encode_update_v2()
    };

    write.send(Message::Binary(update1)).await?;
    info!("First YJS update sent");

    let update2 = {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, " More text!");
        txn.encode_update_v2()
    };

    write.send(Message::Binary(update2)).await?;
    info!("Second YJS update sent");

    send_event(
        &mut write,
        "SessionCommand",
        (),
        Some(SessionCommand {
            command_type: "End",
            project_id: project_id.to_string(),
            user: Some(test_user.clone()),
        }),
    )
    .await?;
    info!("End command sent");

    send_event(
        &mut write,
        "SessionCommand",
        (),
        Some(SessionCommand {
            command_type: "RemoveTask",
            project_id: project_id.to_string(),
            user: None,
        }),
    )
    .await?;
    info!("RemoveTask command sent");

    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => {
                info!("Received: {:?}", msg);
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

async fn send_event<T: Serialize>(
    writer: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    tag: &'static str,
    content: T,
    session_command: Option<SessionCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    let event = Event {
        event: EventData { tag, content },
        session_command,
    };

    writer
        .send(Message::Text(serde_json::to_string(&event)?))
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok(())
}

// async fn wait_for_response(
//     read: &mut futures_util::stream::SplitStream<
//         tokio_tungstenite::WebSocketStream<
//             tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
//         >,
//     >,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     if let Some(msg) = read.next().await {
//         match msg {
//             Ok(msg) => {
//                 info!("Received response: {:?}", msg);
//             }
//             Err(e) => {
//                 error!("Error receiving response: {}", e);
//             }
//         }
//     }
//     Ok(())
// }

fn generate_key() -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use rand::Rng;

    let mut key = [0u8; 16];
    rand::thread_rng().fill(&mut key);
    STANDARD.encode(key)
}
