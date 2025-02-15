use futures_util::Stream;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use yrs::{
    updates::decoder::Decode, updates::encoder::Encode, Doc, ReadTxn, StateVector, Transact, Update,
};

use crate::{storage::kv::DocOps, AppState};

pub mod document {
    tonic::include_proto!("document");
}

use document::document_service_server::DocumentService;
use document::{
    DocumentHistoryRequest, DocumentHistoryResponse, DocumentRequest, DocumentResponse,
    DocumentUpdate, DocumentVersion, RollbackRequest, RollbackResponse,
};

pub struct DocumentServiceImpl {
    state: Arc<AppState>,
}

impl DocumentServiceImpl {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    fn normalize_doc_id(doc_id: &str) -> String {
        doc_id.strip_suffix(":main").unwrap_or(doc_id).to_string()
    }
}

#[tonic::async_trait]
impl DocumentService for DocumentServiceImpl {
    type SyncDocumentStream =
        Pin<Box<dyn Stream<Item = Result<DocumentUpdate, Status>> + Send + 'static>>;

    async fn sync_document(
        &self,
        request: Request<tonic::Streaming<DocumentUpdate>>,
    ) -> Result<Response<Self::SyncDocumentStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(32);
        let state = self.state.clone();

        tokio::spawn(async move {
            while let Some(update) = stream.message().await.unwrap() {
                let doc_id = Self::normalize_doc_id(&update.doc_id);
                let _group = match state.pool.get_or_create_group(&doc_id).await {
                    Ok(group) => group,
                    Err(e) => {
                        tracing::error!("Failed to get or create group: {}", e);
                        break;
                    }
                };

                // Apply update to the document
                let doc = Doc::new();
                let mut txn = doc.transact_mut();
                if let Ok(update) = Update::decode_v1(&update.update_data) {
                    if let Err(e) = txn.apply_update(update) {
                        tracing::error!("Failed to apply update: {}", e);
                    }
                }

                if let Err(e) = tx.send(Ok(update)).await {
                    tracing::error!("Error sending update: {:?}", e);
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    async fn get_latest_document(
        &self,
        request: Request<DocumentRequest>,
    ) -> Result<Response<DocumentResponse>, Status> {
        let doc_id = Self::normalize_doc_id(&request.into_inner().doc_id);
        let store = self.state.pool.get_store();
        let doc = Doc::new();
        let mut txn = doc.transact_mut();

        match store.load_doc(&doc_id, &mut txn).await {
            Ok(true) => {
                drop(txn);
                let read_txn = doc.transact();
                let state = read_txn.encode_diff_v1(&StateVector::default());

                // Get the latest clock from updates
                let clock = match store.get_updates(&doc_id).await {
                    Ok(updates) if !updates.is_empty() => updates.last().unwrap().clock as i32,
                    _ => 0,
                };

                Ok(Response::new(DocumentResponse {
                    doc_id,
                    content: state,
                    clock,
                }))
            }
            Ok(false) => Err(Status::not_found("Document not found")),
            Err(e) => {
                tracing::error!("Failed to get document: {}", e);
                Err(Status::internal("Failed to get document"))
            }
        }
    }

    async fn get_document_history(
        &self,
        request: Request<DocumentHistoryRequest>,
    ) -> Result<Response<DocumentHistoryResponse>, Status> {
        let doc_id = Self::normalize_doc_id(&request.into_inner().doc_id);
        let store = self.state.pool.get_store();

        match store.get_updates(&doc_id).await {
            Ok(updates) => {
                let versions = updates
                    .into_iter()
                    .map(|info| DocumentVersion {
                        version_id: info.clock.to_string(),
                        timestamp: info.timestamp.to_string(),
                        content: info.update.encode_v1(),
                        clock: info.clock as i32,
                    })
                    .collect();

                Ok(Response::new(DocumentHistoryResponse { doc_id, versions }))
            }
            Err(e) => {
                tracing::error!("Failed to get document history: {}", e);
                Err(Status::internal("Failed to get document history"))
            }
        }
    }

    async fn rollback_document(
        &self,
        request: Request<RollbackRequest>,
    ) -> Result<Response<RollbackResponse>, Status> {
        let req = request.into_inner();
        let doc_id = Self::normalize_doc_id(&req.doc_id);
        let version_id = req
            .version_id
            .parse::<u32>()
            .map_err(|_| Status::invalid_argument("Invalid version_id: must be a valid u32"))?;

        let store = self.state.pool.get_store();
        match store.rollback_to(&doc_id, version_id).await {
            Ok(_) => Ok(Response::new(RollbackResponse {
                success: true,
                message: "Document rolled back successfully".to_string(),
            })),
            Err(e) => {
                tracing::error!("Failed to rollback document: {}", e);
                Ok(Response::new(RollbackResponse {
                    success: false,
                    message: format!("Failed to rollback document: {}", e),
                }))
            }
        }
    }
}
