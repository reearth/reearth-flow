package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type JobStatus string

const (
	JobStatusUnknown   JobStatus = "UNKNOWN"
	JobStatusPending   JobStatus = "PENDING"
	JobStatusRunning   JobStatus = "RUNNING"
	JobStatusCompleted JobStatus = "COMPLETED"
	JobStatusFailed    JobStatus = "FAILED"
)

type JobInfo struct {
	ID     id.JobID
	Name   string
	Status JobStatus
}

type Batch interface {
	SubmitJob(ctx context.Context, jobID id.JobID, workflowsURL string, metadataURL string, projectID id.ProjectID) (string, error)
	GetJobStatus(ctx context.Context, jobName string) (JobStatus, error)
	ListJobs(ctx context.Context, projectID id.ProjectID) ([]JobInfo, error)
	CancelJob(ctx context.Context, jobName string) error
}
