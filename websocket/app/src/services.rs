use super::errors::Result;
use std::sync::Arc;
use yrs::sync::{Awareness, SyncMessage};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, Transact, Update};

pub struct YjsService {
    doc: Arc<Doc>,
}

impl YjsService {
    pub fn new(doc: Arc<Doc>) -> Self {
        YjsService { doc }
    }

    pub fn handle_message(&self, data: &[u8]) -> Result<Option<Vec<u8>>> {
        match yrs::sync::Message::decode_v2(data) {
            Ok(yrs::sync::Message::Sync(SyncMessage::SyncStep1(sv))) => {
                let txn = self.doc.transact();
                let update = txn.encode_state_as_update_v2(&sv);
                let sync2 = SyncMessage::SyncStep2(update).encode_v2();
                Ok(Some(sync2))
            }
            Ok(yrs::sync::Message::Sync(SyncMessage::Update(data))) => {
                let mut txn = self.doc.transact_mut();
                txn.apply_update(Update::decode_v2(&data)?);
                Ok(None)
            }
            Ok(yrs::sync::Message::Awareness(update)) => {
                let mut awareness = Awareness::new((*self.doc).clone());
                awareness.apply_update(update)?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
