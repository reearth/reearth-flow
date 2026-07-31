// Package http implements the Y-WebSocket server's HTTP document API, the
// X-API-Secret middleware, and OTLP-instrumented request handling.
package http

import (
	"encoding/json"
	"fmt"
)

// UpdateBytes is a yjs update blob. WARNING: it MUST serialize to/from a JSON
// int array (0–255), NOT base64 — the Go API client decodes `updates` as a JSON
// int array, and stdlib []byte would marshal to a corrupting base64 string.
type UpdateBytes []byte

// MarshalJSON renders the bytes as a JSON int array; nil/empty renders as [], never null.
func (u UpdateBytes) MarshalJSON() ([]byte, error) {
	if len(u) == 0 {
		return []byte("[]"), nil
	}
	ints := make([]int, len(u))
	for i, b := range u {
		ints[i] = int(b)
	}
	return json.Marshal(ints)
}

// UnmarshalJSON parses a JSON int array into bytes, rejecting values outside 0–255.
func (u *UpdateBytes) UnmarshalJSON(data []byte) error {
	var ints []int
	if err := json.Unmarshal(data, &ints); err != nil {
		return err
	}
	out := make(UpdateBytes, len(ints))
	for i, n := range ints {
		if n < 0 || n > 255 {
			return fmt.Errorf("updates[%d] = %d out of byte range 0..255", i, n)
		}
		out[i] = byte(n)
	}
	*u = out
	return nil
}

// DocumentResponse is the document API response shape: id, timestamp (RFC3339),
// updates (int array), version.
type DocumentResponse struct {
	ID        string      `json:"id"`
	Timestamp string      `json:"timestamp"`
	Updates   UpdateBytes `json:"updates"`
	Version   uint64      `json:"version"`
}

// HistoryMetadataItem is one history entry: version + timestamp, without update bytes.
type HistoryMetadataItem struct {
	Version   uint64 `json:"version"`
	Timestamp string `json:"timestamp"`
}

// RollbackRequest is the POST .../rollback body. The handler uses the PATH
// doc_id and the BODY version; the body doc_id is ignored.
type RollbackRequest struct {
	DocID   string `json:"doc_id"`
	Version uint64 `json:"version"`
}

// CreateSnapshotRequest is the POST /api/document/snapshot body (doc_id in body only).
type CreateSnapshotRequest struct {
	DocID   string `json:"doc_id"`
	Name    string `json:"name,omitempty"`
	Version uint64 `json:"version"`
}

// ImportRequest is the POST .../import body: a raw v1 update as an int array.
type ImportRequest struct {
	Data UpdateBytes `json:"data"`
}

// errorResponse is the JSON body for non-2xx responses.
type errorResponse struct {
	Error string `json:"error"`
}

// SnapshotItem is one labelled snapshot in the version history.
type SnapshotItem struct {
	ID        int64  `json:"id"`
	Label     string `json:"label"`
	Timestamp string `json:"timestamp"`
	Size      int64  `json:"size"`
}

// SaveSnapshotRequest is the POST body for creating a named snapshot.
type SaveSnapshotRequest struct {
	Label string `json:"label"`
}

// SnapshotStateResponse is one snapshot's stored state.
//
// Deliberately NOT DocumentResponse. That shape carries a `version` field which
// everywhere else in this API means an update-log clock, and it has no
// omitempty, so reusing it here would ship `"version":0` on every snapshot
// fetch. Zero is not a harmless placeholder in that space: POST .../rollback
// with version 0 prunes every update above clock 0, i.e. the whole log. Naming
// the field snapshot_id keeps the two id spaces impossible to confuse for a
// future consumer.
type SnapshotStateResponse struct {
	ID         string      `json:"id"`
	SnapshotID int64       `json:"snapshot_id"`
	Updates    UpdateBytes `json:"updates"`
}
