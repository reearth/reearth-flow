use bytes::Bytes;
use tracing::{error, warn};
use yrs::encoding::write::Write;
use yrs::sync::awareness::{Awareness, AwarenessUpdate};
use yrs::sync::Message;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::{Encode, Encoder, EncoderV1};
use yrs::{Doc, Update};

pub const MESSAGE_SYNC: u8 = 0;
pub const MESSAGE_AWARENESS: u8 = 1;
pub const MESSAGE_AUTH: u8 = 2;
pub const MESSAGE_QUERY_AWARENESS: u8 = 3;

pub const MESSAGE_SYNC_STEP1: u8 = 0;
pub const MESSAGE_SYNC_STEP2: u8 = 1;
pub const MESSAGE_SYNC_UPDATE: u8 = 2;

/// Merge messages for easier consumption by the client like JavaScript version
///
/// This is useful when the server catches messages from a pubsub/stream.
/// Before the server sends the messages to the clients, we can merge updates,
/// and filter out older awareness messages.
pub fn merge_messages(messages: Vec<Bytes>) -> Vec<Bytes> {
    if messages.len() < 2 {
        return messages;
    }

    let temp_doc = Doc::new();
    let awareness = Awareness::new(temp_doc);
    let mut updates = Vec::new();

    for message in messages {
        if message.is_empty() {
            continue;
        }

        match parse_message(&message) {
            Ok(ParsedMessage::Sync(update_bytes)) => {
                updates.push(update_bytes);
            }
            Ok(ParsedMessage::Awareness(awareness_update)) => {
                if let Err(e) = awareness.apply_update(awareness_update) {
                    warn!("Failed to apply awareness update during merge: {}", e);
                }
            }
            Err(e) => {
                error!("Error parsing message during merge: {}", e);
            }
        }
    }

    let mut result = Vec::new();

    // Merge sync updates
    if !updates.is_empty() {
        match yrs::merge_updates_v1(&updates) {
            Ok(merged_update) => {
                let mut encoder = EncoderV1::new();
                encoder.write_var(MESSAGE_SYNC as u32);
                encoder.write_var(MESSAGE_SYNC_UPDATE as u32);
                encoder.write_buf(&merged_update);
                result.push(Bytes::from(encoder.to_vec()));
            }
            Err(e) => {
                warn!("Failed to merge updates: {}", e);
                // If merging fails, send the last update
                if let Some(last_update) = updates.last() {
                    let mut encoder = EncoderV1::new();
                    encoder.write_var(MESSAGE_SYNC as u32);
                    encoder.write_var(MESSAGE_SYNC_UPDATE as u32);
                    encoder.write_buf(last_update);
                    result.push(Bytes::from(encoder.to_vec()));
                }
            }
        }
    }

    // Add awareness updates if there are any
    let all_clients: Vec<_> = (0..1000).collect(); // This is a simple approach - in practice should track actual clients
    match awareness.update_with_clients(all_clients) {
        Ok(awareness_update) => {
            if !awareness_update.clients.is_empty() {
                let msg = Message::Awareness(awareness_update);
                result.push(Bytes::from(msg.encode_v1()));
            }
        }
        Err(e) => {
            warn!("Failed to create awareness update during merge: {}", e);
        }
    }

    result
}

#[derive(Debug)]
enum ParsedMessage {
    Sync(Vec<u8>),
    Awareness(AwarenessUpdate),
}

fn parse_message(message: &[u8]) -> Result<ParsedMessage, Box<dyn std::error::Error>> {
    if message.is_empty() {
        return Err("Empty message".into());
    }

    let message_type = message[0];

    match message_type {
        MESSAGE_SYNC => {
            if message.len() < 2 {
                return Err("Invalid sync message length".into());
            }

            let sync_type = message[1];
            if sync_type == MESSAGE_SYNC_UPDATE {
                if message.len() < 3 {
                    return Err("Invalid sync update message length".into());
                }
                // Skip message type and sync type, get the update data
                let update_data = &message[2..];
                if let Ok(update) = Update::decode_v1(update_data) {
                    Ok(ParsedMessage::Sync(update.encode_v1()))
                } else {
                    Err("Failed to decode sync update".into())
                }
            } else {
                Err("Unsupported sync message type".into())
            }
        }
        MESSAGE_AWARENESS => {
            if message.len() < 2 {
                return Err("Invalid awareness message length".into());
            }

            // Skip message type, get the awareness data
            let awareness_data = &message[1..];
            match AwarenessUpdate::decode_v1(awareness_data) {
                Ok(awareness_update) => Ok(ParsedMessage::Awareness(awareness_update)),
                Err(e) => Err(format!("Failed to decode awareness update: {}", e).into()),
            }
        }
        _ => Err(format!("Unknown message type: {}", message_type).into()),
    }
}

/// Encode sync step 1 message
pub fn encode_sync_step1(state_vector: &[u8]) -> Bytes {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MESSAGE_SYNC as u32);
    encoder.write_var(MESSAGE_SYNC_STEP1 as u32);
    encoder.write_buf(state_vector);
    Bytes::from(encoder.to_vec())
}

/// Encode sync step 2 message
pub fn encode_sync_step2(diff: &[u8]) -> Bytes {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MESSAGE_SYNC as u32);
    encoder.write_var(MESSAGE_SYNC_STEP2 as u32);
    encoder.write_buf(diff);
    Bytes::from(encoder.to_vec())
}

/// Encode awareness update message
pub fn encode_awareness_update(
    awareness: &Awareness,
    clients: Vec<u64>,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let awareness_update = awareness.update_with_clients(clients)?;
    let msg = Message::Awareness(awareness_update);
    Ok(Bytes::from(msg.encode_v1()))
}

/// Encode awareness user disconnected message
pub fn encode_awareness_user_disconnected(client_id: u64, last_clock: u64) -> Bytes {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MESSAGE_AWARENESS as u32);

    // Create awareness update for disconnection
    let mut awareness_encoder = EncoderV1::new();
    awareness_encoder.write_var(1u32); // one change
    awareness_encoder.write_var(client_id as u32);
    awareness_encoder.write_var((last_clock + 1) as u32);
    awareness_encoder.write_string("null"); // JSON null for disconnection

    encoder.write_buf(awareness_encoder.to_vec());
    Bytes::from(encoder.to_vec())
}
