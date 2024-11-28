use app::MessageType;
use flow_websocket_infra::types::user::User;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::{connect_async_with_config, tungstenite::Message};
use tracing::{error, info};
use url::Url;
use yrs::updates::decoder::Decode;
use yrs::{updates::encoder::Encode, Doc, GetString, ReadTxn, Text, Transact};

#[derive(Serialize, Debug)]
#[serde(tag = "tag", content = "content")]
enum Event {
    Create { room_id: String },
    Join { room_id: String },
    Emit { data: String },
}

#[derive(Serialize)]
struct FlowMessage {
    event: Event,
    session_command: Option<SessionCommand>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "tag", content = "content")]
pub enum SessionCommand {
    Start {
        project_id: String,
        user: User,
    },
    End {
        project_id: String,
        user: User,
    },
    Complete {
        project_id: String,
        user: User,
    },
    CheckStatus {
        project_id: String,
    },
    AddTask {
        project_id: String,
    },
    RemoveTask {
        project_id: String,
    },
    ListAllSnapshotsVersions {
        project_id: String,
    },
    MergeUpdates {
        project_id: String,
        data: Vec<u8>,
        updated_by: Option<String>,
    },
    ProcessStateVector {
        project_id: String,
        state_vector: Vec<u8>,
    },
}

fn create_binary_message(msg_type: MessageType, data: Vec<u8>) -> Vec<u8> {
    let mut message = Vec::with_capacity(data.len() + 1);
    message.push(msg_type._as_byte());
    message.extend_from_slice(&data);
    message
}

async fn create_client(
    user_id: &str,
    room_id: &str,
) -> Result<
    (
        futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
        futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    ),
    Box<dyn std::error::Error>,
