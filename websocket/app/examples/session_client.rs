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
    //Leave,
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
    #[warn(dead_code)]
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let user_id = "test_user";
    let room_id = "room123";
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
    info!("WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();
    let room_id = "room123".to_string();

    let doc = Doc::new();
    let text = doc.get_or_insert_text("test");

    send_event(
        &mut write,
        Event::Create {
            room_id: room_id.clone(),
        },
        None,
    )
    .await?;
    info!("Room created");

    match read.next().await {
        Some(Ok(msg)) => {
            info!("Room creation response: {:?}", msg);
        }
        Some(Err(e)) => {
            error!("Error receiving room creation confirmation: {}", e);
            return Err(e.into());
        }
        None => {
            error!("Connection closed before room creation confirmation");
            return Err("Premature connection close".into());
        }
    }

    send_event(
        &mut write,
        Event::Join {
            room_id: room_id.clone(),
        },
        None,
    )
    .await?;

    let project_id = "test_project3".to_string();
    let user = User::new(user_id.to_string(), None, None);

    send_command(
        &mut write,
        SessionCommand::AddTask {
            project_id: project_id.clone(),
        },
    )
    .await?;
    info!("AddTask command sent");

    send_command(
        &mut write,
        SessionCommand::Start {
            project_id: project_id.clone(),
            user: user.clone(),
        },
    )
    .await?;
    info!("Start command sent");

    let state_vector = {
        let txn = doc.transact();
        let state_vector = txn.state_vector();
        let encode = state_vector.encode_v2();
        create_binary_message(MessageType::Sync, encode)
    };

    write.send(Message::Binary(state_vector)).await?;
    info!("State vector sent");

    let update1 = {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, "Hello, YJS!");
        let update = txn.encode_update_v2();
        create_binary_message(MessageType::Update, update)
    };

    write.send(Message::Binary(update1.clone())).await?;
    info!("First YJS update sent: {} bytes", update1.len());

    let update2 = {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, " More text!");
        let update = txn.encode_update_v2();
        create_binary_message(MessageType::Update, update)
    };

    write.send(Message::Binary(update2.clone())).await?;
    info!("Second YJS update sent: {} bytes", update2.len());

    let update_data = {
        let mut txn = doc.transact_mut();
        text.push(&mut txn, "Hello from merge update!");
        let update = txn.encode_update_v2();
        create_binary_message(MessageType::Update, update)
    };

    write.send(Message::Binary(update_data.clone())).await?;
    info!(
        "MergeUpdates command sent with YJS update: {} bytes",
        update_data.len()
    );

    // Verify the document state after updates
    {
        let txn = doc.transact();
        let content = text.get_string(&txn);
        info!("Current document content: {}", content);

        let diff = text.diff(&txn, |_| None::<()>);
        info!("Document changes: {:?}", diff);

        let state_vector = txn.state_vector();
        info!("Current state vector: {:?}", state_vector);
    }

    send_command(
        &mut write,
        SessionCommand::Complete {
            project_id: project_id.clone(),
            user: user.clone(),
        },
    )
    .await?;
    info!("Complete command sent");

    send_command(
        &mut write,
        SessionCommand::CheckStatus {
            project_id: project_id.clone(),
        },
    )
    .await?;
    info!("CheckStatus command sent");

    send_command(
        &mut write,
        SessionCommand::ListAllSnapshotsVersions {
            project_id: project_id.clone(),
        },
    )
    .await?;
    info!("ListAllSnapshotsVersions command sent");

    send_command(
        &mut write,
        SessionCommand::End {
            project_id: project_id.clone(),
            user: user.clone(),
        },
    )
    .await?;
    info!("End command sent");

    send_command(
        &mut write,
        SessionCommand::RemoveTask {
            project_id: project_id.clone(),
        },
    )
    .await?;
    info!("RemoveTask command sent");

    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => match msg {
                Message::Binary(data) => {
                    info!("Received binary data of length: {}", data.len());

                    match yrs::StateVector::decode_v2(&data) {
                        Ok(sv) => {
                            info!("Successfully decoded as state vector: {:?}", sv);
                            let local_sv = doc.transact().state_vector();
                            info!("Current local state vector before sync: {:?}", local_sv);

                            if sv != local_sv {
                                info!(
                                    "State vector mismatch - local: {:?}, remote: {:?}",
                                    local_sv, sv
                                );

                                let update = {
                                    let txn = doc.transact();
                                    txn.encode_state_as_update_v2(&sv)
                                };
                                let msg = create_binary_message(MessageType::Update, update);
                                write.send(Message::Binary(msg)).await?;
                                info!("Sent state difference update");

                                let new_sv = doc.transact().state_vector();
                                info!("Local state vector after sending update: {:?}", new_sv);

                                let content = text.get_string(&doc.transact());
                                info!("Current document content: {}", content);
                            }
                        }
                        Err(_) => match yrs::Update::decode_v2(&data) {
                            Ok(update) => {
                                let mut txn = doc.transact_mut();
                                match txn.apply_update(update) {
                                    Ok(()) => {
                                        let content = text.get_string(&txn);
                                        info!("Successfully applied update. Content: {}", content);

                                        let diff = text.diff(&txn, |_| None::<()>);
                                        info!("Update resulted in changes: {:?}", diff);

                                        let new_sv = txn.state_vector();
                                        info!("New state vector after update: {:?}", new_sv);
                                    }
                                    Err(e) => {
                                        error!("Failed to apply update: {:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to decode as update: {:?}", e);
                                info!("Raw binary data: {:?}", data);
                            }
                        },
                    }
                }
                Message::Ping(data) => {
                    info!("Received ping message: {:?}", data);
                }
                Message::Text(text) => {
                    info!("Received text message: {}", text);
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        info!("Parsed JSON: {}", serde_json::to_string_pretty(&json)?);
                    }
                }
                Message::Close(frame) => {
                    info!("Received close frame: {:?}", frame);
                    break;
                }
                _ => {
                    info!("Received other message type: {:?}", msg);
                }
            },
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
