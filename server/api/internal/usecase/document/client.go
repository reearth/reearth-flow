package document

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/reearth/reearthx/log"
)

type Client struct {
	baseURL string
	client  *http.Client
}

func NewClient(baseURL string) *Client {
	baseURL = strings.TrimRight(baseURL, "/")
	log.Infof("Creating new document client with base URL: %s", baseURL)
	return &Client{
		baseURL: baseURL,
		client: &http.Client{
			Timeout: 10 * time.Second,
		},
	}
}

func (c *Client) GetLatest(ctx context.Context, docID string) (*Document, error) {
	if c.baseURL == "" {
		return nil, fmt.Errorf("base URL is not configured")
	}

	url := fmt.Sprintf("%s/%s/latest", c.baseURL, docID)
	log.Debugf("Making request to: %s", url)
	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Errorf("failed to close response body: %v", err)
		}
	}(resp.Body)

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("unexpected status code: %d", resp.StatusCode)
	}

	var result []int
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	return &Document{
		ID:        docID,
		Update:    result,
		Clock:     0,
		Timestamp: time.Now(),
	}, nil
}

type rawUpdateHistory struct {
	Update    []int   `json:"update"`
	Clock     int     `json:"clock"`
	Timestamp []int64 `json:"timestamp"`
}

func (r *rawUpdateHistory) Time() time.Time {
	if len(r.Timestamp) >= 9 {
		return time.Date(
			int(r.Timestamp[0]),        // year
			time.Month(r.Timestamp[1]), // month
			int(r.Timestamp[2]),        // day
			int(r.Timestamp[3]),        // hour
			int(r.Timestamp[4]),        // minute
			int(r.Timestamp[5]/1e9),    // second
			int(r.Timestamp[5]%1e9),    // nanosecond
			time.FixedZone("UTC", 0),   // timezone
		)
	}
	return time.Time{}
}

func (c *Client) GetHistory(ctx context.Context, docID string) ([]*History, error) {
	if c.baseURL == "" {
		return nil, fmt.Errorf("base URL is not configured")
	}

	url := fmt.Sprintf("%s/%s/history", c.baseURL, docID)
	log.Debugf("Making request to: %s", url)
	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Errorf("failed to close response body: %v", err)
		}
	}(resp.Body)

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("unexpected status code: %d", resp.StatusCode)
	}

	var updates []rawUpdateHistory
	if err := json.NewDecoder(resp.Body).Decode(&updates); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	history := make([]*History, len(updates))
	for i, update := range updates {
		history[i] = &History{
			Update:    update.Update,
			Clock:     update.Clock,
			Timestamp: update.Time(),
		}
	}

	return history, nil
}

func (c *Client) Rollback(ctx context.Context, id string, clock int) (*Document, error) {
	url := fmt.Sprintf("%s/%s/rollback?clock=%d", c.baseURL, id, clock)
	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
	if err != nil {
		return nil, err
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("unexpected status code: %d", resp.StatusCode)
	}

	var doc Document
	if err := json.NewDecoder(resp.Body).Decode(&doc); err != nil {
		return nil, err
	}

	return &doc, nil
}
