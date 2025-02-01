package notification

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"time"
)

type Payload struct {
	RunID        string   `json:"runId"`
	DeploymentID string   `json:"deploymentId"`
	Status       string   `json:"status"`
	Logs         []string `json:"logs"`
	Outputs      []string `json:"outputs"`
}

type Notifier interface {
	Send(url string, payload Payload) error
}

type HTTPNotifier struct {
	client *http.Client
}

func NewHTTPNotifier() *HTTPNotifier {
	client := &http.Client{
		Timeout: 30 * time.Second,
	}
	return &HTTPNotifier{
		client: client,
	}
}

func (n *HTTPNotifier) Send(url string, payload Payload) error {
	jsonData, err := json.Marshal(payload)
	if err != nil {
		return fmt.Errorf("failed to marshal notification payload: %v", err)
	}

	req, err := http.NewRequest("POST", url, strings.NewReader(string(jsonData)))
	if err != nil {
		return fmt.Errorf("failed to create notification request: %v", err)
	}

	req.Header.Set("Content-Type", "application/json")

	resp, err := n.client.Do(req)
	if err != nil {
		return fmt.Errorf("failed to send notification: %v", err)
	}
	defer func() {
		if err := resp.Body.Close(); err != nil {
			fmt.Printf("failed to close response body: %v\n", err)
		}
	}()

	if resp.StatusCode >= 400 {
		return fmt.Errorf("notification request failed with status: %d", resp.StatusCode)
	}

	return nil
}
