package job

import "fmt"

type CancelRequest struct {
	TriggerID string `json:"triggerId"`
	AuthToken string `json:"authToken,omitempty"`
}

type CancelResponse struct {
	RunID        string `json:"runId"`
	DeploymentID string `json:"deploymentId"`
	Status       string `json:"status"`
}

func (r *CancelRequest) Validate() error {
	if r.TriggerID == "" {
		return fmt.Errorf("triggerId is required")
	}
	return nil
}
