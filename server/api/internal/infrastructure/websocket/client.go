package websocket

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/reearth/reearthx/log"
)

type Config struct {
	GcsBucket   string  `json:"gcs_bucket"`
	GcsEndpoint *string `json:"gcs_endpoint,omitempty"`
	RedisUrl    string  `json:"redis_url"`
	RedisTtl    uint64  `json:"redis_ttl"`
	ServerURL   string  `json:"server_url"`
}

type Client struct {
	config Config
	client *http.Client
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
	if config.ServerURL == "" {
		config.ServerURL = "http://localhost:8000"
	}

	return &Client{
		config: config,
		client: &http.Client{
			Timeout: 30 * time.Second,
		},
	}, nil
}

func (c *Client) Close() error {
	c.client.CloseIdleConnections()
	return nil
}

func (c *Client) GetLatest(ctx context.Context, docID string) (*websocket.Document, error) {
	url := fmt.Sprintf("%s/api/documents/%s", c.config.ServerURL, docID)

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("server returned error: %s, body: %s", resp.Status, string(body))
	}

	var docResp documentResponse
	if err := json.NewDecoder(resp.Body).Decode(&docResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	// Convert bytes to []int
	update := make([]int, len(docResp.Content))
	for i, b := range docResp.Content {
		update[i] = int(b)
	}

	doc := &websocket.Document{
		ID:        docID,
		Update:    update,
		Clock:     int(docResp.Clock),
		Timestamp: time.Now(),
	}

	log.Infof("Returning document: %+v", doc)
	return doc, nil
}

func (c *Client) GetHistory(ctx context.Context, docID string) ([]*websocket.History, error) {
	url := fmt.Sprintf("%s/api/documents/%s/history", c.config.ServerURL, docID)

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("server returned error: %s, body: %s", resp.Status, string(body))
	}

	var histResp documentHistory
	if err := json.NewDecoder(resp.Body).Decode(&histResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	history := make([]*websocket.History, len(histResp.Versions))
	for i, version := range histResp.Versions {
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
	url := fmt.Sprintf("%s/api/documents/%s/rollback", c.config.ServerURL, id)

	data := map[string]interface{}{
		"clock": clock,
	}

	jsonData, err := json.Marshal(data)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewBuffer(jsonData))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("server returned error: %s, body: %s", resp.Status, string(body))
	}

	var docResp documentResponse
	if err := json.NewDecoder(resp.Body).Decode(&docResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	// Convert bytes to []int
	update := make([]int, len(docResp.Content))
	for i, b := range docResp.Content {
		update[i] = int(b)
	}

	return &websocket.Document{
		ID:        id,
		Update:    update,
		Clock:     int(docResp.Clock),
		Timestamp: time.Now(),
	}, nil
}
