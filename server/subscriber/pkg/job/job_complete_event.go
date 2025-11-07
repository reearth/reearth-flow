package job

import "time"

type JobCompleteEvent struct {
	WorkflowID string    `json:"workflowId"`
	JobID      string    `json:"jobId"`
	Result     string    `json:"result"` // "success" or "failed" (LOWERCASE!)
	Timestamp  time.Time `json:"timestamp"`
}
