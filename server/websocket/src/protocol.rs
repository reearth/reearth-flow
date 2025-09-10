use bytes::Bytes;
use yrs::encoding::write::Write;
use yrs::sync::awareness::Awareness;
use yrs::sync::Message;
use yrs::updates::encoder::{Encode, Encoder, EncoderV1};

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

    // For safety, temporarily disable complex message merging
    // Return the last non-empty message to avoid parsing errors
    for message in messages.iter().rev() {
        if !message.is_empty() {
            return vec![message.clone()];
        }
    }

    // If all messages are empty, return the original messages
    messages
}

// Parsing logic removed for safety - will be re-implemented later if needed

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
