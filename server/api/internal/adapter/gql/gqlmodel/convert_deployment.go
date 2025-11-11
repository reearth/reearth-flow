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
		WorkspaceID: IDFrom(d.Workspace()),
		WorkflowURL: d.WorkflowURL(),
		Description: d.Description(),
		Version:     d.Version(),
		CreatedAt:   d.CreatedAt(),
		UpdatedAt:   d.UpdatedAt(),
		ProjectID:   IDFromRef(d.Project()),
		HeadID:      IDFromRef(d.HeadID()),
		IsHead:      d.IsHead(),
		Variables:   ToVariables(d.Variables()),
	}
}

func ToJob(j *job.Job) *Job {
	if j == nil {
		return nil
	}

	job := &Job{
		ID:           ID(j.ID().String()),
		DeploymentID: IDFrom(j.Deployment()),
		WorkspaceID:  IDFrom(j.Workspace()),
		Status:       ToJobStatus(j.Status()),
		StartedAt:    j.StartedAt(),
		CompletedAt:  j.CompletedAt(),
		Variables:    ToVariables(j.Variables()),
	}

	if urls := j.OutputURLs(); len(urls) > 0 {
		job.OutputURLs = urls
	}
	if logsURL := j.LogsURL(); logsURL != "" {
		job.LogsURL = &logsURL
	}
	if workerLogsURL := j.WorkerLogsURL(); workerLogsURL != "" {
		job.WorkerLogsURL = &workerLogsURL
	}
	if userFacingLogsURL := j.UserFacingLogsURL(); userFacingLogsURL != "" {
		job.UserFacingLogsURL = &userFacingLogsURL
	}
	if debug := j.Debug(); debug != nil {
		job.Debug = debug
	}

	return job
}

func ToJobStatus(status job.Status) JobStatus {
	switch status {
	case job.StatusCancelled:
		return JobStatusCancelled
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
