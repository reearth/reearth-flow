package trigger

import (
	"encoding/json"
	"fmt"
)

type ExecutionRequest struct {
	AuthToken       string            `json:"authToken,omitempty"`
	NotificationURL string            `json:"notificationUrl,omitempty"`
	With            map[string]string `json:"with,omitempty"`
}

type ExecutionResponse struct {
	RunID        string `json:"runId"`
	DeploymentID string `json:"deploymentId"`
	Status       string `json:"status"`
}

func (r *ExecutionRequest) Validate() error {
	data, err := json.Marshal(r)
	if err != nil {
		return fmt.Errorf("failed to marshal ExecutionRequest: %v", err.Error())
	}

	return ExecutionValidator.Validate(data)
}
