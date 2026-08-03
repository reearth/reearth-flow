package websocket

type documentResponse struct {
	ID        string `json:"id"`
	Timestamp string `json:"timestamp"`
	Updates   []byte `json:"updates"`
	Version   uint64 `json:"version"`
}

type historyResponse struct {
	Timestamp string `json:"timestamp"`
	Updates   []byte `json:"updates"`
	Version   uint64 `json:"version"`
}

type rollbackRequest struct {
	DocID   string `json:"doc_id"`
	Version uint64 `json:"version"`
}

type createSnapshotRequest struct {
	DocID   string `json:"doc_id"`
	Name    string `json:"name"`
	Version uint64 `json:"version"`
}

type snapshotResponse struct {
	ID        string `json:"id"`
	Timestamp string `json:"timestamp"`
	Name      string `json:"name"`
	Updates   []byte `json:"updates"`
	Version   uint64 `json:"version"`
}

type importDocumentRequest struct {
	Data []int `json:"data"`
}

// snapshotItemResponse is the wire shape for one labelled snapshot, returned
// both by GET .../snapshots (fully populated) and POST .../snapshots (only ID
// and Label populated; Timestamp/Size are the zero value there).
type snapshotItemResponse struct {
	Label     string `json:"label"`
	Timestamp string `json:"timestamp"`
	ID        int64  `json:"id"`
	Size      int64  `json:"size"`
}

type saveSnapshotRequest struct {
	Label string `json:"label"`
}
