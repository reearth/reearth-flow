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
	ServerURL string `json:"server_url"`
}

type Client struct {
	config Config
	client *http.Client
}

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

func NewClient(config Config) (*Client, error) {
	if config.ServerURL == "" {
		config.ServerURL = "http://localhost:8000"
	}

	client := &http.Client{
		Timeout: 30 * time.Second,
	}

	return &Client{
		config: config,
		client: client,
	}, nil
}

func (c *Client) Close() error {
	return nil
}

func (c *Client) GetLatest(ctx context.Context, docID string) (*websocket.Document, error) {
	url := fmt.Sprintf("%s/api/document/%s", c.config.ServerURL, docID)
	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get latest document: %w", err)
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Warnf("failed to close response body: %v", err)
		}
	}(resp.Body)

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("server returned non-200 status: %d %s", resp.StatusCode, body)
	}

	var docResp documentResponse
	if err := json.NewDecoder(resp.Body).Decode(&docResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	updates := make([]int, len(docResp.Updates))
	for i, update := range docResp.Updates {
		updates[i] = int(update)
	}

	timestamp, err := time.Parse(time.RFC3339, docResp.Timestamp)
	if err != nil {
		log.Warnf("failed to parse timestamp: %v, using current time", err)
		timestamp = time.Now()
	}

	doc := &websocket.Document{
		ID:        docResp.ID,
		Updates:   updates,
		Version:   int(docResp.Version),
		Timestamp: timestamp,
	}

	log.Infof("Returning document: %+v", doc)
	return doc, nil
}

func (c *Client) GetHistory(ctx context.Context, docID string) ([]*websocket.History, error) {
	url := fmt.Sprintf("%s/api/document/%s/history", c.config.ServerURL, docID)
	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get document history: %w", err)
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Warnf("failed to close response body: %v", err)
		}
	}(resp.Body)

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("server returned non-200 status: %d %s", resp.StatusCode, body)
	}

	var historyResp []historyResponse
	if err := json.NewDecoder(resp.Body).Decode(&historyResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	history := make([]*websocket.History, len(historyResp))
	for i, item := range historyResp {
		timestamp, err := time.Parse(time.RFC3339, item.Timestamp)
		if err != nil {
			log.Warnf("failed to parse timestamp: %v, using current time", err)
			timestamp = time.Now()
		}

		updates := make([]int, len(item.Updates))
		for j, update := range item.Updates {
			updates[j] = int(update)
		}

		history[i] = &websocket.History{
			Updates:   updates,
			Version:   int(item.Version),
			Timestamp: timestamp,
		}
	}

	return history, nil
}

func (c *Client) GetHistoryMetadata(ctx context.Context, docID string) ([]*websocket.HistoryMetadata, error) {
	url := fmt.Sprintf("%s/api/document/%s/history/metadata", c.config.ServerURL, docID)
	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get document history metadata: %w", err)
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Warnf("failed to close response body: %v", err)
		}
	}(resp.Body)

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("server returned non-200 status: %d %s", resp.StatusCode, body)
	}

	var historyResp []struct {
		Version   uint64 `json:"version"`
		Timestamp string `json:"timestamp"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&historyResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	metadata := make([]*websocket.HistoryMetadata, len(historyResp))
	for i, item := range historyResp {
		timestamp, err := time.Parse(time.RFC3339, item.Timestamp)
		if err != nil {
			log.Warnf("failed to parse timestamp: %v, using current time", err)
			timestamp = time.Now()
		}

		metadata[i] = &websocket.HistoryMetadata{
			Version:   int(item.Version),
			Timestamp: timestamp,
		}
	}

	return metadata, nil
}

func (c *Client) GetHistoryByVersion(ctx context.Context, docID string, version int) (*websocket.History, error) {
	url := fmt.Sprintf("%s/api/document/%s/history/version/%d", c.config.ServerURL, docID, version)
	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to get document history for version %d: %w", version, err)
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Warnf("failed to close response body: %v", err)
		}
	}(resp.Body)

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("server returned non-200 status: %d %s", resp.StatusCode, body)
	}

	var historyResp historyResponse
	if err := json.NewDecoder(resp.Body).Decode(&historyResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	timestamp, err := time.Parse(time.RFC3339, historyResp.Timestamp)
	if err != nil {
		log.Warnf("failed to parse timestamp: %v, using current time", err)
		timestamp = time.Now()
	}

	updates := make([]int, len(historyResp.Updates))
	for j, update := range historyResp.Updates {
		updates[j] = int(update)
	}

	return &websocket.History{
		Version:   int(historyResp.Version),
		Timestamp: timestamp,
		Updates:   updates,
	}, nil
}

func (c *Client) Rollback(ctx context.Context, id string, version int) (*websocket.Document, error) {
	url := fmt.Sprintf("%s/api/document/%s/rollback", c.config.ServerURL, id)

	rollbackReq := rollbackRequest{
		DocID:   id,
		Version: uint64(version),
	}

	reqBody, err := json.Marshal(rollbackReq)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, "POST", url, io.NopCloser(bytes.NewReader(reqBody)))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to rollback document: %w", err)
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Warnf("failed to close response body: %v", err)
		}
	}(resp.Body)

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("server returned non-200 status: %d %s", resp.StatusCode, body)
	}

	var docResp documentResponse
	if err := json.NewDecoder(resp.Body).Decode(&docResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	updates := make([]int, len(docResp.Updates))
	for i, update := range docResp.Updates {
		updates[i] = int(update)
	}

	timestamp, err := time.Parse(time.RFC3339, docResp.Timestamp)
	if err != nil {
		log.Warnf("failed to parse timestamp: %v, using current time", err)
		timestamp = time.Now()
	}

	return &websocket.Document{
		ID:        docResp.ID,
		Updates:   updates,
		Version:   int(docResp.Version),
		Timestamp: timestamp,
	}, nil
}

func (c *Client) FlushToGCS(ctx context.Context, id string) error {
	url := fmt.Sprintf("%s/api/document/%s/flush", c.config.ServerURL, id)

	req, err := http.NewRequestWithContext(ctx, "POST", url, nil)
	if err != nil {
		return fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return fmt.Errorf("failed to flush document to GCS: %w", err)
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Warnf("failed to close response body: %v", err)
		}
	}(resp.Body)

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("server returned non-200 status: %d %s", resp.StatusCode, body)
	}

	return nil
}

func (c *Client) ClearDoc(ctx context.Context, id string) error {
	url := fmt.Sprintf("%s/api/document/%s/clear", c.config.ServerURL, id)

	req, err := http.NewRequestWithContext(ctx, "POST", url, nil)
	if err != nil {
		return fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return fmt.Errorf("failed to clear document: %w", err)
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Warnf("failed to close response body: %v", err)
		}
	}(resp.Body)

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("server returned non-200 status: %d %s", resp.StatusCode, body)
	}

	return nil
}
