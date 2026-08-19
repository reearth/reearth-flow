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

// snapshotItemResponse is one labelled snapshot; POST .../snapshots populates
// only ID and Label.
type snapshotItemResponse struct {
	Label     string `json:"label"`
	Timestamp string `json:"timestamp"`
	ID        int64  `json:"id"`
	Size      int64  `json:"size"`
}

type saveSnapshotRequest struct {
	Label string `json:"label"`
}

// snapshotStateResponse mirrors websocket-go's SnapshotStateResponse. Updates
// arrives as a JSON int array (the server's UpdateBytes marshaller); encoding/json
// decodes that into []byte as readily as it does base64.
type snapshotStateResponse struct {
	ID         string `json:"id"`
	Updates    []byte `json:"updates"`
	SnapshotID int64  `json:"snapshot_id"`
}
