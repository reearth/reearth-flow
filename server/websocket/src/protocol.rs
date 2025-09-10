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

pub fn merge_messages(messages: Vec<Bytes>) -> Vec<Bytes> {
    if messages.len() < 2 {
        return messages;
    }

    for message in messages.iter().rev() {
        if !message.is_empty() {
            return vec![message.clone()];
        }
    }

    messages
}

pub fn encode_sync_step1(state_vector: &[u8]) -> Bytes {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MESSAGE_SYNC as u32);
    encoder.write_var(MESSAGE_SYNC_STEP1 as u32);
    encoder.write_buf(state_vector);
    Bytes::from(encoder.to_vec())
}

pub fn encode_sync_step2(diff: &[u8]) -> Bytes {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MESSAGE_SYNC as u32);
    encoder.write_var(MESSAGE_SYNC_STEP2 as u32);
    encoder.write_buf(diff);
    Bytes::from(encoder.to_vec())
}

pub fn encode_awareness_update(
    awareness: &Awareness,
    clients: Vec<u64>,
) -> Result<Bytes, Box<dyn std::error::Error>> {
    let awareness_update = awareness.update_with_clients(clients)?;
    let msg = Message::Awareness(awareness_update);
    Ok(Bytes::from(msg.encode_v1()))
}

pub fn encode_awareness_user_disconnected(client_id: u64, last_clock: u64) -> Bytes {
    let mut encoder = EncoderV1::new();
    encoder.write_var(MESSAGE_AWARENESS as u32);

    let mut awareness_encoder = EncoderV1::new();
    awareness_encoder.write_var(1u32);
    awareness_encoder.write_var(client_id as u32);
    awareness_encoder.write_var((last_clock + 1) as u32);
    awareness_encoder.write_string("null");

    encoder.write_buf(awareness_encoder.to_vec());
    Bytes::from(encoder.to_vec())
}
