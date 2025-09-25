package websocket

type documentResponse struct {
	ID        string `json:"id"`
	Updates   []byte `json:"updates"`
	Version   uint64 `json:"version"`
	Timestamp string `json:"timestamp"`
}

type historyResponse struct {
	Updates   []byte `json:"updates"`
	Version   uint64 `json:"version"`
	Timestamp string `json:"timestamp"`
}

type rollbackRequest struct {
	DocID   string `json:"doc_id"`
	Version uint64 `json:"version"`
}

type createSnapshotRequest struct {
	DocID   string `json:"doc_id"`
	Version uint64 `json:"version"`
	Name    string `json:"name"`
}

type snapshotResponse struct {
	ID        string `json:"id"`
	Updates   []byte `json:"updates"`
	Version   uint64 `json:"version"`
	Timestamp string `json:"timestamp"`
	Name      string `json:"name"`
}

type importDocumentRequest struct {
	Data []byte `json:"data"`
}
