use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::{connect_async_with_config, tungstenite::Message};
use tracing::{error, info};
use url::Url;
use yrs::{Doc, Text, Transact};

#[derive(Serialize)]
#[serde(tag = "tag", content = "content")]
enum Event {
    Create { room_id: String },
    Join { room_id: String },
    //Leave,
    Emit { data: String },
}

#[derive(Serialize)]
struct FlowMessage {
    event: Event,
    session_command: Option<SessionCommand>,
}

#[derive(Serialize)]
enum SessionCommand {
    Start { project_id: String, user: User },
    End { project_id: String, user: User },
    // Complete {
    //     project_id: String,
    //     user: User,
    // },
    // CheckStatus {
    //     project_id: String,
    // },
    AddTask { project_id: String },
    RemoveTask { project_id: String },
    // ListAllSnapshotsVersions {
    //     project_id: String,
    // },
    // MergeUpdates {
    //     project_id: String,
    //     data: Vec<u8>,
    //     updated_by: Option<String>,
    // },
}

#[derive(Serialize, Clone)]
struct User {
    id: String,
    email: Option<String>,
    name: Option<String>,
    tenant_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let project_id = "test_project";
    let user_id = "test_user";
    let room_id = "room123";

    let url = Url::parse(&format!(
        "ws://127.0.0.1:8080/ws?room_id={}&user_id={}&project_id={}",
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
        Event::Create {
            room_id: room_id.clone(),
        },
        None,
    )
    .await?;
    info!("Room created");

    send_event(
        &mut write,
        Event::Join {
            room_id: room_id.clone(),
        },
        None,
    )
    .await?;
    info!("Joined room");

    let test_user = User {
        id: user_id.to_string(),
        email: Some("test.user@example.com".to_string()),
        name: Some("Test User".to_string()),
        tenant_id: "test_tenant".to_string(),
    };

    send_command(
        &mut write,
        SessionCommand::AddTask {
            project_id: project_id.to_string(),
        },
    )
    .await?;
    info!("AddTask command sent");

    send_command(
        &mut write,
        SessionCommand::Start {
            project_id: project_id.to_string(),
            user: test_user.clone(),
        },
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

    send_command(
        &mut write,
        SessionCommand::End {
            project_id: project_id.to_string(),
            user: test_user.clone(),
        },
    )
    .await?;
    info!("End command sent");

    send_command(
        &mut write,
        SessionCommand::RemoveTask {
            project_id: project_id.to_string(),
        },
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

async fn send_event(
    writer: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    event: Event,
    session_command: Option<SessionCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    let message = FlowMessage {
        event,
        session_command,
    };

    writer
        .send(Message::Text(serde_json::to_string(&message)?))
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok(())
}

async fn send_command(
    writer: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    command: SessionCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let message = FlowMessage {
        event: Event::Emit {
            data: String::new(),
        },
        session_command: Some(command),
    };

    writer
        .send(Message::Text(serde_json::to_string(&message)?))
        .await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok(())
}

fn generate_key() -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use rand::Rng;

    let mut key = [0u8; 16];
    rand::thread_rng().fill(&mut key);
    STANDARD.encode(key)
}
