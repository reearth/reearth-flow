package http

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"time"
)

type APIGateway struct {
	baseURL    string
	httpClient *http.Client
}

func NewAPIGateway(baseURL string) *APIGateway {
	return &APIGateway{
		baseURL: baseURL,
		httpClient: &http.Client{
			Timeout: 10 * time.Second,
		},
	}
}

type JobStatusNotification struct {
	JobID  string `json:"jobId"`
	Status string `json:"status"`
}

func (a *APIGateway) NotifyJobStatusChange(ctx context.Context, jobID string, status string) error {
	notification := JobStatusNotification{
		JobID:  jobID,
		Status: status,
	}

	data, err := json.Marshal(notification)
	if err != nil {
		return fmt.Errorf("failed to marshal notification: %w", err)
	}

	url := fmt.Sprintf("%s/internal/jobs/%s/status", a.baseURL, jobID)
	req, err := http.NewRequestWithContext(ctx, "POST", url, bytes.NewBuffer(data))
	if err != nil {
		return fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")

	resp, err := a.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("failed to send notification: %w", err)
	}
	defer func() {
		if closeErr := resp.Body.Close(); closeErr != nil {
			log.Printf("failed to close response body: %v", closeErr)
		}
	}()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return fmt.Errorf("API server returned status %d", resp.StatusCode)
	}

	return nil
}
