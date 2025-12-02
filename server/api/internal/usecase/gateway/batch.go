package gateway

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type JobStatus string

const (
	JobStatusUnknown   JobStatus = "UNKNOWN"
	JobStatusPending   JobStatus = "PENDING"
	JobStatusRunning   JobStatus = "RUNNING"
	JobStatusCancelled JobStatus = "CANCELLED"
	JobStatusCompleted JobStatus = "COMPLETED"
	JobStatusFailed    JobStatus = "FAILED"
)

type JobInfo struct {
	ID     id.JobID
	Name   string
	Status JobStatus
}

type Batch interface {
	SubmitJob(ctx context.Context, jobID id.JobID, workflowsURL string, metadataURL string, variables map[string]string, projectID id.ProjectID, workspaceID accountsid.WorkspaceID) (string, error)
	GetJobStatus(ctx context.Context, jobName string) (JobStatus, error)
	ListJobs(ctx context.Context, projectID id.ProjectID) ([]JobInfo, error)
	CancelJob(ctx context.Context, jobName string) error
}