> {
    let auth_token = "nyaan";
    let url = Url::parse(&format!(
        "ws://127.0.0.1:8081/{room_id}?user_id={user_id}&token={token}",
        room_id = room_id,
        user_id = user_id,
        token = auth_token
    ))?;

    let request = Request::builder()
        .uri(url.as_str())
        .header("Host", url.host_str().unwrap())
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_key())
        .body(())?;

    let (ws_stream, _) = connect_async_with_config(request, None, false).await?;
    info!("[{}] WebSocket connection established", user_id);

    Ok(ws_stream.split())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let room_id = "sync_room";

    let (mut write1, _read1) = create_client("user1", room_id).await?;
    let (mut write2, mut read2) = create_client("user2", room_id).await?;

    let doc1 = Doc::new();
    let text1 = doc1.get_or_insert_text("test");

    let doc2 = Doc::new();
    let text2 = doc2.get_or_insert_text("test");

    send_event(
        &mut write1,
        Event::Create {
            room_id: room_id.to_string(),
        },
        None,
    )
    .await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    send_event(
        &mut write2,
        Event::Join {
            room_id: room_id.to_string(),
        },
        None,
    )
    .await?;

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let project_id = "test_project3".to_string();
    let user1 = User::new("user1".to_string(), None, None);
    let user2 = User::new("user2".to_string(), None, None);

    send_command(
        &mut write1,
        SessionCommand::AddTask {
            project_id: project_id.clone(),
        },
    )
    .await?;
    info!("[Client 1] AddTask command sent");

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    send_command(
        &mut write1,
        SessionCommand::Start {
            project_id: project_id.clone(),
            user: user1.clone(),
        },
    )
    .await?;
    info!("[Client 1] Start command sent");

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    send_command(
        &mut write2,
        SessionCommand::AddTask {
            project_id: project_id.clone(),
        },
    )
    .await?;
    info!("[Client 2] AddTask command sent");

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    send_command(
        &mut write2,
        SessionCommand::Start {
            project_id: project_id.clone(),
            user: user2.clone(),
        },
    )
    .await?;
    info!("[Client 2] Start command sent");

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    {
        let mut txn = doc1.transact_mut();
        text1.push(&mut txn, "Hello from client 1!");
        let update = txn.encode_update_v2();
        let msg = create_binary_message(MessageType::Update, update);
        write1.send(Message::Binary(msg)).await?;
        info!("[Client 1] Sent update");
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    {
        let state_vector = {
            let txn = doc2.transact();
            let sv = txn.state_vector();
            let encoded = sv.encode_v2();
            create_binary_message(MessageType::Sync, encoded)
        };
        write2.send(Message::Binary(state_vector)).await?;
        info!("[Client 2] Sent state vector for sync");

        let mut sync_complete = false;
        let timeout_duration = tokio::time::Duration::from_secs(5);
        let start_time = tokio::time::Instant::now();

        while !sync_complete && start_time.elapsed() < timeout_duration {
            if let Ok(Some(msg)) =
                tokio::time::timeout(tokio::time::Duration::from_millis(100), read2.next()).await
            {
                match msg {
                    Ok(Message::Binary(data)) => {
                        info!("[Client 2] Received binary data of length: {}", data.len());

                        if let Ok(update) = yrs::Update::decode_v2(&data) {
                            let mut txn = doc2.transact_mut();
                            txn.apply_update(update)?;

                            let content = text2.get_string(&txn);
                            info!("[Client 2] Applied update, new content: {}", content);

                            let new_sv = txn.state_vector();
                            let sv_message =
                                create_binary_message(MessageType::Sync, new_sv.encode_v2());
                            write2.send(Message::Binary(sv_message)).await?;
                            info!("[Client 2] Sent confirmation state vector: {:?}", new_sv);

                            sync_complete = true;
                        } else if let Ok(sv) = yrs::StateVector::decode_v2(&data) {
                            info!("[Client 2] Received state vector: {:?}", sv);
                            let local_sv = doc2.transact().state_vector();

                            if sv != local_sv {
                                info!(
                                    "[Client 2] State vector mismatch - local: {:?}, remote: {:?}",
                                    local_sv, sv
                                );
                                let diff_request = {
                                    let txn = doc2.transact_mut();
                                    let diff = txn.encode_state_as_update_v2(&sv);
                                    create_binary_message(MessageType::Update, diff)
                                };
                                write2.send(Message::Binary(diff_request)).await?;
                                info!("[Client 2] Requested diff update");
                            } else {
                                sync_complete = true;
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        write2.send(Message::Pong(data)).await?;
                        info!("[Client 2] Responded to ping");
                    }
                    Ok(Message::Close(frame)) => {
                        info!("[Client 2] Received close frame: {:?}", frame);
                        break;
                    }
                    _ => {}
                }
            }
        }

        if !sync_complete {
            error!("[Client 2] Sync timeout!");
        }
    }

    {
        let content1 = text1.get_string(&doc1.transact());
        let content2 = text2.get_string(&doc2.transact());

        info!("[Client 1] Content: {}", content1);
        info!("[Client 2] Content: {}", content2);

        if content1 == content2 {
            info!("✅ Documents are in sync!");
        } else {
            error!("❌ Documents are not in sync!");
        }

        let sv1 = doc1.transact().state_vector();
        let sv2 = doc2.transact().state_vector();

        info!("[Client 1] State vector: {:?}", sv1);
        info!("[Client 2] State vector: {:?}", sv2);

        if sv1 == sv2 {
            info!("✅ State vectors match!");
        } else {
            error!("❌ State vectors don't match!");
        }
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    send_command(
        &mut write1,
        SessionCommand::End {
            project_id: project_id.clone(),
            user: user1,
        },
    )
    .await?;
    info!("[Client 1] End command sent");

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    send_command(
        &mut write2,
        SessionCommand::End {
            project_id: project_id.clone(),
            user: user2,
        },
    )
    .await?;
    info!("[Client 2] End command sent");

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

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

    info!("Event sent: {:?}", message.event);
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
        session_command: Some(command.clone()),
    };

    writer
        .send(Message::Text(serde_json::to_string(&message)?))
        .await?;

    info!("Command sent: {:?}", command);
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
