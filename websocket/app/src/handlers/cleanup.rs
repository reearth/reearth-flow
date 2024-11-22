use crate::state::AppState;
use flow_websocket_infra::types::user::User;
use flow_websocket_services::manage_project_edit_session::SessionCommand;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc::Sender;
use tracing::debug;

pub fn perform_cleanup(
    is_cleaning_up: Arc<AtomicBool>,
    room_id: String,
    user: User,
    project_id: Option<String>,
    state: Arc<AppState>,
    cleanup_tx: Sender<()>,
) -> impl Fn() {
    move || {
        if is_cleaning_up
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let state = state.clone();
            let room_id = room_id.clone();
            let user = user.clone();
            let project_id = project_id.clone();
            let cleanup_tx = cleanup_tx.clone();

            tokio::spawn(async move {
                if let Err(e) = state.leave(&room_id, &user.id).await {
                    debug!("Cleanup error: {:?}", e);
                }

                if let Some(project_id) = project_id {
                    if let Err(e) = state
                        .command_tx
                        .send(SessionCommand::End {
                            project_id: project_id.clone(),
                            user: user.clone(),
                        })
                        .await
                    {
                        debug!("Failed to send End command: {:?}", e);
                    }

                    if let Err(e) = state
                        .command_tx
                        .send(SessionCommand::RemoveTask {
                            project_id: project_id.clone(),
                        })
                        .await
                    {
                        debug!("Failed to send RemoveTask command: {:?}", e);
                    }
                }

                let _ = cleanup_tx.send(()).await;
            });
        }
    }
}
