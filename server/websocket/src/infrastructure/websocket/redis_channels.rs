use crate::infrastructure::redis::RedisStore;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, warn};

macro_rules! impl_flush_pending {
    ($(#[$meta:meta])* $fn_name:ident, $batch_size:expr, $redis_method:ident, $update_type:expr) => {
        $(#[$meta])*
        pub async fn $fn_name(
            redis_store: &RedisStore,
            conn: &mut redis::aio::MultiplexedConnection,
            receiver: &mut mpsc::Receiver<Vec<u8>>,
            stream_key: &str,
            instance_id: &u64,
        ) {
            let mut updates = Vec::new();

            while let Ok(update) = receiver.try_recv() {
                updates.push(update);
                if updates.len() >= $batch_size {
                    break;
                }
            }

            if !updates.is_empty() {
                let refs: Vec<&[u8]> = updates.iter().map(|u| u.as_slice()).collect();
                if let Err(e) = redis_store
                    .$redis_method(conn, stream_key, &refs, instance_id)
                    .await
                {
                    warn!(
                        "Failed to batch write {} {} to Redis: {}",
                        updates.len(),
                        $update_type,
                        e
                    );
                } else {
                    debug!("Successfully batched {} {} to Redis", updates.len(), $update_type);
                }
            }
        }
    };
}

pub struct RedisChannels {
    pub write_tx: mpsc::Sender<Vec<u8>>,
    pub write_rx: Arc<Mutex<mpsc::Receiver<Vec<u8>>>>,

    pub awareness_tx: mpsc::Sender<Vec<u8>>,
    pub awareness_rx: Arc<Mutex<mpsc::Receiver<Vec<u8>>>>,
}

impl RedisChannels {
    pub fn new(write_capacity: usize, awareness_capacity: usize) -> Self {
        let (write_tx, write_rx) = mpsc::channel::<Vec<u8>>(write_capacity);
        let write_rx = Arc::new(Mutex::new(write_rx));

        let (awareness_tx, awareness_rx) = mpsc::channel::<Vec<u8>>(awareness_capacity);
        let awareness_rx = Arc::new(Mutex::new(awareness_rx));

        Self {
            write_tx,
            write_rx,
            awareness_tx,
            awareness_rx,
        }
    }

    impl_flush_pending!(
        flush_pending_updates,
        100,
        publish_multiple_updates,
        "updates"
    );

    impl_flush_pending!(
        flush_pending_awareness,
        50,
        publish_multiple_awareness,
        "awareness updates"
    );
}
