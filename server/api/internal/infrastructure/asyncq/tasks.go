package asyncq

import (
	"context"
	"encoding/json"
	"time"

	"github.com/hibiken/asynq"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
)

const (
	TypeWorkflowJob = "workflow:job"
)

type WorkflowJobPayload struct {
	JobID           string                 `json:"job_id"`
	WorkflowURL     string                 `json:"workflow_url"`
	MetadataURL     string                 `json:"metadata_url"`
	Variables       map[string]interface{} `json:"variables"`
	ProjectID       string                 `json:"project_id"`
	WorkspaceID     string                 `json:"workspace_id"`
	DeploymentID    string                 `json:"deployment_id"`
	NotificationURL *string                `json:"notification_url,omitempty"`
	Debug           bool                   `json:"debug"`
}

type JobOptions struct {
	MaxRetry       int                                         `json:"max_retry"`
	Queue          string                                      `json:"queue"`
	ProcessIn      time.Duration                               `json:"process_in"`
	ProcessAt      time.Time                                   `json:"process_at"`
	Deadline       time.Time                                   `json:"deadline"`
	Timeout        time.Duration                               `json:"timeout"`
	Unique         *UniqueOptions                              `json:"unique,omitempty"`
	RetryDelayFunc func(int, error, *asynq.Task) time.Duration `json:"-"`
}

type UniqueOptions struct {
	TTL time.Duration `json:"ttl"`
}

func NewWorkflowJobTask(
	jobID id.JobID,
	workflowURL, metadataURL string,
	variables map[string]interface{},
	projectID id.ProjectID,
	workspaceID accountdomain.WorkspaceID,
	deploymentID id.DeploymentID,
	notificationURL *string,
	debug bool,
) (*asynq.Task, error) {
	payload := WorkflowJobPayload{
		JobID:           jobID.String(),
		WorkflowURL:     workflowURL,
		MetadataURL:     metadataURL,
		Variables:       variables,
		ProjectID:       projectID.String(),
		WorkspaceID:     workspaceID.String(),
		DeploymentID:    deploymentID.String(),
		NotificationURL: notificationURL,
		Debug:           debug,
	}

	payloadBytes, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}

	return asynq.NewTask(TypeWorkflowJob, payloadBytes), nil
}

func ParseWorkflowJobPayload(task *asynq.Task) (WorkflowJobPayload, error) {
	var payload WorkflowJobPayload
	err := json.Unmarshal(task.Payload(), &payload)
	return payload, err
}

type TaskInfo struct {
	ID            string                 `json:"id"`
	Type          string                 `json:"type"`
	Payload       map[string]interface{} `json:"payload"`
	Queue         string                 `json:"queue"`
	MaxRetry      int                    `json:"max_retry"`
	Retried       int                    `json:"retried"`
	LastErr       string                 `json:"last_err"`
	CreatedAt     time.Time              `json:"created_at"`
	ProcessedAt   *time.Time             `json:"processed_at,omitempty"`
	CompletedAt   *time.Time             `json:"completed_at,omitempty"`
	NextProcessAt *time.Time             `json:"next_process_at,omitempty"`
}

type TaskHandler func(ctx context.Context, task *asynq.Task) error
