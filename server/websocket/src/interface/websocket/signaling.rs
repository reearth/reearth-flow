use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, warn};

/// WebRTC signaling message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum SignalingMessage {
    Subscribe { topics: Vec<String> },
    Unsubscribe { topics: Vec<String> },
    Publish { topic: String, #[serde(flatten)] data: serde_json::Value },
    Ping,
}

/// Signaling service manages rooms and peer connections
#[derive(Clone)]
pub struct SignalingService {
    rooms: Arc<DashMap<String, broadcast::Sender<String>>>,
}

impl SignalingService {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
        }
    }

    fn get_or_create_room(&self, room: &str) -> broadcast::Sender<String> {
        self.rooms
            .entry(room.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(1024);
                debug!("Created new signaling room: {}", room);
                tx
            })
            .clone()
    }

    fn cleanup_room(&self, room: &str) {
        if let Some((_, sender)) = self.rooms.remove(room) {
            if sender.receiver_count() == 0 {
                debug!("Cleaned up empty signaling room: {}", room);
            }
        }
    }
}

impl Default for SignalingService {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle a single WebSocket connection for WebRTC signaling
pub async fn handle_signaling_connection(socket: WebSocket, service: SignalingService) {
    let (mut sender, mut receiver) = socket.split();
    
    // Track subscribed rooms for this connection
    let mut subscriptions: Vec<(String, broadcast::Receiver<String>)> = Vec::new();

    loop {
        tokio::select! {
            // Handle incoming messages from the client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<SignalingMessage>(&text) {
                            Ok(SignalingMessage::Subscribe { topics }) => {
                                debug!("Client subscribing to topics: {:?}", topics);
                                for topic in topics {
                                    let room_sender = service.get_or_create_room(&topic);
                                    let room_receiver = room_sender.subscribe();
                                    subscriptions.push((topic.clone(), room_receiver));
                                }
                            }
                            Ok(SignalingMessage::Unsubscribe { topics }) => {
                                debug!("Client unsubscribing from topics: {:?}", topics);
                                subscriptions.retain(|(topic, _)| !topics.contains(topic));
                                for topic in &topics {
                                    service.cleanup_room(topic);
                                }
                            }
                            Ok(SignalingMessage::Publish { topic, data }) => {
                                if let Some(room) = service.rooms.get(&topic) {
                                    // Forward the message to all peers in the room
                                    let mut msg = serde_json::json!({
                                        "type": "publish",
                                        "topic": topic,
                                    });
                                    
                                    // Merge the data fields into the message
                                    if let (Some(msg_obj), Some(data_obj)) = (msg.as_object_mut(), data.as_object()) {
                                        for (key, value) in data_obj {
                                            msg_obj.insert(key.clone(), value.clone());
                                        }
                                    }
                                    
                                    if let Ok(msg_str) = serde_json::to_string(&msg) {
                                        let _ = room.send(msg_str);
                                    }
                                }
                            }
                            Ok(SignalingMessage::Ping) => {
                                // Respond to ping with pong
                                let pong = serde_json::json!({"type": "pong"});
                                if let Ok(pong_str) = serde_json::to_string(&pong) {
                                    let msg = Message::Text(pong_str.into());
                                    if sender.send(msg).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse signaling message: {}", e);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        debug!("Client disconnected from signaling server");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            
            // Handle broadcast messages from rooms
            result = async {
                for (_, receiver) in &mut subscriptions {
                    if let Ok(msg) = receiver.try_recv() {
                        return Some(msg);
                    }
                }
                // Small delay to prevent busy loop
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                None::<String>
            } => {
                if let Some(msg) = result {
                    let ws_msg = Message::Text(msg.into());
                    if sender.send(ws_msg).await.is_err() {
                        break;
                    }
                }
            }
        }
    }

    // Cleanup: unsubscribe from all rooms
    for (topic, _) in subscriptions {
        service.cleanup_room(&topic);
    }
    
    debug!("Signaling connection closed");
}

