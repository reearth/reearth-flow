package websocket

import (
	"context"
	"encoding/json"
	"fmt"
	"time"
	"unsafe"

	"github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/reearth/reearthx/log"
)

// #cgo LDFLAGS: -L${SRCDIR}/../../../../target/release -lwebsocket
// #include <stdlib.h>
// #include <stdint.h>
// extern char* get_latest_document(const char* doc_id, const char* config_json);
// extern char* get_document_history(const char* doc_id, const char* config_json);
// extern char* rollback_document(const char* doc_id, int32_t target_clock, const char* config_json);
// extern void free_string(char* ptr);
import "C"

type Config struct {
	GcsBucket   string  `json:"gcs_bucket"`
	GcsEndpoint *string `json:"gcs_endpoint,omitempty"`
	RedisUrl    string  `json:"redis_url"`
	RedisTtl    uint64  `json:"redis_ttl"`
}

type Client struct {
	config Config
}

type documentResponse struct {
	DocID   string `json:"doc_id"`
	Content []byte `json:"content"`
	Clock   int32  `json:"clock"`
}

type historyVersion struct {
	VersionID string `json:"version_id"`
	Timestamp string `json:"timestamp"`
	Content   []byte `json:"content"`
	Clock     int32  `json:"clock"`
}

type documentHistory struct {
	DocID    string           `json:"doc_id"`
	Versions []historyVersion `json:"versions"`
}

func NewClient(config Config) (*Client, error) {
	return &Client{
		config: config,
	}, nil
}

func (c *Client) Close() error {
	return nil
}

func (c *Client) GetLatest(ctx context.Context, docID string) (*websocket.Document, error) {
	configJSON, err := json.Marshal(c.config)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal config: %w", err)
	}

	log.Infof("GetLatest config: %s", string(configJSON))

	cDocID := C.CString(docID)
	cConfigJSON := C.CString(string(configJSON))
	defer C.free(unsafe.Pointer(cDocID))
	defer C.free(unsafe.Pointer(cConfigJSON))

	log.Infof("Calling FFI get_latest_document with docID: %s", docID)
	result := C.get_latest_document(cDocID, cConfigJSON)
	if result == nil {
		log.Errorf("FFI call returned nil for docID: %s", docID)
		return nil, fmt.Errorf("failed to get latest document")
	}
	defer C.free_string(result)

	resultStr := C.GoString(result)
	log.Infof("FFI result: %s", resultStr)

	var resp documentResponse
	if err := json.Unmarshal([]byte(resultStr), &resp); err != nil {
		log.Errorf("Failed to unmarshal response: %v", err)
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	// Convert bytes to []int
	update := make([]int, len(resp.Content))
	for i, b := range resp.Content {
		update[i] = int(b)
	}

	doc := &websocket.Document{
		ID:        docID,
		Update:    update,
		Clock:     int(resp.Clock),
		Timestamp: time.Now(),
	}
	log.Infof("Returning document: %+v", doc)
	return doc, nil
}

func (c *Client) GetHistory(ctx context.Context, docID string) ([]*websocket.History, error) {
	configJSON, err := json.Marshal(c.config)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal config: %w", err)
	}

	cDocID := C.CString(docID)
	cConfigJSON := C.CString(string(configJSON))
	defer C.free(unsafe.Pointer(cDocID))
	defer C.free(unsafe.Pointer(cConfigJSON))

	result := C.get_document_history(cDocID, cConfigJSON)
	if result == nil {
		return nil, fmt.Errorf("failed to get document history")
	}
	defer C.free_string(result)

	var resp documentHistory
	if err := json.Unmarshal([]byte(C.GoString(result)), &resp); err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	history := make([]*websocket.History, len(resp.Versions))
	for i, version := range resp.Versions {
		timestamp, err := time.Parse(time.RFC3339, version.Timestamp)
		if err != nil {
			return nil, fmt.Errorf("failed to parse timestamp: %w", err)
		}

		// Convert bytes to []int
		update := make([]int, len(version.Content))
		for j, b := range version.Content {
			update[j] = int(b)
		}

		history[i] = &websocket.History{
			Update:    update,
			Clock:     int(version.Clock),
			Timestamp: timestamp,
		}
	}

	return history, nil
}

func (c *Client) Rollback(ctx context.Context, id string, clock int) (*websocket.Document, error) {
	configJSON, err := json.Marshal(c.config)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal config: %w", err)
	}

	cDocID := C.CString(id)
	cConfigJSON := C.CString(string(configJSON))
	defer C.free(unsafe.Pointer(cDocID))
	defer C.free(unsafe.Pointer(cConfigJSON))

	result := C.rollback_document(cDocID, C.int32_t(clock), cConfigJSON)
	if result == nil {
		return nil, fmt.Errorf("failed to rollback document")
	}
	defer C.free_string(result)

	var resp documentResponse
	if err := json.Unmarshal([]byte(C.GoString(result)), &resp); err != nil {
		return nil, fmt.Errorf("failed to unmarshal response: %w", err)
	}

	// Convert bytes to []int
	update := make([]int, len(resp.Content))
	for i, b := range resp.Content {
		update[i] = int(b)
	}

	return &websocket.Document{
		ID:        id,
		Update:    update,
		Clock:     int(resp.Clock),
		Timestamp: time.Now(),
	}, nil
}
