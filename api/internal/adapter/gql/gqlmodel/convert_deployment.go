package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/job"
)

func ToDeployment(d *deployment.Deployment) *Deployment {
	if d == nil {
		return nil
	}

	return &Deployment{
		ID:          IDFrom(d.ID()),
		ProjectID:   IDFrom(d.Project()),
		WorkspaceID: IDFrom(d.Workspace()),
		WorkflowURL: d.WorkflowUrl(),
		Version:     d.Version(),
		CreatedAt:   d.CreatedAt(),
		UpdatedAt:   d.UpdatedAt(),
	}
}

func ToJob(j *job.Job) *Job {
	if j == nil {
		return nil
	}

	return &Job{
		ID:           IDFrom(j.ID()),
		DeploymentID: IDFrom(j.Deployment()),
		WorkspaceID:  IDFrom(j.Workspace()),
		Status:       ToJobStatus(j.Status()),
		StartedAt:    j.StartedAt(),
		CompletedAt:  j.CompletedAt(),
	}
}

func ToJobStatus(status job.Status) JobStatus {
	switch status {
	case job.StatusPending:
		return JobStatusPending
	case job.StatusRunning:
		return JobStatusRunning
	case job.StatusCompleted:
		return JobStatusCompleted
	case job.StatusFailed:
		return JobStatusFailed
	default:
		return JobStatusPending
	}
}
