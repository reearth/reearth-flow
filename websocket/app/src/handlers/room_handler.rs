use flow_websocket_infra::types::user::User;

use crate::{errors::WsError, state::AppState};
use std::sync::Arc;

use super::types::Event;

pub async fn handle_room_event(
    event: &Event,
    room_id: &str,
    state: &Arc<AppState>,
    user: &User,
) -> Result<(), WsError> {
    match event {
        Event::Create { room_id } => {
            state.make_room(room_id.clone()).await?;
        }
        Event::Join { room_id } => {
            state.join(room_id, &user.id).await?;
        }
        Event::Leave => {
            state.leave(room_id, &user.id).await?;
        }
        Event::Emit { data } => {
            state.emit(data).await;
        }
    }
    Ok(())
}
